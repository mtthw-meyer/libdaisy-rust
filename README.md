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

## TODO
* DMA - Get audio data via DMA instead of SAI FIFO. See [Issue 80](https://github.com/stm32-rs/stm32h7xx-hal/issues/80).
* SDRAM - The SDRAM needs to be brought online using [stm32h7-fmc](https://crates.io/crates/stm32h7-fmc).
* MPU - The memory protection unit needs to be configured.
* dcache - Needs to be enabled.
* QSPI - Configur QSPI flash memory.
