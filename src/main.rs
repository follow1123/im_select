mod input_mode_selector;

use std::env;
use std::str::FromStr;
use crate::input_mode_selector::{InputMode, InputModeSelector};

fn main() {
    let input_mode_selector = InputModeSelector::new();
    let mode = input_mode_selector.current_mode();
    let mut args = env::args();
    args.next();
    let Some(input) = args.next() else {
        println!("{mode}");
        return;
    };

    if let Ok(m) = InputMode::from_str(input.as_str()) {
        if m != mode {
            _ = input_mode_selector.switch_input_mode();
        }
    }
}
