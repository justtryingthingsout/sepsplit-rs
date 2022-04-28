use memchr::memmem;
use std::{fs, str, path::{PathBuf}, env};
mod utils;
use utils::*;
use binrw::{BinReaderExt, io::Cursor};

//calculate the end of the Mach-O file, by seeing the last possible offset of all segments
fn calc_size(bytes: &[u8]) -> usize { 
    if bytes.len() < 1024 { return 0 }
    let hdr = cast_struct!(MachHeader, &bytes);
    let mut q = MACHHEADER_SIZE;
    let mut end: u64;
    let mut tsize = 0;

    if !hdr.is_macho() { return 0 }
    else if hdr.is64() { q += 4; }

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
            _ => {}
        }
        q += cmd.cmdsize as usize;
    }

    tsize as usize
}

//main functions

//places the DATA segment specified into where the DATA segment is supposed to be
fn fix_data_segment(image: &mut [u8], data: &[u8]) -> Result<(), String> {
    let mut p = MACHHEADER_SIZE;
    
    let machheader = cast_struct!(MachHeader, &image);
    if !machheader.is_macho() { return Err(String::from("Not macho")) }
    else if machheader.is64() { p += 4; }

    for _ in 0..machheader.ncmds {
        let cur_lcmd = cast_struct!(LoadCommand, &image[p..]);
        match cur_lcmd.cmd.try_into() {
            Ok(CMD::Segment) => {
                let seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname == SEG_DATA {
                    image[range_size!(seg.fileoff as usize, data.len())].copy_from_slice(&data);
                }
            }
            Ok(CMD::Segment64) => {
                let seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if &seg.segname == SEG_DATA {
                    image[range_size!(seg.fileoff as usize, data.len())].copy_from_slice(&data);
                }
            },
            _ => {}
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
            _ => {}
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
            _ => {}
        }
        p += cur_lcmd.cmdsize as usize
    };

    Ok(())
}

//restores the file's LINKEDIT and optionally DATA segments, and saves using the name
fn restore_file(index: usize, buf: &[u8], path: &PathBuf, tail: &str, data_buf: Option<&[u8]>) {
    let file: PathBuf = path.join(format!("sepdump{:02}_{}", index, tail));
    
    let mut tmp = buf.to_owned();
    if let Err(err) = fix_linkedit(&mut tmp) {
        println!("Error in fix_linkedit function: {}", err)
    }
    if let Some(real_data_buf) = data_buf { 
        if let Err(err) = fix_data_segment(&mut tmp, real_data_buf) {
            println!("Error in fix_data_segment function: {}", err)
        };
    }
    fs::write(&file, tmp).unwrap_or_else(|e| panic!("Unable to write to file {}, err: {}", &file.display(), e));
}

