use super::*;
use std::error::Error;
use test_case::test_case;

//add as many test_case macros as is sepfws in testfws, must be named "sepfw.name.bin"
#[test_case("D10.18A373")]
#[test_case("D10.19A346")]
#[test_case("D11.14A403")]
#[test_case("D11.15A372")]
#[test_case("D21.18E199")]
#[test_case("J72b.20A5303i")]
#[test_case("J97.17A844")]
#[test_case("N131b.19R5559e")]
#[test_case("N142b.18R5552f")]
#[test_case("N61.16G192")]
#[test_case("N71m.19F77")]
fn test_fws(fname: &str) -> Result<(), Box<dyn Error>> {
    use assert_cmd::prelude::*;
    use std::process::Command;

    let ref testfwp = Path::new(env!("CARGO_MANIFEST_DIR")).join("testfws");

    Command::cargo_bin("sepsplit-rs")?
        .arg(testfwp.join(format!("sepfw.{fname}.bin")))
        .arg(testfwp.join(format!("testout-{fname}/")))
        .assert()
        .success();
        
    fs::remove_dir_all(testfwp.join(format!("testout-{fname}/")))?; //cleanup

    Ok(())
}