//! Interface abstractions for switches, potentiometer, etc.
#[allow(unused_imports)]
use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};
use stm32h7xx_hal::hal::digital::v2::InputPin;

use debouncr::{debounce_4, Debouncer, Edge, Repeat4};

pub type TransformFn = fn(f32) -> f32;
// Trait for Analog state controls (e.g. potentiometer)
// pub trait AnalogControl {
//     fn update(&mut self);
//     fn set_transform(&mut self, transform: Option<TransformFn>);
//     fn get_value(&self) -> f32;
// }

/// Trait for binary state controls (e.g. a switch)
pub trait SwitchControl {
    fn update(&mut self);
    fn is_high(&self) -> bool;
    fn is_low(&self) -> bool;
    fn is_rising(&self) -> bool;
    fn is_falling(&self) -> bool;
}

pub struct Switch<T> {
    state: Debouncer<u8, Repeat4>,
    falling: bool,
    rising: bool,
    pin: T,
}

impl<T> Switch<T>
where
    T: InputPin,
    <T as InputPin>::Error: core::fmt::Debug,
{
    pub fn new(pin: T) -> Self {
        Self {
            state: debounce_4(),
            falling: false,
            rising: false,
            pin,
        }
    }

    pub fn update(&mut self) {
        // Handle event
        let pressed = self.pin.is_low().unwrap();
        if let Some(edge) = self.state.update(pressed) {
            match edge {
                Edge::Falling => self.falling = true,
                Edge::Rising => self.rising = true,
            }
        } else {
            self.falling = false;
            self.rising = false;
        }
    }

    pub fn is_high(&self) -> bool {
        self.state.is_high()
    }

    pub fn is_low(&self) -> bool {
        self.state.is_low()
    }

    pub fn is_rising(&self) -> bool {
        self.rising
    }

    pub fn is_falling(&self) -> bool {
        self.falling
    }
}

const ANALOG_ARR_SIZE: usize = 4;
const ANALOG_ARR_SIZE_F32: f32 = ANALOG_ARR_SIZE as f32;

pub struct AnalogControl<T> {
    state: [f32; ANALOG_ARR_SIZE],
    scale: f32,
    transform: Option<TransformFn>,
    pub pin: T,
    index: usize,
}

impl<T> AnalogControl<T> {
    pub fn new(pin: T, scale: f32) -> Self {
        Self {
            state: [0.0; ANALOG_ARR_SIZE],
            scale,
            transform: None,
            pin,
            index: 0,
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn set_transform(&mut self, transform: TransformFn) {
        self.transform = Some(transform);
    }

    pub fn update(&mut self, value: u32) {
        self.state[self.index] = value as f32 / self.scale;
        self.index = (self.index + 1) % ANALOG_ARR_SIZE;
    }

    pub fn get_value(&self) -> f32 {
        let mut value = self.state.iter().sum();
        value = value / ANALOG_ARR_SIZE_F32;
        if let Some(tfn) = self.transform {
            value = tfn(value);
        }
        value
    }
}
