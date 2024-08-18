//! examples/sdram.rs
#![no_main]
#![no_std]

#[rtic::app(
    device = stm32h7xx_hal::stm32,
    peripherals = true,
)]
mod app {
    use log::info;

    use embedded_sdmmc::{VolumeManager, TimeSource, Timestamp, VolumeIdx};
    use libdaisy::{
        gpio,
        // Includes a panic handler and optional logging facilities
        logger,
        prelude::*,
        sdmmc,
        system::System,
    };
    use stm32h7xx_hal::{
        sdmmc::{SdCard, Sdmmc},
        stm32::SDMMC1,
    };

    #[shared]
    struct Shared {}

    #[local]
    struct Local {}

    struct FakeTime;

    impl TimeSource for FakeTime {
        fn get_timestamp(&self) -> Timestamp {
            Timestamp {
                year_since_1970: 52, //2022
                zero_indexed_month: 0,
                zero_indexed_day: 0,
                hours: 0,
                minutes: 0,
                seconds: 1,
            }
        }
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        logger::init();

        let device = ctx.device;
        let mut ccdr = System::init_clocks(device.PWR, device.RCC, &device.SYSCFG);

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        let mut gpio = gpio::GPIO::init(
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

        let mut sd = sdmmc::init(
            gpio.daisy1.take().unwrap(),
            gpio.daisy2.take().unwrap(),
            gpio.daisy3.take().unwrap(),
            gpio.daisy4.take().unwrap(),
            gpio.daisy5.take().unwrap(),
            gpio.daisy6.take().unwrap(),
            device.SDMMC1,
            ccdr.peripheral.SDMMC1,
            &mut ccdr.clocks,
        );

        gpio.led.set_low();
        if let Ok(_) = <Sdmmc<SDMMC1, SdCard>>::init(&mut sd, 50.MHz()) {
            info!("Got SD Card!");
            let mut sd_fatfs = VolumeManager::new(sd.sdmmc_block_device(), FakeTime);
            if let Ok(sd_fatfs_volume) = sd_fatfs.get_volume(VolumeIdx(0)) {
                if let Ok(sd_fatfs_root_dir) = sd_fatfs.open_root_dir(&sd_fatfs_volume) {
                    sd_fatfs
                        .iterate_dir(&sd_fatfs_volume, &sd_fatfs_root_dir, |entry| {
                            info!("{:?}", entry);
                        })
                        .unwrap();
                    sd_fatfs.close_dir(&sd_fatfs_volume, sd_fatfs_root_dir);
                    gpio.led.set_high();
                } else {
                    info!("Failed to get root dir");
                }
            } else {
                info!("Failed to get volume 0");
            }
        } else {
            info!("Failed to init SD Card");
        }

        (Shared {}, Local {}, init::Monotonics())
    }

    #[idle]
    fn idle(_ctx: idle::Context) -> ! {
        loop {
            cortex_m::asm::nop();
        }
    }
}
