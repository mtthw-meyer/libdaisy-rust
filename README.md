# [Documentation](https://docs.rs/libdaisy)

# libdaisy
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
# A Flashing Utility
* [Electro-smith web programmer](https://electro-smith.github.io/Programmer/)

OR

* [dfu-util](http://dfu-util.sourceforge.net/)

OR

* [Probe.rs](https://probe.rs/)

This requires a debug probe of some sort (e.g. ST link) and allows for fast debugging messages via RTT.

cargo embed --features log-rtt --example passthru

## Build Examples
cargo objcopy --example blinky --release -- -O binary blinky.bin

cargo objcopy --example passthru --release -- -O binary passthru.bin

[cargo-binutils-url]: https://github.com/rust-embedded/cargo-binutils

# Minimum supported Rust version
The Minimum Supported Rust Version (MSRV) at the moment is 1.68.2
# Demos

[Looper](https://github.com/mtthw-meyer/daisy-looper) - Basic one button looper.
