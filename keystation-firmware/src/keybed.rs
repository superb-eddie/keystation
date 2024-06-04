use arduino_hal::port::mode::{Floating, Input};
use arduino_hal::port::Pin;

use crate::millis::millis;
use crate::shift::ShiftRegister;

// The key matrix has 8 outputs and 14 inputs to read from 49 total keys.
// Each key has two contacts, to calculate velocity.

const KEYS: usize = 49;

// Ownership rules means we can't just pass `pins` to the constructor, a macro gets around that
#[macro_export]
macro_rules! keybed_init {
    ( $p:expr ) => {{
        let shift = ShiftRegister::new(
            $p.d13.into_output().downgrade(),
            $p.d12.into_output().downgrade(),
            $p.d11.into_output().downgrade(),
        );

        let keys_a: [Pin<Input<Floating>>; 7] = [
            $p.d6.downgrade(),
            $p.d3.downgrade(),
            $p.a5.downgrade(),
            $p.a3.downgrade(),
            $p.a1.downgrade(),
            $p.d10.downgrade(),
            $p.d8.downgrade(),
        ];

        let keys_b: [Pin<Input<Floating>>; 7] = [
            $p.d5.downgrade(),
            $p.d4.downgrade(),
            $p.a4.downgrade(),
            $p.a2.downgrade(),
            $p.a0.downgrade(),
            $p.d9.downgrade(),
            $p.d7.downgrade(),
        ];

        Keybed::new(shift, keys_a, keys_b)
    }};
}

#[derive(Copy, Clone)]
pub enum Key {
    // Key goes through these states in order
    // B up, A up
    Up,
    // B down, A up (Key may go to "up" after "partial")
    DownPartial(u32), // millis when B triggered
    // B down, A down (Key may only go to "down" after "partial")
    Down(u32), // millis travel time
}

pub struct Keybed {
    // Shift register selects which 7 keys we're reading
    shift: ShiftRegister,

    // Each key has two contacts under it
    // When a key is pressed, it hits 'b' first and then 'a'
    keys_a: [Pin<Input<Floating>>; 7],
    keys_b: [Pin<Input<Floating>>; 7],

    pub keys: [Key; KEYS],
}

impl Keybed {
    pub fn new(
        mut shift: ShiftRegister,
        keys_a: [Pin<Input<Floating>>; 7],
        keys_b: [Pin<Input<Floating>>; 7],
    ) -> Self {
        shift.enable();

        return Self {
            shift,
            keys_a,
            keys_b,
            keys: [Key::Up; KEYS],
        };
    }

    pub fn scan(&mut self) {
        // Scan key matrix
        for i in 0..8 {
            if i == 0 {
                self.shift.push_high()
            } else {
                self.shift.push_low()
            }
            for j in 0..7 {
                let (key_index, _) =
                    ((j * 8) + (if i == 0 { 8usize } else { i })).overflowing_sub(1);
                if key_index >= KEYS {
                    //  This matrix support more keys than we actually have
                    continue;
                }

                // let key = &mut keys[key_index];
                let a_down = self.keys_a[j].is_high();
                let b_down = self.keys_b[j].is_high();

                if !b_down {
                    // Key is always up if B is up
                    self.keys[key_index] = Key::Up;
                    continue;
                }

                match (a_down, self.keys[key_index]) {
                    (false, Key::Up) => {
                        // Key is being pressed
                        self.keys[key_index] = Key::DownPartial(millis());
                    }
                    (true, Key::DownPartial(at)) => {
                        // Key was fully pressed
                        let travel_time = millis().saturating_sub(at);
                        self.keys[key_index] = Key::Down(travel_time);
                    }
                    (false, Key::Down(_velocity)) => {
                        // Key is being lifted
                        self.keys[key_index] = Key::Up
                    }
                    _ => {
                        //     Nothing happens in every other case
                    }
                }
            }
        }
    }
}
