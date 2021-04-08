//!Configure MPU

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

/// Refer to ARM®v7-M Architecture Reference Manual ARM DDI 0403
/// Version E.b Section B3.5
use log::info;

const MEMFAULTENA: u32 = 1 << 16;
const REGION_FULL_ACCESS: u32 = 0x03;
const REGION_ENABLE: u32 = 0x01;

pub fn disable(mpu: &mut cortex_m::peripheral::MPU, scb: &mut cortex_m::peripheral::SCB) {
    unsafe {
        /* Make sure outstanding transfers are done */
        cortex_m::asm::dmb();
        scb.shcsr.modify(|r| r & !MEMFAULTENA);
        /* Disable the MPU and clear the control register*/
        mpu.ctrl.write(0);
    }
}

pub fn enable(mpu: &mut cortex_m::peripheral::MPU, scb: &mut cortex_m::peripheral::SCB) {
    const MPU_ENABLE: u32 = 0x01;
    const MPU_DEFAULT_MMAP_FOR_PRIVILEGED: u32 = 0x04;

    unsafe {
        mpu.ctrl
            .modify(|r| r | MPU_DEFAULT_MMAP_FOR_PRIVILEGED | MPU_ENABLE);

        scb.shcsr.modify(|r| r | MEMFAULTENA);

        // Ensure MPU settings take effect
        cortex_m::asm::dsb();
        cortex_m::asm::isb();
    }
}

fn log2minus1(sz: u32) -> u32 {
    for x in 5..=31 {
        if sz == (1 << x) {
            return x - 1;
        }
    }
    panic!("Unknown memory region size!");
}

pub fn dma_init(
    mpu: &mut cortex_m::peripheral::MPU,
    scb: &mut cortex_m::peripheral::SCB,
    location: *mut u32,
    size: usize,
) {
    disable(mpu, scb);

    const REGION_NUMBER0: u32 = 0x00;
    const REGION_SHAREABLE: u32 = 0x01;
    const REGION_TEX: u32 = 0b001;
    const REGION_CB: u32 = 0b00;

    assert_eq!(
        size & (size - 1),
        0,
        "Memory region size must be a power of 2"
    );
    assert_eq!(
        size & 0x1F,
        0,
        "Memory region size must be 32 bytes or more"
    );

    info!("Memory Size 0x{:x}", log2minus1(size as u32));

    // Configure region 0
    //
    // Strongly ordered
    unsafe {
        mpu.rnr.write(REGION_NUMBER0);
        mpu.rbar.write((location as u32) & !0x1F);
        mpu.rasr.write(
            (REGION_FULL_ACCESS << 24)
                | (REGION_TEX << 19)
                | (REGION_SHAREABLE << 18)
                | (REGION_CB << 16)
                | (log2minus1(size as u32) << 1)
                | REGION_ENABLE,
        );
    }

    enable(mpu, scb);
}

pub fn sdram_init(
    mpu: &mut cortex_m::peripheral::MPU,
    scb: &mut cortex_m::peripheral::SCB,
    location: *mut u32,
    size: usize,
) {
    disable(mpu, scb);

    // SDRAM
    const REGION_NUMBER1: u32 = 0x01;

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

    enable(mpu, scb);
}
