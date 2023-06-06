//clippy config
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(
    clippy::use_self, //issues with this from nursery
    clippy::cast_possible_truncation //can't do much about this
)]

use memchr::memmem;

use std::{
    str, 
    path::Path,
    process, 
    io::{Write, BufWriter}, 
    ffi::c_void,
    fs
};

#[macro_use]
mod utils;

#[allow(clippy::wildcard_imports)]
use utils::*;

use binrw::{
    io::Cursor, 
    BinRead,
    BinReaderExt,
    BinWriterExt
};

use uuid::Uuid;

#[allow(warnings)]
mod bindings {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

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
            Ok(Cmd::Segment) => {
                let seg = cast_struct!(Segment, &bytes[q+LOADCOMMAND_SIZE..]);
                end = u64::from(seg.fileoff + seg.filesize);
                if tsize < end { tsize = end; }
            },
            Ok(Cmd::Segment64) => {
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
            Ok(Cmd::Segment) => {
                let seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if seg.segname == SEG_DATA {
                    let segoff = dataoff.unwrap_or(seg.fileoff as usize);
                    image[range_size(segoff, data.len())].copy_from_slice(data);
                }
            }
            Ok(Cmd::Segment64) => {
                let seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if seg.segname == SEG_DATA {
                    let segoff = dataoff.unwrap_or(seg.fileoff as usize);
                    image[range_size(segoff, data.len())].copy_from_slice(data);
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
            Ok(Cmd::Segment) => {
                let seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if seg.segname != SEG_PAGEZERO && min > u64::from(seg.vmaddr) { 
                    min = u64::from(seg.vmaddr); 
                }
            },
            Ok(Cmd::Segment64) => {
                let seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if seg.segname != SEG_PAGEZERO && min > seg.vmaddr { 
                    min = seg.vmaddr; 
                }
            },
            _ => ()
        }
        p += cur_lcmd.cmdsize as usize;
    };

    let mut delta: u64;
    p = MACHHEADER_SIZE + if machheader.is64() {4} else {0};

    for _ in 0..machheader.ncmds {
        let cur_lcmd = cast_struct!(LoadCommand, &image[p..]);
        match cur_lcmd.cmd.try_into() {
            Ok(Cmd::Segment) => {
                let mut seg = cast_struct!(Segment, &image[p+LOADCOMMAND_SIZE..]);
                if seg.segname == SEG_LINKEDIT  {
                    delta = u64::from(seg.vmaddr) - min - u64::from(seg.fileoff);
                    seg.fileoff += delta as u32;
                }
                let mut buf = Vec::new();
                write_struct!(seg, buf);
                image[range_size(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(&buf);
            },
            Ok(Cmd::Segment64) => {
                let mut seg = cast_struct!(Segment64, &image[p+LOADCOMMAND_SIZE..]);
                if seg.segname == SEG_LINKEDIT  { 
                    delta = seg.vmaddr - min - seg.fileoff;
                    seg.fileoff += delta;
                }
                let mut buf = Vec::new();
                write_struct!(seg, buf);
                image[range_size(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(&buf);
            },
            Ok(Cmd::SymTab) => {
                /* what xerub's code did (translated into Rust):
                    let mut seg = cast_struct!(SymTab, &image[p+LOADCOMMAND_SIZE..]);
                    if seg.stroff != 0 { seg.stroff += delta as u32};
                    if seg.symoff != 0 { seg.symoff += delta as u32};
                    
                this does not work because there aren't even any symbols in the binaries. */

                let seg = SymTab::default();
                let mut buf = Vec::new();
                write_struct!(seg, buf);
                image[range_size(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(&buf);
            },
            Ok(Cmd::DySymTab) => {
                // same reasons as above
                let seg = DySymTab::default();
                let mut buf = Vec::new();
                write_struct!(seg, buf);
                image[range_size(p+LOADCOMMAND_SIZE, buf.len())].copy_from_slice(&buf);
            }
            _ => ()
        }
        p += cur_lcmd.cmdsize as usize;
    };

    Ok(())
}

//restores the file's LINKEDIT and optionally DATA segments, and saves using the name
fn restore_file(index: usize, buf: &[u8], path: &Path, tail: &str, data_buf: Option<&[u8]>, dataoff: Option<usize>) {
    let file: &Path = &path.join(format!("sepdump{index:02}_{tail}"));
    
    let mut tmp = buf.to_owned();
    if let Err(err) = fix_linkedit(&mut tmp) {
        eprintln!("Error in fix_linkedit function: {err}");
    }
    if let Some(data_seg) = data_buf { 
        if let Err(err) = fix_data_segment(&mut tmp, data_seg, dataoff) {
            eprintln!("Error in fix_data_segment function: {err}");
        };
    }
    filewrite(file, &tmp);
}

//splits the SEP apps from the 64-bit SEP Firmware by reading the structs
fn split64(hdr_offset: usize, kernel: &[u8], outdir: &Path, mut outbuf: BufWriter<Box<dyn Write>>, ver: u8) -> Result<(), std::io::Error> {
    writeln!(&mut outbuf, "detected 64 bit SEP")?;
    let hdr = cast_struct_args!(SEPDataHDR64, &kernel[hdr_offset..], (ver, ));
    let mut off = hdr_offset + SEPHDR_SIZE 
                    + if ver == 4 { 56 } else if hdr.ar_min_size == 0 { 0 } else { 24 } //see top of utils.rs file
                    - if hdr.stack_size == 0 && ver != 4 { 24 } else { 0 };

    let mut n_apps = hdr.n_apps;
    let mut n_shlibs = hdr.n_shlibs;
    
    if hdr.n_apps == 0 { 
        off += 0x100;
        n_apps = u32::from_le_bytes(kernel[range_size(hdr_offset+0x210, 4)].try_into().unwrap());
        n_shlibs = u32::from_le_bytes(kernel[range_size(hdr_offset+0x214, 4)].try_into().unwrap());
    }

    //first part of image, boot
    let bootout = outdir.join("sepdump00_boot");
    filewrite(&bootout, &kernel[..hdr.kernel_base_paddr as usize]);
    writeln!(&mut outbuf, "boot             size {sz:#x}", sz=hdr.kernel_base_paddr as usize)?;

    //second part, kernel
    let mut sz = calc_size(&kernel[hdr.kernel_base_paddr as usize..]);
    let mut uuid = Uuid::from_bytes_le(hdr.kernel_uuid).hyphenated().to_string();
    if sz == 0 {
        filewrite(&outdir.join("sepdump01_kernel"), &kernel[hdr.kernel_base_paddr as usize..hdr.kernel_max_paddr as usize]);
        sz = (hdr.kernel_max_paddr - hdr.kernel_base_paddr) as usize;
    } else {
        restore_file(1, &kernel[range_size(hdr.kernel_base_paddr as usize, sz)], outdir, "kernel", None, None);
    }
    writeln!(&mut outbuf, "kernel           size {sz:#x},  UUID {uuid}")?;

    //SEPOS aka "rootserver"
    let mut tail = strslice(&hdr.init_name); //get the name of the first image (SEPOS) without spaces;
    uuid = Uuid::from_bytes_le(hdr.init_uuid).hyphenated().to_string();
    sz = calc_size(&kernel[hdr.init_base_paddr as usize..]);
    restore_file(2, &kernel[range_size(hdr.init_base_paddr as usize, sz)], outdir, tail, None, None);
    writeln!(&mut outbuf, "{tail:<16} size {sz:#x}, UUID {uuid}")?;

    //the rest of the apps
    let sepappsize = SEPAPP_64_SIZE 
                     - if hdr.srcver.get_major() < 1300 { 8 } else { 0 } 
                     + match hdr.srcver.get_major() {
                        2000.. => 36,
                        1700.. => 4,
                        _ => 0
                       }; //similar to reasons as top of utils.rs
    let mut app;
    let mut i = 0;
    while i < n_apps as usize {
        app = cast_struct_args!(SEPApp64, &kernel[off..], (ver, ));
        tail = strslice(&app.app_name);
        let data_buf = &kernel[range_size(app.phys_data as usize, app.size_data as usize)].to_owned();
        restore_file(i + 3, &kernel[range_size(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), None);
        let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
        writeln!(&mut outbuf, "{tail:<16} phys_text {:>#8x}, virt {:>#7x}, size_text {:>#8x}, phys_data {:#x}, size_data {:>#7x}, entry {:#x},\n                 UUID {uuid}",
            app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
        off += sepappsize;
        i += 1;
    }
    let max = (n_apps + n_shlibs) as usize;
    while i < max {
        app = cast_struct_args!(SEPApp64, &kernel[off..], (ver, ));
        tail = strslice(&app.app_name);
        let data_buf = &kernel[range_size(app.phys_data as usize, app.size_data as usize)].to_owned();
        restore_file(i + 3, &kernel[range_size(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), Some(app.size_text as usize));
        let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
        writeln!(&mut outbuf, "{tail:<16} phys_text {:>#8x}, virt {:>#7x}, size_text {:>#8x}, phys_data {:#x}, size_data {:>#7x}, entry {:#x},\n                 UUID {uuid}",
            app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
        off += sepappsize;
        i += 1;
    }
    outbuf.flush()
}

//splits the SEP apps from the 32-bit SEP Firmware by reading the structs
fn split32(kernel: &[u8], outdir: &Path, mut sep_info: SEPinfo, mut outbuf: BufWriter<Box<dyn Write>>) -> Result<(), std::io::Error> {
    writeln!(&mut outbuf, "detected 32 bit SEP")?;

    //index 0: boot
    let mut bootout = outdir.join("sepdump00_boot");
    filewrite(&bootout, &kernel[..0x1000]); 
    writeln!(&mut outbuf, "boot         size 0x1000")?;

    //index 1: kernel
    let mut st = 0x1000;
    let mut sz = calc_size(&kernel[st..]); //most SEP fws
    
    if sz == 0 {
        if kernel[range_size(st, 4)] == [0; 4] {
            //J97 SEP Firmware
            st = 0x4000;
            sz = calc_size(&kernel[st..]); 
            restore_file(1, &kernel[range_size(st, sz)], outdir, "kernel", None, None);
        } else {
            //N71 SEP or newer SEP Firmware
            bootout = outdir.join("sepdump01_kernel");
            filewrite(&bootout, &kernel[range_size(st, 0xe000)]);
            sz = 0xe000;
        }
    } else {
        restore_file(1, &kernel[range_size(st, sz)], outdir, "kernel", None, None);
    }

    writeln!(&mut outbuf, "kernel       size {sz:#x}")?;

    //check for newer SEP
    let tmp = cast_struct!(SEPAppOld, &kernel[sep_info.sep_app_pos..]);
    if tmp.size == 0 {
        //64 bit SEP struct in 32 bit SEP

        //number of apps must be valid in this case
        let n_apps = sep_info.sepapps.unwrap();
        let shlib = sep_info.shlibs.unwrap_or(0);

        let mut app = cast_struct_args!(SEPApp64, &kernel[sep_info.sep_app_pos..], (if shlib == 0 { 0 } else { 4 }, ));
        let sepappsize = SEPAPP_64_SIZE + match app.srcver.get_major() {
            2100.. => 36,
            1700.. => 4,
            _ => 0
        };
        let mut tail;

        //dump struct from start of kernel
        bootout = outdir.join("sepdump-struct.extra");
        filewrite(&bootout, &kernel[range_size(app.phys_text as usize, 0x1000)]);
        writeln!(&mut outbuf, "struct       size 0x1000")?;
        app.phys_text += 0x1000;
        app.size_text -= 0x1000;

        let mut i = 2;
        while i < n_apps {
            if i != 2 {
                app = cast_struct_args!(SEPApp64, &kernel[sep_info.sep_app_pos..], (if shlib == 0 { 0 } else { 4 }, ));
            }
            tail = strslice(&app.app_name);
            let data_buf = &kernel[range_size(app.phys_data as usize, app.size_data as usize)].to_owned();
            restore_file(i, &kernel[range_size(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), None);
            let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
            writeln!(&mut outbuf, "{tail:-12} phys_text {:#08x}, virt {:#06x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x},\n             UUID {uuid}",
                app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
            sep_info.sep_app_pos += sepappsize;
            i += 1;
        }

        if shlib != 0 {
            let max = n_apps + shlib + 2;
            while i < max {
                app = cast_struct_args!(SEPApp64, &kernel[sep_info.sep_app_pos..], (4, ));
                tail = strslice(&app.app_name);
                let data_buf = &kernel[range_size(app.phys_data as usize, app.size_data as usize)].to_owned();
                restore_file(i, &kernel[range_size(app.phys_text as usize, (app.size_text + app.size_data) as usize)], outdir, tail, Some(data_buf), Some(app.size_text as usize));
                let uuid = Uuid::from_bytes_le(app.app_uuid).hyphenated().to_string();
                writeln!(&mut outbuf, "{tail:-12} phys_text {:#08x}, virt {:#06x}, size_text {:#08x}, phys_data {:#x}, size_data {:#07x}, entry {:#x},\n             UUID {uuid}",
                    app.phys_text, app.virt, app.size_text, app.phys_data, app.size_data, app.ventry)?;
                sep_info.sep_app_pos += sepappsize;
                i += 1;
            }
        }
    } else { //older SEP
        /*
            preparation for loop, find offset of "SEPOS" string and 
            calculate size of structs based off "SEPD" string and previous string
        */
        let tailoff = memmem::find(&kernel[sep_info.sep_app_pos..], b"SEPOS       ").unwrap_or_else(|| panic!("Could not find SEPOS string")); //offset of the name in the struct
        sep_info.sepapp_size = memmem::find(&kernel[range_size(sep_info.sep_app_pos+tailoff, 128)], b"SEPD").unwrap_or_else(|| panic!("Could not find SEPD string")); 

        for index in 2.. {
            let (tail, mut apps);
            assert!(sep_info.sep_app_pos != 0, "SEPApp position is 0!");
            apps = cast_struct!(SEPAppOld, &kernel[sep_info.sep_app_pos..]);
            if apps.phys == 0 { //end of structs, nothing else to do
                return outbuf.flush() 
            } else if index == 2 { //need SEPOS kernel's offset to dump structs
                bootout = outdir.join("sepdump-extra_struct");
                filewrite(&bootout, &kernel[range_size(apps.phys as usize, 0x1000)]); 
                writeln!(&mut outbuf, "struct       size 0x1000")?;
                apps.phys += 0x1000;
                apps.size -= 0x1000;
            }
            tail = strslice(&kernel[range_size(sep_info.sep_app_pos + tailoff, 12)]);
            let uuid = Uuid::from_bytes_le(kernel[range_size(sep_info.sep_app_pos + tailoff + 12, 16)].try_into().unwrap()).hyphenated().to_string();
            writeln!(&mut outbuf, "{tail:-12} phys {:#08x}, virt {:#x}, size {:#08x}, entry {:#x},\n             UUID {uuid}", 
                      apps.phys,  apps.virt,  apps.size,  apps.entry)?;
            sep_info.sep_app_pos += sep_info.sepapp_size;
            restore_file(index, &kernel[range_size(apps.phys as usize, apps.size as usize)], outdir, tail, None, None);
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
        sep_app_pos: monitorstr.args_off as usize + KRNLBOOTARGS_SIZE, 
        sepapp_size: SEPAPP_SIZE.to_owned(),
        sepapps: krnlbastr.num_apps.lt(&0xFF).then_some(krnlbastr.num_apps as usize),
        shlibs: krnlbastr.num_shlibs.ne(&0).then_some(krnlbastr.num_shlibs as usize),
    }
}

//find the offset of the SEP HDR struct for 64-bit
fn find_off(krnl: &[u8]) -> (u64, u8) { 
    if &krnl[range_size(0x1004, 16)] == b"Built by legion2" { 
        //iOS 15 and below
        let hdr = cast_struct!(Legion64Old, &krnl[0x1000..]);
        (u64::from(hdr.structoff), hdr.subversion as u8)
    } else if &krnl[range_size(0x103c, 16)] == b"Built by legion2" {
        //iOS 16
        let hdr16 = cast_struct!(Legion64, &krnl[0x1000..]);
        let uuid = Uuid::from_bytes_le(hdr16.uuid).hyphenated().to_string();
        println!("HDR UUID: {uuid}");
        (u64::from(hdr16.structoff), hdr16.subversion as u8)
    } else if &krnl[range_size(0x408, 16)] == b"Built by legion2" {
        let hdr = cast_struct!(Legion32, &krnl[0x400..]);
        (u64::from(hdr.off), hdr.subversion as u8)
    } else {
        eprintln!("[!] Invalid or unknown kernel inputted, exiting.");
        process::exit(1)
    }
}

//test that the kernel is valid, find_off will verify other cases
fn test_krnl(krnl: &[u8]) -> Option<Vec<u8>> {
    if krnl[..2] == [0x30, 0x83] {
        eprintln!("[!] IMG4 Header detected, please extract (and decrypt) the SEP firmware first. Exiting.");
        process::exit(1)
    } else if &krnl[8..16] == b"eGirBwRD" { //LZVN compression, "DRawBridGe"
        use bindings::lzvn_decode;
        let start = if krnl[range_size(0x10000, 4)] == [0,0,0,0] { 0x20000 } else { 0x10000 };
        let startptr: *const c_void = krnl[start..].as_ptr().cast();
        let startlen = krnl.len() - start;

        let mut destlen: usize = u32::from_le_bytes(
            krnl[range_size(0x18, 4)].try_into().unwrap() //infallable, taking slice of 4 bytes ad converting into array wih len 4
        ).try_into().unwrap();
        let mut destbuf: Vec<u8> = vec![0; destlen as usize];
        let destptr: *mut c_void = destbuf.as_mut_ptr().cast();

        loop {
            let complen = unsafe { 
                lzvn_decode(destptr, destlen, startptr, startlen) 
            };
            assert_ne!(complen, 0, "Decompression failed (truncated input?)");
            if complen == destlen { break; } 
            else if complen < destlen {
                destbuf.truncate(complen as usize);
                break;
            }
            destlen *= 2; //the SEP firmware may have lied to us about the decompressed size
            destbuf.resize(destlen as usize, 0);
        }
        return Some(destbuf);
    }
    None
}

/// The main logic of the program.
/// # Arguments
/// * `filein` - The input file to read from
/// * `outdir` - The output directory to write to
/// * `verbose` - The verbosity level (0 for no output, 1 for normal output)
/// # Errors
/// * Input file errors (permissions, not found, etc.)
/// * Errors while writing to the output directory
/// * Errors while writing to stdout
pub fn sepsplit(filein: &str, outdir: &Path, verbose: usize) -> Result<(), std::io::Error> {
    let mut krnl: Vec<u8> = fs::read(filein)?;
    if let Some(newkrnl) = test_krnl(&krnl) {
        krnl = newkrnl;
    }
    let (hdr_offset, ver) = find_off(&krnl);
    
    
    //fast stdout
    let stdout = std::io::stdout();
    let outlock = stdout.lock();
    let outbuf: BufWriter<Box<dyn Write>> = std::io::BufWriter::new(
        if verbose == 1 {
            Box::new(outlock)
        } else {
            Box::new(std::io::sink())
        }
    );

    if ver == 1 { //32-bit SEP
        let septype = sep32_structs(&krnl);
        split32(&krnl, outdir, septype, outbuf)
    } else { //64-bit SEP
        split64(hdr_offset as usize, &krnl, outdir, outbuf, ver)
    }
}

use core::ffi::{c_char, CStr};

/// Calls the main logic of the program with FFI.
/// # Arguments
/// * `filein` - the path to the extracted SEP firmware
/// * `outdir` - the path to the output directory
/// * `verbose` - the verbosity level (0 for no output, 1 for normal output)
/// # Returns
/// * 0 on success
/// * 1 on failure
/// # Safety
/// * `filein` must be a null terminated char array with valid UTF-8 characters and also be a path to a file
/// * `outdir` must be a null terminated char array with valid UTF-8 characters and also be a path to a already existing directory
#[no_mangle]
pub unsafe extern "C" fn split(filein: *const c_char, outdir: *const c_char, verbose: usize) -> isize {
    let Ok(filein) = unsafe { CStr::from_ptr(filein) }.to_str() else { return 1 };
    let Ok(outdir) = unsafe { CStr::from_ptr(outdir) }.to_str() else { return 1 };
    let outdir = Path::new(outdir);
    match sepsplit(filein, outdir, verbose) {
        Ok(_) => 0,
        Err(_) => 1
    }
}