#![allow(dead_code)]
#![allow(unused_variables)]

use rtt_target::rprintln as println;
pub use stm32h7xx_hal::hal::digital::v2::OutputPin;

use cortex_m::peripheral::DWT;
use cortex_mpu;

use rtic;

use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::rcc;
use stm32h7xx_hal::rcc::rec::ResetEnable;
use stm32h7xx_hal::sai::*;
use stm32h7xx_hal::stm32::rcc::d2ccip1r::SAI1SEL_A;
use stm32h7xx_hal::stm32::{TIM1, TIM12, TIM17, TIM2};
use stm32h7xx_hal::timer::{Event, Timer};
use stm32h7xx_hal::{device, dma, dma::DmaExt, gpio, interrupt, sai, stm32};

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
pub const BUFFER_SIZE: usize = BLOCK_SIZE_MAX * 2;

pub type IoBuffer = [u32; BUFFER_SIZE];

const SLOTS: u8 = 2;
const FIRST_BIT_OFFSET: u8 = 0;

// 805306368 805306368

#[link_section = ".sram1_bss"]
#[no_mangle]
static mut buf_tx: IoBuffer = [0; BUFFER_SIZE];
#[link_section = ".sram1_bss"]
#[no_mangle]
static mut buf_rx: IoBuffer = [0; BUFFER_SIZE];

pub struct System {
    pub gpio: crate::gpio::GPIO,
    pub audio: sai::Sai<stm32::SAI1, sai::I2S>,
    pub exit: stm32::EXTI,
    pub syscfg: stm32::SYSCFG,
}

