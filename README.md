# sepsplit-rs
A utility to split a SEP firmware into it's various modules, made in Rust.

## Building
1. Install `cargo` if you haven't already, instructions are [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. Run `cargo install --path /path/to/sepsplit-rs/`
3. Run the executable with `sepsplit-rs`

## Usage
`sepsplit-rs /path/to/sep-firmware.bin [output folder]`<br />
The SEP firmware has to be decrypted and extracted.

## Testing
1. `cd` into the project
2. Run `./download_testfws.sh` to download test SEP Firmwares
3. Run the tests with `cargo test`

## Credits
- xerub for the [original sepsplit](https://gist.github.com/xerub/0161aacd7258d31c6a27584f90fa2e8c) and the [fork of LZVN](https://github.com/xerub/LZVN)
- matteyeux for helping me test this program
- mrmacete for the [64-bit version](https://github.com/matteyeux/sepsplit/commit/abea72789e82f07d73fe4892cf96b4b8b44802dc)