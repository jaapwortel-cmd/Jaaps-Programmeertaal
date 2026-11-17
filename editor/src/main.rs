//! Jaap's Programmeertaal: de editor voor Windows.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
#[cfg(windows)]
mod win;

/// Het eerste kind-venster (het tekstvak in een tekstvenster).
#[cfg(windows)]
pub(crate) fn win_kind(h: win::HWND) -> win::HWND {
    #[link(name = "user32")]
    extern "system" {
        fn GetWindow(h: win::HWND, cmd: u32) -> win::HWND;
    }
    unsafe { GetWindow(h, 5) }
}

#[cfg(windows)]
fn main() {
    app::start();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("De editor van Jaap's Programmeertaal werkt alleen op Windows.");
    eprintln!("Op andere computers kun je de opdrachtregel gebruiken: jaaptaal controleer programma.txt");
}
