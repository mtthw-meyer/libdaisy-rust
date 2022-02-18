use stm32h7xx_hal::{gpio::{gpiod, Analog}, hal::digital::v2::InputPin};

pub enum BoardVersion {
    /** Daisy Seed Rev4
     *  This is the original Daisy Seed */
    DaisySeed,
    /** Daisy Seed 1.1 (aka Daisy Seed Rev5)
     *  This is a pin-compatible version of the Daisy Seed
     *  that uses the WM8731 codec instead of the AK4430 */
    DaisySeed1_1,
}

/**
 * Fall through is Daisy Seed v1 (aka Daisy Seed rev4)
 * PD3 tied to gnd is Daisy Seed v1.1 (aka Daisy Seed rev5)
 * PD4 tied to gnd reserved for future hardware
 */
pub fn check_board_version(pin: gpiod::PD3<Analog>) -> BoardVersion {
    let pin = pin.into_pull_up_input();
    if pin.is_low().unwrap() {
        BoardVersion::DaisySeed1_1
    } else {
        BoardVersion::DaisySeed
    }
}
