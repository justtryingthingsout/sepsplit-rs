use memchr::memmem;
use std::{
    fs::{self, File}, 
    str, 
    path::{Path, PathBuf}, 
    env, 
    process::exit, 
    io::{Write, BufWriter, StdoutLock}
};
mod utils;
use utils::*;
use binrw::{BinReaderExt, io::Cursor, BinRead};
use uuid::Uuid;

//calculate the end of the Mach-O file, by seeing the last possible offset of all segments
fn calc_size(bytes: &[u8]) -> usize { 
    if bytes.len() < 1024 { return 0 }
    let hdr = cast_struct!(MachHeader, bytes);
    let mut q = MACHHEADER_SIZE;
    let mut end: u64;
    let mut tsize = 0;

    if !hdr.is_macho() { return 0 }
    else if hdr.is64() { q += 4 }

    //check segments in mach-o file
    for _ in 0..hdr.ncmds {
        let cmd = cast_struct!(LoadCommand, &bytes[q..]);
        match cmd.cmd.try_into() {
            Ok(CMD::Segment) => {
                let seg = cast_struct!(Segment, &bytes[q+LOADCOMMAND_SIZE..]);
                end = (seg.fileoff + seg.filesize) as u64;
                if tsize < end { tsize = end; }
            },
            Ok(CMD::Segment64) => {
                let seg = cast_struct!(Segment64, &bytes[q+LOADCOMMAND_SIZE..]);
                end = seg.fileoff + seg.filesize;
                if tsize < end { tsize = end; }
            },
            _ => ()
        }
        q += cmd.cmdsize as usize;
    }

    tsize as usize
}

//main functions

//places the DATA segment specified into where the DATA segment is supposed to be
fn fix_data_segment(image: &mut [u8], data: &[u8], dataoff: Option<usize>) -> Result<(), String> {
    let mut p = MACHHEADER_SIZE;
    
    let machheader = cast_struct!(MachHeader, image);
    if !machheader.is_macho() { return Err(String::from("Not macho")) }
    else if machheader.is64() { p += 4; }

    for _ in 0..machheader.ncmds {
        let cur_lcmd = cast_struct!(LoadCommand, &image[p..]);
        match cur_lcmd.cmd.try_into() {
            Ok(CMD::Segment) => {
                let seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname == SEG_DATA {
                    let segoff = match dataoff {
                        Some(a) => a,
                        None => seg.fileoff as usize,
                    };
                    image[range_size!(segoff, data.len())].copy_from_slice(data);
                }
            }
            Ok(CMD::Segment64) => {
                let seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname == SEG_DATA {
                    let segoff = match dataoff {
                        Some(a) => a,
                        None => seg.fileoff as usize,
                    };
                    image[range_size!(segoff, data.len())].copy_from_slice(data);
                }
            },
            _ => ()
        }
        p += cur_lcmd.cmdsize as usize;
    };

    Ok(())
}

