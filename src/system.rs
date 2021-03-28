#![allow(dead_code)]
// #![allow(unused_variables)]

use cortex_m::peripheral::DWT;
use log::info;

use core::{mem, slice};

use stm32h7xx_hal::adc;
use stm32h7xx_hal::delay::Delay;
use stm32h7xx_hal::gpio::{gpiod, gpioe, gpiof, gpiog, gpioh, gpioi, Analog};
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::rcc;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::stm32::TIM2;
use stm32h7xx_hal::timer::Event;
use stm32h7xx_hal::timer::Timer;

use stm32_fmc::devices::as4c16m32msa_6;

use crate::audio::Audio;
use crate::*;

const HSE_CLOCK_MHZ: MegaHertz = MegaHertz(16);
const HCLK_MHZ: MegaHertz = MegaHertz(200);
const HCLK2_MHZ: MegaHertz = MegaHertz(200);

// PCLKx
const PCLK_HZ: Hertz = Hertz(CLOCK_RATE_HZ.0 / 4);
// 49_152_344
// PLL1
const PLL1_P_HZ: Hertz = CLOCK_RATE_HZ;
const PLL1_Q_HZ: Hertz = Hertz(CLOCK_RATE_HZ.0 / 18);
const PLL1_R_HZ: Hertz = Hertz(CLOCK_RATE_HZ.0 / 32);
// PLL2
const PLL2_P_HZ: Hertz = Hertz(4_000_000);
const PLL2_Q_HZ: Hertz = Hertz(PLL2_P_HZ.0 / 2); // No divder given, what's the default?
const PLL2_R_HZ: Hertz = Hertz(PLL2_P_HZ.0 / 4); // No divder given, what's the default?
                                                 // PLL3
                                                 // 48Khz * 256 = 12_288_000
const PLL3_P_HZ: Hertz = Hertz(AUDIO_SAMPLE_HZ.0 * 257);
const PLL3_Q_HZ: Hertz = Hertz(PLL3_P_HZ.0 / 4);
const PLL3_R_HZ: Hertz = Hertz(PLL3_P_HZ.0 / 16);

const SLOTS: u8 = 2;
const FIRST_BIT_OFFSET: u8 = 0;

// #[link_section = ".sdram_bss"]
// #[no_mangle]
// static mut start_of_sdram: u32 = 0;

/// Configure pins for the FMC controller
macro_rules! fmc_pins {
    ($($pin:expr),*) => {
        (
            $(
                $pin.into_push_pull_output()
                    .set_speed(stm32h7xx_hal::gpio::Speed::VeryHigh)
                    .into_alternate_af12()
                    .internal_pull_up(true)
            ),*
        )
    };
}

pub struct System {
    pub gpio: crate::gpio::GPIO,
    pub audio: Audio,
    pub exti: stm32::EXTI,
    pub syscfg: stm32::SYSCFG,
    pub adc1: adc::Adc<stm32::ADC1, adc::Disabled>,
    pub adc2: adc::Adc<stm32::ADC2, adc::Disabled>,
    pub timer2: Timer<TIM2>,
    pub sdram: &'static mut [f32],
}

impl System {
    pub fn init_clocks(pwr: stm32::PWR, rcc: stm32::RCC, syscfg: &stm32::SYSCFG) -> rcc::Ccdr {
        // Power
        let pwr = pwr.constrain();
        let vos = pwr.vos0(syscfg).freeze();
        rcc.constrain()
            .use_hse(HSE_CLOCK_MHZ)
            .sys_ck(CLOCK_RATE_HZ)
            .pclk1(PCLK_HZ) // DMA clock
            // PLL1
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .pll1_p_ck(PLL1_P_HZ)
            .pll1_q_ck(PLL1_Q_HZ)
            .pll1_r_ck(PLL1_R_HZ)
            // PLL2
            .pll2_p_ck(PLL2_P_HZ) // Default adc_ker_ck_input
            // .pll2_q_ck(PLL2_Q_HZ)
            // .pll2_r_ck(PLL2_R_HZ)
            // PLL3
            .pll3_strategy(rcc::PllConfigStrategy::Iterative)
            .pll3_p_ck(PLL3_P_HZ)
            .pll3_q_ck(PLL3_Q_HZ)
            .pll3_r_ck(PLL3_R_HZ)
            .freeze(vos, &syscfg)
    }

