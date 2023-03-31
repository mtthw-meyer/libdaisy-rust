//! GPIO module. Contains pins by Daisy names.
//! Provides access to the Seed LED and codec reset.
use stm32h7xx_hal::gpio;
use stm32h7xx_hal::gpio::gpioc::PC7;

// use stm32h7xx_hal::hal::digital::v2::InputPin;
#[allow(unused_imports)]
use stm32h7xx_hal::gpio::{Alternate, Analog, Input, Output, Pull, PushPull};

pub use gpio::gpioa::PA0 as Daisy25;
pub use gpio::gpioa::PA1 as Daisy24;
pub use gpio::gpioa::PA2 as Daisy28;
pub use gpio::gpioa::PA3 as Daisy16;
pub use gpio::gpioa::PA4 as Daisy23;
pub use gpio::gpioa::PA5 as Daisy22;
pub use gpio::gpioa::PA6 as Daisy19;
pub use gpio::gpioa::PA7 as Daisy18;
pub use gpio::gpiob::PB1 as Daisy17;
pub use gpio::gpiob::PB12 as Daisy0;
pub use gpio::gpiob::PB14 as Daisy29;
pub use gpio::gpiob::PB15 as Daisy30;
pub use gpio::gpiob::PB4 as Daisy9;
pub use gpio::gpiob::PB5 as Daisy10;
pub use gpio::gpiob::PB6 as Daisy13;
pub use gpio::gpiob::PB7 as Daisy14;
pub use gpio::gpiob::PB8 as Daisy11;
pub use gpio::gpiob::PB9 as Daisy12;
pub use gpio::gpioc::PC0 as Daisy15;
pub use gpio::gpioc::PC1 as Daisy20;
pub use gpio::gpioc::PC10 as Daisy2;
pub use gpio::gpioc::PC11 as Daisy1;
pub use gpio::gpioc::PC12 as Daisy6;
pub use gpio::gpioc::PC4 as Daisy21;
pub use gpio::gpioc::PC8 as Daisy4;
pub use gpio::gpioc::PC9 as Daisy3;
pub use gpio::gpiod::PD11 as Daisy26;
pub use gpio::gpiod::PD2 as Daisy5;
pub use gpio::gpiog::PG10 as Daisy7;
pub use gpio::gpiog::PG11 as Daisy8;
pub use gpio::gpiog::PG9 as Daisy27;

pub type SeedLed = PC7<Output<PushPull>>;

use crate::*;

/// GPIO struct for holding Daisy GPIO pins
#[allow(clippy::upper_case_acronyms)]
pub struct GPIO {
    pub led: SeedLed,
    codec: gpio::gpiob::PB11<Output<PushPull>>,
    pub daisy0: Option<gpio::gpiob::PB12<Analog>>,
    pub daisy1: Option<gpio::gpioc::PC11<Analog>>,
    pub daisy2: Option<gpio::gpioc::PC10<Analog>>,
    pub daisy3: Option<gpio::gpioc::PC9<Analog>>,
    pub daisy4: Option<gpio::gpioc::PC8<Analog>>,
    pub daisy5: Option<gpio::gpiod::PD2<Analog>>,
    pub daisy6: Option<gpio::gpioc::PC12<Analog>>,
    pub daisy7: Option<gpio::gpiog::PG10<Analog>>,
    pub daisy8: Option<gpio::gpiog::PG11<Analog>>,
    pub daisy9: Option<gpio::gpiob::PB4<Alternate<0>>>,
    pub daisy10: Option<gpio::gpiob::PB5<Analog>>,
    pub daisy11: Option<gpio::gpiob::PB8<Analog>>,
    pub daisy12: Option<gpio::gpiob::PB9<Analog>>,
    pub daisy13: Option<gpio::gpiob::PB6<Analog>>,
    pub daisy14: Option<gpio::gpiob::PB7<Analog>>,
    pub daisy15: Option<gpio::gpioc::PC0<Analog>>,
    pub daisy16: Option<gpio::gpioa::PA3<Analog>>,
    pub daisy17: Option<gpio::gpiob::PB1<Analog>>,
    pub daisy18: Option<gpio::gpioa::PA7<Analog>>,
    pub daisy19: Option<gpio::gpioa::PA6<Analog>>,
    pub daisy20: Option<gpio::gpioc::PC1<Analog>>,
    pub daisy21: Option<gpio::gpioc::PC4<Analog>>,
    pub daisy22: Option<gpio::gpioa::PA5<Analog>>,
    pub daisy23: Option<gpio::gpioa::PA4<Analog>>,
    pub daisy24: Option<gpio::gpioa::PA1<Analog>>,
    pub daisy25: Option<gpio::gpioa::PA0<Analog>>,
    pub daisy26: Option<gpio::gpiod::PD11<Analog>>,
    pub daisy27: Option<gpio::gpiog::PG9<Analog>>,
    pub daisy28: Option<gpio::gpioa::PA2<Analog>>,
    pub daisy29: Option<gpio::gpiob::PB14<Analog>>,
    pub daisy30: Option<gpio::gpiob::PB15<Analog>>,
}

