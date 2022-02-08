use crate::delay_ms;

use stm32h7xx_hal::{
    gpio::{gpiob, Analog},
    hal::digital::v2::OutputPin,
};

pub fn init(codec_gpio: gpiob::PB11<Analog>) {
    // Reset the AK4556 codec chip
    let mut codec_gpio = codec_gpio.into_push_pull_output();
    codec_gpio.set_low().unwrap();
    delay_ms(5);
    codec_gpio.set_high().unwrap();
}
