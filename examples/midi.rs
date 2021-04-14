//! examples/midi.rs
#![no_main]
#![no_std]
use log::info;

use libdaisy::{gpio, prelude::*, system::System};
use stm32h7xx_hal::{
    block, stm32,
    timer::{Event, Timer},
};

#[derive(Copy, Clone)]
enum NoteState {
    On,
    Off,
    Idle,
}

impl NoteState {
    pub fn next(&self) -> Self {
        match self {
            Self::On => Self::Off,
            Self::Off => Self::Idle,
            Self::Idle => Self::On,
        }
    }
}

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
    monotonic = rtic::cyccnt::CYCCNT,
)]
const APP: () = {
    struct Resources {
        seed_led: gpio::SeedLed,
        timer2: Timer<stm32::TIM2>,
        serial_tx: stm32h7xx_hal::serial::Tx<stm32h7xx_hal::stm32::USART1>,
    }

    #[init]
    fn init(ctx: init::Context) -> init::LateResources {
        libdaisy::logger::init();

        let device = ctx.device;
        let mut ccdr = System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);
        let mut timer2 = device
            .TIM2
            .timer(100.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        info!("Startup done!");

        timer2.set_freq(100.ms());

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        let mut gpio = crate::gpio::GPIO::init(
            gpioc.pc7,
            gpiob.pb11,
            Some(gpiob.pb12),
            Some(gpioc.pc11),
            Some(gpioc.pc10),
            Some(gpioc.pc9),
            Some(gpioc.pc8),
            Some(gpiod.pd2),
            Some(gpioc.pc12),
            Some(gpiog.pg10),
            Some(gpiog.pg11),
            Some(gpiob.pb4),
            Some(gpiob.pb5),
            Some(gpiob.pb8),
            Some(gpiob.pb9),
            Some(gpiob.pb6),
            Some(gpiob.pb7),
            Some(gpioc.pc0),
            Some(gpioa.pa3),
            Some(gpiob.pb1),
            Some(gpioa.pa7),
            Some(gpioa.pa6),
            Some(gpioc.pc1),
            Some(gpioc.pc4),
            Some(gpioa.pa5),
            Some(gpioa.pa4),
            Some(gpioa.pa1),
            Some(gpioa.pa0),
            Some(gpiod.pd11),
            Some(gpiog.pg9),
            Some(gpioa.pa2),
            Some(gpiob.pb14),
            Some(gpiob.pb15),
        );

        let (serial_tx, _) = libdaisy::serial::midi(
            gpio.daisy13.take().unwrap(),
            gpio.daisy14.take().unwrap(),
            device.USART1,
            ccdr.peripheral.USART1,
            &ccdr.clocks,
        )
        .unwrap()
        .split();

        init::LateResources {
            seed_led: gpio.led,
            timer2,
            serial_tx,
        }
    }

    #[task(binds = TIM2, resources = [timer2, seed_led, serial_tx] )]
    fn send(ctx: send::Context) {
        static mut NOTE: NoteState = NoteState::Idle;
        static mut NOTE_NUM: u8 = 0;

        ctx.resources.timer2.clear_irq();

        *NOTE = NOTE.next();

        match NOTE {
            NoteState::On => {
                block!(ctx.resources.serial_tx.write(0x90)).unwrap();
                block!(ctx.resources.serial_tx.write(*NOTE_NUM)).unwrap();
                block!(ctx.resources.serial_tx.write(0x40)).unwrap();
                ctx.resources.seed_led.set_high().unwrap();
            }
            NoteState::Off => {
                ctx.resources.seed_led.set_low().unwrap();
                block!(ctx.resources.serial_tx.write(0x80)).unwrap();
                block!(ctx.resources.serial_tx.write(*NOTE_NUM)).unwrap();
                block!(ctx.resources.serial_tx.write(0x40)).unwrap();
            }
            NoteState::Idle => {
                *NOTE_NUM = (*NOTE_NUM + 1) & 0x7F;
            }
        };
    }
};
