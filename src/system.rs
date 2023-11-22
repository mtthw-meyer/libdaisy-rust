//! Contains setup for Daisy board hardware.
#![allow(dead_code)]
// #![allow(unused_variables)]

use hal::rcc::CoreClocks;
use log::info;

use stm32h7xx_hal::{
    adc,
    delay::Delay,
    prelude::*,
    rcc, stm32,
    stm32::TIM2,
    time::{Hertz, MegaHertz, MilliSeconds},
    timer::Event,
    timer::Timer,
};

use crate::audio::Audio;
use crate::*;

const HSE_CLOCK_MHZ: MegaHertz = MegaHertz::from_raw(16);
const HCLK_MHZ: MegaHertz = MegaHertz::from_raw(200);
const HCLK2_MHZ: MegaHertz = MegaHertz::from_raw(200);

// PCLKx
const PCLK_HZ: Hertz = Hertz::from_raw(CLOCK_RATE_HZ.raw() / 4);
// 49_152_344
// PLL1
const PLL1_P_HZ: Hertz = CLOCK_RATE_HZ;
const PLL1_Q_HZ: Hertz = Hertz::from_raw(CLOCK_RATE_HZ.raw() / 18);
const PLL1_R_HZ: Hertz = Hertz::from_raw(CLOCK_RATE_HZ.raw() / 32);
// PLL2
const PLL2_P_HZ: Hertz = Hertz::from_raw(4_000_000);

const PLL3_P_HZ: Hertz = Hertz::from_raw(AUDIO_SAMPLE_HZ.raw() * 256);

pub struct System {
    pub gpio: crate::gpio::GPIO,
    pub audio: audio::Audio,
    pub exti: stm32::EXTI,
    pub syscfg: stm32::SYSCFG,
    pub adc1: adc::Adc<stm32::ADC1, adc::Disabled>,
    pub adc2: adc::Adc<stm32::ADC2, adc::Disabled>,
    pub timer2: Timer<TIM2>,
    pub sdram: &'static mut [f32],
    pub flash: crate::flash::Flash,
    pub clocks: CoreClocks,
    pub uart: crate::uart::UART,
}

impl System {
    /// Initialize clocks
    pub fn init_clocks(pwr: stm32::PWR, rcc: stm32::RCC, syscfg: &stm32::SYSCFG) -> rcc::Ccdr {
        // Power
        let pwr = pwr.constrain();
        let vos = pwr.vos0(syscfg).freeze();

        rcc.constrain()
            .use_hse(HSE_CLOCK_MHZ.convert())
            .sys_ck(CLOCK_RATE_HZ)
            .pclk1(PCLK_HZ) // DMA clock
            // PLL1
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .pll1_p_ck(PLL1_P_HZ)
            .pll1_q_ck(PLL1_Q_HZ)
            .pll1_r_ck(PLL1_R_HZ)
            // PLL2
            .pll2_p_ck(PLL2_P_HZ) // Default adc_ker_ck_input
            // PLL3
            .pll3_strategy(rcc::PllConfigStrategy::FractionalNotLess)
            .pll3_p_ck(PLL3_P_HZ) // used for SAI1
            .freeze(vos, &syscfg)
    }

    /// Setup cache
    pub fn init_cache(
        scb: &mut cortex_m::peripheral::SCB,
        cpuid: &mut cortex_m::peripheral::CPUID,
    ) {
        scb.enable_icache();
        scb.enable_dcache(cpuid);
    }

    /// Enable debug
    pub fn init_debug(dcb: &mut cortex_m::peripheral::DCB, dwt: &mut cortex_m::peripheral::DWT) {
        dcb.enable_trace();
        cortex_m::peripheral::DWT::unlock();
        dwt.enable_cycle_counter();
    }

    /// Batteries included initialization
    pub fn init(mut core: rtic::export::Peripherals, device: stm32::Peripherals) -> System {
        info!("Starting system init");
        let mut ccdr = Self::init_clocks(device.PWR, device.RCC, &device.SYSCFG);

        // log_clocks(&ccdr);
        let mut delay = Delay::new(core.SYST, ccdr.clocks);
        // Setup ADCs
        let (adc1, adc2) = adc::adc12(
            device.ADC1,
            device.ADC2,
            4.MHz(),
            &mut delay,
            ccdr.peripheral.ADC12,
            &ccdr.clocks,
        );

        Self::init_debug(&mut core.DCB, &mut core.DWT);

        // Timers
        let mut timer2 = device.TIM2.timer(
            MilliSeconds::from_ticks(100).into_rate(),
            ccdr.peripheral.TIM2,
            &mut ccdr.clocks,
        );
        timer2.listen(Event::TimeOut);

        // let mut timer3 = device
        //     .TIM3
        //     .timer(1.ms(), ccdr.peripheral.TIM3, &mut ccdr.clocks);
        // timer3.listen(Event::TimeOut);

        info!("Setting up GPIOs...");
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
        let sdram = crate::sdram::Sdram::new(
            device.FMC,
            ccdr.peripheral.FMC,
            &ccdr.clocks,
            &mut delay,
            &mut core.SCB,
            &mut core.MPU,
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
        )
        .into();

        info!("Setup up Audio...");
        let audio = Audio::new(
            device.DMA1,
            ccdr.peripheral.DMA1,
            device.SAI1,
            ccdr.peripheral.SAI1,
            gpioe.pe2,
            gpioe.pe3,
            gpioe.pe4,
            gpioe.pe5,
            gpioe.pe6,
            &ccdr.clocks,
            &mut core.MPU,
            &mut core.SCB,
        );

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
        Self::init_cache(&mut core.SCB, &mut core.CPUID);

        info!("System init done!");

        //setup flash
        let flash = crate::flash::Flash::new(
            device.QUADSPI,
            ccdr.peripheral.QSPI,
            &ccdr.clocks,
            gpiof.pf6,
            gpiof.pf7,
            gpiof.pf8,
            gpiof.pf9,
            gpiof.pf10,
            gpiog.pg6,
        );

        let uart = crate::uart::UART {
            usart1: Some((device.USART1, ccdr.peripheral.USART1)),
            usart3: Some((device.USART3, ccdr.peripheral.USART3)),
            uart4: Some((device.UART4, ccdr.peripheral.UART4)),
            uart5: Some((device.UART5, ccdr.peripheral.UART5)),
        };

        System {
            gpio,
            audio,
            exti: device.EXTI,
            syscfg: device.SYSCFG,
            adc1,
            adc2,
            timer2,
            sdram,
            flash,
            uart,
            clocks: ccdr.clocks,
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
