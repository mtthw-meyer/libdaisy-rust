//! Audio module, handles audio startup and I/O
//! As well as converting between the S24 input and f32 for processing
use stm32h7xx_hal::gpio::{gpioe, Analog};
use stm32h7xx_hal::rcc;
use stm32h7xx_hal::stm32::rcc::d2ccip1r::SAI1SEL_A;
use stm32h7xx_hal::traits::i2s::FullDuplex;
use stm32h7xx_hal::{sai, sai::*, stm32};

// use core::marker::PhantomData;
const FBIPMAX: f32 = 0.999985;
const FBIPMIN: f32 = -FBIPMAX;
const F32_TO_S24_SCALE: f32 = 8388608.0; // 2 ** 23
const S24_TO_F32_SCALE: f32 = 1.0 / F32_TO_S24_SCALE;
const S24_SIGN: i32 = 0x800000;

// Process samples at 1000 Hz
// With a circular buffer(*2) in stereo (*2)
pub const BLOCK_SIZE_MAX: usize = 48;
pub const BUFFER_SIZE: usize = BLOCK_SIZE_MAX * 2 * 2;

pub type IoBuffer = [u32; BUFFER_SIZE];

// 805306368 805306368

#[link_section = ".sram1_bss"]
#[no_mangle]
static mut buf_tx: IoBuffer = [0; BUFFER_SIZE];
#[link_section = ".sram1_bss"]
#[no_mangle]
static mut buf_rx: IoBuffer = [0; BUFFER_SIZE];

type StereoIteratorHandle = fn(StereoIterator, &mut Output);

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct S24(pub i32);

impl From<i32> for S24 {
    fn from(x: i32) -> S24 {
        S24(x)
    }
}

impl From<u32> for S24 {
    fn from(x: u32) -> S24 {
        S24(x as i32)
    }
}

impl From<S24> for i32 {
    fn from(x: S24) -> i32 {
        x.0
    }
}

impl From<S24> for u32 {
    fn from(x: S24) -> u32 {
        x.0 as u32
    }
}

impl From<f32> for S24 {
    fn from(x: f32) -> S24 {
        let x = if x <= FBIPMIN {
            FBIPMIN
        } else if x >= FBIPMAX {
            FBIPMAX
        } else {
            x
        };
        S24((x * F32_TO_S24_SCALE) as i32)
    }
}

impl From<S24> for f32 {
    fn from(x: S24) -> f32 {
        ((x.0 ^ S24_SIGN) - S24_SIGN) as f32 * S24_TO_F32_SCALE
    }
}

pub struct Audio {
    pub stream: sai::Sai<stm32::SAI1, sai::I2S>,
    pub input: Input,
    pub output: Output,
}

impl Audio {
    pub fn init(
        sai1: rcc::rec::Sai1,
        sai1_d: stm32::SAI1,
        clocks: &rcc::CoreClocks,
        ee2: gpioe::PE2<Analog>,
        ee3: gpioe::PE3<Analog>,
        ee4: gpioe::PE4<Analog>,
        ee5: gpioe::PE5<Analog>,
        ee6: gpioe::PE6<Analog>,
    ) -> Self {
        let sai1_rec = sai1.kernel_clk_mux(SAI1SEL_A::PLL3_P);
        let master_config = I2SChanConfig::new(I2SDir::Tx).set_frame_sync_active_high(true);
        let slave_config = I2SChanConfig::new(I2SDir::Rx)
            .set_sync_type(I2SSync::Internal)
            .set_frame_sync_active_high(true);

        let pins_a = (
            ee2.into_alternate_af6(),       // MCLK_A
            ee5.into_alternate_af6(),       // SCK_A
            ee4.into_alternate_af6(),       // FS_A
            ee6.into_alternate_af6(),       // SD_A
            Some(ee3.into_alternate_af6()), // SD_B
        );

        let dev_audio = sai1_d.i2s_ch_a(
            pins_a,
            crate::AUDIO_SAMPLE_HZ,
            I2SDataSize::BITS_24,
            sai1_rec,
            &clocks,
            master_config,
            Some(slave_config),
        );
        unsafe { Self::new(dev_audio, &mut buf_rx, &mut buf_tx) }
    }
    pub fn new(
        mut stream: sai::Sai<stm32::SAI1, sai::I2S>,
        input: &'static mut IoBuffer,
        output: &'static mut IoBuffer,
    ) -> Self {
        stream.listen(SaiChannel::ChannelB, Event::Data);
        stream.enable();
        stream.try_send(0, 0).unwrap();
        Audio {
            stream,
            input: Input { buffer: input },
            output: Output::new(output),
        }
    }

    pub fn read(&mut self) {
        self.stream
            .clear_irq(sai::SaiChannel::ChannelB, sai::Event::Data);
        self.output.reset();
        if let Ok((left, right)) = self.stream.try_read() {
            self.input.buffer[0] = left;
            self.input.buffer[1] = right;
        }
    }

    pub fn send(&mut self) {
        let left = self.output.buffer[0];
        let right = self.output.buffer[1];
        self.stream.try_send(left, right).unwrap();
    }

    // pub fn get_left(&self)
}

pub struct Input {
    buffer: &'static mut IoBuffer,
}

impl Input {
    /// Get StereoIterator(interleaved) iterator
    pub fn get_stereo_iter(&self) -> Option<StereoIterator> {
        Some(StereoIterator::new(&self.buffer[..2]))
    }
}

pub struct Output {
    index: usize,
    buffer: &'static mut IoBuffer,
}

impl Output {
    fn new(buffer: &'static mut IoBuffer) -> Self {
        Self { index: 0, buffer }
    }

    fn reset(&mut self) {
        self.index = 0;
    }

    pub fn push(&mut self, data: (f32, f32)) -> Result<(), ()> {
        if self.index < (BLOCK_SIZE_MAX * 2) {
            self.buffer[self.index] = S24::from(data.0).into();
            self.buffer[self.index + 1] = S24::from(data.1).into();
            self.index += 2;
            return Ok(());
        }
        Err(())
    }
}

pub struct StereoIterator<'a> {
    index: usize,
    buf: &'a [u32],
}

impl<'a> StereoIterator<'a> {
    fn new(buf: &'a [u32]) -> Self {
        Self { index: 0, buf }
    }
}

impl Iterator for StereoIterator<'_> {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buf.len() {
            self.index += 2;
            Some((
                S24(self.buf[self.index - 2] as i32).into(),
                S24(self.buf[self.index - 1] as i32).into(),
            ))
        } else {
            None
        }
    }
}

pub struct Mono<'a> {
    index: usize,
    buf: &'a [i32],
}

impl<'a> Mono<'a> {
    fn new(buf: &'a [i32]) -> Self {
        Self { index: 0, buf }
    }
}

impl Iterator for Mono<'_> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.buf.len() {
            self.index += 2;
            Some(S24(self.buf[self.index - 1]).into())
        } else {
            None
        }
    }
}
