// ======================================================================
// Looptijd van Jaap's Programmeertaal.
// Dit stuk gaat mee in elk programma dat je bouwt. Het zorgt voor rekenen,
// vergelijken, tonen, vensters en knoppen. Het gebruikt alleen de standaard
// Rust-bibliotheek en Windows zelf. Geen AI, geen internet, geen extra pakketten.
//
// Starten met --tekst: alles komt als tekst in de uitvoer (voor proeven).
// Met --druk Naam drukt de proef daarna op de knop Naam.
// ======================================================================
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(dead_code, unused_variables, unused_mut, unreachable_code, unused_imports, non_snake_case, unused_braces)]

#[derive(Clone, Debug)]
enum W {
    G(f64),
    T(String),
    B(bool),
}

struct Fout {
    regel: usize,
    tekst: String,
}

type R<T> = Result<T, Fout>;

fn fout<T>(regel: usize, tekst: String) -> R<T> {
    Err(Fout { regel, tekst })
}

fn schrijf_getal(g: f64) -> String {
    if g.fract() == 0.0 && g.abs() < 1e15 {
        return format!("{}", g as i64);
    }
    let mut s = format!("{:.10}", g);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s.replace('.', ",")
}

impl W {
    fn tekst(&self) -> String {
        match self {
            W::G(g) => schrijf_getal(*g),
            W::T(t) => t.clone(),
            W::B(true) => "waar".to_string(),
            W::B(false) => "niet waar".to_string(),
        }
    }
    fn soort(&self) -> &'static str {
        match self {
            W::G(_) => "een getal",
            W::T(_) => "tekst",
            W::B(_) => "waar of niet waar",
        }
    }
}

const MAX_STAPPEN: u64 = 10_000_000;
const MAX_HERHALINGEN: f64 = 1_000_000.0;

fn reken(a: W, teken: u8, b: W, r: usize) -> R<W> {
    if teken == 4 {
        let (x, y) = (a.tekst(), b.tekst());
        return Ok(W::T(if x.is_empty() || y.is_empty() { format!("{}{}", x, y) } else { format!("{} {}", x, y) }));
    }
    let woord = ["plus", "min", "keer", "gedeeld door"][teken as usize];
    let (x, y) = match (&a, &b) {
        (W::G(x), W::G(y)) => (*x, *y),
        (W::G(_), ander) | (ander, _) => {
            return fout(r, format!("Rekenen ({}) kan alleen met getallen, maar hier staat {}.", woord, ander.soort()))
        }
    };
    let uit = match teken {
        0 => x + y,
        1 => x - y,
        2 => x * y,
        _ => {
            if y == 0.0 {
                return fout(r, "Delen door nul kan niet.".to_string());
            }
            x / y
        }
    };
    if !uit.is_finite() {
        return fout(r, "Het getal wordt te groot om mee te rekenen.".to_string());
    }
    Ok(W::G(uit))
}

fn negatief(a: W, r: usize) -> R<W> {
    match a {
        W::G(g) => Ok(W::G(-g)),
        ander => fout(r, format!("Min ervoor kan alleen bij een getal, maar hier staat {}.", ander.soort())),
    }
}

fn vergelijk(a: W, b: W, op: u8, r: usize) -> R<bool> {
    if op <= 1 {
        let gelijk = match (&a, &b) {
            (W::G(x), W::G(y)) => x == y,
            (W::T(x), W::T(y)) => x.to_lowercase() == y.to_lowercase(),
            (W::B(x), W::B(y)) => x == y,
            _ => false,
        };
        return Ok(if op == 0 { gelijk } else { !gelijk });
    }
    let (x, y) = match (&a, &b) {
        (W::G(x), W::G(y)) => (*x, *y),
        (W::G(_), ander) | (ander, _) => {
            return fout(r, format!("Groter of kleiner kan ik alleen bij getallen bepalen, maar hier staat {}.", ander.soort()))
        }
    };
    Ok(match op {
        2 => x > y,
        3 => x < y,
        4 => x >= y,
        _ => x <= y,
    })
}

fn waar(a: W, r: usize) -> R<bool> {
    match a {
        W::B(b) => Ok(b),
        ander => fout(r, format!("Hier verwacht ik waar of niet waar, maar hier staat {}.", ander.soort())),
    }
}

fn aantal(a: W, r: usize) -> R<u64> {
    match a {
        W::G(g) if g >= 0.0 && g.fract() == 0.0 && g <= MAX_HERHALINGEN => Ok(g as u64),
        W::G(g) if g > MAX_HERHALINGEN => {
            fout(r, "Zo vaak herhalen mag niet. Het kan hoogstens een miljoen keer.".to_string())
        }
        W::G(g) => fout(r, format!("Het aantal keer moet een heel getal zijn, en niet {}.", schrijf_getal(g))),
        ander => fout(r, format!("Hoe vaak? Hier verwacht ik een getal, maar hier staat {}.", ander.soort())),
    }
}

