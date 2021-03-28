use stm32_fmc::devices::as4c16m32msa_6;
use stm32h7xx_hal::{
    gpio::{gpiod, gpioe, gpiof, gpiog, gpioh, gpioi, Analog},
    hal::blocking::delay::DelayUs,
    prelude::*,
    rcc, stm32,
};

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

pub struct Sdram {
    inner: *mut u32,
}

impl Sdram {
    pub fn new<D: DelayUs<u8>>(
        fmc_d: stm32::FMC,
        fmc_p: rcc::rec::Fmc,
        clocks: &rcc::CoreClocks,
        delay: &mut D,
        scb: &mut cortex_m::peripheral::SCB,
        mpu: &mut cortex_m::peripheral::MPU,
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
    ) -> Self {
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

        let ram_ptr = fmc_d
            .sdram(sdram_pins, as4c16m32msa_6::As4c16m32msa {}, fmc_p, clocks)
            .init(delay);
        mpu_sdram_init(mpu, scb, ram_ptr, Self::bytes());
        Self { inner: ram_ptr }
    }

    pub const fn bytes() -> usize {
        64 * 1024 * 1024
    }
}

impl<T: Sized> Into<&'static mut [T]> for Sdram {
    fn into(self) -> &'static mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.inner as *mut T,
                Self::bytes() / core::mem::size_of::<T>(),
            )
        }
    }
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
pub fn mpu_sdram_init(
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

    //info!("SDRAM Memory Size 0x{:x}", log2minus1(size as u32));

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
