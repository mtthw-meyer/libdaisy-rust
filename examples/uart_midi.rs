//! examples/uart_midi.rs
#![no_main]
#![no_std]

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true
)]
mod app {
    use libdaisy::logger;
    use log::info;

    use stm32h7xx_hal::prelude::*;
    use stm32h7xx_hal::serial::Serial;
    use stm32h7xx_hal::stm32::USART1;

    use libdaisy::system;
    use libdaisy::uart;

    use midi_port::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        midi: midi_port::MidiInPort<Serial<USART1>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();
        let mut system = system::System::init(ctx.core, ctx.device);
        info!("Startup done!");

        let midi = MidiInPort::new(uart::serial(
            system.gpio.daisy13.take().unwrap().into_alternate(),
            system.gpio.daisy14.take().unwrap().into_alternate(),
            stm32h7xx_hal::serial::config::Config::default()
                .baudrate(31_250.bps())
                .parity_none(),
            system.uart.usart1.take().unwrap(),
            &system.clocks,
        ));

        (Shared {}, Local { midi }, init::Monotonics())
    }

    #[idle(local=[midi])]
    fn idle(ctx: idle::Context) -> ! {
        loop {
            ctx.local.midi.poll_uart();

            if let Some(message) = ctx.local.midi.get_message() {
                info!("{:?}", message);
            }
        }
    }
}