struct Staat {
    waarden: Vec<Option<W>>,
    namen: &'static [&'static str],
    acties: &'static [&'static str],
    stappen: u64,
    scherm: Scherm,
}

impl Staat {
    fn lees(&self, i: usize, r: usize) -> R<W> {
        match &self.waarden[i] {
            Some(w) => Ok(w.clone()),
            None => {
                let n = self.namen[i];
                fout(r, format!("{} heeft nog geen waarde. Zet eerst een regel zoals: Onthoud {} is 0.", n, n))
            }
        }
    }
    fn zet(&mut self, i: usize, w: W) {
        self.waarden[i] = Some(w);
    }
    fn verander(&mut self, i: usize, w: W, r: usize) -> R<()> {
        if self.waarden[i].is_none() {
            let n = self.namen[i];
            return fout(r, format!("Je kunt {} pas veranderen als je het eerst hebt onthouden.", n));
        }
        self.waarden[i] = Some(w);
        Ok(())
    }
    fn tel_op(&mut self, i: usize, w: W, erbij: bool, r: usize) -> R<()> {
        let oud = self.lees(i, r)?;
        let nieuw = reken(oud, if erbij { 0 } else { 1 }, w, r)?;
        self.waarden[i] = Some(nieuw);
        Ok(())
    }
    fn stap(&mut self, r: usize) -> R<()> {
        self.stappen += 1;
        if self.stappen > MAX_STAPPEN {
            return fout(
                r,
                "Het programma doet te veel stappen, meer dan tien miljoen. Ik heb het gestopt, zodat de computer niet vastloopt."
                    .to_string(),
            );
        }
        Ok(())
    }
    fn toon(&mut self, w: W) {
        let t = w.tekst();
        self.scherm.toon(&t);
    }
    fn grens(&mut self, r: usize, max: u64) {
        let t = format!("Let op: de herhaling van regel {} is gestopt na {} keer. Dat was de grens die je had gezet.", r, max);
        self.scherm.toon(&t);
    }
    fn venster(&mut self, titel: &str) {
        self.scherm.venster(titel);
    }
    fn knop(&mut self, naam: &str) {
        let actie = self.acties.iter().position(|a| a.to_lowercase() == naam.to_lowercase());
        self.scherm.knop(naam, actie);
    }
    fn onderdeel(&mut self, soort: u8, naam: &str) {
        self.scherm.onderdeel(soort, naam);
    }
    fn zoek(&self, naam: &str, r: usize) -> R<usize> {
        match self.scherm.onderdelen.iter().position(|o| o.naam.to_lowercase() == naam.to_lowercase()) {
            Some(i) => Ok(i),
            None => fout(r, format!("Het onderdeel {} is er (nog) niet. Is het venster waar het in staat al gemaakt?", naam)),
        }
    }
    fn waarde_van(&mut self, naam: &str, als_getal: bool, r: usize) -> R<W> {
        let i = self.zoek(naam, r)?;
        let w = self.scherm.lees(i);
        if !als_getal {
            return Ok(w);
        }
        let t = w.tekst();
        let schoon: String = t.trim().replace(' ', "").replace('.', "").replace(',', ".");
        match schoon.parse::<f64>() {
            Ok(g) if g.is_finite() && !t.trim().is_empty() => Ok(W::G(g)),
            _ => {
                if t.trim().is_empty() {
                    fout(r, format!("In {} staat nog niets. Daar moet een getal in.", naam))
                } else {
                    fout(r, format!("In {} staat geen getal, maar: {}", naam, t.trim()))
                }
            }
        }
    }
    fn zet_in(&mut self, naam: &str, w: W, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        if !matches!(self.scherm.onderdelen[i].soort, 0 | 1 | 2) {
            return fout(r, format!("In {} kan geen tekst. Dat kan alleen bij een tekstvak of een label.", naam));
        }
        self.scherm.zet_tekst(i, &w.tekst());
        Ok(())
    }
    fn voeg_keuze(&mut self, naam: &str, w: W, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        if !matches!(self.scherm.onderdelen[i].soort, 4 | 5) {
            return fout(r, format!("{} is geen keuzelijst.", naam));
        }
        self.scherm.voeg_keuze(i, &w.tekst());
        Ok(())
    }
    fn vink(&mut self, naam: &str, aan: bool, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        if self.scherm.onderdelen[i].soort != 3 {
            return fout(r, format!("{} is geen selectievakje.", naam));
        }
        self.scherm.vink(i, aan);
        Ok(())
    }
    fn focus(&mut self, naam: &str, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        self.scherm.focus(i);
        Ok(())
    }
}

