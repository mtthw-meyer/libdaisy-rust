//! examples/toggle.rs
#![no_main]
#![no_std]
use libdaisy_rust::hid;
use libdaisy_rust::*;

use arr_macro::arr;

use core::alloc;

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        // #[init(Some( hid::Interface { switch_controls: arr![None; 20], analogue_controls: arr![None; 12], ready: false }))]
        // #[init(None)]
        // hid: Option<hid::Interface>,
        gpio: gpio::GPIO,
    }

    #[init( schedule = [interface_handler] )]
    fn init(ctx: init::Context) -> init::LateResources {
        let system = system::System::init(ctx.core, ctx.device);

        let now = ctx.start;
        ctx.schedule
            .interface_handler(now)
            .unwrap();

        init::LateResources {
            gpio: system.gpio,
        }
    }

    #[idle( resources = [gpio] )]
    fn idle(mut ctx: idle::Context) -> ! {
        // let mut hid = ctx.resources.hid;
        let mut hid = hid::Interface { switch_controls: arr![None; 20], analogue_controls: arr![None; 12], ready: false };
        let mut gpio = ctx.resources.gpio;

        let mut daisy28: Option<_> = None;
        gpio.lock(|gpio| {
            daisy28 = gpio.daisy28.take();
        });
        
        let daisy28 = daisy28.expect("Failed to get pin!").into_pull_up_input();
        
        let mut switch1 = hid::Switch::new(&daisy28);
        hid.register_switch(&mut switch1);
        // ctx.resources.hid = &mut Some(hid);
        // Register controls
        // hid.lock(|hid| hid.register_switch(&mut switch1) );
        // hid.register_analogue();

        // hid.lock(|hid| hid.set_ready() );
        loop {
            cortex_m::asm::nop();
        }
    }

    #[task( schedule = [interface_handler], resources = [gpio] )]
    fn interface_handler(ctx: interface_handler::Context) {
        // ctx.resources.hid.update();

        // let a = &ctx.resources.gpio.daisy28;
    }

    extern "C" {
        fn TIM4();
    }
};
