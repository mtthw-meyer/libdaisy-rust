//! Interface abstractions for switches, potentiometer, etc.
#[allow(unused_imports)]
use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};
use stm32h7xx_hal::hal::digital::v2::InputPin;

use debouncr::{debounce_4, Debouncer, Edge, Repeat4};

// Trait for analogue state controls (e.g. potentiometer)
pub trait AnalogueControl {
    fn update(&mut self);
    fn value(&self);
}

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
