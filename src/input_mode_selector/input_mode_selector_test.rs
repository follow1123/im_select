use super::*;
use std::thread;
use std::time::Duration;


#[test]
fn test_find_ime_button_window_handle() {
    _ = InputModeSelector::new()
}

#[test]
fn test_get_current_mode() {
    let ims = InputModeSelector::new();
    _ = ims.current_mode()
}

#[test]
fn test_switch_mode() {
    let ims = InputModeSelector::new();
    let mode_b = ims.current_mode();
    let is_switched = ims.switch_input_mode();
    assert!(is_switched);
    thread::sleep(Duration::from_millis(10));
    let mode_a = ims.current_mode();
    assert!(mode_a != mode_b);
}
