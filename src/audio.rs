//! Audio module, handles audio startup and I/O
//! As well as converting between the S24 input and f32 for processing
use log::info;

use stm32h7xx_hal::traits::i2s::FullDuplex;
use stm32h7xx_hal::{dma, sai, sai::*, stm32};

use crate::system::{DmaBuffer, BLOCK_SIZE_MAX, DMA_BUFFER_SIZE};

// use core::marker::PhantomData;
const FBIPMAX: f32 = 0.999985;
const FBIPMIN: f32 = -FBIPMAX;
const F32_TO_S24_SCALE: f32 = 8388608.0; // 2 ** 23
const S24_TO_F32_SCALE: f32 = 1.0 / F32_TO_S24_SCALE;
const S24_SIGN: i32 = 0x800000;
pub const MAX_TRANSFER_SIZE: usize = BLOCK_SIZE_MAX * 2;

type ProgramBuffer = [u32; MAX_TRANSFER_SIZE];

type DmaInputStream = dma::Transfer<
    dma::dma::Stream1<stm32::DMA1>,
    stm32::SAI1,
    dma::PeripheralToMemory,
    &'static mut [u32; DMA_BUFFER_SIZE],
    dma::DBTransfer,
>;

type DmaOutputStream = dma::Transfer<
    dma::dma::Stream0<stm32::DMA1>,
    stm32::SAI1,
    dma::MemoryToPeripheral,
    &'static mut [u32; DMA_BUFFER_SIZE],
    dma::DBTransfer,
>;

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
    sai: sai::Sai<stm32::SAI1, sai::I2S>,
    input: Input,
    output: Output,
    input_stream: DmaInputStream,
    output_stream: DmaOutputStream,
}

impl Audio {
    pub fn new(
        mut sai: sai::Sai<stm32::SAI1, sai::I2S>,
        mut input_stream: DmaInputStream,
        mut output_stream: DmaOutputStream,
        dma_input_buffer: &'static DmaBuffer,
        dma_output_buffer: &'static mut DmaBuffer,
    ) -> Self {
        input_stream.start(|_sai1_rb| {
            sai.enable_dma(SaiChannel::ChannelB);
        });

        output_stream.start(|sai1_rb| {
            sai.enable_dma(SaiChannel::ChannelA);

            // wait until sai1's fifo starts to receive data
            info!("Sai1 fifo waiting to receive data.");
            while sai1_rb.cha.sr.read().flvl().is_empty() {}
            info!("Audio started!");
            sai.enable();
            sai.try_send(0, 0).unwrap();
        });
        let input = Input::new(dma_input_buffer);
        let output = Output::new(dma_output_buffer);
        info!(
            "{:?}, {:?}",
            &input.buffer[0] as *const u32, &output.buffer[0] as *const u32
        );
        Audio {
            sai,
            input_stream,
            output_stream,
            input,
            output,
        }
    }

    fn read(&mut self) -> bool {
        // Check interrupt(s)
        if self.input_stream.get_half_transfer_flag() {
            self.input_stream.clear_half_transfer_interrupt();
            self.input.set_index(0);
            self.output.set_index(0);
            true
        } else if self.input_stream.get_transfer_complete_flag() {
            self.input_stream.clear_transfer_complete_interrupt();
            self.input.set_index(MAX_TRANSFER_SIZE);
            self.output.set_index(MAX_TRANSFER_SIZE);
            true
        } else {
            false
        }
    }

    pub fn passthru(&mut self) {
        // Copy data
        if self.read() {
            let mut index = 0;
            let mut out_index = self.output.index;
            while index < MAX_TRANSFER_SIZE {
                self.output.buffer[out_index] = self.input.buffer[index + self.input.index];
                self.output.buffer[out_index + 1] = self.input.buffer[index + self.input.index + 1];
                index += 2;
                out_index += 2;
            }
        }
    }

    pub fn get_stereo(&mut self, data: &mut [(f32, f32); MAX_TRANSFER_SIZE / 2]) {
        // Needs an error condition of some sort
        if self.read() {
            let mut i = 0;
            for (left, right) in
                StereoIterator::new(&self.input.buffer[self.input.index..MAX_TRANSFER_SIZE])
            {
                data[i] = (left, right);
                i += 1;
            }
        }
    }

    fn get_stereo_iter(&mut self) -> Option<StereoIterator> {
        if self.read() {
            return Some(StereoIterator::new(
                &self.input.buffer[self.input.index..MAX_TRANSFER_SIZE],
            ));
        }
        None
    }

    pub fn push_stereo(&mut self, data: (f32, f32)) -> Result<(), ()> {
        return self.output.push(data);
    }
}

pub struct Input {
    index: usize,
    buffer: &'static DmaBuffer,
}

impl Input {
    /// Create a new Input from a DmaBuffer
    fn new(buffer: &'static DmaBuffer) -> Self {
        Self { index: 0, buffer }
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    /// Get StereoIterator(interleaved) iterator
    pub fn get_stereo_iter(&self) -> Option<StereoIterator> {
        Some(StereoIterator::new(&self.buffer[..2]))
    }
}

pub struct Output {
    index: usize,
    buffer: &'static mut DmaBuffer,
}

impl Output {
    /// Create a new Input from a DmaBuffer
    fn new(buffer: &'static mut DmaBuffer) -> Self {
        Self { index: 0, buffer }
    }

    fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn push(&mut self, data: (f32, f32)) -> Result<(), ()> {
        if self.index < (MAX_TRANSFER_SIZE * 2) {
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
