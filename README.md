# sepsplit-rs
SEP firmware splitter, made in rust.

# Building:
First, install `cargo` if you haven't already, instructions are [here](https://doc.rust-lang.org/cargo/getting-started/installation.html).
Then, run `cargo build --release`.
Afterwards, run the executable with: `target/release/sepsplit-rs`

# Usage:
`sepsplit-rs /path/to/sep-firmware.bin [output folder]`
The SEP firmware has to be decrypted, extracted, and decompressed.

# Credits:
- xerub for the original sepsplit
- matteyeux for helping me test this program
- mrmacete for the 64-bit version