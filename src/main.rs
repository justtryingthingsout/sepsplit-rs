/*
    sepsplit-rs - A tool to split SEPOS firmware into its individual modules
    Copyright (C) 2024 plzdonthaxme

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::{
    fs, 
    path::PathBuf, 
    env, 
    process, 
};
use sepsplit_rs::sepsplit;

#[cfg(test)]
mod tests;

fn main() -> Result<(), std::io::Error> {
    //why I don't use a crate for parsing arguments? idk, I'm more used to C
    let argv: Vec<String> = std::env::args().collect();
    let arglen = argv.len();

    if arglen < 2 {
        eprintln!("[!] Not enough arguments\n\
                   sepsplit-rs - tool to split SEPOS firmware into its individual modules, by @plzdonthaxme\n\
                   Usage: {prog} <SEPOS.bin> [output folder]", prog=&argv[0]);
        process::exit(1)
    }

    let outdir = &if arglen > 2 {
        PathBuf::from(&argv[2])
    } else {
        env::current_dir().unwrap_or_else(|e| panic!("Cannot get current dir: {e}")) //if output dir is specified, use it
    };
    fs::create_dir_all(outdir)?;
    sepsplit(&argv[1], outdir, 1)
}