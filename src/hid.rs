//! Interface abstractions for switches, potentiometers, etc.
#[allow(unused_imports)]
use stm32h7xx_hal::gpio::{Analog, Input, Output, PullDown, PullUp, PushPull};
use stm32h7xx_hal::hal::digital::v2::{InputPin, OutputPin};

use debouncr::{debounce_4, Debouncer, Edge, Repeat4};
use micromath::F32Ext;

/// Define the types for a transformation function for AnalogControl
pub type TransformFn = fn(f32) -> f32;

/// If the switch is a pull-up or pull-down type
pub enum SwitchType {
    PullUp,
    PullDown,
}

/// LED blink status
pub enum BlinkStatus {
    Disabled,
    On,
    Off,
}

/// Process state information from a 2 state switch.
/// [Debouncr](https://github.com/dbrgn/debouncr/) with a 4 sample array is used for debouncing.
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
    /// Create a new Switch.
    pub fn new(pin: T, switch_type: SwitchType) -> Self {
        Self {
            pin,
            state: debounce_4(false),
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

    /// Set the threshold in number of calls to update.
    pub fn set_held_thresh(&mut self, held_threshold: Option<u32>) {
        self.held_threshold = if let Some(held_threshold) = held_threshold {
            Some(held_threshold)
        } else {
            None
        };
    }

    /// Set the threshold in number of calls to update.
    pub fn set_double_thresh(&mut self, double_threshold: Option<u32>) {
        self.double_threshold = if let Some(double_threshold) = double_threshold {
            Some(double_threshold)
        } else {
            None
        };
    }

    /// Read the state of the switch and update status. This should be called on a timer.
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

    /// If the switch state is high
    pub fn is_high(&self) -> bool {
        self.state.is_high()
    }

    /// If the switch state is low
    pub fn is_low(&self) -> bool {
        self.state.is_low()
    }

    /// If the switch is pressed
    pub fn is_pressed(&self) -> bool {
        match self.switch_type {
            SwitchType::PullUp => self.pin.is_low().unwrap(),
            SwitchType::PullDown => self.pin.is_high().unwrap(),
        }
    }

    /// If the switch is rising
    pub fn is_rising(&self) -> bool {
        self.rising
    }

    /// If the switch is falling
    pub fn is_falling(&self) -> bool {
        self.falling
    }

    /// If the switch is being held
    pub fn is_held(&self) -> bool {
        if let Some(held_threshold) = self.held_threshold {
            return self.falling && self.held_counter >= held_threshold;
        }
        false
    }

    /// If the switch pressed twice inside the provided threshold
    pub fn is_double(&self) -> bool {
        self.double_press
    }
}

const ANALOG_ARR_SIZE: usize = 4;
const ANALOG_ARR_SIZE_F32: f32 = ANALOG_ARR_SIZE as f32;

/// Contains the state of an analog control (e.g. a potentiometer).
pub struct AnalogControl<T> {
    state: [f32; ANALOG_ARR_SIZE],
    scale: f32,
    transform: Option<TransformFn>,
    pin: T,
    index: usize,
}

impl<T> AnalogControl<T> {
    /// Create a new AnalogControl.
    pub fn new(pin: T, scale: f32) -> Self {
        Self {
            state: [0.0; ANALOG_ARR_SIZE],
            scale,
            transform: None,
            pin,
            index: 0,
        }
    }

    /// Set the scaling
    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Provide an optional transformation function.
    ///
    /// # Example
    ///
    /// ```rust
    /// // Transform linear input into logarithmic
    ///let mut control1 = hid::AnalogControl::new(daisy15, adc1_max);
    ///control1.set_transform(|x| x * x);
    ///```
    pub fn set_transform(&mut self, transform: TransformFn) {
        self.transform = Some(transform);
    }

    /// Update control value. This should be called on a timer.
    /// Typically you would read data from an ADC and supply it to this.
    ///
    /// # Example
    ///
    /// ```rust
    /// if let Ok(data) = adc1.read(control1.get_pin()) {
    ///    control1.update(data);
    /// }
    /// ```
    pub fn update(&mut self, value: u32) {
        self.state[self.index] = value as f32 / self.scale;
        self.index = (self.index + 1) % ANALOG_ARR_SIZE;
    }

    /// Get the value of the control with any applied scaling and/or transformation.
    pub fn get_value(&self) -> f32 {
        let mut value = self.state.iter().sum();
        value /= ANALOG_ARR_SIZE_F32;
        if let Some(tfn) = self.transform {
            value = tfn(value);
        }
        value
    }

    /// Get the pin associated with this control.
    pub fn get_pin(&mut self) -> &mut T {
        &mut self.pin
    }
}

/// Basic LED implementation with a PWM like functional. Does not implement PWM via hardware.
pub struct Led<T> {
    pin: T,
    /// inverts the brightness level
    invert: bool,
    /// resolution is the number of brightness levels
    resolution: u32,
    brightness: f32,
    pwm: f32,
    blink_on: Option<u32>,
    blink_off: Option<u32>,
    blink_counter: u32,
    blink_status: BlinkStatus,
}

impl<T> Led<T>
where
    T: OutputPin,
{
    /// Create a new LED.
    pub fn new(pin: T, invert: bool, resolution: u32) -> Self {
        Self {
            pin,
            invert,
            resolution,
            brightness: 0.0,
            pwm: 0.0,
            blink_on: None,
            blink_off: None,
            blink_counter: 0,
            blink_status: BlinkStatus::Disabled,
        }
    }

    /// Set the brightness of the LED from 0.0 to 1.0.
    pub fn set_brightness(&mut self, value: f32) {
        let value = if value > 1.0 {
            1.0
        } else if value < 0.0 {
            0.0
        } else {
            value
        };
        match self.invert {
            // Bias for slower transitions in the low brightness range
            // TODO configurable?
            true => self.brightness = value.sqrt(),
            false => self.brightness = value * value,
        }
    }

    /// Enable blink functionality.
    /// Times are in resolution multiplied by blink_on/blink_off.
    pub fn set_blink(&mut self, blink_on: f32, blink_off: f32) {
        self.blink_on = Some((blink_on * self.resolution as f32) as u32);
        self.blink_off = Some((blink_off * self.resolution as f32) as u32);
    }

    /// Disable blink.
    pub fn clear_blink(&mut self) {
        self.blink_on = None;
        self.blink_off = None;
        self.blink_status = BlinkStatus::Disabled;
    }

    /// Update LED status. This should be called on a timer.
    pub fn update(&mut self) {
        // Calculate blink status
        if let (Some(blink_on), Some(blink_off)) = (self.blink_on, self.blink_off) {
            self.blink_counter += 1;
            self.blink_status = match self.blink_status {
                BlinkStatus::On => {
                    if self.blink_counter > blink_on {
                        self.blink_counter = 0;
                        BlinkStatus::Off
                    } else {
                        BlinkStatus::On
                    }
                }
                BlinkStatus::Off => {
                    if self.blink_counter > blink_off {
                        self.blink_counter = 0;
                        BlinkStatus::On
                    } else {
                        BlinkStatus::Off
                    }
                }
                BlinkStatus::Disabled => BlinkStatus::On,
            };
        };

        self.pwm += 1.0 / self.resolution as f32;
        if self.pwm > 1.0 {
            self.pwm -= 1.0;
        }

        let is_bright = if self.brightness > self.pwm {
            true
        } else {
            false
        };
        match self.blink_status {
            BlinkStatus::On => self.pin.set_high().ok().unwrap(),
            BlinkStatus::Off => self.pin.set_low().ok().unwrap(),
            BlinkStatus::Disabled => {
                if (is_bright && !self.invert) || (!is_bright && self.invert) {
                    self.pin.set_high().ok().unwrap();
                } else {
                    self.pin.set_low().ok().unwrap();
                }
            }
        };
    }
}
