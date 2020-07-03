// use embedded_hal;
pub use stm32h7xx_hal::hal::digital::v2::InputPin;
use stm32h7xx_hal::gpio;
pub use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};

pub enum Never {}

pub trait Switch<Mode: InputPin> {
    fn test(&self);
}

// impl <Mode> for dyn InputPin<Error = Never> {
//     fn test(&self) {

//     }
// }
