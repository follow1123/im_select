use winapi::um::winuser::GetSystemMetrics;

extern crate winapi;

// 计算原始坐标
fn calc_original_pos(scale: u16, x: i32, y: i32) -> (i32, i32) {
    let original_x = (x as f32 * (scale as f32 / 100.0)).ceil() as i32;
    let original_y = (y as f32 * (scale as f32 / 100.0)).ceil() as i32;
    (original_x, original_y)
}

#[test]
fn pop_window() {
    unsafe {
        let width = GetSystemMetrics(0);  // SM_CXSCREEN
        let height = GetSystemMetrics(1); // SM_CYSCREEN

        let (a,  b) = calc_original_pos(125, width, height);
        // println!("Width: {}", width);
        // println!("Height: {}", height);
        println!("Width: {}", a);
        println!("Height: {}", b);

    }
}