/// Een onderdeel op het scherm. Soort: 0 tekstvak, 1 groot tekstvak, 2 label, 3 selectievakje, 4 keuzelijst, 5 uitklaplijst.
struct Ond {
    soort: u8,
    naam: String,
    tekst: String,
    aan: bool,
    keuzes: Vec<String>,
    gekozen: Option<usize>,
    h: usize,
}

const SOORT_WOORD: [&str; 6] = ["tekstvak", "groot tekstvak", "label", "selectievakje", "keuzelijst", "uitklaplijst"];

type Hoofd = fn(&mut Staat) -> R<()>;
type BijKnop = fn(&mut Staat, usize) -> R<()>;

fn foutzin(f: &Fout) -> String {
    format!("Er ging iets mis op regel {}: {}", f.regel, f.tekst)
}

// ---------------------------------------------------------------------
// Tekst-stand: voor proeven en voor computers zonder Windows.
// ---------------------------------------------------------------------
fn start_tekst(namen: &'static [&'static str], acties: &'static [&'static str], hoofd: Hoofd, bij_knop: BijKnop, stappen: Vec<(String, String)>) {
    let mut s = Staat { waarden: vec![None; namen.len()], namen, acties, stappen: 0, scherm: Scherm::tekst() };
    if let Err(f) = hoofd(&mut s) {
        println!("{}", foutzin(&f));
        std::process::exit(1);
    }
    for (soort, waarde) in stappen {
        let (naam, inhoud) = waarde.split_once('=').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((waarde.clone(), String::new()));
        if soort != "--druk" {
            let Some(i) = s.scherm.onderdelen.iter().position(|o| o.naam.to_lowercase() == naam.to_lowercase()) else {
                println!("Er is geen onderdeel met de naam {}.", naam);
                continue;
            };
            match soort.as_str() {
                "--typ" => {
                    println!("[typ in {}] {}", naam, inhoud);
                    s.scherm.onderdelen[i].tekst = inhoud;
                }
                "--vink" => {
                    println!("[vink aan] {}", naam);
                    s.scherm.onderdelen[i].aan = true;
                }
                _ => {
                    let o = &mut s.scherm.onderdelen[i];
                    o.gekozen = o.keuzes.iter().position(|k| k.to_lowercase() == inhoud.to_lowercase());
                    println!("[kies in {}] {}", naam, inhoud);
                }
            }
            continue;
        }
        let naam = waarde;
        println!("[druk op] {}", naam);
        let knop = s.scherm.knoppen.iter().find(|k| k.0.to_lowercase() == naam.to_lowercase()).map(|k| k.1);
        match knop {
            None => println!("Er is geen knop met de naam {}.", naam),
            Some(None) => println!("Deze knop doet nog niets."),
            Some(Some(a)) => {
                s.stappen = 0;
                if let Err(f) = bij_knop(&mut s, a) {
                    println!("{}", foutzin(&f));
                    std::process::exit(1);
                }
            }
        }
    }
}

fn start(namen: &'static [&'static str], acties: &'static [&'static str], hoofd: Hoofd, bij_knop: BijKnop) {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tekststand = args.iter().any(|a| a == "--tekst") || !cfg!(windows);
    if tekststand {
        let mut stappen = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if matches!(args[i].as_str(), "--druk" | "--typ" | "--vink" | "--kies") && i + 1 < args.len() {
                stappen.push((args[i].clone(), args[i + 1].clone()));
                i += 1;
            }
            i += 1;
        }
        start_tekst(namen, acties, hoofd, bij_knop, stappen);
        return;
    }
    #[cfg(windows)]
    win::start(namen, acties, hoofd, bij_knop);
}

// ---------------------------------------------------------------------
// Het scherm.
// ---------------------------------------------------------------------
struct Scherm {
    tekststand: bool,
    /// (naam, actie) van elke knop, in volgorde.
    knoppen: Vec<(String, Option<usize>)>,
    onderdelen: Vec<Ond>,
    uitvoer: Vec<String>,
    #[cfg(windows)]
    win: win::WinScherm,
}

