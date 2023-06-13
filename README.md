# sepsplit-rs
A utility to split a SEP firmware into its various modules, made in Rust.

## Building
1. Install `cargo` if you haven't already, instructions are [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)
2. Run `cargo install --path /path/to/sepsplit-rs/`
3. Run the executable with `sepsplit-rs`

### Note for Windows
In order to get the program to compile, you may need to 
* Install LLVM as shown [here](https://rust-lang.github.io/rust-bindgen/requirements.html#windows)
* Either comment out or modify line 52 in `lzvn_decode.c` in the lzvn repo to `#define _LZVN_DEBUG_DUMP(...)` if you are using the MSVC compiler.

## Usage
### As a binary
`sepsplit-rs /path/to/sep-firmware.bin [output folder]`<br />
The SEP firmware has to be decrypted and extracted.

### As a library
1. Use `./src/seplib.h` as the header for importing the function. <br />
2. Compile a static library with `cargo rustc --lib --crate-type staticlib`.<br />
3. Finally, run the main logic of the program with `split(const char* filein, const char* outdir, unsigned int verbose)`, replacing the parameters with arguments with the necessary safety requirements listed in the header.

## Testing
1. `cd` into the project
2. Run `./download_testfws.sh` to download test SEP Firmwares
3. Run the tests with `cargo test`

## Credits
- xerub for the [original sepsplit](https://gist.github.com/xerub/0161aacd7258d31c6a27584f90fa2e8c) and the [fork of LZVN](https://github.com/xerub/LZVN)
- matteyeux for helping me test this program
- mrmacete for the [64-bit version](https://github.com/matteyeux/sepsplit/commit/abea72789e82f07d73fe4892cf96b4b8b44802dc)
