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
    error::Error,
    path::Path,
    fs
};
use test_case::test_case;

//add as many test_case macros as is sepfws in testfws, must be named "sepfw.name.bin"
#[test_case("D10.18A373")]
#[test_case("D10.19A346")]
#[test_case("D11.14A403")]
#[test_case("D11.15A372")]
#[test_case("D21.18E199")]
#[test_case("D28.21A5248v")]
#[test_case("J72b.20A5303i")]
#[test_case("J97.17A844")]
#[test_case("N131b.19R5559e")]
#[test_case("N142b.18R5552f")]
#[test_case("N61.16G192")]
#[test_case("N71m.19F77")]
fn test_fws(fname: &str) -> Result<(), Box<dyn Error>> {
    use assert_cmd::prelude::*;
    use std::process::Command;

    let testfwp = &Path::new(env!("CARGO_MANIFEST_DIR")).join("testfws");
    
    Command::cargo_bin("sepsplit-rs")?
        .arg(testfwp.join(format!("sepfw.{fname}.bin")))
        .arg(testfwp.join(format!("testout-{fname}/")))
        .assert()
        .success();

    assert!(testfwp.join(format!("testout-{fname}/")).exists());
        
    fs::remove_dir_all(testfwp.join(format!("testout-{fname}/")))?; //cleanup

    Ok(())
}