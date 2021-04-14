use stm32h7xx_hal::{
    gpio::Analog,
    prelude::*,
    rcc,
    serial::{config::InvalidConfig, Serial, SerialExt},
    stm32::USART1,
};

/// Get a configured serial MIDI device
pub fn midi(
    daisy13: crate::gpio::Daisy13<Analog>,
    daisy14: crate::gpio::Daisy14<Analog>,
    usart1_d: USART1,
    usart1_p: rcc::rec::Usart1,
    clocks: &rcc::CoreClocks,
) -> Result<Serial<stm32h7xx_hal::stm32::USART1>, InvalidConfig> {
    let (tx, rx) = (daisy13.into_alternate_af7(), daisy14.into_alternate_af7());
    usart1_d.serial(
        (tx, rx),
        stm32h7xx_hal::serial::config::Config::default()
            .baudrate(31_250.bps())
            .parity_none(),
        usart1_p,
        &clocks,
    )
}
