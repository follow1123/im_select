use std::iter::once;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::ctypes::c_void as cvd;

use winapi::shared::minwindef::{BOOL, FALSE, TRUE};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::wingdi::{CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetBitmapBits, GetObjectW, SelectObject, BITMAP};
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{FindWindowExW, FindWindowW, GetClientRect, GetDC, PostMessageW, PrintWindow, ReleaseDC, PW_CLIENTONLY, WM_LBUTTONDOWN, WM_LBUTTONUP};
use winapi::um::winbase::{GlobalAlloc, GlobalFree, GPTR};

// 转换 LPCWSTR 字符串
fn l(text: &str) -> LPCWSTR {
    let wide: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().chain(once(0)).collect();
    wide.as_ptr()
}

// 获取系统托盘处输入指示器句柄
fn get_ime_handle() -> HWND {
    unsafe {
        let system_tray = FindWindowW(l("Shell_TrayWnd"), null_mut());
        if system_tray.is_null() { panic!("can not find 'Shell_TrayWnd' window"); }
        let tray_notify = FindWindowExW(system_tray, null_mut(), l("TrayNotifyWnd"), null_mut());
        if tray_notify.is_null() { panic!("can not find 'TrayNotifyWnd' window"); }
        let input_indicator = FindWindowExW(tray_notify, null_mut(), l("TrayInputIndicatorWClass"), null_mut());
        if tray_notify.is_null() { panic!("can not find 'TrayInputIndicatorWClass' window"); }
        let ime_mode = FindWindowExW(input_indicator, null_mut(), l("IMEModeButton"), null_mut());
        if ime_mode.is_null() { panic!("can not find 'IMEModeButton' window"); }
        return ime_mode;
    }
}

// 切换输入法模式
fn _switch_input_mode(hwnd: HWND) -> bool {
    unsafe {
        let btn_down_ok: BOOL = PostMessageW(hwnd, WM_LBUTTONDOWN, 0, 0);
        let btn_up_ok: BOOL = PostMessageW(hwnd, WM_LBUTTONUP, 0, 0);
        return btn_down_ok == TRUE && btn_up_ok == TRUE;
    }
}

// 获取当前输入模式
fn get_input_mode(hwnd: HWND) -> u8 {
    unsafe {
        // 从窗口中获取位图
        let hdc_window = GetDC(hwnd);
        if hdc_window.is_null() { panic!("'GetDC' error"); }
        let hdc_mem = CreateCompatibleDC(hdc_window);
        if hdc_mem.is_null() { panic!("'CreateCompatibleDC' error"); }
        let mut rect: MaybeUninit<RECT> = MaybeUninit::uninit();
        let result: BOOL = GetClientRect(hwnd, rect.as_mut_ptr());
        if result == FALSE { panic!("'GetClientRect' error") }
        let rect = *rect.as_mut_ptr();
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        let h_bitmap = CreateCompatibleBitmap(hdc_window, width, height);
        if h_bitmap.is_null() { panic!("'CreateCompatibleBitmap' error"); }
        let h_gdi_obj = SelectObject(hdc_mem, h_bitmap as *mut cvd);
        if h_gdi_obj.is_null() { panic!("'SelectObject' error"); }
        // 打印窗口内容到内存DC
        let result: BOOL = PrintWindow(hwnd, hdc_mem, PW_CLIENTONLY);
        if result == FALSE { panic!("'PrintWindow' error") }
        let mut bmp: MaybeUninit<BITMAP> = MaybeUninit::uninit();
        let result: BOOL = GetObjectW(h_bitmap as *mut cvd, std::mem::size_of::<BITMAP>() as i32, bmp.as_mut_ptr() as *mut cvd);
        if result == FALSE { panic!("'GetObjectW' error") }
        let bmp = *bmp.as_mut_ptr();
        // 获取位图的数据
        let pixel_size = bmp.bmWidthBytes * bmp.bmHeight;
        let p_pixels = GlobalAlloc(GPTR, pixel_size as usize);
        let result: BOOL = GetBitmapBits(h_bitmap, pixel_size, p_pixels);
        let pixels = p_pixels as *mut u8;
        if result == FALSE { panic!("'GetBitmapBits' error") }
        // 处理位图
        // 判断输入模式
        // 从右下角开始扫描
        let mut mode: u8 = 2;
        let (mut flag, mut flag_x, mut flag_y) = (0, -1, -1);
        for y in (0..bmp.bmHeight).rev() {
            if flag_y != -1 && flag_y != y { break; }
            for x in (0..bmp.bmWidth).rev() {
                // 32位位图
                let offset = y * bmp.bmWidthBytes + x * 4;
                let r = *pixels.add(offset as usize);
                if r == 0 { continue; }
                if flag_y == -1 { flag_y = y; }
                if flag != 0 {
                    if x != flag_x - 1 {
                        mode = 1;
                        break;
                    }
                }
                flag = r;
                flag_x = x
            }
        }
        // 释放资源
        GlobalFree(p_pixels);
        DeleteObject(h_bitmap as *mut cvd);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_window);
        return mode;
    }
}

fn main() {
    let hwnd = get_ime_handle();
    //let _ = switch_input_mode(hwnd);
    let mode = get_input_mode(hwnd);
    print!("{mode}");
    //let mut arg_iter = env::args();
    //arg_iter.next();
    //
    //// 获取当前输入法模式
    //let (cur_code, input_indicator_handle) = get_cur_code();
    //if let Some(arg) = arg_iter.next() {
    //    let err_msg = "参数必须为1(英)/2(中)";
    //    let code: u8 = arg.parse().expect(err_msg);
    //    if code != 1 && code != 2 {
    //        panic!("{}", err_msg)
    //    }
    //    // 参数和当前的输入法不同则切换输入法
    //    if code != cur_code {
    //        unsafe{ switch_input_mode(input_indicator_handle) }
    //    }
    //    process::exit(0);
    //}
    //// 没有参数则直接打印当前的输入模式
    //println!("{}", cur_code);
}
