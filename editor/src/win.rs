//! De Windows-functies die de editor gebruikt. Rechtstreeks uit Windows zelf,
//! zonder extra pakketten.
#![allow(non_snake_case, clippy::upper_case_acronyms, dead_code)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HANDLE = *mut c_void;
pub type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: Option<WndProc>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HANDLE,
    pub hIcon: HANDLE,
    pub hCursor: HANDLE,
    pub hbrBackground: HANDLE,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HANDLE,
}

#[repr(C)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: usize,
    pub lParam: isize,
    pub time: u32,
    pub pt: POINT,
}

#[repr(C)]
#[derive(Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct ACCEL {
    pub fVirt: u8,
    pub key: u16,
    pub cmd: u16,
}

#[repr(C)]
pub struct OPENFILENAMEW {
    pub lStructSize: u32,
    pub hwndOwner: HWND,
    pub hInstance: HANDLE,
    pub lpstrFilter: *const u16,
    pub lpstrCustomFilter: *mut u16,
    pub nMaxCustFilter: u32,
    pub nFilterIndex: u32,
    pub lpstrFile: *mut u16,
    pub nMaxFile: u32,
    pub lpstrFileTitle: *mut u16,
    pub nMaxFileTitle: u32,
    pub lpstrInitialDir: *const u16,
    pub lpstrTitle: *const u16,
    pub Flags: u32,
    pub nFileOffset: u16,
    pub nFileExtension: u16,
    pub lpstrDefExt: *const u16,
    pub lCustData: isize,
    pub lpfnHook: *mut c_void,
    pub lpTemplateName: *const u16,
    pub pvReserved: *mut c_void,
    pub dwReserved: u32,
    pub FlagsEx: u32,
}

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
    pub fn CreateWindowExW(ex: u32, class: *const u16, name: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: HANDLE, inst: HANDLE, param: *mut c_void) -> HWND;
    pub fn DefWindowProcW(h: HWND, m: u32, w: usize, l: isize) -> isize;
    pub fn ShowWindow(h: HWND, cmd: i32) -> i32;
    pub fn GetMessageW(m: *mut MSG, h: HWND, a: u32, b: u32) -> i32;
    pub fn TranslateMessage(m: *const MSG) -> i32;
    pub fn DispatchMessageW(m: *const MSG) -> isize;
    pub fn PostQuitMessage(code: i32);
    pub fn MessageBoxW(h: HWND, text: *const u16, title: *const u16, kind: u32) -> i32;
    pub fn SendMessageW(h: HWND, m: u32, w: usize, l: isize) -> isize;
    pub fn PostMessageW(h: HWND, m: u32, w: usize, l: isize) -> i32;
    pub fn SetFocus(h: HWND) -> HWND;
    pub fn GetFocus() -> HWND;
    pub fn GetAncestor(h: HWND, flags: u32) -> HWND;
    pub fn DestroyWindow(h: HWND) -> i32;
    pub fn MoveWindow(h: HWND, x: i32, y: i32, w: i32, hh: i32, repaint: i32) -> i32;
    pub fn GetClientRect(h: HWND, r: *mut RECT) -> i32;
    pub fn LoadCursorW(inst: HANDLE, name: *const u16) -> HANDLE;
    pub fn SetProcessDPIAware() -> i32;
    pub fn GetDC(h: HWND) -> HANDLE;
    pub fn ReleaseDC(h: HWND, dc: HANDLE) -> i32;
    pub fn CreateMenu() -> HANDLE;
    pub fn CreatePopupMenu() -> HANDLE;
    pub fn AppendMenuW(menu: HANDLE, flags: u32, id: usize, text: *const u16) -> i32;
    pub fn DeleteMenu(menu: HANDLE, positie: u32, flags: u32) -> i32;
    pub fn GetMenuItemCount(menu: HANDLE) -> i32;
    pub fn SetMenu(h: HWND, menu: HANDLE) -> i32;
    pub fn CreateAcceleratorTableW(accel: *const ACCEL, count: i32) -> HANDLE;
    pub fn TranslateAcceleratorW(h: HWND, accel: HANDLE, m: *const MSG) -> i32;
    pub fn GetKeyState(key: i32) -> i16;
    pub fn SetWindowTextW(h: HWND, text: *const u16) -> i32;
    pub fn GetWindowTextW(h: HWND, buf: *mut u16, max: i32) -> i32;
    pub fn GetWindowTextLengthW(h: HWND) -> i32;
    pub fn MessageBeep(kind: u32) -> i32;
    pub fn SetForegroundWindow(h: HWND) -> i32;
    pub fn IsWindow(h: HWND) -> i32;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(name: *const u16) -> HANDLE;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateFontW(h: i32, w: i32, esc: i32, ori: i32, weight: i32, italic: u32, underline: u32, strike: u32, charset: u32, outp: u32, clip: u32, quality: u32, pitch: u32, face: *const u16) -> HANDLE;
    pub fn GetDeviceCaps(dc: HANDLE, index: i32) -> i32;
}

