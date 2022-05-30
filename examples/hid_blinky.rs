//! examples/hid_blinky.rs
#![no_main]
#![no_std]

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    use log::info;
    // Includes a panic handler and optional logging facilities
    use libdaisy::logger;

    use stm32h7xx_hal::{adc, prelude::*, stm32, time::MilliSeconds, timer::Timer};

    use libdaisy::gpio::*;
    use libdaisy::hid;
    use libdaisy::prelude::*;
    use libdaisy::system;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led1: hid::Led<SeedLed>,
        adc1: adc::Adc<stm32::ADC1, adc::Enabled>,
        control1: hid::AnalogControl<Daisy15<Analog>>,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);

        system
            .timer2
            .set_freq(MilliSeconds::from_ticks(1).into_rate());

        let mut led1 = hid::Led::new(system.gpio.led, false, 1000);
        led1.set_brightness(0.5);

        let mut adc1 = system.adc1.enable();
        adc1.set_resolution(adc::Resolution::SIXTEENBIT);
        let adc1_max = adc1.slope() as f32;

        let daisy15 = system
            .gpio
            .daisy15
            .take()
            .expect("Failed to get pin daisy15!")
            .into_analog();

        let control1 = hid::AnalogControl::new(daisy15, adc1_max);

        (
            Shared {},
            Local {
                led1,
                adc1,
                control1,
                timer2: system.timer2,
            },
            init::Monotonics(),
        )
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task(binds = TIM2, local = [timer2, adc1, control1, led1])]
    fn interface_handler(ctx: interface_handler::Context) {
        ctx.local.timer2.clear_irq();
        let adc1 = ctx.local.adc1;
        let led1 = ctx.local.led1;
        let control1 = ctx.local.control1;

        if let Ok(data) = adc1.read(control1.get_pin()) {
            control1.update(data);
        }

        let value = control1.get_value();
        led1.set_blink(value, 1.0 - value);
        info!("{} {}", value, 1.0 - value);
        led1.update();
    }
}