//main function of the program, splits the SEP apps from the file by reading the structs
fn split(hdr_offset: Option<usize>, kernel: &Vec<u8>, outdir: PathBuf, sepinfo: Option<SEPinfo>) {
    if let Some(hdr_offset) = hdr_offset {
        println!("detected 64 bit SEP");
        let hdr = cast_struct_binread!(SEPDataHDR64, &kernel[hdr_offset..]);
        let mut off = hdr_offset + SEPHDR_SIZE 
                      - if hdr.stack_size == 0 {24} else {0} 
                      + if hdr.shm_size == 0 {0} else {24}; //see top of utils.rs file
        
        //first part of image, boot
        let bootout = outdir.join("sepdump00_boot");
        fs::write(&bootout, &kernel[..hdr.kernel_base_paddr as usize]).unwrap_or_else(|e| panic!("Unable to write to file {}, err: {}", &bootout.display(), e));
        println!("boot         size {:#x}", hdr.kernel_base_paddr as usize);

        //second part, kernel
        let mut sz = calc_size(&kernel[hdr.kernel_base_paddr as usize..]);
        restore_file(1, &kernel[range_size!(hdr.kernel_base_paddr as usize, sz)], &outdir, "kernel", None);
        println!("kernel       size {:#x}", sz);

        //SEPOS aka "rootserver"
        let mut tail = str::from_utf8(&hdr.init_name)
                       .unwrap_or_else(|e| panic!("Could not convert name to utf-8, err: {}", e))
                       .split_whitespace()
                       .next()
                       .unwrap(); //get the name of the first image (SEPOS) without spaces
        sz = calc_size(&kernel[hdr.init_base_paddr as usize..]);
        restore_file(2, &kernel[range_size!(hdr.init_base_paddr as usize, sz)], &outdir, tail, None);
        println!("{:-12} size {:#x}", tail, sz as usize);

        //the rest of the apps
        let sepappsize = SEPAPP_64_SIZE 
                         - if hdr.srcver.get_major() < 1300 { 8 } else { 0 } 
                         + if hdr.srcver.get_major() > 1700 { 4 } else { 0 }; //similar to reasons as top of utils.rs
        let mut app;
        for i in 0..hdr.n_apps as usize {
            app = cast_struct_binread!(SEPApp64, &kernel[off..]);
            tail = str::from_utf8(&app.app_name).unwrap_or_else(|e| panic!("Could not convert name to utf-8, err: {}", e)).split_whitespace().next().unwrap();
            let data_buf = &kernel[range_size!(app.phys_data as usize, app.size_data as usize)].to_owned();
            restore_file(i + 3, &kernel[range_size!(app.phys_text as usize, (app.size_text + app.size_data) as usize)], &outdir, tail, Some(data_buf));
            println!("{:-12} phys_text {:#x}, virt {:#x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x}",
                tail, app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry);
            off += sepappsize;
        }
    } else if let Some(mut sep_info) = sepinfo {
        println!("detected 32 bit SEP");

        //index 0: boot
        let mut bootout = outdir.join("sepdump00_boot");
        fs::write(&bootout, &kernel[..0x1000]).unwrap_or_else(|e| panic!("Unable to write to file {}, err: {}", &bootout.display(), e)); 
        println!("boot         size 0x1000");

        //index 1: kernel
        let mut st = 0x1000;
        let mut sz = calc_size(&kernel[st..]); //most SEP fws
        
        if sz == 0 {
            if &kernel[st..st+4] == &[0, 0, 0, 0] {
                //J97 SEP Firmware
                st = 0x4000;
                sz = calc_size(&kernel[st..]); 
                restore_file(1, &kernel[range_size!(st, sz)], &outdir, "kernel", None);
            } else {
                //N71 SEP or newer SEP Firmware
                bootout = outdir.join("sepdump01_kernel");
                fs::write(&bootout, &kernel[range_size!(st, 0xe000)]).unwrap_or_else(|e| panic!("Unable to write to file {}, err: {}", &bootout.display(), e)); 
                sz = 0xe000;
            }
        } else {
            restore_file(1, &kernel[range_size!(st, sz)], &outdir, "kernel", None);
        }

        println!("kernel       size {:#x}", sz);

        //check for newer SEP
        let tmp = cast_struct!(SEPAppOld, &kernel[sep_info.sep_app_pos..]);
        if tmp.size == 0 {
            //64 bit SEP struct in 32 bit SEP
            let mut app = cast_struct_binread!(SEPApp64, &kernel[sep_info.sep_app_pos..]);
            let sepappsize = SEPAPP_64_SIZE + if app.srcver.get_major() > 1700 { 4 } else { 0 };
            let mut tail;

            //dump struct from start of kernel
            bootout = outdir.join("sepdump-extra_struct");
            fs::write(&bootout, &kernel[range_size!(app.phys_text as usize, 0x1000)]).unwrap_or_else(|e| panic!("Unable to write to file {}, err: {}", &bootout.display(), e)); 
            println!("struct       size {:#x}", 0x1000);
            app.phys_text += 0x1000;
            app.size_text -= 0x1000;


            for i in 2.. {
                if i != 1 {
                    app = cast_struct_binread!(SEPApp64, &kernel[sep_info.sep_app_pos..]);
                }
                if app.phys_text == 0 { return }
                tail = str::from_utf8(&app.app_name).unwrap_or_else(|e| panic!("Could not convert name to utf-8, error: {}", e)).split_whitespace().next().unwrap();
                let data_buf = &kernel[range_size!(app.phys_data as usize, app.size_data as usize)].to_owned();
                restore_file(i, &kernel[range_size!(app.phys_text as usize, (app.size_text + app.size_data) as usize)], &outdir, tail, Some(data_buf));
                println!("{:-12} phys_text {:#08x}, virt {:#06x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x}",
                    tail, app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry);
                sep_info.sep_app_pos += sepappsize;
            }
        }

        /*
            preparation for loop, find offset of "SEPOS" string and 
            calculate size of structs based off "SEPD" string and previous string
        */
        let tailoff = memmem::find(&kernel[sep_info.sep_app_pos..], b"SEPOS       ").unwrap_or_else(|| panic!("Could not find SEPOS string")); //offset of the name in the struct
        sep_info.sepapp_size = memmem::find(&kernel[range_size!(sep_info.sep_app_pos+tailoff, 128)], b"SEPD").unwrap_or_else(|| panic!("Could not find SEPD string")); 

        for index in 2.. {
            let (tail, mut apps);
            if sep_info.sep_app_pos == 0 { panic!("SEPApp position is 0!"); }
            apps = cast_struct!(SEPAppOld, &kernel[sep_info.sep_app_pos..]);
            if apps.phys == 0 { return } //end of structs, nothing else to do
            else if index == 2 { //need SEPOS kernel's offset to dump structs
                bootout = outdir.join("sepdump-extra_struct");
                fs::write(&bootout, &kernel[range_size!(apps.phys as usize, 0x1000)]).unwrap_or_else(|e| panic!("Unable to write to file {}, error: {}", &bootout.display(), e)); 
                println!("struct       size {:#x}", 0x1000);
                apps.phys += 0x1000;
                apps.size -= 0x1000;
            }
            tail = str::from_utf8(&kernel[range_size!(sep_info.sep_app_pos + tailoff, 12)]).unwrap_or_else(|e| panic!("error when trying to convert to utf-8: {}", e)).split_whitespace().next().unwrap();
            println!("{:-12} phys {:#08x}, virt {:#x}, size {:#08x}, entry {:#x}", 
                      tail,  apps.phys,  apps.virt,  apps.size,  apps.entry);
            sep_info.sep_app_pos += sep_info.sepapp_size;
            restore_file(index, &kernel[range_size!(apps.phys as usize, apps.size as usize)], &outdir, tail, None);
        }
    }
}

