use debouncr::{debounce_4, Debouncer, Edge, Repeat4};

// use embedded_hal;
use stm32h7xx_hal::gpio;
use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};
use stm32h7xx_hal::hal::digital::v2::InputPin;

const CONTROL_ARRAY_SIZE: usize = 20;


/// Struct to handle updating all controls globally
/// Register any control that implements `SwitchControl` or `AnalogueControl`
pub struct Interface<'a> {
    pub switch_controls: [Option<&'a mut dyn SwitchControl>; 20],
    pub analogue_controls: [Option<&'a mut dyn AnalogueControl>; 12],
    pub ready: bool,
}

// Is this safe?
unsafe impl Send for Interface<'_> {}

impl<'a> Interface<'a> {
    pub fn register_switch(&mut self, control: &'a mut dyn SwitchControl) -> Result<(), ()> {
        if let Some(index) = self.switch_controls.iter().position(|x| x.is_none()) {
            self.switch_controls[index] = Some(control);
            return Ok(());
        }
        Err(())
    }

    pub fn register_analogue(&mut self, control: &'a mut dyn AnalogueControl) -> Result<(), ()> {
        if let Some(index) = self.switch_controls.iter().position(|x| x.is_none()) {
            self.analogue_controls[index] = Some(control);
            return Ok(());
        }
        Err(())
    }

    pub fn set_ready(&mut self) {
        self.ready = true;
    }

    pub fn update(&mut self) {
        if self.ready {
            for control in &mut self.switch_controls {
                if let Some(control) = control {
                    control.update();
                }
            }
            for control in &mut self.analogue_controls {
                if let Some(control) = control {
                    control.update();
                }
            }
        }
    }
}

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

pub struct Switch<'a, T> {
    state: Debouncer<u8, Repeat4>,
    falling: bool,
    rising: bool,
    pin: &'a dyn InputPin<Error = T>,
}

impl<'a, T> Switch<'a, T> {
    pub fn new(pin:  &'a dyn InputPin<Error = T> ) -> Self {
        Self {
            state: debounce_4(),
            falling: false,
            rising: false,
            pin,
        }
    }
}

impl<T> SwitchControl for Switch<'_, T> {
    fn update(&mut self) {
        // Handle event
        if let Some(edge) = self.state.update(false) {
            match edge {
                Edge::Falling => self.falling = true,
                Edge::Rising => self.rising = true,
            }
        } else {
            self.falling = true;
            self.rising = true;
        }
    }

    fn is_high(&self) -> bool {
        self.state.is_high()
    }

    fn is_low(&self) -> bool {
        self.state.is_low()
    }

    fn is_rising(&self) -> bool {
        self.rising
    }
        
    fn is_falling(&self) -> bool {
        self.falling
    }
}