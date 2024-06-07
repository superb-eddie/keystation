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
        shift: ShiftRegister,
        keys_a: [Pin<Input<Floating>>; 7],
        keys_b: [Pin<Input<Floating>>; 7],
    ) -> Self {
        return Self {
            shift,
            keys_a,
            keys_b,
            keys: [Key::Up; KEYS],
        };
    }

    fn next_state(a_down: bool, b_down: bool, key: Key) -> Option<Key> {
        // | B | A | STATE | NEW_STATE | DESC                                        |
        // |---|---|-------|-----------|---------------------------------------------|
        // | 0 | 0 | Up    | -         | Key is in the neutral position              |
        // | 1 | 0 | Up    | DownP     | A key press has started                     |
        // | 0 | 1 | Up    | -         | Physically impossible                       |
        // | 1 | 1 | Up    | Down      | Key was pressed faster than we could detect |
        // | 0 | 0 | DownP | Up        | Key was pressed halfway, then released      |
        // | 1 | 0 | DownP | -         | Key is travelling                           |
        // | 0 | 1 | DownP | Up        | Physically impossible                       |
        // | 1 | 1 | DownP | Down      | Key was pressed all the way                 |
        // | 0 | 0 | Down  | Up        | Key press has finished                      |
        // | 1 | 0 | Down  | -         | Key has been released                       |
        // | 0 | 1 | Down  | Up        | Physically impossible.                      |
        // | 1 | 1 | Down  | -         | Key is being held down                      |

        return match (b_down, a_down, key) {
            (true, false, Key::Up) => {
                // Key is being pressed
                Some(Key::DownPartial(millis()))
            }
            (true, true, Key::DownPartial(at)) => {
                // Key was fully pressed
                let travel_time = millis().saturating_sub(at);
                Some(Key::Down(travel_time))
            }
            (true, true, Key::Up) => {
                // Key was fully depressed before we could register the DownPartial
                // Report as the smallest resolution
                Some(Key::Down(2))
            }
            (false, _, _) => {
                // If contact b is up, the key should be up
                Some(Key::Up)
            }
            _ => None,
        };
    }

    pub fn scan(&mut self, mut key_update: impl FnMut(usize, Key)) {
        // Scan key matrix
        self.shift.enable();
        for i in 0..8 {
            if i == 0 {
                self.shift.push_high()
            } else {
                self.shift.push_low()
            }
            for j in 0..7 {
                // Calculate a key index where the leftmost key is 0
                let (key_index, _) =
                    ((j * 8) + (if i == 0 { 8usize } else { i })).overflowing_sub(1);
                if key_index >= KEYS {
                    //  This matrix support more keys than we actually have
                    continue;
                }

                let a_down = self.keys_a[j].is_high();
                let b_down = self.keys_b[j].is_high();

                let state = self.keys[key_index];
                if let Some(new_state) = Self::next_state(a_down, b_down, state) {
                    match (state, new_state) {
                        // Down state is always reported
                        (_, Key::Down(_)) => key_update(key_index, new_state),
                        // Up state is only reported when key had been fully depressed
                        (Key::Down(_), Key::Up) => key_update(key_index, new_state),
                        _ => {}
                    }
                    self.keys[key_index] = new_state
                }
            }
        }
        self.shift.disable();
    }
}
