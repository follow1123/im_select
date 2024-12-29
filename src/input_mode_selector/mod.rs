#[cfg(test)]
mod input_mode_selector_test;

use std::fmt::Display;
use std::iter::once;
use std::mem::MaybeUninit;
use std::os::windows::ffi::OsStrExt;


use std::ptr::null_mut;
use std::str::FromStr;

use winapi::ctypes::c_void as cvd;
use winapi::shared::minwindef::{BOOL, FALSE, TRUE};
use winapi::shared::windef::{HWND, RECT};
use winapi::um::wingdi::{CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetBitmapBits, GetObjectW, SelectObject, BITMAP};
use winapi::um::winnt::LPCWSTR;
use winapi::um::winuser::{FindWindowExW, FindWindowW, GetClientRect, GetDC, PostMessageW, PrintWindow, ReleaseDC, PW_CLIENTONLY, WM_LBUTTONDOWN, WM_LBUTTONUP};
use winapi::um::winbase::{GlobalAlloc, GlobalFree, GPTR};

#[derive(PartialEq, Debug)]
pub enum ParseInputModeError {
    NotNumber,
    NotSpecificValue,
}

#[derive(PartialEq)]
pub enum InputMode {
    En,
    Zh
}

impl Display for InputMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::En => f.write_str("1"),
            Self::Zh => f.write_str("2"),
        }
    }
}

impl TryFrom<u8> for InputMode {
    type Error = ParseInputModeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(InputMode::En),
            2 => Ok(InputMode::Zh),
            _ => Err(ParseInputModeError::NotSpecificValue),
        }
    }
}

impl FromStr for InputMode {
    type Err = ParseInputModeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Ok(mode) = s.parse::<u8>() else {
            return Err(ParseInputModeError::NotNumber);
        };
        Self::try_from(mode)
    }
}

// 转换 LPCWSTR 字符串
fn l(text: &str) -> LPCWSTR {
    let wide: Vec<u16> = std::ffi::OsStr::new(text).encode_wide().chain(once(0)).collect();
    wide.as_ptr()
}

pub struct InputModeSelector {
    hwnd: HWND
}

impl InputModeSelector {

    /// 获取系统托盘处输入指示器句柄
    pub fn new() -> Self {
        unsafe {
            let system_tray = FindWindowW(l("Shell_TrayWnd"), null_mut());
            if system_tray.is_null() { panic!("can not find 'Shell_TrayWnd' window"); }
            let tray_notify = FindWindowExW(system_tray, null_mut(), l("TrayNotifyWnd"), null_mut());
            if tray_notify.is_null() { panic!("can not find 'TrayNotifyWnd' window"); }
            let input_indicator = FindWindowExW(tray_notify, null_mut(), l("TrayInputIndicatorWClass"), null_mut());
            if tray_notify.is_null() { panic!("can not find 'TrayInputIndicatorWClass' window"); }
            let ime_mode = FindWindowExW(input_indicator, null_mut(), l("IMEModeButton"), null_mut());
            if ime_mode.is_null() { panic!("can not find 'IMEModeButton' window"); }
            Self { hwnd: ime_mode }
        }
    }

    // 获取当前输入模式
    pub fn current_mode(&self) -> InputMode {
        unsafe {
            let hwnd = self.hwnd;
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
                if mode != 2 || (flag_y != -1 && flag_y != y) { break; }
                for x in (0..bmp.bmWidth).rev() {
                    // 32位位图
                    let offset = y * bmp.bmWidthBytes + x * 4;
                    let r = *pixels.add(offset as usize);
                    if r == 0 { continue; }
                    if flag_y == -1 { flag_y = y; }
                    if flag != 0 && x != flag_x - 1 {
                        mode = 1;
                        break;
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
            InputMode::try_from(mode).unwrap()
        }
    }
    // 切换输入法模式
    pub fn switch_input_mode(&self) -> bool {
        unsafe {
            let hwnd = self.hwnd;
            let btn_down_ok: BOOL = PostMessageW(hwnd, WM_LBUTTONDOWN, 0, 0);
            let btn_up_ok: BOOL = PostMessageW(hwnd, WM_LBUTTONUP, 0, 0);
            btn_down_ok == TRUE && btn_up_ok == TRUE
        }
    }
}
