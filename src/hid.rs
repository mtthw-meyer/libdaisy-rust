//! Interface abstractions for switches, potentiometer, etc.
#[allow(unused_imports)]
use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};
use stm32h7xx_hal::hal::digital::v2::InputPin;

use stm32h7xx_hal::pac;
use stm32h7xx_hal::time::MilliSeconds as MS;

use crate::MILICYCLES;

use debouncr::{debounce_4, Debouncer, Edge, Repeat4};

pub type TransformFn = fn(f32) -> f32;

pub enum SwitchType {
    PullUp,
    PullDown,
}

pub struct Switch<T> {
    pin: T,
    state: Debouncer<u8, Repeat4>,
    falling: bool,
    rising: bool,
    switch_type: SwitchType,
    double_threshold: Option<MS>,
    held_threshold: Option<MS>,
    was_pressed: bool,
    start: u32,
    last_press: u32,
    single_press: bool,
    double_press: bool,
}

impl<T> Switch<T>
where
    T: InputPin,
    <T as InputPin>::Error: core::fmt::Debug,
{
    pub fn new(pin: T, switch_type: SwitchType) -> Self {
        Self {
            pin,
            state: debounce_4(),
            falling: false,
            rising: false,
            switch_type,
            double_threshold: None,
            held_threshold: None,
            was_pressed: false,
            start: 0,
            last_press: 0,
            single_press: false,
            double_press: false,
        }
    }

    pub fn set_held_thresh<S>(&mut self, held_threshold: Option<S>)
    where
        S: Into<MS>,
    {
        self.held_threshold = if let Some(held_threshold) = held_threshold {
            Some(held_threshold.into())
        } else {
            None
        };
    }

    pub fn set_double_thresh<S>(&mut self, double_threshold: Option<S>)
    where
        S: Into<MS>,
    {
        self.double_threshold = if let Some(double_threshold) = double_threshold {
            Some(double_threshold.into())
        } else {
            None
        };
    }

    pub fn update(&mut self) {
        let is_pressed = self.is_pressed();

        // Handle event
        if let Some(edge) = self.state.update(is_pressed) {
            match edge {
                Edge::Falling => self.falling = true,
                Edge::Rising => self.rising = true,
            }
        } else {
            self.falling = false;
            self.rising = false;
        }

        if is_pressed {
            if !self.was_pressed {
                self.start = pac::DWT::get_cycle_count();
                self.was_pressed = true;
            }
        }
        // Handle edge on release
        else if self.was_pressed {
            if let Some(double_threshold) = self.double_threshold {
                let now = pac::DWT::get_cycle_count();

                // If it's a double press set it to true
                if self.single_press && (now - self.last_press) / MILICYCLES < double_threshold.0 {
                    self.double_press = true;
                    self.single_press = false;
                // Else set the last press to now
                } else {
                    self.last_press = now;
                    self.single_press = true;
                }
            }
            self.was_pressed = false;
        }
    }

    pub fn is_high(&self) -> bool {
        self.state.is_high()
    }

    pub fn is_low(&self) -> bool {
        self.state.is_low()
    }

    pub fn is_pressed(&self) -> bool {
        match self.switch_type {
            SwitchType::PullUp => self.pin.is_low().unwrap(),
            SwitchType::PullDown => self.pin.is_high().unwrap(),
        }
    }

    pub fn is_rising(&self) -> bool {
        self.rising
    }

    pub fn is_falling(&self) -> bool {
        self.falling
    }

    pub fn is_held(&self) -> bool {
        if let Some(held_threshold) = self.held_threshold {
            if self.is_pressed() {
                return (pac::DWT::get_cycle_count() - self.start) / MILICYCLES > held_threshold.0;
            }
        }
        false
    }

    pub fn is_double(&self) -> bool {
        self.double_press
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