    pub fn init_sdram(
        fmc_d: stm32::FMC,
        fmc_p: rcc::rec::Fmc,
        clocks: &rcc::CoreClocks,
        dd0: gpiod::PD0<Analog>,
        dd1: gpiod::PD1<Analog>,
        dd8: gpiod::PD8<Analog>,
        dd9: gpiod::PD9<Analog>,
        dd10: gpiod::PD10<Analog>,
        dd14: gpiod::PD14<Analog>,
        dd15: gpiod::PD15<Analog>,
        ee0: gpioe::PE0<Analog>,
        ee1: gpioe::PE1<Analog>,
        ee7: gpioe::PE7<Analog>,
        ee8: gpioe::PE8<Analog>,
        ee9: gpioe::PE9<Analog>,
        ee10: gpioe::PE10<Analog>,
        ee11: gpioe::PE11<Analog>,
        ee12: gpioe::PE12<Analog>,
        ee13: gpioe::PE13<Analog>,
        ee14: gpioe::PE14<Analog>,
        ee15: gpioe::PE15<Analog>,
        ff0: gpiof::PF0<Analog>,
        ff1: gpiof::PF1<Analog>,
        ff2: gpiof::PF2<Analog>,
        ff3: gpiof::PF3<Analog>,
        ff4: gpiof::PF4<Analog>,
        ff5: gpiof::PF5<Analog>,
        ff11: gpiof::PF11<Analog>,
        ff12: gpiof::PF12<Analog>,
        ff13: gpiof::PF13<Analog>,
        ff14: gpiof::PF14<Analog>,
        ff15: gpiof::PF15<Analog>,
        gg0: gpiog::PG0<Analog>,
        gg1: gpiog::PG1<Analog>,
        gg2: gpiog::PG2<Analog>,
        gg4: gpiog::PG4<Analog>,
        gg5: gpiog::PG5<Analog>,
        gg8: gpiog::PG8<Analog>,
        gg15: gpiog::PG15<Analog>,
        hh2: gpioh::PH2<Analog>,
        hh3: gpioh::PH3<Analog>,
        hh5: gpioh::PH5<Analog>,
        hh8: gpioh::PH8<Analog>,
        hh9: gpioh::PH9<Analog>,
        hh10: gpioh::PH10<Analog>,
        hh11: gpioh::PH11<Analog>,
        hh12: gpioh::PH12<Analog>,
        hh13: gpioh::PH13<Analog>,
        hh14: gpioh::PH14<Analog>,
        hh15: gpioh::PH15<Analog>,
        ii0: gpioi::PI0<Analog>,
        ii1: gpioi::PI1<Analog>,
        ii2: gpioi::PI2<Analog>,
        ii3: gpioi::PI3<Analog>,
        ii4: gpioi::PI4<Analog>,
        ii5: gpioi::PI5<Analog>,
        ii6: gpioi::PI6<Analog>,
        ii7: gpioi::PI7<Analog>,
        ii9: gpioi::PI9<Analog>,
        ii10: gpioi::PI10<Analog>,
    ) -> stm32_fmc::Sdram<stm32h7xx_hal::fmc::FMC, as4c16m32msa_6::As4c16m32msa> {
        let sdram_pins = fmc_pins! {
            // A0-A12
            ff0, ff1, ff2, ff3,
            ff4, ff5, ff12, ff13,
            ff14, ff15, gg0, gg1,
            gg2,
            // BA0-BA1
            gg4, gg5,
            // D0-D31
            dd14, dd15, dd0, dd1,
            ee7, ee8, ee9, ee10,
            ee11, ee12, ee13, ee14,
            ee15, dd8, dd9, dd10,
            hh8, hh9, hh10, hh11,
            hh12, hh13, hh14, hh15,
            ii0, ii1, ii2, ii3,
            ii6, ii7, ii9, ii10,
            // NBL0 - NBL3
            ee0, ee1, ii4, ii5,
            hh2,   // SDCKE0
            gg8,   // SDCLK
            gg15,  // SDNCAS
            hh3,   // SDNE0
            ff11,  // SDRAS
            hh5    // SDNWE
        };

        fmc_d.sdram(sdram_pins, as4c16m32msa_6::As4c16m32msa {}, fmc_p, clocks)
    }

    pub fn init_cache(scb: &mut cortex_m::peripheral::SCB) {
        // Setup cache
        scb.invalidate_icache();
        scb.enable_icache();
        // core.SCB.clean_invalidate_dcache(&mut core.CPUID);
        // core.SCB.enable_dcache(&mut core.CPUID);
    }

    pub fn init_adc(
        adc1: stm32::ADC1,
        adc2: stm32::ADC2,
        adc12: rcc::rec::Adc12,
        delay: &mut Delay,
        clocks: &rcc::CoreClocks,
    ) -> (
        adc::Adc<stm32::ADC1, adc::Disabled>,
        adc::Adc<stm32::ADC2, adc::Disabled>,
    ) {
        adc::adc12(adc1, adc2, delay, adc12, clocks)
    }