impl System {
    pub fn init(_: rtic::Peripherals, device: stm32::Peripherals) -> System {
        // pub fn init(mut core: cortex_m::peripheral::Peripherals, device: stm32::Peripherals) -> System {

        println!("Starting system init");
        // Power
        let pwr = device.PWR.constrain();
        let vos = pwr.vos0(&device.SYSCFG).freeze();

        // Clocks
        let mut ccdr = device
            .RCC
            .constrain()
            .use_hse(HSE_CLOCK_MHZ)
            .sys_ck(CLOCK_RATE_HZ)
            .pclk1(PCLK_HZ) // DMA clock
            // PLL1
            .pll1_strategy(rcc::PllConfigStrategy::Iterative)
            .pll1_p_ck(PLL1_P_HZ)
            .pll1_q_ck(PLL1_Q_HZ)
            .pll1_r_ck(PLL1_R_HZ)
            // PLL2
            // .pll2_p_ck(PLL2_P_HZ)
            // .pll2_q_ck(PLL2_Q_HZ)
            // .pll2_r_ck(PLL2_R_HZ)
            // PLL3
            .pll3_strategy(rcc::PllConfigStrategy::Iterative)
            .pll3_p_ck(PLL3_P_HZ)
            .pll3_q_ck(PLL3_Q_HZ)
            .pll3_r_ck(PLL3_R_HZ)
            .freeze(vos, &device.SYSCFG);

        // let pll3_ck_hz = ccdr.clocks.pll3_p_ck().unwrap();
        // Figure 48 shows the recommended PLL initialization sequence in integer and fractional
        // mode. The PLLx are supposed to be disabled at the start of the initialization sequence:
        // 1. Initialize the PLLs registers according to the required frequency.
        // – Set PLLxFRACEN of RCC PLLs Configuration Register (RCC_PLLCFGR) to ‘0’
        // for integer mode.
        // – For fractional mode, set FRACN to the required initial value (FracInitValue) and
        // then set PLLxFRACEN to ‘1’.
        // 2. Once the PLLxON bit is set to ‘1’, the user application has to wait until PLLxRDY bit is
        // set to ‘1’. If the PLLx is in fractional mode, the PLLxFRACEN bit must not be set back
        // to ‘0’ as long as PLLxRDY = ‘0’.
        // 3. Once the PLLxRDY bit is set to ‘1’, the PLLx is ready to be used.
        // 4. If the application intends to tune the PLLx frequency on-the-fly (possible only in
        // fractional mode), then:
        // a) PLLxFRACEN must be set to ‘0’,
        // When PLLxFRACEN = ‘0’, the Sigma-Delta modulator is still operating with the
        // value latched into SH_REG.
        // b) A new value must be uploaded into PLLxFRACR (FracValue(n)).
        // c) PLLxFRACEN must be set to ‘1’, in order to latch the content of PLLxFRACR into
        // its shadow register.

        print_clocks(&ccdr);
        // TODO - Use stm32h7-fmc to setup SDRAM?
        // https://crates.io/crates/stm32h7-fmc
        // https://github.com/electro-smith/libDaisy/blob/04479d151dc275203a02e64fbfa2ab2bf6c0a91a/src/dev_sdram.c

        let mut core = device::CorePeripherals::take().unwrap();
        // MPU
        // Configure MPU per Seed
        // https://github.com/electro-smith/libDaisy/blob/04479d151dc275203a02e64fbfa2ab2bf6c0a91a/src/sys_system.c
        // core.MPU.
        // let mpu = unsafe { cortex_mpu::Mpu::new(core.MPU) };

        // Timers
        // TODO
        // ?
        core.DCB.enable_trace();
        DWT::unlock();
        core.DWT.enable_cycle_counter();

        let mut timer2 = device
            .TIM2
            .timer(1000.ms(), ccdr.peripheral.TIM2, &mut ccdr.clocks);
        timer2.listen(Event::TimeOut);

        let mut timer3 = device
            .TIM3
            .timer(1.ms(), ccdr.peripheral.TIM3, &mut ccdr.clocks);
        timer3.listen(Event::TimeOut);

        // println!("Setting up GPIOs...");
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
        // println!("Setting up QSPI...");
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
        println!("Setup up SAI...");

        let sai1_rec = ccdr.peripheral.SAI1.kernel_clk_mux(SAI1SEL_A::PLL3_P);
        let master_config =
            I2SChanConfig::new(I2SDir::Tx).set_frame_sync_active_high(true);
        let slave_config = I2SChanConfig::new(I2SDir::Rx)
            .set_sync_type(I2SSync::Internal)
            .set_frame_sync_active_high(true);

        let audio = device.SAI1.i2s_ch_a(
            pins_a,
            AUDIO_SAMPLE_HZ,
            I2SDataSize::BITS_24,
            sai1_rec,
            &ccdr.clocks,
            master_config,
            Some(slave_config),
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
        let mut gpio = crate::gpio::GPIO::init(gpioa, gpiob, gpioc, gpiod, gpiog);
        gpio.reset_codec();

        // Setup cache
        core.SCB.invalidate_icache();
        core.SCB.enable_icache();
        // core.SCB.clean_invalidate_dcache(&mut core.CPUID);
        // core.SCB.enable_dcache(&mut core.CPUID);

        println!("System init done!");

        System {
            gpio,
            audio,
            exit: device.EXTI,
            syscfg: device.SYSCFG,
        }
    }
}

fn print_clocks(ccdr: &stm32h7xx_hal::rcc::Ccdr) {
    println!("Core {}", ccdr.clocks.c_ck());
    println!("pclk1 {}", ccdr.clocks.pclk1());
    println!("pclk2 {}", ccdr.clocks.pclk2());
    println!("pclk3 {}", ccdr.clocks.pclk2());
    println!("pclk4 {}", ccdr.clocks.pclk4());
    // println!(
    //     log,
    //     "PLL1\nP: {:?}\nQ: {:?}\nR: {:?}",
    //     ccdr.clocks.pll1_p_ck(),
    //     ccdr.clocks.pll1_q_ck(),
    //     ccdr.clocks.pll1_r_ck()
    // );
    // println!(
    //     log,
    //     "PLL2\nP: {:?}\nQ: {:?}\nR: {:?}",
    //     ccdr.clocks.pll2_p_ck(),
    //     ccdr.clocks.pll2_q_ck(),
    //     ccdr.clocks.pll2_r_ck()
    // );
    println!(
        "PLL3\nP: {:?}\nQ: {:?}\nR: {:?}",
        ccdr.clocks.pll3_p_ck(),
        ccdr.clocks.pll3_q_ck(),
        ccdr.clocks.pll3_r_ck()
    );
}
