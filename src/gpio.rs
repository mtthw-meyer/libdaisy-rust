use stm32h7xx_hal::gpio;
use stm32h7xx_hal::gpio::gpioc::PC7;
pub use stm32h7xx_hal::gpio::{Alternate, Analog, Input, Output, PullUp, PushPull};
use stm32h7xx_hal::stm32::Interrupt;

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

pub struct GPIO {
    pub led: SeedLed,
    pub daisy0: gpio::gpiob::PB12<Analog>,
    pub daisy1: gpio::gpioc::PC11<Analog>,
    pub daisy2: gpio::gpioc::PC10<Analog>,
    pub daisy3: gpio::gpioc::PC9<Analog>,
    pub daisy4: gpio::gpioc::PC8<Analog>,
    pub daisy5: gpio::gpiod::PD2<Analog>,
    pub daisy6: gpio::gpioc::PC12<Analog>,
    pub daisy7: gpio::gpiog::PG10<Analog>,
    pub daisy8: gpio::gpiog::PG11<Analog>,
    pub daisy9: gpio::gpiob::PB4<Alternate<gpio::AF0>>,
    pub daisy10: gpio::gpiob::PB5<Analog>,
    pub daisy11: gpio::gpiob::PB8<Analog>,
    pub daisy12: gpio::gpiob::PB9<Analog>,
    pub daisy13: gpio::gpiob::PB6<Analog>,
    pub daisy14: gpio::gpiob::PB7<Analog>,
    pub daisy15: gpio::gpioc::PC0<Analog>,
    pub daisy16: gpio::gpioa::PA3<Analog>,
    pub daisy17: gpio::gpiob::PB1<Analog>,
    pub daisy18: gpio::gpioa::PA7<Analog>,
    pub daisy19: gpio::gpioa::PA6<Analog>,
    pub daisy20: gpio::gpioc::PC1<Analog>,
    pub daisy21: gpio::gpioc::PC4<Analog>,
    pub daisy22: gpio::gpioa::PA5<Analog>,
    pub daisy23: gpio::gpioa::PA4<Analog>,
    pub daisy24: gpio::gpioa::PA1<Analog>,
    pub daisy25: gpio::gpioa::PA0<Analog>,
    pub daisy26: gpio::gpiod::PD11<Analog>,
    pub daisy27: gpio::gpiog::PG9<Analog>,
    pub daisy28: gpio::gpioa::PA2<Analog>,
    pub daisy29: gpio::gpiob::PB14<Analog>,
    pub daisy30: gpio::gpiob::PB15<Analog>,
}

impl GPIO {
    pub fn init(
        gpioa: gpio::gpioa::Parts,
        gpiob: gpio::gpiob::Parts,
        gpioc: gpio::gpioc::Parts,
        gpiod: gpio::gpiod::Parts,
        gpiog: gpio::gpiog::Parts,
    ) -> GPIO {
        let led = gpioc.pc7.into_push_pull_output();
        GPIO {
            led,
            daisy0: gpiob.pb11,
            daisy1: gpioc.pc11,
            daisy2: gpioc.pc10,
            daisy3: gpioc.pc9,
            daisy4: gpioc.pc8,
            daisy5: gpiod.pd2,
            daisy6: gpioc.pc12,
            daisy7: gpiog.pg10,
            daisy8: gpiog.pg11,
            daisy9: gpiob.pb4,
            daisy10: gpiob.pb5,
            daisy11: gpiob.pb8,
            daisy12: gpiob.pb9,
            daisy13: gpiob.pb6,
            daisy14: gpiob.pb7,
            daisy15: gpioc.pc0,
            daisy16: gpioa.pa3,
            daisy17: gpiob.pb1,
            daisy18: gpioa.pa7,
            daisy19: gpioa.pa6,
            daisy20: gpioc.pc1,
            daisy21: gpioc.pc4,
            daisy22: gpioa.pa5,
            daisy23: gpioa.pa4,
            daisy24: gpioa.pa1,
            daisy25: gpioa.pa0,
            daisy26: gpiod.pd11,
            daisy27: gpiog.pg9,
            daisy28: gpioa.pa2,
            daisy29: gpiob.pb14,
            daisy30: gpiob.pb15,
        }
    }
}
