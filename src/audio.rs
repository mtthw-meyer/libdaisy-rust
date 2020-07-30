//! Audio module, handles audio startup and I/O
//! As well as converting between the S24 input and f32 for processing
use stm32h7xx_hal::{sai, sai::*, stm32};

use crate::system::{IoBuffer, BLOCK_SIZE_MAX};

// use core::marker::PhantomData;
const FBIPMAX: f32 = 0.999985;
const FBIPMIN: f32 = -FBIPMAX;
const F32_TO_S24_SCALE: f32 = 8388608.0; // 2 ** 23
const S24_TO_F32_SCALE: f32 = 1.0 / F32_TO_S24_SCALE;
const S24_SIGN: i32 = 0x800000;

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
