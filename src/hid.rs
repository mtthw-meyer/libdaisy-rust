//! Interface abstractions for switches, potentiometer, etc.
#[allow(unused_imports)]
use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};
use stm32h7xx_hal::hal::digital::v2::InputPin;

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
    double_threshold: Option<u32>,
    held_threshold: Option<u32>,
    was_pressed: bool,
    held_counter: u32,
    last_press_counter: u32,
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
            held_counter: 0,
            last_press_counter: 0,
            single_press: false,
            double_press: false,
        }
    }

    pub fn set_held_thresh(&mut self, held_threshold: Option<u32>) {
        self.held_threshold = if let Some(held_threshold) = held_threshold {
            Some(held_threshold)
        } else {
            None
        };
    }

    pub fn set_double_thresh(&mut self, double_threshold: Option<u32>) {
        self.double_threshold = if let Some(double_threshold) = double_threshold {
            Some(double_threshold)
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

        // Handle double press logic
        if let Some(double_threshold) = self.double_threshold {
            // If we exceed the threshold for a double press reset it
            // Otherwise the counter will eventually wrap around and panic
            if self.single_press {
                self.last_press_counter += 1;
                if self.last_press_counter > double_threshold {
                    self.single_press = false;
                }
            }

            if self.falling {
                if self.single_press && self.last_press_counter < double_threshold {
                    self.double_press = true;
                    self.single_press = false;
                } else {
                    self.single_press = true;
                    self.last_press_counter = 0;
                }
            } else {
                self.double_press = false;
            }
        }

        // Handle held counter
        if is_pressed {
            self.held_counter += 1;
        }
        if self.rising {
            self.held_counter = 0;
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
            return self.falling && self.held_counter >= held_threshold;
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
        value /= ANALOG_ARR_SIZE_F32;
        if let Some(tfn) = self.transform {
            value = tfn(value);
        }
        value
    }
}
