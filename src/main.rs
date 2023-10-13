use std::process::Command;
use std::{env, process, ptr};

use winapi::shared::windef::{HGDIOBJ, POINT, HWND, RECT};
use winapi::um::winuser::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN, WM_LBUTTONDOWN, WM_LBUTTONUP, PostMessageW, FindWindowExW, FindWindowW, GetWindowRect};

use winapi::um::wingdi::{CreateCompatibleDC, DeleteDC, GetPixel, SelectObject};
use winapi::um::winuser::{GetDC, ReleaseDC};

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

fn _get_hex_color(pixel_color: u32) -> String{
    // 提取颜色的红、绿、蓝分量
    let red = pixel_color & 0xFF;
    let green = (pixel_color >> 8) & 0xFF;
    let blue = (pixel_color >> 16) & 0xFF;

    // 将分量转换为十六进制并格式化为字符串,并打印
    format!("#{:02X}{:02X}{:02X}", red, green, blue)
}

// 获取系统实际分辨率
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

// 获取系统当前的分辨率
fn get_cur_res() -> (i32, i32) {
    (unsafe { GetSystemMetrics(SM_CXSCREEN) }, unsafe { GetSystemMetrics(SM_CYSCREEN) })
}

// 计算当前系统缩放的百分比
fn get_scale() -> f32 {
    let (x, _) = get_act_res();
    let (x1, _) = get_cur_res();
    x as f32 / x1 as f32
}

// 根据缩放百分比计算原始坐标
fn calc_original_pos(scale: f32, x: i32, y: i32) -> (i32, i32) {
    let original_x = (x as f32 * scale).ceil() as i32;
    let original_y = (y as f32 * scale).ceil() as i32;
    (original_x, original_y)
}

// 切换输入法模式
unsafe fn switch_input_mode(input_indicator_handle: HWND) {
    // 获取到输入窗口的子按键的句柄
    let im_mode_button_handle = FindWindowExW(input_indicator_handle, ptr::null_mut(), "IMEModeButton\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

    // 模拟点击操作
    PostMessageW(im_mode_button_handle, WM_LBUTTONDOWN, 0, 0);
    PostMessageW(im_mode_button_handle, WM_LBUTTONUP, 0, 0);
}

// 获取到系统托盘内输入指示的窗口句柄
unsafe fn get_input_indicator_handle() -> HWND {
    let taskbar_handle = FindWindowW("Shell_TrayWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

    let notify_handle = FindWindowExW(taskbar_handle, ptr::null_mut(), "TrayNotifyWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());
    // 查找通知区域图标的句柄
    FindWindowExW(notify_handle, ptr::null_mut(), "TrayInputIndicatorWClass\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null())
}

fn get_cur_code() -> (u8, HWND) {
    let scale = get_scale();
    let mut code: u8 = 2;
    unsafe{
        // 查找通知区域图标的句柄
        let input_indicator_handle = get_input_indicator_handle();

        // 获取窗口的位置和大小
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect(input_indicator_handle, &mut rect);

        let window_width = rect.right - rect.left;
        let window_height =  rect.bottom - rect.top;

        // 根据背景颜色和输入模式问题不重合的部分判断当前的输入模式
        let (bg_x, bg_y) = calc_original_pos(scale, rect.left + 1, rect.top + 1);
        let (font_x, font_y) = calc_original_pos(scale, rect.left + (window_width / 2), rect.bottom - (window_height as f32 / 2.6) as i32);

        let bg_color = get_pixel_color(bg_x, bg_y);
        let font_color = get_pixel_color(font_x, font_y);

        if bg_color == font_color {
            code = 1;
        }

        return (code, input_indicator_handle);
    }
}

fn main() {
    let mut arg_iter = env::args();
    arg_iter.next();

    // 获取当前输入法模式
    let (cur_code, input_indicator_handle) = get_cur_code();
    if let Some(arg) = arg_iter.next() {
        let err_msg = "参数必须为1(英)/2(中)";
        let code: u8 = arg.parse().expect(err_msg);
        if code != 1 && code != 2 {
            panic!("{}", err_msg)
        }
        // 参数和当前的输入法不同则切换输入法
        if code != cur_code {
            unsafe{ switch_input_mode(input_indicator_handle) }
        }
        process::exit(0);
    }
    // 没有参数则直接打印当前的输入模式
    println!("{}", cur_code);
}