    ///Batteries included initializion
    pub fn init(mut core: cortex_m::Peripherals, device: stm32::Peripherals) -> System {
        info!("Starting system init");
        let mut ccdr = Self::init_clocks(device.PWR, device.RCC, &device.SYSCFG);

        // log_clocks(&ccdr);

        let mut delay = Delay::new(core.SYST, ccdr.clocks);
        // Setup ADCs
        let (adc1, adc2) = Self::init_adc(
            device.ADC1,
            device.ADC2,
            ccdr.peripheral.ADC12,
            &mut delay,
            &ccdr.clocks,
        );

        // Timers
        // TODO
        // ?
        core.DCB.enable_trace();
        DWT::unlock();
        core.DWT.enable_cycle_counter();

        let mut timer2 = device
            .TIM2
            .timer(100.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        // let mut timer3 = device
        //     .TIM3
        //     .timer(1.ms(), ccdr.peripheral.TIM3, &mut ccdr.clocks);
        // timer3.listen(Event::TimeOut);

        // info!("Setting up GPIOs...");
        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);
        let gpioh = device.GPIOH.split(ccdr.peripheral.GPIOH);
        let gpioi = device.GPIOI.split(ccdr.peripheral.GPIOI);

        // Configure SDRAM
        info!("Setting up SDRAM...");
        let mut sdram = Self::init_sdram(
            device.FMC,
            ccdr.peripheral.FMC,
            &ccdr.clocks,
            gpiod.pd0,
            gpiod.pd1,
            gpiod.pd8,
            gpiod.pd9,
            gpiod.pd10,
            gpiod.pd14,
            gpiod.pd15,
            gpioe.pe0,
            gpioe.pe1,
            gpioe.pe7,
            gpioe.pe8,
            gpioe.pe9,
            gpioe.pe10,
            gpioe.pe11,
            gpioe.pe12,
            gpioe.pe13,
            gpioe.pe14,
            gpioe.pe15,
            gpiof.pf0,
            gpiof.pf1,
            gpiof.pf2,
            gpiof.pf3,
            gpiof.pf4,
            gpiof.pf5,
            gpiof.pf11,
            gpiof.pf12,
            gpiof.pf13,
            gpiof.pf14,
            gpiof.pf15,
            gpiog.pg0,
            gpiog.pg1,
            gpiog.pg2,
            gpiog.pg4,
            gpiog.pg5,
            gpiog.pg8,
            gpiog.pg15,
            gpioh.ph2,
            gpioh.ph3,
            gpioh.ph5,
            gpioh.ph8,
            gpioh.ph9,
            gpioh.ph10,
            gpioh.ph11,
            gpioh.ph12,
            gpioh.ph13,
            gpioh.ph14,
            gpioh.ph15,
            gpioi.pi0,
            gpioi.pi1,
            gpioi.pi2,
            gpioi.pi3,
            gpioi.pi4,
            gpioi.pi5,
            gpioi.pi6,
            gpioi.pi7,
            gpioi.pi9,
            gpioi.pi10,
        );

        let ram: &mut [f32] = unsafe {
            let ram_ptr: *mut u32 = sdram.init(&mut delay);
            info!("SDRAM ptr: {:?}", ram_ptr);
            let sdram_size_bytes: usize = 64 * 1024 * 1024;
            mpu_sdram_init(&mut core.MPU, &mut core.SCB, ram_ptr, sdram_size_bytes);

            info!("Initialised MPU...");

            slice::from_raw_parts_mut(
                ram_ptr as *mut f32,
                sdram_size_bytes / mem::size_of::<u32>(),
            )
        };

        // TODO - QSPI
        // info!("Setting up QSPI...");
        /*
            dsy_gpio_pin *pin_group;
            qspi_handle.device = DSY_QSPI_DEVICE_IS25LP064A;
            qspi_handle.mode   = DSY_QSPI_MODE_DSY_MEMORY_MAPPED;
            pin_group          = qspi_handle.pin_config;

            pin_group[DSY_QSPI_PIN_IO0] = dsy_pin(DSY_GPIOF, 8);
            pin_group[DSY_QSPI_PIN_IO1] = dsy_pin(DSY_GPIOF, 9);
            pin_group[DSY_QSPI_PIN_IO2] = dsy_pin(DSY_GPIOF, 7);
            pin_group[DSY_QSPI_PIN_IO3] = dsy_pin(DSY_GPIOF, 6);
            pin_group[DSY_QSPI_PIN_CLK] = dsy_pin(DSY_GPIOF, 10);
            pin_group[DSY_QSPI_PIN_NCS] =
            dsy_pin(DSY_GPIOG, 6);
        */
        info!("Setup up SAI...");

        let audio = Audio::init(
            ccdr.peripheral.SAI1,
            device.SAI1,
            &ccdr.clocks,
            gpioe.pe2,
            gpioe.pe3,
            gpioe.pe4,
            gpioe.pe5,
            gpioe.pe6,
        );

        // ccdr.peripheral.DMA1.enable().reset();
        // ccdr.peripheral.DMA1.enable().reset();
        // let dma1_channels = device.DMA1.split();
        // let mut stream0 = dma1_channels.0;
        // let mut stream1 = dma1_channels.1;
        // unsafe {
        //     stream0.set_memory_address(buf_tx[..].as_ptr() as u32, true);
        //     stream1.set_memory_address(buf_rx[..].as_ptr() as u32, true);
        // }

        // Setup GPIOs
        let gpio = crate::gpio::GPIO::init(
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

        // Setup cache
        Self::init_cache(&mut core.SCB);

        info!("System init done!");

        System {
            gpio,
            audio,
            exti: device.EXTI,
            syscfg: device.SYSCFG,
            adc1,
            adc2,
            timer2,
            sdram: ram,
        }
    }
}

fn log_clocks(ccdr: &stm32h7xx_hal::rcc::Ccdr) {
    info!("Core {}", ccdr.clocks.c_ck());
    info!("hclk {}", ccdr.clocks.hclk());
    info!("pclk1 {}", ccdr.clocks.pclk1());
    info!("pclk2 {}", ccdr.clocks.pclk2());
    info!("pclk3 {}", ccdr.clocks.pclk2());
    info!("pclk4 {}", ccdr.clocks.pclk4());
    info!(
        "PLL1\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll1_p_ck(),
        ccdr.clocks.pll1_q_ck(),
        ccdr.clocks.pll1_r_ck()
    );
    info!(
        "PLL2\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll2_p_ck(),
        ccdr.clocks.pll2_q_ck(),
        ccdr.clocks.pll2_r_ck()
    );
    info!(
        "PLL3\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll3_p_ck(),
        ccdr.clocks.pll3_q_ck(),
        ccdr.clocks.pll3_r_ck()
    );
}

// MPU
// Configure MPU per Seed
// https://github.com/electro-smith/libDaisy/blob/04479d151dc275203a02e64fbfa2ab2bf6c0a91a/src/sys_system.c
// core.MPU.
// let mpu = unsafe { cortex_mpu::Mpu::new(core.MPU) };

/// Configure MPU for external SDRAM
///
/// Based on example from:
/// https://github.com/richardeoin/stm32h7-fmc/blob/master/examples/stm32h747i-disco.rs
///
/// Memory address in location will be 32-byte aligned.
///
/// # Panics
///
/// Function will panic if `size` is not a power of 2. Function
/// will panic if `size` is not at least 32 bytes.
fn mpu_sdram_init(
    mpu: &mut cortex_m::peripheral::MPU,
    scb: &mut cortex_m::peripheral::SCB,
    location: *mut u32,
    size: usize,
) {
    /// Refer to ARM®v7-M Architecture Reference Manual ARM DDI 0403
    /// Version E.b Section B3.5
    const MEMFAULTENA: u32 = 1 << 16;

    unsafe {
        /* Make sure outstanding transfers are done */
        cortex_m::asm::dmb();

        scb.shcsr.modify(|r| r & !MEMFAULTENA);

        /* Disable the MPU and clear the control register*/
        mpu.ctrl.write(0);
    }

    const REGION_NUMBER1: u32 = 0x01;
    const REGION_FULL_ACCESS: u32 = 0x03;
    const REGION_ENABLE: u32 = 0x01;

    assert_eq!(
        size & (size - 1),
        0,
        "SDRAM memory region size must be a power of 2"
    );
    assert_eq!(
        size & 0x1F,
        0,
        "SDRAM memory region size must be 32 bytes or more"
    );
    fn log2minus1(sz: u32) -> u32 {
        for x in 5..=31 {
            if sz == (1 << x) {
                return x - 1;
            }
        }
        panic!("Unknown SDRAM memory region size!");
    }

    info!("SDRAM Memory Size 0x{:x}", log2minus1(size as u32));

    // Configure region 1
    //
    // Strongly ordered
    unsafe {
        mpu.rnr.write(REGION_NUMBER1);
        mpu.rbar.write((location as u32) & !0x1F);
        mpu.rasr
            .write((REGION_FULL_ACCESS << 24) | (log2minus1(size as u32) << 1) | REGION_ENABLE);
    }

    const MPU_ENABLE: u32 = 0x01;
    const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

    // Enable
    unsafe {
        mpu.ctrl
            .modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

        scb.shcsr.modify(|r| r | MEMFAULTENA);

        // Ensure MPU settings take effect
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
}