#[link(name = "shell32")]
extern "system" {
    pub fn SHGetFolderPathW(h: HWND, csidl: i32, token: HANDLE, flags: u32, path: *mut u16) -> i32;
}

#[link(name = "comdlg32")]
extern "system" {
    pub fn GetOpenFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
    pub fn GetSaveFileNameW(ofn: *mut OPENFILENAMEW) -> i32;
}

pub const WM_DESTROY: u32 = 0x0002;
pub const WM_SIZE: u32 = 0x0005;
pub const WM_ACTIVATE: u32 = 0x0006;
pub const WM_SETFOCUS: u32 = 0x0007;
pub const WM_CLOSE: u32 = 0x0010;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_KEYDOWN: u32 = 0x0100;
pub const WM_COMMAND: u32 = 0x0111;
pub const WM_APP: u32 = 0x8000;

pub const EM_GETSEL: u32 = 0x00B0;
pub const EM_SETSEL: u32 = 0x00B1;
pub const EM_SCROLLCARET: u32 = 0x00B7;
pub const EM_GETMODIFY: u32 = 0x00B8;
pub const EM_SETMODIFY: u32 = 0x00B9;
pub const EM_GETLINECOUNT: u32 = 0x00BA;
pub const EM_LINEINDEX: u32 = 0x00BB;
pub const EM_REPLACESEL: u32 = 0x00C2;
pub const EM_SETLIMITTEXT: u32 = 0x00C5;
pub const EM_EMPTYUNDOBUFFER: u32 = 0x00CD;
pub const EM_LINEFROMCHAR: u32 = 0x00C9;

pub const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
pub const WS_CHILD: u32 = 0x4000_0000;
pub const WS_CLIPCHILDREN: u32 = 0x0200_0000;
pub const WS_VISIBLE: u32 = 0x1000_0000;
pub const WS_TABSTOP: u32 = 0x0001_0000;
pub const WS_VSCROLL: u32 = 0x0020_0000;
pub const WS_HSCROLL: u32 = 0x0010_0000;
pub const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;
pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_AUTOVSCROLL: u32 = 0x0040;
pub const ES_AUTOHSCROLL: u32 = 0x0080;
pub const ES_NOHIDESEL: u32 = 0x0100;
pub const ES_READONLY: u32 = 0x0800;
pub const ES_WANTRETURN: u32 = 0x1000;
pub const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
pub const SW_SHOWMAXIMIZED: i32 = 3;
pub const SW_SHOW: i32 = 5;

pub const MB_OK: u32 = 0x0;
pub const MB_YESNOCANCEL: u32 = 0x3;
pub const MB_YESNO: u32 = 0x4;
pub const MB_OKCANCEL: u32 = 0x0001;
pub const IDOK: i32 = 1;
pub const MB_ICONERROR: u32 = 0x10;
pub const MB_ICONQUESTION: u32 = 0x20;
pub const MB_ICONWARNING: u32 = 0x30;
pub const MB_ICONINFORMATION: u32 = 0x40;
pub const IDYES: i32 = 6;
pub const IDNO: i32 = 7;

pub const MF_STRING: u32 = 0x0;
pub const MF_POPUP: u32 = 0x10;
pub const MF_SEPARATOR: u32 = 0x800;
pub const MF_BYPOSITION: u32 = 0x400;
pub const LB_ADDSTRING: u32 = 0x0180;
pub const LB_RESETCONTENT: u32 = 0x0184;
pub const LB_SETCURSEL: u32 = 0x0186;
pub const LB_GETCURSEL: u32 = 0x0188;
pub const LB_GETCOUNT: u32 = 0x018B;
pub const LBS_NOTIFY: u32 = 0x0001;
pub const LBN_DBLCLK: usize = 2;
pub const EN_CHANGE: usize = 0x0300;
pub const EM_UNDO: u32 = 0x00C7;
pub const EM_LINELENGTH: u32 = 0x00C1;
pub const VK_RETURN: usize = 0x0D;
pub const WM_NEXTDLGCTL: u32 = 0x0028;
pub const MF_GRAYED: u32 = 0x1;
pub const FVIRTKEY: u8 = 1;
pub const FSHIFT: u8 = 4;
pub const FCONTROL: u8 = 8;

pub const VK_TAB: usize = 0x09;
pub const VK_ESCAPE: usize = 0x1B;
pub const VK_SHIFT: i32 = 0x10;
pub const VK_CONTROL: i32 = 0x11;
pub const VK_F1: u16 = 0x70;

pub const OFN_OVERWRITEPROMPT: u32 = 0x2;
pub const OFN_NOCHANGEDIR: u32 = 0x8;
pub const OFN_PATHMUSTEXIST: u32 = 0x800;
pub const OFN_FILEMUSTEXIST: u32 = 0x1000;
pub const OFN_EXPLORER: u32 = 0x8_0000;

pub const GA_ROOT: u32 = 2;
pub const LOGPIXELSY: i32 = 90;

/// Tekst klaarmaken voor Windows (UTF-16 met een nul aan het eind).
pub fn w(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}
