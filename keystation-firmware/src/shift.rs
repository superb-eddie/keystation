

use arduino_hal::port::mode::Output;
use arduino_hal::port::{Pin};

pub struct ShiftRegister
{
    enable: Pin<Output>,
    input: Pin<Output>,
    clock: Pin<Output>
}

impl ShiftRegister
{

    pub fn new(enable: Pin<Output>, input: Pin<Output>, clock: Pin<Output>) -> Self {

        return Self {
            enable, input, clock
        }
    }

    pub fn enable(&mut self) {
        self.enable.set_high();
    }

    pub fn disable(&mut self) {
        self.enable.set_low();
    }

    pub fn clock(&mut self) {
        self.clock.set_high();
        self.clock.set_low();
    }

    pub fn push_high(&mut self) {
        self.input.set_high();
        self.clock();
        self.input.set_low();
    }

    pub fn push_low(&mut self) {
        self.clock();
    }
}