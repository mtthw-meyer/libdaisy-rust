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
use stm32h7xx_hal::stm32::rcc::d2ccip1r::SAI1SEL_A;

use crate::*;

const HSE_CLOCK_MHZ: MegaHertz = MegaHertz(16);
const HCLK_MHZ: MegaHertz = MegaHertz(200);

// PCLKx
const PCLK_HZ_DIV2: Hertz  = Hertz(CLOCK_RATE_HZ.0 / 2);
// 49_152_344
// PLL1
const PLL1_P_HZ: Hertz  = CLOCK_RATE_HZ;
const PLL1_Q_HZ: Hertz  = Hertz(CLOCK_RATE_HZ.0 / 18);
const PLL1_R_HZ: Hertz  = Hertz(CLOCK_RATE_HZ.0 / 32);
// PLL2
const PLL2_P_HZ: Hertz = Hertz(3_125_000);
const PLL2_Q_HZ: Hertz = Hertz(PLL2_P_HZ.0 / 2); // No divder given, what's the default?
const PLL2_R_HZ: Hertz = Hertz(PLL2_P_HZ.0 / 4); // No divder given, what's the default?
// PLL3
const PLL3_P_HZ: Hertz = Hertz(AUDIO_SAMPLE_HZ.0 * 256); // 48Khz * 256 = 12_288_000
const PLL3_Q_HZ: Hertz = Hertz(PLL3_P_HZ.0 / 4);
const PLL3_R_HZ: Hertz = Hertz(PLL3_P_HZ.0 / 16);

const BLOCK_SIZE_MAX: usize = 48;
const BUFFER_SIZE: usize = BLOCK_SIZE_MAX * 2;

#[no_mangle]
#[link_section = ".sdram1_bss"]
static mut buf_rx: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE];
#[no_mangle]
#[link_section = ".sdram1_bss"]
static mut buf_tx: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE];

#[allow(non_snake_case)]
pub struct System {
    pub log: Log,
    pub gpio: crate::gpio::GPIO,
    pub audio: sai::Sai<stm32::SAI1, sai::I2S>,
    pub EXTI: stm32::EXTI,
    pub SYSCFG: stm32::SYSCFG,
    
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
        // println!(log, "Starting system init");
        // Power
        let pwr = device.PWR.constrain();
        let vos = pwr.freeze();

        // Clocks
        let mut ccdr = device
            .RCC
            .constrain()
            .use_hse(HSE_CLOCK_MHZ)
            .sys_ck(CLOCK_RATE_HZ)
            .pclk1(PCLK_HZ_DIV2) // DMA clock
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

        // TODO - Use stm32h7-fmc to setup SDRAM
        // println!(log, "Setting up SDRAM...");

        // TODO - Timer
        // println!(log, "Setting up timers...");
        // Initialize (enable) the monotonic timer (CYCCNT)
        core.DCB.enable_trace();
        // required on Cortex-M7 devices that software lock the DWT (e.g. STM32F7)
        DWT::unlock();
        core.DWT.enable_cycle_counter();

        let mut timer2 = device
            .TIM2
            .timer(250.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        // println!(log, "Setting up GPIOs...");
        let gpioa = device.GPIOA.split(ccdr.peripheral.GPIOA);
        let gpiob = device.GPIOB.split(ccdr.peripheral.GPIOB);
        let gpioc = device.GPIOC.split(ccdr.peripheral.GPIOC);
        let gpiod = device.GPIOD.split(ccdr.peripheral.GPIOD);
        let gpioe = device.GPIOE.split(ccdr.peripheral.GPIOE);
        let gpiof = device.GPIOF.split(ccdr.peripheral.GPIOF);
        let gpiog = device.GPIOG.split(ccdr.peripheral.GPIOG);

        let pins_a = (
            gpioe.pe2.into_alternate_af6(),       // MCLK_A
            gpioe.pe5.into_alternate_af6(),       // SCK_A
            gpioe.pe4.into_alternate_af6(),       // FS_A
            gpioe.pe6.into_alternate_af6(),       // SD_A
            Some(gpioe.pe3.into_alternate_af6()), // SD_B
        );

        // TODO - QSPI
        // println!(log, "Setting up QSPI...");
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
            pin_group[DSY_QSPI_PIN_NCS] = dsy_pin(DSY_GPIOG, 6);
        */

        // println!(log, "Setup up SAI...");
        
        let sai1_per = ccdr.peripheral.SAI1.kernel_clk_mux(SAI1SEL_A::PLL3_P);
        let mut audio = device.SAI1.i2s_ch_a(
            pins_a,
            48.khz(),
            sai::I2SBitRate::BITS_24,
            sai1_per,
            &ccdr.clocks,
        );

        unsafe {
            audio.config_dma(device.DMA1, &buf_rx[0], &buf_tx[0]);
        }

        audio.enable();

        // Setup GPIOs
        let gpio = crate::gpio::GPIO::init(gpioa, gpiob, gpioc, gpiod, gpiog);

        // println!(log, "System init done!");

        System {
            log,
            gpio,
            audio,
            EXTI: device.EXTI,
            SYSCFG: device.SYSCFG,
        }
    }
}

fn print_clocks(log: &mut Log, ccdr: &stm32h7xx_hal::rcc::Ccdr) {
    println!(log, "Core {}", ccdr.clocks.c_ck());
    println!(log, "pclk1 {}", ccdr.clocks.pclk1());
    println!(log, "pclk2 {}", ccdr.clocks.pclk2());
    println!(log, "pclk3 {}", ccdr.clocks.pclk2());
    println!(log, "pclk4 {}", ccdr.clocks.pclk4());
    println!(
        log,
        "PLL1\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll1_p_ck(),
        ccdr.clocks.pll1_q_ck(),
        ccdr.clocks.pll1_r_ck()
    );
    println!(
        log,
        "PLL2\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll2_p_ck(),
        ccdr.clocks.pll2_q_ck(),
        ccdr.clocks.pll2_r_ck()
    );
    println!(
        log,
        "PLL3\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll3_p_ck(),
        ccdr.clocks.pll3_q_ck(),
        ccdr.clocks.pll3_r_ck()
    );
}
