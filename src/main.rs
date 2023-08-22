
use winapi::um::winuser::{keybd_event, VK_CONTROL, VK_SPACE};


use winapi::um::winuser::{GetDC, ReleaseDC};
use winapi::um::wingdi::GetPixel;

use std::{env, process};


fn get_pixel_color(x: i32, y: i32) -> u32 {
    let hdc = unsafe { GetDC(std::ptr::null_mut()) };
    let color = unsafe { GetPixel(hdc, x, y) };
    unsafe { ReleaseDC(std::ptr::null_mut(), hdc) };
    color
}

fn switch_input_mode() {
    // 模拟按下 Ctrl + Space 键
    unsafe {
        keybd_event(VK_CONTROL as u8, 0, 0, 0);
        keybd_event(VK_SPACE as u8, 0, 0, 0);
        keybd_event(VK_SPACE as u8, 0, 0x0002, 0);
        keybd_event(VK_CONTROL as u8, 0, 0x0002, 0);
    }
}

fn main() {
    let mut arg_iter = env::args();
    arg_iter.next();
    // 获取中英输入模式状态的坐标
    let x = 2348;
    let y = 1422;
    // 根据`中`和`英`不同像素分布判断当前是什么模式
    let input_code = match get_pixel_color(x, y) {
        0xFFFFFF => 1,
        0x101010 => 2,
        _ => process::exit(1),
    };


    // 获取命令行输入的参数
    let arg = match arg_iter.next() {
        Some(arg) => arg,
        None => {
            println!("{}", input_code);
            process::exit(0);
        },
    };

    let arg_code: u32 = arg.parse().expect("参数解析错误！");
    // 解析输入的参数，并切换为对应的输入法
    if input_code == arg_code {
        process::exit(0);
    } {
        switch_input_mode();
    }

}