impl GPIO {
    /// Initialize the GPIOs
    pub fn init(
        seed_led: gpio::gpioc::PC7<Analog>,
        codec: gpio::gpiob::PB11<Analog>,
        daisy0: Option<gpio::gpiob::PB12<Analog>>,
        daisy1: Option<gpio::gpioc::PC11<Analog>>,
        daisy2: Option<gpio::gpioc::PC10<Analog>>,
        daisy3: Option<gpio::gpioc::PC9<Analog>>,
        daisy4: Option<gpio::gpioc::PC8<Analog>>,
        daisy5: Option<gpio::gpiod::PD2<Analog>>,
        daisy6: Option<gpio::gpioc::PC12<Analog>>,
        daisy7: Option<gpio::gpiog::PG10<Analog>>,
        daisy8: Option<gpio::gpiog::PG11<Analog>>,
        daisy9: Option<gpio::gpiob::PB4<Alternate<0>>>,
        daisy10: Option<gpio::gpiob::PB5<Analog>>,
        daisy11: Option<gpio::gpiob::PB8<Analog>>,
        daisy12: Option<gpio::gpiob::PB9<Analog>>,
        daisy13: Option<gpio::gpiob::PB6<Analog>>,
        daisy14: Option<gpio::gpiob::PB7<Analog>>,
        daisy15: Option<gpio::gpioc::PC0<Analog>>,
        daisy16: Option<gpio::gpioa::PA3<Analog>>,
        daisy17: Option<gpio::gpiob::PB1<Analog>>,
        daisy18: Option<gpio::gpioa::PA7<Analog>>,
        daisy19: Option<gpio::gpioa::PA6<Analog>>,
        daisy20: Option<gpio::gpioc::PC1<Analog>>,
        daisy21: Option<gpio::gpioc::PC4<Analog>>,
        daisy22: Option<gpio::gpioa::PA5<Analog>>,
        daisy23: Option<gpio::gpioa::PA4<Analog>>,
        daisy24: Option<gpio::gpioa::PA1<Analog>>,
        daisy25: Option<gpio::gpioa::PA0<Analog>>,
        daisy26: Option<gpio::gpiod::PD11<Analog>>,
        daisy27: Option<gpio::gpiog::PG9<Analog>>,
        daisy28: Option<gpio::gpioa::PA2<Analog>>,
        daisy29: Option<gpio::gpiob::PB14<Analog>>,
        daisy30: Option<gpio::gpiob::PB15<Analog>>,
    ) -> GPIO {
        let led = seed_led.into_push_pull_output();
        let codec = codec.into_push_pull_output();
        let mut gpio = Self {
            led,
            codec,
            daisy0,
            daisy1,
            daisy2,
            daisy3,
            daisy4,
            daisy5,
            daisy6,
            daisy7,
            daisy8,
            daisy9,
            daisy10,
            daisy11,
            daisy12,
            daisy13,
            daisy14,
            daisy15,
            daisy16,
            daisy17,
            daisy18,
            daisy19,
            daisy20,
            daisy21,
            daisy22,
            daisy23,
            daisy24,
            daisy25,
            daisy26,
            daisy27,
            daisy28,
            daisy29,
            daisy30,
        };
        gpio.reset_codec();
        gpio
    }

    /// Reset the AK4556 codec chip
    pub fn reset_codec(&mut self) {
        self.codec.set_low();
        delay_ms(5);
        self.codec.set_high();
    }
}
