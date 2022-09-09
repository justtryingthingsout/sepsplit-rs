#![feature(stmt_expr_attributes)]
#![feature(register_tool)]
#![register_tool(c2rust)]
#![feature(type_ascription)]
#![feature(core_intrinsics)]

mod sep;
mod utils;
mod lzvn;

use std::{
    fs::{self},
    path::{PathBuf},
    env,
    process
};

use sep::*;

fn main() -> Result<(), std::io::Error> {
    //why I don't use a crate for parsing arguments? idk, I'm more used to C
    let argv: Vec<String> = std::env::args().collect();
    let argc = argv.len();

    if argc < 2 {
        eprintln!("[!] Not enough arguments\n\
                   sepsplit-rs - tool to split SEPOS firmware into its individual modules, by @plzdonthaxme\n\
                   Usage: {prog} <SEPOS.bin> [output folder]", prog=&argv[0]);
        process::exit(1)
    }

    let mut krnl: Vec<u8> = fs::read(&argv[1]).unwrap_or_else(|e| panic!("[-] Cannot read kernel, err: {e}"));
    if let Some(newkrnl) = test_krnl(&krnl) {
        krnl = newkrnl;
    }
    let outdir = &if argc > 2 {
        PathBuf::from(&argv[2])
    } else {
        env::current_dir().unwrap_or_else(|e| panic!("Cannot get current dir: {e}")) //if output dir is specified, use it
    };
    fs::create_dir_all(outdir).unwrap_or_else(|e| panic!("Failed to create folder(s): {e}"));
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