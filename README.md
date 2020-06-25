# libdaisy-rust
Hardware Abstraction Layer implementation for Daisy boards.

## Requirements
[cargo-binutils][cargo-binutils-url]
``` console
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview
```

## Build Examples
cargo objcopy --example blinky --release -- -O binary blinky.bin
cargo objcopy --example button --release -- -O binary button.bin

[cargo-binutils-url]: https://github.com/rust-embedded/cargo-binutils

## TODO
* Audio I/O
* Probe.rs integration
* ?
