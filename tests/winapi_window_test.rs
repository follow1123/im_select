extern crate winapi;

use std::{ptr, mem, ffi::OsString, os::windows::prelude::OsStringExt, thread};
use winapi::{um::{winuser::{FindWindowW, FindWindowExW, GetWindowTextW, GetWindowLongPtrW, GWL_STYLE, GWL_EXSTYLE, WM_LBUTTONDOWN, WM_LBUTTONUP, GetClassNameW, GetWindowRect, IsWindowVisible, MSG, TranslateMessage, PostMessageW, GetPropW, GWLP_USERDATA, GetWindowModuleFileNameW, PeekMessageW, PM_REMOVE, DispatchMessageW, WM_QUIT}, winnt::HANDLE}, shared::{windef::{RECT, HWND}, minwindef::LPARAM}};

#[test]
fn input_area() {
    unsafe {
        let taskbar_handle = FindWindowW("Shell_TrayWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        let notify_handle = FindWindowExW(taskbar_handle, ptr::null_mut(), "TrayNotifyWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        // 查找通知区域图标的句柄
        let input_indicator_handle = FindWindowExW(notify_handle, ptr::null_mut(), "TrayInputIndicatorWClass\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());
        // 获取窗口的样式属性
        let window_style = GetWindowLongPtrW(input_indicator_handle, GWL_STYLE) as u32;

        // 获取窗口的扩展样式属性
        let window_ex_style = GetWindowLongPtrW(input_indicator_handle, GWL_EXSTYLE) as u32;

        // 获取窗口的位置和大小
        let mut rect: RECT = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        GetWindowRect(input_indicator_handle, &mut rect);

        // 获取窗口的类名
        let mut class_name = [0u16; 128];

        let class_name_length = GetClassNameW( input_indicator_handle, class_name.as_mut_ptr(), class_name.len() as i32,) as usize;

        let class_name_str = String::from_utf16_lossy(&class_name[0..class_name_length]);

        // 获取窗口标题
        let mut window_title = [0u16; 128];
        let title_length = GetWindowTextW(input_indicator_handle, window_title.as_mut_ptr(), window_title.len() as i32,) as usize;
        let title = String::from_utf16_lossy(&window_title[0..title_length]);

        println!("窗口标题: {}", title);
        // 检查窗口是否可见
        let is_visible = IsWindowVisible(input_indicator_handle) != 0;
        println!("窗口是否可见: {}", is_visible);
        // 打印窗口属性
        println!("窗口样式属性: 0x{:X}", window_style);
        println!("窗口扩展样式属性: 0x{:X}", window_ex_style);
        println!("窗口位置: ({}, {})", rect.left, rect.top);
        println!("窗口大小: {} x {}", rect.right - rect.left, rect.bottom - rect.top);
        println!("窗口类名: {}", class_name_str);

// (1942, 1143) : #1E1E1E
    }
}

#[test]
fn input_area1() {
    unsafe {
        let taskbar_handle = FindWindowW("Shell_TrayWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        let notify_handle = FindWindowExW(taskbar_handle, ptr::null_mut(), "TrayNotifyWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());
        // 查找通知区域图标的句柄
        let input_indicator_handle = FindWindowExW(notify_handle, ptr::null_mut(), "TrayInputIndicatorWClass\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        let im_mode_button_handle = FindWindowExW(input_indicator_handle, ptr::null_mut(), "IMEModeButton\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        let mut class_name = [0u16; 256];
        let class_name_length = GetClassNameW(im_mode_button_handle, class_name.as_mut_ptr(), class_name.len() as i32);
        let class_name = OsString::from_wide(&class_name[..class_name_length as usize]);

        println!("子窗口类名: {:?}", class_name);

        // EnumPropsW(im_mode_button_handle, Some(enum_props_callback));

        let prop_value = GetPropW(im_mode_button_handle, "MSAA_*FCFFFFFF00000000\0".encode_utf16().collect::<Vec<u16>>().as_ptr());

        hwnd_click(im_mode_button_handle);
        println!("属性值: {:?}", prop_value);

        // EnumChildWindows(input_indicator_handle, Some(enum_child_windows_callback), 0);

    }
}

unsafe fn hwnd_click(hwnd: HWND) {
    PostMessageW(hwnd, WM_LBUTTONDOWN, 0, 0);
    PostMessageW(hwnd, WM_LBUTTONUP, 0, 0);
}

unsafe fn _peek_msg(){
// 创建消息结构体
    let mut msg: MSG = unsafe { mem::zeroed() };

    loop {
        // 使用 PeekMessageW 函数检查消息队列
        let has_message = unsafe {
            PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE)
        };

        if has_message != 0 {
            // 打印消息内容
            println!("Received message: 0x{:X}, wParam: {}, lParam: {}", msg.message, msg.wParam, msg.lParam);

            // 处理消息
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // 检查是否收到退出消息
            if msg.message == WM_QUIT {
                break;
            }
        }

        // 在这里执行你的应用程序逻辑
        // 例如：更新界面、处理输入等

        // 延迟一段时间，以免 CPU 占用过高
        thread::sleep(std::time::Duration::from_millis(10));
    }
}

unsafe extern "system" fn _enum_child_windows_callback(hwnd: HWND, _lparam: LPARAM) -> i32 {
    // 检查子窗口是否可见
    if IsWindowVisible(hwnd) == 0 {
        return 1; // 返回非零值以继续枚举
    }

    print_name(hwnd);

    // EnumPropsW(hwnd, Some(enum_props_callback));
    // EnumPropsExW(input_indicator_handle, Some(enum_propsex_callback), 0);

    // 返回非零值以继续枚举
    1
}

extern "system" fn _enum_props_callback(hwnd: HWND, prop_name: *const u16, _prop_data: *mut winapi::ctypes::c_void) -> i32 {
    // 将 *const u16 转换为 Rust 字符串
    let rust_string = {
        let wide_chars = {
            let mut len = 0;
            while unsafe { *prop_name.offset(len) } != 0 {
                len += 1;
            }
            unsafe { std::slice::from_raw_parts(prop_name, len as usize) }
        };
        
        let os_string = OsString::from_wide(wide_chars);
        os_string.to_string_lossy().into_owned()
    };

    // 打印属性名称
    println!("属性名称: {}", rust_string);

    // 获取属性的值并打印
    let prop_value: HANDLE = unsafe {
        GetPropW(hwnd, prop_name)
    };

    // 打印属性的值
    println!("属性值: {:?}", prop_value);

    // 返回非零值以继续枚举，返回零值以中止枚举
    1
}

unsafe fn print_name(hwnd: HWND) {
    let mut class_name = [0u16; 256];
    let class_name_length = GetClassNameW(hwnd, class_name.as_mut_ptr(), class_name.len() as i32);
    let class_name = OsString::from_wide(&class_name[..class_name_length as usize]);

    println!("窗口类名: {:?}", class_name);
}

unsafe fn _print_title(hwnd: HWND){
    let mut window_title = [0u16; 128];
    let title_length = GetWindowTextW(hwnd, window_title.as_mut_ptr(), window_title.len() as i32,) as usize;
    let title = String::from_utf16_lossy(&window_title[0..title_length]);

    println!("窗口标题: {}", title);
}

unsafe fn _print_window_data(hwnd: HWND){
    let retrieved_user_data = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as usize;
    println!("Retrieved user data: {}", retrieved_user_data);
}

unsafe fn _print_module_info(hwnd: HWND){
    // 获取窗口关联模块的文件路径
    let mut buffer: Vec<u16> = vec![0; 1024]; // 用于存储文件路径的缓冲区
    let buffer_size = buffer.len() as u32;

    let module_name_length = GetWindowModuleFileNameW(hwnd, buffer.as_mut_ptr(), buffer_size);

    if module_name_length == 0 {
        println!("获取模块文件名失败");
        return;
    }

    // 从缓冲区中提取文件路径
    let module_name = OsString::from_wide(&buffer[..(module_name_length as usize)]);
    let module_name_str = module_name.to_string_lossy();

    println!("窗口关联模块的文件路径: {}", module_name_str);
}

extern "system" fn _enum_propsex_callback(hwnd: HWND, prop_name: *const u16, _prop_data: *mut winapi::ctypes::c_void) -> i32 {
    // 将 *const u16 转换为 Rust 字符串
    let rust_string = {
        let wide_chars = {
            let mut len = 0;
            while unsafe { *prop_name.offset(len) } != 0 {
                len += 1;
            }
            unsafe { std::slice::from_raw_parts(prop_name, len as usize) }
        };
        
        let os_string = OsString::from_wide(wide_chars);
        os_string.to_string_lossy().into_owned()
    };

    // 打印属性名称
    println!("属性名称: {}", rust_string);

    // 获取属性的值并打印
    let prop_value: HANDLE = unsafe {
        GetPropW(hwnd, prop_name)
    };

    // 打印属性的值
    println!("属性值: {:?}", prop_value);

    // 返回非零值以继续枚举，返回零值以中止枚举
    1
}

#[test]
fn input_area2() {
    unsafe{
        let taskbar_handle = FindWindowW("Shell_TrayWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        let notify_handle = FindWindowExW(taskbar_handle, ptr::null_mut(), "TrayNotifyWnd\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());
        // 查找通知区域图标的句柄
        let input_indicator_handle = FindWindowExW(notify_handle, ptr::null_mut(), "TrayInputIndicatorWClass\0".encode_utf16().collect::<Vec<u16>>().as_ptr(), ptr::null());

        print_name(input_indicator_handle);

        // EnumPropsExW(input_indicator_handle, Some(enum_propsex_callback), 0);
        // hwnd_click(im_mode_button_handle);
        // EnumChildWindows(input_indicator_handle, Some(enum_child_windows_callback), 0);
    }
}
