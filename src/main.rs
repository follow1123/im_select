use std::cell::RefCell;
use std::collections::HashMap;
use std::{env, process, thread};

use winapi::shared::windef::{HGDIOBJ, POINT};
use winapi::um::winuser::{
    keybd_event, GetCursorPos, GetKeyState,
    GetSystemMetrics, VK_CONTROL, VK_DOWN, VK_LEFT, VK_RIGHT,
    VK_SPACE, VK_UP,
};

use winapi::um::wingdi::{CreateCompatibleDC, DeleteDC, GetPixel, SelectObject};
use winapi::um::winuser::{GetDC, ReleaseDC};

// 获取当前坐标像素点的颜色
fn get_pixel_color(x: i32, y: i32) -> String {
    let hex_color;
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
        hex_color = format!("#{:02X}{:02X}{:02X}", red, green, blue);
    }
    hex_color
}

// 获取系统分辨率
fn get_system_res() -> (i32, i32) {
    let width: i32;
    let height: i32;
    unsafe {
        width = GetSystemMetrics(0); // SM_CXSCREEN
        height = GetSystemMetrics(1); // SM_CYSCREEN
    }
    (width, height)
}

// 切换输入法模式
fn switch_input_mode() {
    // 模拟按下 Ctrl + Space 键
    unsafe {
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(VK_SPACE as u8, 0, 0, 0);
        keybd_event(VK_SPACE as u8, 0, 0x0002, 0);
        keybd_event(VK_CONTROL as u8, 0, 0x0002, 0);
    }
}

// 计算原始坐标
fn calc_original_pos(scale: u16, x: i32, y: i32) -> (i32, i32) {
    let original_x = (x as f32 * (scale as f32 / 100.0)).ceil() as i32;
    let original_y = (y as f32 * (scale as f32 / 100.0)).ceil() as i32;
    (original_x, original_y)
}

// 获取当前鼠标的位置信息
fn get_cursor_pos() -> (i32, i32) {
    let mut cursor_pos: POINT = POINT { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut cursor_pos);
    }
    (cursor_pos.x, cursor_pos.y)
}

// 取色器
fn color_picker() {
    let (x, y) = get_cursor_pos();

    let x = RefCell::new(x);
    let y = RefCell::new(y);
    let scale = 125;

    let mut key_map: HashMap<i32, Box<dyn FnMut()>> = HashMap::new();
    key_map.insert( VK_SPACE, Box::new(|| {
        process::exit(0);
    }));

    key_map.insert(VK_UP, Box::new(|| {
        let mut m_y = y.borrow_mut();
        *m_y += 1;
        let (o_x, o_y) = calc_original_pos(scale, *x.borrow(), *m_y);
        println!("({}, {}) : {}", o_x, o_y, get_pixel_color(o_x, o_y))
    }));

    key_map.insert(VK_DOWN, Box::new(|| {
        let mut m_y = y.borrow_mut();
        *m_y += 1;
        let (o_x, o_y) = calc_original_pos(scale, *x.borrow(), *m_y);
        println!("({}, {}) : {}", o_x, o_y, get_pixel_color(o_x, o_y))
    }));

    key_map.insert(VK_LEFT, Box::new(|| {
        let mut m_x = x.borrow_mut();
        *m_x -= 1;
        let (o_x, o_y) = calc_original_pos(scale, *m_x, *y.borrow());
        println!("({}, {}) : {}", o_x, o_y, get_pixel_color(o_x, o_y))
    }));

    key_map.insert(VK_RIGHT, Box::new(|| {
        let mut m_x = x.borrow_mut();
        *m_x += 1;
        let (o_x, o_y) = calc_original_pos(scale, *m_x, *y.borrow());
        println!("({}, {}) : {}", o_x, o_y, get_pixel_color(o_x, o_y))
    }));
    unsafe {
        // 循环检查空格键是否按下
        loop {
            for (key, callback) in key_map.iter_mut() {
                if (GetKeyState(*key) as u16 & 0x8000) != 0 {
                    callback();
                }
            }
            thread::sleep(std::time::Duration::from_millis(60));
        }
    }
}

fn main() {
    let mut arg_iter = env::args();
    arg_iter.next();
    // 获取中英输入模式状态的坐标
    let x = 2348;
    let y = 1422;

    // if let Some(arg) = arg_iter.next() {
    if let Some(arg) = Some("-p") {
        if "-p" == arg {
            color_picker();
            process::exit(0);
        }
    }

    // 根据`中`和`英`不同像素分布判断当前是什么模式
    // let input_code = match get_pixel_color(x, y) {
    //     "#FFFFFF" => 1,
    //     "#101010" => 2,
    //     _ => process::exit(1),
    // };
    let input_code = 1;
    //
    // 获取命令行输入的参数
    let arg = match arg_iter.next() {
        Some(arg) => arg,
        None => {
            println!("{}", input_code);
            process::exit(0);
        }
    };

    let arg_code: u32 = arg.parse().expect("参数解析错误！");
    // 解析输入的参数，并切换为对应的输入法
    if input_code == arg_code {
        process::exit(0);
    }
    switch_input_mode();
}
