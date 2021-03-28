#![allow(dead_code)]
// #![allow(unused_variables)]

use log::info;
use stm32h7xx_hal::{
    adc, delay::Delay, prelude::*, rcc, stm32, stm32::TIM2, timer::Event, timer::Timer,
};

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

    /// Setup cache
    pub fn init_cache(scb: &mut cortex_m::peripheral::SCB) {
        scb.invalidate_icache();
        scb.enable_icache();
        // core.SCB.clean_invalidate_dcache(&mut core.CPUID);
        // core.SCB.enable_dcache(&mut core.CPUID);
    }

    /// Set ADCs
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

    /// Enable debug
    pub fn init_debug(dcb: &mut cortex_m::peripheral::DCB, dwt: &mut cortex_m::peripheral::DWT) {
        dcb.enable_trace();
        cortex_m::peripheral::DWT::unlock();
        dwt.enable_cycle_counter();
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

        Self::init_debug(&mut core.DCB, &mut core.DWT);

        //Timers
        let mut timer2 = device
            .TIM2
            .timer(100.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        // let mut timer3 = device
        //     .TIM3
        //     .timer(1.ms(), ccdr.peripheral.TIM3, &mut ccdr.clocks);
        // timer3.listen(Event::TimeOut);

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
        info!("Initialised MPU...");

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

        info!("Setting up GPIOs...");
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
            sdram,
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
