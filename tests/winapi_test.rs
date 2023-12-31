extern crate winapi;

use std::{process::Command, ptr};

use winapi::{
    shared::windef::{HGDIOBJ, POINT, RECT},
    um::{
        wingdi::{CreateCompatibleDC, DeleteDC, GetPixel, SelectObject},
        winuser::{GetDC, ReleaseDC, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, FindWindowW, FindWindowExW, GetWindowRect, GetCursorPos},
    },
};


fn get_act_res() -> (i32, i32){
    let output = Command::new("wmic")
        .args(["path", "Win32_VideoController", "get", "CurrentHorizontalResolution,CurrentVerticalResolution"])
        .output()
        .expect("exec error");

    // 将命令的标准输出转换为字符串
    let stdout_str = String::from_utf8_lossy(&output.stdout);

    if !output.status.success() {
        panic!("无法执行 wmic命令")
    }

    let mut res_iter = stdout_str.split_whitespace().into_iter();
    res_iter.next();
    res_iter.next();
    (res_iter.next().expect("无法获取实际宽度").parse().expect("解析为数字失败"),
        res_iter.next().expect("无法获取实际高度").parse().expect("解析为数字失败"))
}

fn get_cur_res() -> (i32, i32) {
    (unsafe { GetSystemMetrics(SM_CXSCREEN) }, unsafe { GetSystemMetrics(SM_CYSCREEN) })
}

fn get_scale() -> f32 {
    let (x, _) = get_act_res();
    let (x1, _) = get_cur_res();
    x as f32 / x1 as f32
}

// 计算原始坐标
fn calc_original_pos(scale: f32, x: i32, y: i32) -> (i32, i32) {
    let original_x = (x as f32 * scale).ceil() as i32;
    let original_y = (y as f32 * scale).ceil() as i32;
    (original_x, original_y)
}

// 获取当前坐标像素点的颜色
fn get_pixel_color(x: i32, y: i32) -> u32 {
    let pixel_color;
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
        pixel_color = GetPixel(hdc_screen, point.x, point.y);

        // 释放资源
        SelectObject(hdc_compatible, old_bitmap);
        DeleteDC(hdc_compatible);
        ReleaseDC(std::ptr::null_mut(), hdc_screen);


    }
    pixel_color
}

fn get_hex_color(pixel_color: u32) -> String{
    // 提取颜色的红、绿、蓝分量
    let red = pixel_color & 0xFF;
    let green = (pixel_color >> 8) & 0xFF;
    let blue = (pixel_color >> 16) & 0xFF;

    // 将分量转换为十六进制并格式化为字符串,并打印
    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}


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

    let (x, y) = get_cursor_pos();
    println!("cursor: [{}, {}]", x, y);

    let scale = get_scale();

    unsafe{
        let taskbar_handle = FindWindowW("Shell_TrayWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        let notify_handle = FindWindowExW(taskbar_handle, ptr::null_mut(), "TrayNotifyWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        // 查找通知区域图标的句柄
        let input_indicator_handle = FindWindowExW(notify_handle, ptr::null_mut(), "TrayInputIndicatorWClass\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        // 获取窗口的位置和大小
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect(input_indicator_handle, &mut rect);
        // println!("scale: {}", scale);
        // let mut x_offset = 6;
        // let mut y_offset = 10;

        let window_width = rect.right - rect.left;
        let window_height =  rect.bottom - rect.top;

        let (bg_x, bg_y) = calc_original_pos(scale, rect.left + 1, rect.top + 1);
        let (font_x, font_y) = calc_original_pos(scale, rect.left + (window_width / 2), rect.bottom - (window_height as f32 / 2.6) as i32);

        let bg_color = get_pixel_color(bg_x, bg_y);
        let font_color = get_pixel_color(font_x, font_y);

        // if rect.bottom - rect.top > 30 {
        //     x_offset = 6;
        //     y_offset = 15;
        // }
        // let (x, y) = (rect.right - x_offset, rect.bottom - y_offset);
        // let (x, y) = calc_original_pos(scale, x, y);
        // let hex_color = get_pixel_color(x, y);
        // println!("图形位置: ({}, {})", x, y);
        println!("bg :[{}, {}] {}", bg_x, bg_y, get_hex_color(bg_color));
        println!("font : [{}, {}] {}", font_x, font_y, get_hex_color(font_color));
        println!("窗口位置: ({}, {})", rect.left, rect.top);
        println!("窗口大小: {} x {}", rect.right - rect.left, rect.bottom - rect.top);

    }

}
