//! examples/usb_midi.rs
#![no_main]
#![no_std]

//based on https://github.com/stm32-rs/stm32h7xx-hal/blob/master/examples/usb_rtic.rs
//and https://github.com/btrepp/usbd-midi

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    use libdaisy::{gpio, prelude::*, system::System};
    use stm32h7xx_hal::{
        rcc::rec::UsbClkSel,
        stm32,
        time::MilliSeconds,
        timer::{Event, Timer},
        usb_hs::{UsbBus, USB2},
    };

    use num_enum::TryFromPrimitive;
    use usb_device::prelude::*;
    use usbd_midi::{
        data::{
            byte::{from_traits::FromClamped, u7::U7},
            midi::{channel::Channel as MidiChannel, message::Message, notes::Note},
            usb::constants::USB_CLASS_NONE,
            usb_midi::{
                midi_packet_reader::MidiPacketBufferReader,
                usb_midi_event_packet::UsbMidiEventPacket,
            },
        },
        midi_device::MidiClass,
    };

    // Warning: EP_MEMORY may only be used for the UsbBusAllocator. Any
    // additional references are UB.
    static mut EP_MEMORY: [u32; 1024] = [0; 1024];

    #[shared]
    struct Shared {
        usb: (
            UsbDevice<'static, UsbBus<USB2>>,
            MidiClass<'static, UsbBus<USB2>>,
        ),
    }

    #[local]
    struct Local {
        seed_led: gpio::SeedLed,
        timer2: Timer<stm32::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        let device = ctx.device;
        let mut ccdr = System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);
        let _ = ccdr.clocks.hsi48_ck().expect("HSI48 must run");
        ccdr.peripheral.kernel_usb_clk_mux(UsbClkSel::Hsi48);

        /*
        unsafe {
            let pwr = &*stm32::PWR::ptr();
            pwr.cr3.modify(|_, w| w.usbregen().set_bit());
            while pwr.cr3.read().usb33rdy().bit_is_clear() {}
        }
        */

        let mut timer2 = device.TIM2.timer(
            MilliSeconds::from_ticks(200).into_rate(),
            ccdr.peripheral.TIM2,
            &mut ccdr.clocks,
        );
        timer2.listen(Event::TimeOut);

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        let gpio = gpio::GPIO::init(
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

        let (pin_dm, pin_dp) = { (gpioa.pa11.into_alternate(), gpioa.pa12.into_alternate()) };
        //float makes this a device
        gpioa.pa10.into_floating_input();
        let usb = USB2::new(
            device.OTG2_HS_GLOBAL,
            device.OTG2_HS_DEVICE,
            device.OTG2_HS_PWRCLK,
            pin_dm,
            pin_dp,
            ccdr.peripheral.USB2OTG,
            &ccdr.clocks,
        );

        #[allow(static_mut_refs)]
        let usb_bus = cortex_m::singleton!(
            : usb_device::class_prelude::UsbBusAllocator<UsbBus<USB2>> =
                UsbBus::new(usb, unsafe { &mut EP_MEMORY })
        )
        .unwrap();

        let midi = MidiClass::new(usb_bus, 1, 1).unwrap();

        let usb_dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x16c0, 0x5e4))
            .strings(&[StringDescriptors::default().product("daisy midi")]).unwrap()
            .device_class(USB_CLASS_NONE)
            .build();

        (
            Shared {
                usb: (usb_dev, midi),
            },
            Local {
                seed_led: gpio.led,
                timer2,
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

    #[task(
        binds = TIM2,
        shared = [usb],
        local = [timer2, on: bool = true, note_num: u8 = 0]
    )]
    fn send_note(mut ctx: send_note::Context) {
        ctx.local.timer2.clear_irq();
        let (local, shared) = (&mut ctx.local, &mut ctx.shared);
        shared.usb.lock(|(_usb_dev, midi)| {
            let on = *local.on;
            let note_num = *local.note_num;
            let chan = MidiChannel::Channel1;
            let note = Note::try_from_primitive(note_num).unwrap();
            let vel = U7::from_clamped(127);
            let packet = UsbMidiEventPacket::from_midi(
                usbd_midi::data::usb_midi::cable_number::CableNumber::Cable0,
                if on {
                    Message::NoteOn(chan, note, vel)
                } else {
                    Message::NoteOff(chan, note, vel)
                },
            );
            if midi.send_message(packet).is_ok() {
                *local.on = !on;
                if !on {
                    *local.note_num = (note_num + 1) % 127;
                }
            }
        });
    }

    #[task(binds = OTG_FS, shared = [usb], local = [seed_led])]
    fn usb_event(mut ctx: usb_event::Context) {
        let (local, shared) = (&mut ctx.local, &mut ctx.shared);
        shared.usb.lock(|(usb_dev, midi)| {
            let led = &mut local.seed_led;

            if !usb_dev.poll(&mut [midi]) {
                return;
            }

            let mut buffer = [0; 64];
            if let Ok(size) = midi.read(&mut buffer) {
                let buffer_reader = MidiPacketBufferReader::new(&buffer, size);
                for packet in buffer_reader.into_iter() {
                    if let Ok(packet) = packet {
                        match packet.message {
                            Message::NoteOn(_, _, U7::MIN) | Message::NoteOff(..) => {
                                led.set_low();
                            }
                            Message::NoteOn(..) => {
                                led.set_high();
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }
}