impl Scherm {
    fn tekst() -> Self {
        Scherm {
            tekststand: true,
            knoppen: Vec::new(),
            onderdelen: Vec::new(),
            uitvoer: Vec::new(),
            #[cfg(windows)]
            win: win::WinScherm::default(),
        }
    }
    fn onderdeel(&mut self, soort: u8, naam: &str) {
        let mut o = Ond { soort, naam: naam.to_string(), tekst: String::new(), aan: false, keuzes: Vec::new(), gekozen: None, h: 0 };
        if soort == 2 {
            o.tekst = naam.to_string();
        }
        if self.tekststand {
            println!("[{}] {}", SOORT_WOORD[soort as usize], naam);
        } else {
            #[cfg(windows)]
            {
                o.h = self.win.onderdeel(soort, naam);
            }
        }
        self.onderdelen.push(o);
    }
    fn lees(&self, i: usize) -> W {
        let o = &self.onderdelen[i];
        if !self.tekststand {
            #[cfg(windows)]
            return win::lees(o);
        }
        match o.soort {
            3 => W::B(o.aan),
            4 | 5 => W::T(o.gekozen.and_then(|k| o.keuzes.get(k).cloned()).unwrap_or_default()),
            _ => W::T(o.tekst.clone()),
        }
    }
    fn zet_tekst(&mut self, i: usize, t: &str) {
        let o = &mut self.onderdelen[i];
        o.tekst = t.to_string();
        if self.tekststand {
            println!("[{} {}] {}", SOORT_WOORD[o.soort as usize], o.naam, t);
            return;
        }
        #[cfg(windows)]
        win::zet_tekst(o.h, t);
    }
    fn voeg_keuze(&mut self, i: usize, t: &str) {
        let o = &mut self.onderdelen[i];
        o.keuzes.push(t.to_string());
        if o.gekozen.is_none() {
            o.gekozen = Some(0);
        }
        if !self.tekststand {
            #[cfg(windows)]
            win::voeg_keuze(o.h, o.soort, t, o.keuzes.len() == 1);
        }
    }
    fn vink(&mut self, i: usize, aan: bool) {
        let o = &mut self.onderdelen[i];
        o.aan = aan;
        if self.tekststand {
            println!("[selectievakje {}] {}", o.naam, if aan { "aan" } else { "uit" });
            return;
        }
        #[cfg(windows)]
        win::vink(o.h, aan);
    }
    fn focus(&mut self, i: usize) {
        if self.tekststand {
            println!("[focus] {}", self.onderdelen[i].naam);
            return;
        }
        #[cfg(windows)]
        win::focus(self.onderdelen[i].h);
    }
    fn toon(&mut self, t: &str) {
        if self.tekststand {
            println!("{}", t);
            return;
        }
        #[cfg(windows)]
        {
            if self.win.vensters.is_empty() {
                self.uitvoer.push(t.to_string());
            } else {
                win::melding(self.win.actief(), t, "Melding");
            }
        }
    }
    fn venster(&mut self, titel: &str) {
        if self.tekststand {
            println!("[venster] {}", titel);
            return;
        }
        #[cfg(windows)]
        self.win.venster(titel);
    }
    fn knop(&mut self, naam: &str, actie: Option<usize>) {
        self.knoppen.push((naam.to_string(), actie));
        if self.tekststand {
            println!("[knop] {}", naam);
            return;
        }
        #[cfg(windows)]
        self.win.knop(naam, self.knoppen.len() - 1);
    }
}

// ---------------------------------------------------------------------
// Windows: echte vensters en knoppen, voorleesbaar en met het toetsenbord.
// ---------------------------------------------------------------------
#[cfg(windows)]
mod win {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    type HWND = *mut c_void;
    type HANDLE = *mut c_void;
    type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

    #[repr(C)]
    struct WNDCLASSEXW {
        cbSize: u32,
        style: u32,
        lpfnWndProc: Option<WndProc>,
        cbClsExtra: i32,
        cbWndExtra: i32,
        hInstance: HANDLE,
        hIcon: HANDLE,
        hCursor: HANDLE,
        hbrBackground: HANDLE,
        lpszMenuName: *const u16,
        lpszClassName: *const u16,
        hIconSm: HANDLE,
    }

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct MSG {
        hwnd: HWND,
        message: u32,
        wParam: usize,
        lParam: isize,
        time: u32,
        pt: POINT,
    }

