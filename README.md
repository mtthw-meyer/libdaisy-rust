# libdaisy-rust
Hardware Abstraction Layer implementation for Daisy boards.

## Requirements
* Hardware target
```
$ rustup target add thumbv7em-none-eabihf
```

* [cargo-binutils][cargo-binutils-url]
``` console
$ cargo install cargo-binutils

$ rustup component add llvm-tools-preview
```
Some flashing utility such as
* [Electro-smith web programmer](https://electro-smith.github.io/Programmer/)

OR

* [dfu-util](http://dfu-util.sourceforge.net/)

## Optional
* Other flashing tools such as [Probe.rs](https://probe.rs/)

## Build Examples
cargo objcopy --example blinky --release -- -O binary blinky.bin

cargo objcopy --example toggle --release -- -O binary toggle.bin

cargo objcopy --example passthru --release -- -O binary passthru.bin

[cargo-binutils-url]: https://github.com/rust-embedded/cargo-binutils
