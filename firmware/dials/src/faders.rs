use arduino_hal::adc;
use arduino_hal::hal::port::{PC0, PC1};
use arduino_hal::port::mode::Analog;
use arduino_hal::port::Pin;

#[macro_export]
macro_rules! faders_init {
    ( $p:expr, $a:expr ) => {crate::faders::Faders::new(
        $p.a0.into_analog_input(&mut $a),
        $p.a1.into_analog_input(&mut $a),
    )};
}

pub struct Faders {
    pitch: Pin<Analog, PC0>,
    modulation: Pin<Analog, PC1>,
}

impl Faders {
    pub fn new(
        pitch: Pin<Analog, PC0>,
        modulation: Pin<Analog, PC1>,
    ) -> Self {
        Self { pitch, modulation }
    }
    pub fn read_volume(&mut self, adc: &mut adc::Adc) -> u16 {
        adc.read_blocking(&adc::channel::ADC7)
    }

    pub fn read_pitch(&mut self, adc: &mut adc::Adc) -> u16 {
        self.pitch.analog_read(adc)
    }

    pub fn read_modulation(&mut self, adc: &mut adc::Adc) -> u16 {
        self.modulation.analog_read(adc)
    }
}