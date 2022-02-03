//! examples/knob.rs
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

    use stm32h7xx_hal::adc;
    use stm32h7xx_hal::stm32;
    use stm32h7xx_hal::timer::Timer;

    use libdaisy::gpio::*;
    use libdaisy::hid;
    use libdaisy::prelude::*;
    use libdaisy::system;
    use stm32h7xx_hal::time::Hertz;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        led1: hid::Led<Daisy28<Output<PushPull>>>,
        adc1: adc::Adc<stm32::ADC1, adc::Enabled>,
        control1: hid::AnalogControl<Daisy21<Analog>>,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);

        let duty_cycle = 50;
        let resolution = 20;

        system.timer2.set_freq(Hertz(duty_cycle * resolution));

        let daisy28 = system
            .gpio
            .daisy28
            .take()
            .expect("Failed to get pin 28!")
            .into_push_pull_output();

        let led1 = hid::Led::new(daisy28, false, resolution);

        let mut adc1 = system.adc1.enable();
        adc1.set_resolution(adc::Resolution::SIXTEENBIT);
        let adc1_max = adc1.max_sample() as f32;

        let daisy21 = system
            .gpio
            .daisy21
            .take()
            .expect("Failed to get pin 21!")
            .into_analog();

        let control1 = hid::AnalogControl::new(daisy21, adc1_max);

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

    #[task( binds = TIM2, local = [timer2, adc1, control1, led1] )]
    fn interface_handler(ctx: interface_handler::Context) {
        ctx.local.timer2.clear_irq();
        let adc1 = ctx.local.adc1;
        let led1 = ctx.local.led1;
        let control1 = ctx.local.control1;

        if let Ok(data) = adc1.read(control1.get_pin()) {
            control1.update(data);
        }

        led1.set_brightness(control1.get_value());
        info!("{}", control1.get_value());
        led1.update();
    }
}
