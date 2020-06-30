#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use cortex_m::peripheral::DWT;

use rtic;
use stm32h7xx_hal::gpio;
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::sai;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};

use crate::*;

const HSE_CLOCK_MHZ: MegaHertz = MegaHertz(16);
const HCLK_MHZ: MegaHertz = MegaHertz(200);

// PLL1
const PLL1_P_HZ: MegaHertz = CLOCK_RATE_MHZ;
const PLL1_Q_HZ: Hertz = Hertz(160_000_000);
const PLL1_R_HZ: Hertz = Hertz(120_000_000);
// PLL2
const PLL2_P_HZ: Hertz = Hertz(25_000_000);
const PLL2_Q_HZ: Hertz = Hertz(100_000_000);
const PLL2_R_HZ: Hertz = Hertz(100_000_000);
// PLL3
const PLL3_P_HZ: Hertz = Hertz(49_152_344);
const PLL3_Q_HZ: Hertz = Hertz(24_576_172);
const PLL3_R_HZ: Hertz = Hertz(12_288_086);

#[allow(non_snake_case)]
pub struct System {
    pub log: Log,
    pub gpio: crate::gpio::GPIO,
    pub sai1: sai::Sai<stm32::SAI1, sai::I2S>,
    pub EXTI: stm32::EXTI,
    pub SYSCFG: stm32::SYSCFG,
    //pub audio: ?.
}

impl System {
    pub fn init(mut core: rtic::Peripherals, device: stm32::Peripherals) -> System {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                use cortex_m_log::printer::semihosting::Semihosting;
                use cortex_m_log::modes::InterruptFree;
                let mut log = Semihosting::<InterruptFree, _>::stdout().unwrap();
            }
            else {
                let mut log = Dummy::new();
            }
        }
        println!(log, "Starting system init");

        // Power
        let pwr = device.PWR.constrain();
        let vos = pwr.freeze();

        // Clocks
        let mut ccdr = device
            .RCC
            .constrain()
            .use_hse(HSE_CLOCK_MHZ)
            .sys_ck(CLOCK_RATE_MHZ)
            // PLL1
            .pll1_p_ck(PLL1_P_HZ)
            .pll1_q_ck(PLL1_Q_HZ)
            .pll1_r_ck(PLL1_R_HZ)
            // PLL2
            .pll2_p_ck(PLL2_P_HZ)
            .pll2_q_ck(PLL2_Q_HZ)
            .pll2_r_ck(PLL2_R_HZ)
            // PLL3
            .pll3_p_ck(PLL3_P_HZ)
            .pll3_q_ck(PLL3_Q_HZ)
            .pll3_r_ck(PLL3_R_HZ)
            .freeze(vos, &device.SYSCFG);

        print_clocks(&mut log, &ccdr);

        let mut timer2 = device
            .TIM2
            .timer(250.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        core.DWT.enable_cycle_counter();

        let pins_a = (
            gpioe.pe2.into_alternate_af6(),       // MCLK_A
            gpioe.pe5.into_alternate_af6(),       // SCK_A
            gpioe.pe4.into_alternate_af6(),       // FS_A
            gpioe.pe6.into_alternate_af6(),       // SD_A
            Some(gpioe.pe3.into_alternate_af6()), // SD_B
        );

        println!(log, "Setup up SAI...");
        let sai1 = device.SAI1.i2s_ch_a(
            pins_a,
            48.khz(),
            sai::I2SBitRate::BITS_24,
            ccdr.peripheral.SAI1,
            &ccdr.clocks,
        );

        // Setup GPIOs
        let gpio = crate::gpio::GPIO::init(gpioa, gpiob, gpioc, gpiod, gpiog);

        println!(log, "System init done!");

        System {
            log,
            gpio,
            sai1,
            EXTI: device.EXTI,
            SYSCFG: device.SYSCFG,
        }
    }
}

fn print_clocks(log: &mut Log, ccdr: &stm32h7xx_hal::rcc::Ccdr) {
    println!(log, "Core {}", ccdr.clocks.c_ck());
    println!(
        log,
        "PLL1 P: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll1_p_ck(),
        ccdr.clocks.pll1_q_ck(),
        ccdr.clocks.pll1_r_ck()
    );
    println!(
        log,
        "PLL2 P: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll2_p_ck(),
        ccdr.clocks.pll2_q_ck(),
        ccdr.clocks.pll2_r_ck()
    );
    println!(
        log,
        "PLL3 P: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll3_p_ck(),
        ccdr.clocks.pll3_q_ck(),
        ccdr.clocks.pll3_r_ck()
    );
}
