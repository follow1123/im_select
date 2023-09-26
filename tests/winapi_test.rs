extern crate winapi;

use winapi::{
    shared::windef::{HGDIOBJ, POINT},
    um::{
        wingdi::{CreateCompatibleDC, DeleteDC, GetDeviceCaps, GetPixel, SelectObject, LOGPIXELSX},
        winuser::{GetCursorPos, GetDC, ReleaseDC, GetDpiForSystem},
    },
};

// 获取当前鼠标的位置信息
fn get_cursor_pos() -> (i32, i32) {
    // 获取当前鼠标坐标
    let mut point: POINT = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut point);
    }

    let x = point.x;
    let y = point.y;

    let scale_percent = 125; // 缩放百分比（例如 125%）

    // 计算原始坐标
    let original_x = (x as f32 * (scale_percent as f32 / 100.0)).ceil() as i32;
    let original_y = (y as f32 * (scale_percent as f32 / 100.0)).ceil() as i32;
    // 输出原始分辨率坐标
    (original_x, original_y)
}


#[test]
fn screen_size() {
}

#[test]
fn cursor_pos() {
    let (x, y) = get_cursor_pos();
    println!("pos is: {} x {}", x, y);
}


#[test]
fn pixel_color() {
    let (x, y) = get_cursor_pos();

    unsafe {
        let hdc_screen = GetDC(std::ptr::null_mut()); // 获取屏幕的设备上下文
        let hdc_compatible = CreateCompatibleDC(hdc_screen); // 创建兼容的设备上下文

        // 创建一个 POINT 结构来指定坐标
        let mut point = POINT { x, y };

        // 将兼容的设备上下文选入屏幕设备上下文
        let old_bitmap: HGDIOBJ = SelectObject(hdc_compatible, hdc_screen as HGDIOBJ);

        // 使用 ClientToScreen 将坐标转换为屏幕坐标
        winapi::um::winuser::ClientToScreen(std::ptr::null_mut(), &mut point);

        // 使用 GetPixel 获取指定像素的颜色
        let pixel_color = GetPixel(hdc_screen, point.x, point.y);

        // 释放资源
        SelectObject(hdc_compatible, old_bitmap);
        DeleteDC(hdc_compatible);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);

        // 提取颜色的红、绿、蓝分量
        let red = pixel_color & 0xFF;
        let green = (pixel_color >> 8) & 0xFF;
        let blue = (pixel_color >> 16) & 0xFF;

        // 将分量转换为十六进制并格式化为字符串
        let hex_color = format!("#{:02X}{:02X}{:02X}", red, green, blue);

        println!("Pixel Color (Hex): {}", hex_color);
    }
}

// assert_eq!((1920, 1080), (screen_width, screen_height))
