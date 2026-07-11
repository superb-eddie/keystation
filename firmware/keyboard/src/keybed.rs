use arduino_hal::port::mode::{Floating, Input};
use arduino_hal::port::Pin;

use crate::shift::ShiftRegister;
use shared::millis::millis;

// The key matrix has 8 outputs and 14 inputs to read from 49 total keys.
// Each key has two contacts, to calculate velocity.

const KEYS: usize = 49;

// Borrow checker won't let constructor borrow Pins and it would be cumbersome to do this elsewhere
#[macro_export]
macro_rules! keybed_init {
    ( $p:expr ) => {{
        let shift = ShiftRegister::new(
            $p.d2.into_output().downgrade(),
            $p.d5.into_output().downgrade(),
            $p.d4.into_output().downgrade(),
        );

        let keys_a: [Pin<Input<Floating>>; 7] = [
            $p.d13.downgrade(),
            $p.a1.downgrade(),
            $p.a3.downgrade(),
            $p.a5.downgrade(),
            $p.d6.downgrade(),
            $p.d11.downgrade(),
            $p.d9.downgrade(),
        ];

        let keys_b: [Pin<Input<Floating>>; 7] = [
            $p.a0.downgrade(),
            $p.a2.downgrade(),
            $p.a4.downgrade(),
            $p.d7.downgrade(),
            $p.d12.downgrade(),
            $p.d10.downgrade(),
            $p.d8.downgrade(),
        ];

        Keybed::new(shift, keys_a, keys_b)
    }};
}

#[derive(Copy, Clone)]
pub enum KeyState {
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

    pub key_states: [KeyState; KEYS],
}

impl Keybed {
    pub fn new(
        mut shift: ShiftRegister,
        keys_a: [Pin<Input<Floating>>; 7],
        keys_b: [Pin<Input<Floating>>; 7],
    ) -> Self {
        shift.enable();

        Self {
            shift,
            keys_a,
            keys_b,
            key_states: [KeyState::Up; KEYS],
        }
    }

    fn next_state(a_down: bool, b_down: bool, state: KeyState) -> Option<KeyState> {
        // There are two contacts under each key.
        // As a key is pressed down it will first touch contact B, then it will contact A

        // | B | A | STATE | NEW_STATE | DESC                                        |
        // |---|---|-------|-----------|---------------------------------------------|
        // | 0 | 0 | Up    | Up        | Key is in the neutral position              |
        // | 1 | 0 | Up    | DownP     | A key press has started                     |
        // | 0 | 1 | Up    | Up        | Physically impossible                       |
        // | 1 | 1 | Up    | Down      | Key was pressed faster than we could detect |
        // | 0 | 0 | DownP | Up        | Key was pressed halfway, then released      |
        // | 1 | 0 | DownP | DownP     | Key is travelling                           |
        // | 0 | 1 | DownP | Up        | Physically impossible                       |
        // | 1 | 1 | DownP | Down      | Key was pressed all the way                 |
        // | 0 | 0 | Down  | Up        | Key press has finished                      |
        // | 1 | 0 | Down  | Down      | Key has been released                       |
        // | 0 | 1 | Down  | Up        | Physically impossible.                      |
        // | 1 | 1 | Down  | Down      | Key is being held down                      |

        match (b_down, a_down, state) {
            // Key touched first contact
            (true, false, KeyState::Up) => Some(KeyState::DownPartial(millis())),
            // Key touched both contacts, calculate travel time
            (true, true, KeyState::DownPartial(at)) => Some(KeyState::Down(millis().saturating_sub(at))),
            // Key touched both contacts before we could register the first, report as the smallest resolution
            (true, true, KeyState::Up) => Some(KeyState::Down(2)),
            // Key is always up if the first contact is up
            (false, _, _) => Some(KeyState::Up),
            // Anything else is either impossible or shouldn't change the current state
            _ => None,
        }
    }

    // Scan key matrix
    pub fn scan(&mut self, mut key_update: impl FnMut(usize, KeyState)) {

        // The keybed has 8 sections
        for i in 0..8 {
            // The shift register is used to select each section of the keybed in order
            if i == 0 {
                self.shift.push_high()
            } else {
                self.shift.push_low()
            }

            // Each section of the keybed has 7 keys
            for j in 0..7 {
                // Calculate index of key_state
                // The leftmost key on the keyboard is 0 and the rightmost key is KEYS-1
                // This calculation is funky because I wired things bad, it could be made simple with some work
                let (key_index, _) =
                    ((j * 8) + (if i == 0 { 8usize } else { i })).overflowing_sub(1);
                if key_index >= KEYS {
                    // This matrix supports more keys than we actually have
                    // We can't break because key_index does not increase sequentially with every iteration
                    continue;
                }

                let a_down = self.keys_a[j].is_high();
                let b_down = self.keys_b[j].is_high();

                let state = self.key_states[key_index];
                if let Some(new_state) = Self::next_state(a_down, b_down, state) {
                    match (state, new_state) {
                        // Down state is always reported
                        (_, KeyState::Down(_)) => key_update(key_index, new_state),
                        // Up state is only reported when key had been fully depressed
                        (KeyState::Down(_), KeyState::Up) => key_update(key_index, new_state),
                        _ => {}
                    }
                    self.key_states[key_index] = new_state
                }
            }
        }
    }
}
