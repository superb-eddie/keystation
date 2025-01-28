use std::thread;
use std::time::Duration;

use crossbeam::channel::Sender;
use rppal::gpio::{Event, Gpio, Trigger};

use crate::midi_sender::MidiEvent;
use crate::user_interface::{Button, UIEvent};

const SUSTAIN_GPIO_PIN: u8 = 20;

const DPAD_U_GPIO_PIN: u8 = 17;
const DPAD_D_GPIO_PIN: u8 = 22;
const DPAD_R_GPIO_PIN: u8 = 23;
const DPAD_L_GPIO_PIN: u8 = 27;
const DPAD_C_GPIO_PIN: u8 = 4;

const A_GPIO_PIN: u8 = 5;
const B_GPIO_PIN: u8 = 13;

// Start a thread to poll for gpio interrupts and translate them to events
pub fn start_gpio_driver(
    midi_channel: Sender<MidiEvent>,
    ui_channel: Sender<UIEvent>,
) -> anyhow::Result<()> {
    let gpio = Gpio::new()?;
    let debounce_duration = Some(Duration::from_millis(1));

    let mut sustain_pin = gpio.get(SUSTAIN_GPIO_PIN)?.into_input_pulldown();
    sustain_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut dpad_u_pin = gpio.get(DPAD_U_GPIO_PIN)?.into_input_pullup();
    dpad_u_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut dpad_d_pin = gpio.get(DPAD_D_GPIO_PIN)?.into_input_pullup();
    dpad_d_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut dpad_r_pin = gpio.get(DPAD_R_GPIO_PIN)?.into_input_pullup();
    dpad_r_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut dpad_l_pin = gpio.get(DPAD_L_GPIO_PIN)?.into_input_pullup();
    dpad_l_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut dpad_c_pin = gpio.get(DPAD_C_GPIO_PIN)?.into_input_pullup();
    dpad_c_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut a_pin = gpio.get(A_GPIO_PIN)?.into_input_pullup();
    a_pin.set_interrupt(Trigger::Both, debounce_duration)?;
    let mut b_pin = gpio.get(B_GPIO_PIN)?.into_input_pullup();
    b_pin.set_interrupt(Trigger::Both, debounce_duration)?;

    thread::spawn(move || {
        let pins = vec![
            &sustain_pin,
            &dpad_u_pin,
            &dpad_d_pin,
            &dpad_r_pin,
            &dpad_l_pin,
            &dpad_c_pin,
            &a_pin,
            &b_pin,
        ];

        loop {
            let (pin, interrupt) = gpio.poll_interrupts(&pins, false, None).unwrap().unwrap();

            match pin.pin() {
                DPAD_U_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::DpadUp),
                DPAD_D_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::DpadDown),
                DPAD_R_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::DpadRight),
                DPAD_L_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::DpadLeft),
                DPAD_C_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::DpadCenter),
                A_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::A),
                B_GPIO_PIN => send_button_event(&ui_channel, interrupt, Button::B),
                SUSTAIN_GPIO_PIN => {
                    midi_channel
                        .try_send(match interrupt.trigger {
                            Trigger::RisingEdge => MidiEvent::SustainOn,
                            Trigger::FallingEdge => MidiEvent::SustainOff,
                            _ => break,
                        })
                        .expect("couldn't send midi event");
                }
                _ => {}
            }
        }
    });

    Ok(())
}

fn send_button_event(ui_channel: &Sender<UIEvent>, interrupt: Event, button: Button) {
    ui_channel
        .try_send(match interrupt.trigger {
            Trigger::RisingEdge => UIEvent::Up(button),
            Trigger::FallingEdge => UIEvent::Down(button),
            _ => return,
        })
        .expect("couldn't send ui event");
}