    #[repr(C)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
        fn CreateWindowExW(ex: u32, class: *const u16, name: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: HANDLE, inst: HANDLE, param: *mut c_void) -> HWND;
        fn DefWindowProcW(h: HWND, m: u32, w: usize, l: isize) -> isize;
        fn ShowWindow(h: HWND, cmd: i32) -> i32;
        fn GetMessageW(m: *mut MSG, h: HWND, a: u32, b: u32) -> i32;
        fn TranslateMessage(m: *const MSG) -> i32;
        fn DispatchMessageW(m: *const MSG) -> isize;
        fn IsDialogMessageW(h: HWND, m: *mut MSG) -> i32;
        fn PostQuitMessage(code: i32);
        fn MessageBoxW(h: HWND, text: *const u16, title: *const u16, kind: u32) -> i32;
        fn SendMessageW(h: HWND, m: u32, w: usize, l: isize) -> isize;
        fn PostMessageW(h: HWND, m: u32, w: usize, l: isize) -> i32;
        fn SetFocus(h: HWND) -> HWND;
        fn GetFocus() -> HWND;
        fn SetForegroundWindow(h: HWND) -> i32;
        fn GetAncestor(h: HWND, flags: u32) -> HWND;
        fn DestroyWindow(h: HWND) -> i32;
        fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, w: i32, hh: i32, flags: u32) -> i32;
        fn LoadCursorW(inst: HANDLE, name: *const u16) -> HANDLE;
        fn SetProcessDPIAware() -> i32;
        fn GetDC(h: HWND) -> HANDLE;
        fn ReleaseDC(h: HWND, dc: HANDLE) -> i32;
        fn GetClientRect(h: HWND, r: *mut RECT) -> i32;
        fn MoveWindow(h: HWND, x: i32, y: i32, w: i32, hh: i32, repaint: i32) -> i32;
        fn GetWindow(h: HWND, cmd: u32) -> HWND;
        fn SetWindowTextW(h: HWND, text: *const u16) -> i32;
        fn GetWindowTextW(h: HWND, buf: *mut u16, max: i32) -> i32;
        fn GetWindowTextLengthW(h: HWND) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> HANDLE;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateFontW(h: i32, w: i32, esc: i32, ori: i32, weight: i32, italic: u32, underline: u32, strike: u32, charset: u32, outp: u32, clip: u32, quality: u32, pitch: u32, face: *const u16) -> HANDLE;
        fn GetDeviceCaps(dc: HANDLE, index: i32) -> i32;
    }

    const WM_DESTROY: u32 = 0x0002;
    const WM_SIZE: u32 = 0x0005;
    const WM_CLOSE: u32 = 0x0010;
    const WM_SETFONT: u32 = 0x0030;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_COMMAND: u32 = 0x0111;
    const BM_CLICK: u32 = 0x00F5;
    const EM_SETSEL: u32 = 0x00B1;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_CLIPCHILDREN: u32 = 0x0200_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const WS_VSCROLL: u32 = 0x0020_0000;
    const WS_EX_CONTROLPARENT: u32 = 0x0001_0000;
    const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_READONLY: u32 = 0x0800;
    const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
    const SW_SHOW: i32 = 5;
    const MB_ICONINFORMATION: u32 = 0x40;
    const MB_ICONERROR: u32 = 0x10;
    const MB_SETFOREGROUND: u32 = 0x0001_0000;
    const VK_ESCAPE: usize = 0x1B;
    const VK_RETURN: usize = 0x0D;
    const GA_ROOT: u32 = 2;
    const GW_CHILD: u32 = 5;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const LOGPIXELSY: i32 = 90;
    const KNOP_ID: usize = 1000;
    const WS_BORDER: u32 = 0x0080_0000;
    const ES_AUTOHSCROLL: u32 = 0x0080;
    const ES_WANTRETURN: u32 = 0x1000;
    const BS_AUTOCHECKBOX: u32 = 0x0003;
    const LBS_NOTIFY: u32 = 0x0001;
    const CBS_DROPDOWNLIST: u32 = 0x0003;
    const BM_GETCHECK: u32 = 0x00F0;
    const BM_SETCHECK: u32 = 0x00F1;
    const LB_ADDSTRING: u32 = 0x0180;
    const LB_SETCURSEL: u32 = 0x0186;
    const LB_GETCURSEL: u32 = 0x0188;
    const LB_GETTEXT: u32 = 0x0189;
    const LB_GETTEXTLEN: u32 = 0x018A;
    const CB_ADDSTRING: u32 = 0x0143;
    const CB_GETCURSEL: u32 = 0x0147;
    const CB_GETLBTEXT: u32 = 0x0148;
    const CB_GETLBTEXTLEN: u32 = 0x0149;
    const CB_SETCURSEL: u32 = 0x014E;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    thread_local! {
        static STAAT: RefCell<Option<Staat>> = RefCell::new(None);
        static BIJ_KNOP: Cell<Option<BijKnop>> = Cell::new(None);
        static OPEN: Cell<usize> = Cell::new(0);
        static KNOP_HWNDS: RefCell<Vec<HWND>> = RefCell::new(Vec::new());
        static UITVOER_VAK: Cell<HWND> = Cell::new(std::ptr::null_mut());
    }

    #[derive(Default)]
    pub(super) struct WinScherm {
        pub(super) vensters: Vec<HWND>,
        /// Per venster: waar het volgende onderdeel komt, en of er al iets de focus heeft.
        volgende_y: Vec<i32>,
        heeft_focus: Vec<bool>,
        pub(super) klik_venster: Option<HWND>,
        lettertype: Option<HANDLE>,
        schaal: f32,
    }

    pub(super) fn melding(eigenaar: HWND, tekst: &str, titel: &str) {
        unsafe {
            MessageBoxW(eigenaar, w(tekst).as_ptr(), w(titel).as_ptr(), MB_ICONINFORMATION | MB_SETFOREGROUND);
        }
    }

    fn foutmelding(eigenaar: HWND, tekst: &str) {
        unsafe {
            MessageBoxW(eigenaar, w(tekst).as_ptr(), w("Fout in het programma").as_ptr(), MB_ICONERROR | MB_SETFOREGROUND);
        }
    }

    impl WinScherm {
        pub(super) fn actief(&self) -> HWND {
            self.klik_venster.or(self.vensters.last().copied()).unwrap_or(null_mut())
        }

        fn voorbereiden(&mut self) {
            if self.lettertype.is_some() {
                return;
            }
            unsafe {
                let dc = GetDC(null_mut());
                let dpi = GetDeviceCaps(dc, LOGPIXELSY);
                ReleaseDC(null_mut(), dc);
                self.schaal = if dpi > 0 { dpi as f32 / 96.0 } else { 1.0 };
                let hoogte = -((22.0 * self.schaal) as i32);
                self.lettertype = Some(CreateFontW(hoogte, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w("Segoe UI").as_ptr()));
            }
        }

        fn s(&self, px: i32) -> i32 {
            (px as f32 * self.schaal) as i32
        }

        pub(super) fn venster(&mut self, titel: &str) {
            self.voorbereiden();
            unsafe {
                let h = CreateWindowExW(
                    WS_EX_CONTROLPARENT,
                    klasse().as_ptr(),
                    w(titel).as_ptr(),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    self.s(460),
                    self.s(220),
                    null_mut(),
                    null_mut(),
                    GetModuleHandleW(null()),
                    null_mut(),
                );
                OPEN.with(|o| o.set(o.get() + 1));
                self.vensters.push(h);
                self.volgende_y.push(20);
                self.heeft_focus.push(false);
                SetForegroundWindow(h);
                SetFocus(h);
            }
        }

        /// Zet een Windows-onderdeel onder het vorige in het laatste venster.
        fn plaats(&mut self, klasse: &str, tekst: &str, stijl: u32, ex: u32, hoogte: i32, id: usize, focus: bool) -> HWND {
            let Some(&venster) = self.vensters.last() else { return null_mut() };
            let y = *self.volgende_y.last().unwrap();
            unsafe {
                let h = CreateWindowExW(
                    ex,
                    w(klasse).as_ptr(),
                    w(tekst).as_ptr(),
                    WS_CHILD | WS_VISIBLE | stijl,
                    self.s(20),
                    y,
                    self.s(400),
                    self.s(hoogte),
                    venster,
                    id as HANDLE,
                    GetModuleHandleW(null()),
                    null_mut(),
                );
                if let Some(f) = self.lettertype {
                    SendMessageW(h, WM_SETFONT, f as usize, 1);
                }
                let nieuw_y = y + self.s(hoogte) + self.s(12);
                *self.volgende_y.last_mut().unwrap() = nieuw_y;
                SetWindowPos(venster, null_mut(), 0, 0, self.s(460), nieuw_y + self.s(60), SWP_NOMOVE | SWP_NOZORDER);
                if focus && !*self.heeft_focus.last().unwrap() {
                    *self.heeft_focus.last_mut().unwrap() = true;
                    SetFocus(h);
                }
                h
            }
        }

        pub(super) fn knop(&mut self, naam: &str, nummer: usize) {
            if self.vensters.is_empty() {
                return;
            }
            let h = self.plaats("BUTTON", naam, WS_TABSTOP, 0, 52, KNOP_ID + nummer, true);
            KNOP_HWNDS.with(|k| k.borrow_mut().push(h));
        }

        /// Maakt een tekstvak, label, selectievakje of keuzelijst. Geeft het Windows-onderdeel terug.
        pub(super) fn onderdeel(&mut self, soort: u8, naam: &str) -> usize {
            if self.vensters.is_empty() {
                return 0;
            }
            // Een tekstvak of lijst krijgt een label erboven: zo leest de schermlezer de naam voor.
            let label = |s: &mut Self| {
                s.plaats("STATIC", &format!("{}:", naam), 0, 0, 30, 0, false);
            };
            let h = match soort {
                0 => {
                    label(self);
                    self.plaats("EDIT", "", WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL, WS_EX_CLIENTEDGE, 38, 0, true)
                }
                1 => {
                    label(self);
                    self.plaats(
                        "EDIT",
                        "",
                        WS_TABSTOP | WS_BORDER | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN,
                        WS_EX_CLIENTEDGE,
                        200,
                        0,
                        true,
                    )
                }
                2 => self.plaats("STATIC", naam, 0, 0, 34, 0, false),
                3 => self.plaats("BUTTON", naam, WS_TABSTOP | BS_AUTOCHECKBOX, 0, 36, 0, true),
                4 => {
                    label(self);
                    self.plaats("LISTBOX", "", WS_TABSTOP | WS_BORDER | WS_VSCROLL | LBS_NOTIFY, 0, 160, 0, true)
                }
                _ => {
                    label(self);
                    let h = self.plaats("COMBOBOX", "", WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST, 0, 240, 0, true);
                    // De open lijst is 240 hoog, het dichte vak maar 40: schuif de volgende plek terug.
                    *self.volgende_y.last_mut().unwrap() -= self.s(200);
                    h
                }
            };
            h as usize
        }
    }

    fn tekst_van(h: HWND) -> String {
        unsafe {
            let n = GetWindowTextLengthW(h);
            let mut buf = vec![0u16; n as usize + 1];
            let m = GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
            String::from_utf16_lossy(&buf[..m.max(0) as usize])
        }
    }

    pub(super) fn lees(o: &Ond) -> W {
        let h = o.h as HWND;
        unsafe {
            match o.soort {
                3 => W::B(SendMessageW(h, BM_GETCHECK, 0, 0) == 1),
                4 | 5 => {
                    let (cur, len, get) = if o.soort == 4 { (LB_GETCURSEL, LB_GETTEXTLEN, LB_GETTEXT) } else { (CB_GETCURSEL, CB_GETLBTEXTLEN, CB_GETLBTEXT) };
                    let i = SendMessageW(h, cur, 0, 0);
                    if i < 0 {
                        return W::T(String::new());
                    }
                    let n = SendMessageW(h, len, i as usize, 0).max(0) as usize;
                    let mut buf = vec![0u16; n + 1];
                    SendMessageW(h, get, i as usize, buf.as_mut_ptr() as isize);
                    W::T(String::from_utf16_lossy(&buf[..n]))
                }
                _ => W::T(tekst_van(h).replace("\r\n", "\n")),
            }
        }
    }

    pub(super) fn zet_tekst(h: usize, t: &str) {
        unsafe {
            SetWindowTextW(h as HWND, w(&t.replace("\r\n", "\n").replace('\n', "\r\n")).as_ptr());
        }
    }

    pub(super) fn voeg_keuze(h: usize, soort: u8, t: &str, eerste: bool) {
        let (voeg, kies) = if soort == 4 { (LB_ADDSTRING, LB_SETCURSEL) } else { (CB_ADDSTRING, CB_SETCURSEL) };
        unsafe {
            SendMessageW(h as HWND, voeg, 0, w(t).as_ptr() as isize);
            if eerste {
                SendMessageW(h as HWND, kies, 0, 0);
            }
        }
    }

    pub(super) fn vink(h: usize, aan: bool) {
        unsafe {
            SendMessageW(h as HWND, BM_SETCHECK, aan as usize, 0);
        }
    }

    pub(super) fn focus(h: usize) {
        unsafe {
            SetFocus(h as HWND);
        }
    }

    fn klasse() -> Vec<u16> {
        w("JaapsProgrammeertaalVenster")
    }

    fn druk(nummer: usize, venster: HWND) {
        let Some(mut staat) = STAAT.with(|s| s.borrow_mut().take()) else { return };
        let actie = staat.scherm.knoppen.get(nummer).and_then(|k| k.1);
        staat.scherm.win.klik_venster = Some(venster);
        match actie {
            None => melding(venster, "Deze knop doet nog niets.", "Melding"),
            Some(a) => {
                staat.stappen = 0;
                if let Some(f) = BIJ_KNOP.with(|b| b.get()) {
                    if let Err(fout) = f(&mut staat, a) {
                        foutmelding(venster, &foutzin(&fout));
                    }
                }
            }
        }
        staat.scherm.win.klik_venster = None;
        STAAT.with(|s| *s.borrow_mut() = Some(staat));
    }

    unsafe extern "system" fn venster_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
        match m {
            WM_COMMAND => {
                let id = wp & 0xFFFF;
                let code = (wp >> 16) & 0xFFFF;
                if code == 0 && id >= KNOP_ID {
                    druk(id - KNOP_ID, h);
                    return 0;
                }
            }
            WM_SIZE => {
                let vak = UITVOER_VAK.with(|u| u.get());
                if !vak.is_null() && GetAncestor(vak, GA_ROOT) == h {
                    let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    GetClientRect(h, &mut r);
                    MoveWindow(vak, 0, 0, r.right, r.bottom, 1);
                }
            }
            WM_CLOSE => {
                DestroyWindow(h);
                return 0;
            }
            WM_DESTROY => {
                let over = OPEN.with(|o| {
                    o.set(o.get().saturating_sub(1));
                    o.get()
                });
                if over == 0 {
                    PostQuitMessage(0);
                }
                return 0;
            }
            _ => {}
        }
        DefWindowProcW(h, m, wp, lp)
    }

    fn uitvoervenster(regels: &[String]) {
        unsafe {
            let h = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                klasse().as_ptr(),
                w("Uitvoer van het programma").as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                700,
                500,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null()),
                null_mut(),
            );
            OPEN.with(|o| o.set(o.get() + 1));
            let tekst = regels.join("\r\n");
            let vak = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w("EDIT").as_ptr(),
                w(&tekst).as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
                0,
                0,
                100,
                100,
                h,
                null_mut(),
                GetModuleHandleW(null()),
                null_mut(),
            );
            UITVOER_VAK.with(|u| u.set(vak));
            let dc = GetDC(null_mut());
            let dpi = GetDeviceCaps(dc, LOGPIXELSY);
            ReleaseDC(null_mut(), dc);
            let f = CreateFontW(-(22 * dpi / 96), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w("Segoe UI").as_ptr());
            SendMessageW(vak, WM_SETFONT, f as usize, 1);
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetClientRect(h, &mut r);
            MoveWindow(vak, 0, 0, r.right, r.bottom, 1);
            SetForegroundWindow(h);
            SetFocus(vak);
            SendMessageW(vak, EM_SETSEL, 0, 0);
        }
    }

    pub(super) fn start(namen: &'static [&'static str], acties: &'static [&'static str], hoofd: Hoofd, bij_knop: BijKnop) {
        unsafe {
            SetProcessDPIAware();
            let naam = klasse();
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(venster_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(null()),
                hIcon: null_mut(),
                hCursor: LoadCursorW(null_mut(), 32512 as *const u16),
                hbrBackground: 16 as HANDLE, // kleur van knoppen + 1
                lpszMenuName: null(),
                lpszClassName: naam.as_ptr(),
                hIconSm: null_mut(),
            };
            RegisterClassExW(&wc);
        }
        BIJ_KNOP.with(|b| b.set(Some(bij_knop)));
        let mut staat = Staat {
            waarden: vec![None; namen.len()],
            namen,
            acties,
            stappen: 0,
            scherm: Scherm { tekststand: false, knoppen: Vec::new(), onderdelen: Vec::new(), uitvoer: Vec::new(), win: WinScherm::default() },
        };
        if let Err(f) = hoofd(&mut staat) {
            foutmelding(staat.scherm.win.actief(), &foutzin(&f));
            return;
        }
        if staat.scherm.win.vensters.is_empty() {
            let mut regels = std::mem::take(&mut staat.scherm.uitvoer);
            if regels.is_empty() {
                regels.push("Het programma is klaar. Er was niets om te tonen.".to_string());
            }
            regels.push(String::new());
            regels.push("Druk op Escape om te sluiten.".to_string());
            uitvoervenster(&regels);
        } else if let Some(&laatste) = staat.scherm.win.vensters.last() {
            unsafe {
                SetForegroundWindow(laatste);
            }
            // Tekst die getoond werd voordat er een venster was, gaat niet verloren.
            let eerder = std::mem::take(&mut staat.scherm.uitvoer);
            if !eerder.is_empty() {
                melding(laatste, &eerder.join("\n"), "Melding");
            }
        }
        STAAT.with(|s| *s.borrow_mut() = Some(staat));

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                let wortel = GetAncestor(msg.hwnd, GA_ROOT);
                if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE && !wortel.is_null() {
                    PostMessageW(wortel, WM_CLOSE, 0, 0);
                    continue;
                }
                if msg.message == WM_KEYDOWN && msg.wParam == VK_RETURN {
                    let focus = GetFocus();
                    if KNOP_HWNDS.with(|k| k.borrow().contains(&focus)) {
                        SendMessageW(focus, BM_CLICK, 0, 0);
                        continue;
                    }
                }
                if !wortel.is_null() && IsDialogMessageW(wortel, &mut msg) != 0 {
                    continue;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
// ===================== Einde van de looptijd ==========================
