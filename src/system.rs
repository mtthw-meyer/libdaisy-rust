#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use cortex_m::peripheral::DWT;

use rtic;
use stm32h7xx_hal::gpio;
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;
use stm32h7xx_hal::interrupt;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::rcc::rec::ResetEnable;
use stm32h7xx_hal::sai;
use stm32h7xx_hal::stm32;
use stm32h7xx_hal::stm32::rcc::d2ccip1r::SAI1SEL_A;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};

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
const PLL2_P_HZ: Hertz = Hertz(3_125_000);
const PLL2_Q_HZ: Hertz = Hertz(PLL2_P_HZ.0 / 2); // No divder given, what's the default?
const PLL2_R_HZ: Hertz = Hertz(PLL2_P_HZ.0 / 4); // No divder given, what's the default?
                                                 // PLL3
                                                 // 48Khz * 256 = 12_288_000
const PLL3_P_HZ: Hertz = Hertz(AUDIO_SAMPLE_HZ.0 * 257);
const PLL3_Q_HZ: Hertz = Hertz(PLL3_P_HZ.0 / 4);
const PLL3_R_HZ: Hertz = Hertz(PLL3_P_HZ.0 / 16);

const BLOCK_SIZE_MAX: usize = 48;
const BUFFER_SIZE: usize = BLOCK_SIZE_MAX * 2;

#[link_section = ".sram1_bss"]
#[no_mangle]
static mut buf_rx: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE];
#[link_section = ".sram1_bss"]
#[no_mangle]
static mut buf_tx: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE];

#[allow(non_snake_case)]
pub struct System {
    pub log: Log,
    pub gpio: crate::gpio::GPIO,
    // pub audio: sai::Sai<stm32::SAI1, sai::I2S>,
    pub EXTI: stm32::EXTI,
    pub SYSCFG: stm32::SYSCFG,
}

impl System {
    pub fn init(mut core: rtic::Peripherals, device: stm32::Peripherals) -> System {
        // pub fn init(mut core: cortex_m::peripheral::Peripherals, device: stm32::Peripherals) -> System {
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
            // .pclk1(PCLK_HZ) // DMA clock
            // PLL1
            // .pll1_p_ck(PLL1_P_HZ)
            // .pll1_q_ck(PLL1_Q_HZ)
            // .pll1_r_ck(PLL1_R_HZ)
            // PLL2
            // .pll2_p_ck(PLL2_P_HZ)
            // .pll2_q_ck(PLL2_Q_HZ)
            // .pll2_r_ck(PLL2_R_HZ)
            // PLL3
            .pll3_p_ck(PLL3_P_HZ)
            // .pll3_q_ck(PLL3_Q_HZ)
            // .pll3_r_ck(PLL3_R_HZ)
            .freeze(vos, &device.SYSCFG);

        // print_clocks(&mut log, &ccdr);
        // TODO - Use stm32h7-fmc to setup SDRAM?
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
            .timer(1000.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        // timer2.listen(Event::TimeOut);

        let mut timer3 = device
            .TIM3
            .timer(1.ms(), ccdr.peripheral.TIM3, &mut ccdr.clocks);
        timer3.listen(Event::TimeOut);

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
            pin_group[DSY_QSPI_PIN_NCS] =
            dsy_pin(DSY_GPIOG, 6);
        */
        println!(log, "Setup up SAI...");

        let sai1_rec = ccdr.peripheral.SAI1.kernel_clk_mux(SAI1SEL_A::PLL3_P);
        ccdr.peripheral.DMA1.enable().reset();

        let buf_rx_base_addr: u32;
        let buf_tx_base_addr: u32;
        unsafe {
            buf_rx_base_addr = buf_rx.as_ptr() as u32;
            buf_tx_base_addr = buf_rx.as_ptr() as u32;
        }
        let mut audio = device.SAI1.i2s_ch_a(
            pins_a,
            AUDIO_SAMPLE_HZ,
            sai::I2SBitRate::BITS_24,
            sai1_rec,
            &ccdr.clocks,
            Some((0, buf_rx_base_addr)),
            Some((1, buf_tx_base_addr)),
            AUDIO_BLOCK_SIZE,
        );

        audio.enable();
        unsafe {
            // core.NVIC.set_priority(interrupt::DMA1_STR0, 32);
            // stm32::NVIC::unmask(interrupt::DMA1_STR0);
            // stm32::NVIC::unmask(interrupt::SAI1);
        };

        // Setup GPIOs
        let gpio = crate::gpio::GPIO::init(gpioa, gpiob, gpioc, gpiod, gpiog);

        println!(log, "System init done!");

        println!(
            log,
            "PLL3\nP: {:?}\nQ: {:?}\nR: {:?}",
            ccdr.clocks.pll3_p_ck(),
            ccdr.clocks.pll3_q_ck(),
            ccdr.clocks.pll3_r_ck()
        );

        unsafe {
            let ptr = &*stm32::SAI1::ptr() as *const _ as *const u32;
            // println!(log, "{:#010X?}: {:#010X}", ptr, *ptr);
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(1), *(ptr.offset(1)));
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(2), *(ptr.offset(2)));
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(3), *(ptr.offset(3)));
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(4), *(ptr.offset(4)));
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(5), *(ptr.offset(5)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(6), *(ptr.offset(6)));
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(7), *(ptr.offset(7)));
            // println!(log, "{:#010X?}: {:#010X}", ptr.offset(8), *(ptr.offset(8)));

            let ptr = &*stm32::DMA1::ptr() as *const _ as *const u32;
            println!(log, "{:#010X?}: {:#010X}", ptr, *ptr);
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(1), *(ptr.offset(1)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(4), *(ptr.offset(4)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(5), *(ptr.offset(5)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(6), *(ptr.offset(6)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(7), *(ptr.offset(7)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(8), *(ptr.offset(8)));
            println!(log, "{:#010X?}: {:#010X}", ptr.offset(9), *(ptr.offset(9)));
        }

        System {
            log,
            gpio,
            // audio,
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
