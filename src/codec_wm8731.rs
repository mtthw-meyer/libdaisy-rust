use stm32h7xx_hal::{device::I2C2, i2c::I2c};
use wm8731_alt::{self, command::sampling::Mclk12M288, prelude::*, Wm8731};

use crate::delay_ms;

const W8731_ADDR: u8 = 0x1A;

pub fn init(i2c: I2c<I2C2>) {
    // Instantiate the driver with an i2c interface.
    // It also resets the codec.
    let interface = I2CInterface::new(i2c, W8731_ADDR);
    let mut wm8731 = Wm8731::new(interface);

    // Set Line Inputs to 0DB
    let cmd = left_line_in().invol().db(InVoldB::P0DB).into_command();
    wm8731.send(cmd);
    delay_ms(10);

    let cmd = right_line_in().invol().db(InVoldB::P0DB).into_command();
    wm8731.send(cmd);
    delay_ms(10);

    // Set Headphones To Mute.
    let cmd = left_headphone_out()
        .hpvol()
        .db(HpVoldB::MUTE)
        .into_command();
    wm8731.send(cmd);
    delay_ms(10);

    let cmd = right_headphone_out()
        .hpvol()
        .db(HpVoldB::MUTE)
        .into_command();
    wm8731.send(cmd);
    delay_ms(10);

    // Analog and Digital Routing.
    let cmd = analogue_audio_path()
        .mutemic()
        .set_bit()
        .insel()
        .set_bit()
        .dacsel()
        .set_bit()
        .into_command();
    wm8731.send(cmd);
    delay_ms(10);

    // Configure power management.
    let cmd = power_down()
        .micpd()
        .set_bit()
        .clkoutpd()
        .set_bit()
        .oscpd()
        .set_bit()
        .into_command();

    wm8731.send(cmd);
    delay_ms(10);

    // Digital Format.
    let cmd = digital_audio_interface()
        .format()
        .left_justified()
        .iwl()
        .iwl_24_bits()
        .ms()
        .slave()
        .into_command();
    wm8731.send(cmd);
    delay_ms(10);

    // Sample rate.
    // For a 12.288 MHz master clock.
    let cmd = sampling_with_mclk(Mclk12M288)
        .sample_rate()
        .adc48k_dac48k()
        .into_command();
    wm8731.send(cmd);
    delay_ms(10);

    // Enable.
    let cmd = active_control().inactive().into_command();
    wm8731.send(cmd);
    delay_ms(10);

    let cmd = active_control().active().into_command();
    wm8731.send(cmd);
    delay_ms(10);
}
