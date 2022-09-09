# sepsplit-rs
SEP firmware splitter, made in Rust.

# Building:
First, install `cargo` if you haven't already, instructions are [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).
Then, run `cargo install`.
Afterwards, run the executable with: `sepsplit-rs`

# Usage:
`sepsplit-rs /path/to/sep-firmware.bin [output folder]`
The SEP firmware has to be decrypted and extracted.

# Credits:
- xerub for the original sepsplit and the fork of LZVN
- matteyeux for helping me test this program
- mrmacete for the 64-bit version