//gets the position of the SEPApp struct and a temporary SEPApp size, using structs in the SEP
fn sep32_structs(krnl: &Vec<u8>) -> SEPinfo {
    let legionstr = cast_struct!(Legion32, &krnl[0x400..]);
    let monitorstr = cast_struct!(SEPMonitorBootArgs, &krnl[legionstr.off as usize..]);
    SEPinfo {
        sep_app_pos: (monitorstr.args_off as usize + KRNLBOOTARGS_SIZE), 
        sepapp_size: SEPAPP_SIZE.to_owned(),
    }
}

//find the offset of the SEP HDR struct for 64-bit
fn find_off(krnl: &Vec<u8>) -> u64 { 
    let legionstroff = memmem::find(&krnl[0..8192], b"Built by legion2").unwrap_or_else(|| { 
        eprintln!("[!] Invalid kernel inputted, exiting.");
        std::process::exit(1)
    });
    u64::from_le_bytes(krnl[range_size!(legionstroff+16, 8)].try_into().unwrap_or_else(|e| panic!("Error trying to get a u64, message: {}", e)))
}

//test that the kernel is valid, find_off will verify other cases
fn test_krnl(krnl: &[u8], fw: &String) {
    if &krnl[..2] == &[0x30, 0x83] {
        eprintln!("[!] IMG4 Header detected, please extract the SEP firmware first. Exiting.");
        std::process::exit(1)
    } else if &krnl[8..16] == b"eGirBwRD" {
        eprintln!("[!] LZVN Header detected, please decompress the SEP firmware first.\n\
                  To extract, run these commands (assuming you have lzvn installed):\n\
                  `dd if={} of=sep.compressed skip=1 bs=65536`\n\
                  `lzvn -d sep.compressed sep.bin`\n\
                  then run this program again with the decompressed file.\n\
                  Exiting.", fw);
        std::process::exit(1)
    }
}


fn main() {
    //why I don't use a crate for parsing arguments? idk, I'm more used to C
    let argv: Vec<String> = std::env::args().collect();
    let argc = argv.len();

    if argc < 2 {
        eprintln!("[!] Not enough arguments\n\
                   sepsplit-rs - tool to split SEPOS firmware into its individual modules, by @plzdonthaxme\n\
                   Usage: {} <SEPOS.bin> [output folder]", &argv[0]);
        return
    }

    let krnl: Vec<u8> = fs::read(&argv[1]).unwrap_or_else(|e| panic!("[-] Cannot read kernel, err: {}", e));
    test_krnl(&krnl[..16], &argv[1]);
    let outdir: PathBuf = if argc > 2 {PathBuf::from(&argv[2])} else {env::current_dir().unwrap()}; //if output dir is specified, use it
    let hdr_offset = find_off(&krnl); //will give 0 when 32-bit SEP
    
    if hdr_offset == 0 { //32-bit SEP
        let septype = sep32_structs(&krnl);
        split(None, &krnl, outdir, Some(septype));
    } else { //64-bit SEP
        split(Some(hdr_offset as usize), &krnl, outdir, None);
    }
}