//fixes LINKEDIT offsets
fn fix_linkedit(image: &mut [u8]) -> Result<(), String> {
    let mut min: u64 = u64::MAX;
    let mut p = MACHHEADER_SIZE;
    
    let machheader = cast_struct!(MachHeader, &image[..MACHHEADER_SIZE]);
    if !machheader.is_macho() { return Err(String::from("Not macho")) }
    else if machheader.is64() { p += 4; }

    for _ in 0..machheader.ncmds {
        let cur_lcmd = cast_struct!(LoadCommand, &image[p..]);
        match cur_lcmd.cmd.try_into() {
            Ok(CMD::Segment) => {
                let seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname != SEG_PAGEZERO && min > seg.vmaddr as u64 { min = seg.vmaddr as u64; }
            },
            Ok(CMD::Segment64) => {
                let seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname != SEG_PAGEZERO && min > seg.vmaddr { min = seg.vmaddr; }
            },
            _ => ()
        }
        p += cur_lcmd.cmdsize as usize
    };

    let mut delta: u64;
    p = MACHHEADER_SIZE + if machheader.is64() {4} else {0};

    for _ in 0..machheader.ncmds {
        let cur_lcmd = cast_struct!(LoadCommand, &image[p..]);
        match cur_lcmd.cmd.try_into() {
            Ok(CMD::Segment) => {
                let mut seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname == SEG_LINKEDIT  {
                    delta = seg.vmaddr as u64 - min - seg.fileoff as u64;
                    seg.fileoff += delta as u32;
                }
                let buf = &bincode::serialize(&seg).unwrap();
                image[range_size!(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(buf)
            },
            Ok(CMD::Segment64) => {
                let mut seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname == SEG_LINKEDIT  { 
                    delta = seg.vmaddr - min - seg.fileoff;
                    seg.fileoff += delta;
                }
                let buf = &bincode::serialize(&seg).unwrap();
                image[range_size!(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(buf)
            },
            Ok(CMD::SymTab) => {
                /* what xerub's code did (translated into Rust):
                    let mut seg = cast_struct!(SymTab, &image[p+LOADCOMMAND_SIZE..]);
                    if seg.stroff != 0 { seg.stroff += delta as u32};
                    if seg.symoff != 0 { seg.symoff += delta as u32};
                    
                this does not work because there aren't even any symbols in the binaries. */

                let seg = SymTab {
                    stroff: 0,
                    symoff: 0,
                    nsyms: 0,
                    strsize: 0,
                };
                let buf = &bincode::serialize(&seg).unwrap();
                image[range_size!(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(buf)
            },
            _ => ()
        }
        p += cur_lcmd.cmdsize as usize
    };

    Ok(())
}

//restores the file's LINKEDIT and optionally DATA segments, and saves using the name
fn restore_file(index: usize, buf: &[u8], path: &Path, tail: &str, data_buf: Option<&[u8]>, dataoff: Option<usize>) {
    let file: &Path = &path.join(format!("sepdump{index:02}_{tail}"));
    
    let mut tmp = buf.to_owned();
    if let Err(err) = fix_linkedit(&mut tmp) {
        eprintln!("Error in fix_linkedit function: {err}")
    }
    if let Some(data_seg) = data_buf { 
        if let Err(err) = fix_data_segment(&mut tmp, data_seg, dataoff) {
            eprintln!("Error in fix_data_segment function: {err}")
        };
    }
    filewrite!(file, &tmp);
}

//splits the SEP apps from the 64-bit SEP Firmware by reading the structs
fn split64(hdr_offset: usize, kernel: &[u8], outdir: &Path, mut outbuf: BufWriter<StdoutLock>, ver: u8) -> Result<(), std::io::Error> {
    writeln!(&mut outbuf, "detected 64 bit SEP")?;
    let hdr = cast_struct_args!(SEPDataHDR64, &kernel[hdr_offset..], (ver, ));
    let mut off = hdr_offset + SEPHDR_SIZE 
                    + if ver == 4 { 56 } else if hdr.ar_min_size == 0 { 0 } else { 24 } //see top of utils.rs file
                    - if hdr.stack_size == 0 && ver != 4 { 24 } else { 0 };
    
    //first part of image, boot
    let bootout = outdir.join("sepdump00_boot");
    filewrite!(&bootout, &kernel[..hdr.kernel_base_paddr as usize]);
    writeln!(&mut outbuf, "boot         size {sz:#x}", sz=hdr.kernel_base_paddr as usize)?;

    //second part, kernel
    let mut sz = calc_size(&kernel[hdr.kernel_base_paddr as usize..]);
    let mut uuid = Uuid::from_bytes_le(hdr.kernel_uuid).hyphenated().to_string();
    if sz == 0 {
        filewrite!(&bootout, &kernel[hdr.kernel_base_paddr as usize..hdr.kernel_max_paddr as usize]);
    } else {
        restore_file(1, &kernel[range_size!(hdr.kernel_base_paddr as usize, sz)], outdir, "kernel", None, None);
    }
    writeln!(&mut outbuf, "kernel       size {sz:#x}, UUID {uuid}")?;

    //SEPOS aka "rootserver"
    let mut tail = strslice!(&hdr.init_name, "init_name"); //get the name of the first image (SEPOS) without spaces;
    uuid = Uuid::from_bytes_le(hdr.init_uuid).hyphenated().to_string();
    sz = calc_size(&kernel[hdr.init_base_paddr as usize..]);
    restore_file(2, &kernel[range_size!(hdr.init_base_paddr as usize, sz)], outdir, tail, None, None);
    writeln!(&mut outbuf, "{tail:-12} size {sz:#x}, UUID {uuid}")?;

    //the rest of the apps
    let sepappsize = SEPAPP_64_SIZE 
                     - if hdr.srcver.get_major() < 1300 { 8 } else { 0 } 
                     + if hdr.srcver.get_major() > 1700 { 
                         if hdr.srcver.get_major() > 2000 { 36 } else { 4 }
                       } else { 0 }; //similar to reasons as top of utils.rs
    let mut app;
    dbg!(off, sepappsize);
    for i in 0..hdr.n_apps as usize {
        app = cast_struct_args!(SEPApp64, &kernel[off..], (ver, ));
        tail = strslice!(&app.app_name, "app_name");
        let data_buf = &kernel[range_size!(app.phys_data as usize, app.size_data as usize)].to_owned();
        restore_file(i + 3, &kernel[range_size!(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), None);
        let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
        writeln!(&mut outbuf, "{tail:-12} phys_text {:#x}, virt {:#x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x},\n             UUID {uuid}",
            app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
        off += sepappsize;
    }
    for i in 0..hdr.n_shlibs as usize {
        app = cast_struct_args!(SEPApp64, &kernel[off..], (ver, ));
        tail = strslice!(&app.app_name, "app_name");
        let data_buf = &kernel[range_size!(app.phys_data as usize, app.size_data as usize)].to_owned();
        restore_file(i + 3, &kernel[range_size!(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), Some(app.size_text as usize));
        let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
        writeln!(&mut outbuf, "{tail:-12} phys_text {:#x}, virt {:#x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x},\n             UUID {uuid}",
            app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
        off += sepappsize;
    }
    outbuf.flush()
}

//splits the SEP apps from the 32-bit SEP Firmware by reading the structs
fn split32(kernel: &[u8], outdir: &Path, mut sep_info: SEPinfo, mut outbuf: BufWriter<StdoutLock>) -> Result<(), std::io::Error> {
    writeln!(&mut outbuf, "detected 32 bit SEP")?;

    //index 0: boot
    let mut bootout = outdir.join("sepdump00_boot");
    filewrite!(&bootout, &kernel[..0x1000]); 
    writeln!(&mut outbuf, "boot         size 0x1000")?;

    //index 1: kernel
    let mut st = 0x1000;
    let mut sz = calc_size(&kernel[st..]); //most SEP fws
    
    if sz == 0 {
        if kernel[range_size!(st, 4)] == [0; 4] {
            //J97 SEP Firmware
            st = 0x4000;
            sz = calc_size(&kernel[st..]); 
            restore_file(1, &kernel[range_size!(st, sz)], outdir, "kernel", None, None);
        } else {
            //N71 SEP or newer SEP Firmware
            bootout = outdir.join("sepdump01_kernel");
            filewrite!(&bootout, &kernel[range_size!(st, 0xe000)]);
            sz = 0xe000;
        }
    } else {
        restore_file(1, &kernel[range_size!(st, sz)], outdir, "kernel", None, None);
    }

    writeln!(&mut outbuf, "kernel       size {sz:#x}")?;

    //check for newer SEP
    let tmp = cast_struct!(SEPAppOld, &kernel[sep_info.sep_app_pos..]);
    if tmp.size == 0 {
        //64 bit SEP struct in 32 bit SEP

        //number of apps must be valid in this case
        let n_apps = sep_info.sepapps.unwrap();

        let mut app = cast_struct_binread!(SEPApp64, &kernel[sep_info.sep_app_pos..]);
        let sepappsize = SEPAPP_64_SIZE + if app.srcver.get_major() > 1700 { 4 } else { 0 };
        let mut tail;

        //dump struct from start of kernel
        bootout = outdir.join("sepdump-extra_struct");
        filewrite!(&bootout, &kernel[range_size!(app.phys_text as usize, 0x1000)]);
        writeln!(&mut outbuf, "struct       size 0x1000")?;
        app.phys_text += 0x1000;
        app.size_text -= 0x1000;

        for i in 2..n_apps {
            if i != 2 {
                app = cast_struct_binread!(SEPApp64, &kernel[sep_info.sep_app_pos..]);
            }
            tail = strslice!(&app.app_name, "app_name");
            let data_buf = &kernel[range_size!(app.phys_data as usize, app.size_data as usize)].to_owned();
            restore_file(i, &kernel[range_size!(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), None);
            let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
            writeln!(&mut outbuf, "{tail:-12} phys_text {:#08x}, virt {:#06x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x},\n             UUID {uuid}",
                app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
            sep_info.sep_app_pos += sepappsize;
        }
        return outbuf.flush()
    } else { //older SEP
        /*
            preparation for loop, find offset of "SEPOS" string and 
            calculate size of structs based off "SEPD" string and previous string
        */
        let tailoff = memmem::find(&kernel[sep_info.sep_app_pos..], b"SEPOS       ").unwrap_or_else(|| panic!("Could not find SEPOS string")); //offset of the name in the struct
        sep_info.sepapp_size = memmem::find(&kernel[range_size!(sep_info.sep_app_pos+tailoff, 128)], b"SEPD").unwrap_or_else(|| panic!("Could not find SEPD string")); 

        for index in 2.. {
            let (tail, mut apps);
            if sep_info.sep_app_pos == 0 { panic!("SEPApp position is 0!") }
            apps = cast_struct!(SEPAppOld, &kernel[sep_info.sep_app_pos..]);
            if apps.phys == 0 { //end of structs, nothing else to do
                return outbuf.flush() 
            } else if index == 2 { //need SEPOS kernel's offset to dump structs
                bootout = outdir.join("sepdump-extra_struct");
                filewrite!(&bootout, &kernel[range_size!(apps.phys as usize, 0x1000)]); 
                writeln!(&mut outbuf, "struct       size 0x1000")?;
                apps.phys += 0x1000;
                apps.size -= 0x1000;
            }
            tail = strslice!(&kernel[range_size!(sep_info.sep_app_pos + tailoff, 12)], "name");
            let uuid = Uuid::from_bytes_le(kernel[range_size!(sep_info.sep_app_pos + tailoff + 12, 16)].try_into().unwrap()).hyphenated().to_string();
            writeln!(&mut outbuf, "{tail:-12} phys {:#08x}, virt {:#x}, size {:#08x}, entry {:#x},\n             UUID {uuid}", 
                      apps.phys,  apps.virt,  apps.size,  apps.entry)?;
            sep_info.sep_app_pos += sep_info.sepapp_size;
            restore_file(index, &kernel[range_size!(apps.phys as usize, apps.size as usize)], outdir, tail, None, None);
        }
    }
    outbuf.flush()
}

//gets the position of the SEPApp struct and a temporary SEPApp size, using structs in the SEP
fn sep32_structs(krnl: &[u8]) -> SEPinfo {
    let legionstr = cast_struct!(Legion32, &krnl[0x400..]);
    let monitorstr = cast_struct!(SEPMonitorBootArgs, &krnl[legionstr.off as usize..]);
    let krnlbastr = cast_struct!(SEPKernBootArgs, &krnl[monitorstr.args_off as usize..]);
    SEPinfo {
        sep_app_pos: (monitorstr.args_off as usize + KRNLBOOTARGS_SIZE), 
        sepapp_size: SEPAPP_SIZE.to_owned(),
        sepapps: if krnlbastr.num_apps > 0xFF { None } else { Some(krnlbastr.num_apps as usize) },
    }
}

//find the offset of the SEP HDR struct for 64-bit
fn find_off(krnl: &[u8]) -> (u64, u8) { 
    if &krnl[range_size!(0x1004, 16)] == b"Built by legion2" { 
        //iOS 15 and below
        let hdr = cast_struct!(Legion64Old, &krnl[0x1000..]);
        (hdr.structoff as u64, hdr.subversion as u8)
    } else if &krnl[range_size!(0x103c, 16)] == b"Built by legion2" {
        //iOS 16
        let hdr16 = cast_struct!(Legion64, &krnl[0x1000..]);
        let uuid = Uuid::from_bytes_le(hdr16.uuid).hyphenated().to_string();
        println!("HDR UUID: {uuid}");
        (hdr16.structoff as u64, hdr16.subversion as u8)
    } else if &krnl[range_size!(0x408, 16)] == b"Built by legion2" {
        let hdr = cast_struct!(Legion32, &krnl[0x400..]);
        (hdr.off as u64, hdr.subversion as u8)
    } else {
        eprintln!("[!] Invalid or unknown kernel inputted, exiting.");
        exit(1)
    }
}

//test that the kernel is valid, find_off will verify other cases
fn test_krnl(krnl: &[u8], fw: &String) {
    if krnl[..2] == [0x30, 0x83] {
        eprintln!("[!] IMG4 Header detected, please extract the SEP firmware first. Exiting.");
        exit(1)
    } else if &krnl[8..16] == b"eGirBwRD" {
        eprintln!("[!] LZVN Header detected, please decompress the SEP firmware first.\n\
                  To extract, run these commands (assuming you have lzvn installed):\n\
                  `dd if={fw} of=sep.compressed skip=1 bs=65536`\n\
                  `lzvn -d sep.compressed sep.bin`\n\
                  then run this program again with the decompressed file.\n\
                  Exiting.");
        exit(1)
    }
}


fn main() -> Result<(), std::io::Error> {
    //why I don't use a crate for parsing arguments? idk, I'm more used to C
    let argv: Vec<String> = std::env::args().collect();
    let argc = argv.len();

    if argc < 2 {
        eprintln!("[!] Not enough arguments\n\
                   sepsplit-rs - tool to split SEPOS firmware into its individual modules, by @plzdonthaxme\n\
                   Usage: {prog} <SEPOS.bin> [output folder]", prog=&argv[0]);
        exit(1)
    }

    let krnl: Vec<u8> = fs::read(&argv[1]).unwrap_or_else(|e| panic!("[-] Cannot read kernel, err: {e}"));
    test_krnl(&krnl[..16], &argv[1]);
    let outdir = &if argc > 2 {
        PathBuf::from(&argv[2])
    } else {
        env::current_dir().unwrap_or_else(|e| panic!("Cannot get current dir: {e}")) //if output dir is specified, use it
    };
    let (hdr_offset, ver) = find_off(&krnl);
    
    //fast stdout
    let stdout = std::io::stdout();
    let outlock = stdout.lock();
    let outbuf = std::io::BufWriter::new(outlock);

    if ver == 1 { //32-bit SEP
        let septype = sep32_structs(&krnl);
        split32(&krnl, outdir, septype, outbuf)
    } else { //64-bit SEP
        split64(hdr_offset as usize, &krnl, outdir, outbuf, ver)
    }
}