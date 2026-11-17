// ======================================================================
// Installatieprogramma van Jaap's Programmeertaal.
// Dit wordt gebouwd voor elk programma waarvan je de installatie beschrijft.
// Het bevat jouw programma en installeert het op een Windows-computer:
// - alleen voor jou (zonder beheerdersrechten), of voor alle gebruikers (met beheerdersrechten);
// - met snelkoppelingen in het startmenu en op het bureaublad, als je dat beschreef;
// - met een licentie waarmee je akkoord moet gaan, als je die beschreef;
// - met een vermelding bij "Apps" van Windows, zodat je het ook weer kunt verwijderen.
// Alles gaat via meldingsvensters met Ja en Nee: die leest elke schermlezer voor.
//
// Opdrachtregel (voor proeven): --stil  --voor-mij | --voor-iedereen  --map <map>  --verwijder
// ======================================================================
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(dead_code, non_snake_case, unused_imports, unused_variables)]

#[cfg(not(windows))]
fn main() {
    println!("Dit installatieprogramma werkt alleen op Windows.");
}

#[cfg(windows)]
fn main() {
    win::main();
}

#[cfg(windows)]
mod win {
    use super::*;
    use std::ffi::c_void;
    use std::path::{Path, PathBuf};
    use std::ptr::{null, null_mut};

    type HWND = *mut c_void;
    type HANDLE = *mut c_void;

    #[repr(C)]
    struct GUID {
        d1: u32,
        d2: u16,
        d3: u16,
        d4: [u8; 8],
    }

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(h: HWND, text: *const u16, title: *const u16, kind: u32) -> i32;
    }
    #[link(name = "shell32")]
    extern "system" {
        fn SHGetFolderPathW(h: HWND, csidl: i32, token: HANDLE, flags: u32, path: *mut u16) -> i32;
        fn ShellExecuteW(h: HWND, op: *const u16, file: *const u16, params: *const u16, dir: *const u16, show: i32) -> isize;
        fn IsUserAnAdmin() -> i32;
    }
    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(r: *mut c_void, flags: u32) -> i32;
        fn CoCreateInstance(clsid: *const GUID, outer: *mut c_void, ctx: u32, iid: *const GUID, out: *mut *mut c_void) -> i32;
    }
    #[link(name = "advapi32")]
    extern "system" {
        fn RegCreateKeyExW(key: HANDLE, sub: *const u16, res: u32, class: *const u16, opt: u32, sam: u32, sec: *mut c_void, out: *mut HANDLE, disp: *mut u32) -> i32;
        fn RegSetValueExW(key: HANDLE, name: *const u16, res: u32, kind: u32, data: *const u8, len: u32) -> i32;
        fn RegCloseKey(key: HANDLE) -> i32;
        fn RegDeleteTreeW(key: HANDLE, sub: *const u16) -> i32;
    }

    const MB_OK: u32 = 0;
    const MB_YESNOCANCEL: u32 = 3;
    const MB_YESNO: u32 = 4;
    const MB_ICONERROR: u32 = 0x10;
    const MB_ICONQUESTION: u32 = 0x20;
    const MB_ICONINFORMATION: u32 = 0x40;
    const MB_SETFOREGROUND: u32 = 0x1_0000;
    const IDYES: i32 = 6;
    const IDNO: i32 = 7;
    const HKEY_CURRENT_USER: HANDLE = 0x8000_0001u32 as i32 as isize as HANDLE;
    const HKEY_LOCAL_MACHINE: HANDLE = 0x8000_0002u32 as i32 as isize as HANDLE;
    const KEY_WRITE: u32 = 0x2_0006;
    const REG_SZ: u32 = 1;
    const REG_DWORD: u32 = 4;
    const CSIDL_PROGRAMS: i32 = 0x02;
    const CSIDL_DESKTOPDIRECTORY: i32 = 0x10;
    const CSIDL_COMMON_PROGRAMS: i32 = 0x17;
    const CSIDL_COMMON_DESKTOPDIRECTORY: i32 = 0x19;
    const CSIDL_LOCAL_APPDATA: i32 = 0x1C;
    const CSIDL_PROGRAM_FILES: i32 = 0x26;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    struct Keuzes {
        stil: bool,
        voor_iedereen: Option<bool>,
        map: Option<PathBuf>,
        verwijder: bool,
        /// Na de toestemming van de beheerder: de vragen zijn al gesteld.
        al_gevraagd: bool,
    }

    fn keuzes() -> Keuzes {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let mut k = Keuzes { stil: false, voor_iedereen: None, map: None, verwijder: false, al_gevraagd: false };
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--stil" => k.stil = true,
                "--voor-mij" => k.voor_iedereen = Some(false),
                "--voor-iedereen" => k.voor_iedereen = Some(true),
                "--verwijder" => k.verwijder = true,
                "--al-gevraagd" => k.al_gevraagd = true,
                "--map" if i + 1 < args.len() => {
                    k.map = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
                _ => {}
            }
            i += 1;
        }
        k
    }

    fn titel() -> String {
        if VERSIE.is_empty() {
            format!("{} installeren", NAAM)
        } else {
            format!("{} {} installeren", NAAM, VERSIE)
        }
    }

    fn vraag(tekst: &str, knoppen: u32) -> i32 {
        unsafe { MessageBoxW(null_mut(), w(tekst).as_ptr(), w(&titel()).as_ptr(), knoppen | MB_SETFOREGROUND) }
    }

    fn zeg(k: &Keuzes, tekst: &str, fout: bool) {
        if k.stil {
            println!("{}", tekst);
        } else {
            vraag(tekst, MB_OK | if fout { MB_ICONERROR } else { MB_ICONINFORMATION });
        }
    }

    fn systeemmap(csidl: i32) -> Option<PathBuf> {
        let mut buf = [0u16; 520];
        let r = unsafe { SHGetFolderPathW(null_mut(), csidl, null_mut(), 0, buf.as_mut_ptr()) };
        if r != 0 {
            return None;
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(0);
        Some(PathBuf::from(String::from_utf16_lossy(&buf[..n])))
    }

    fn is_beheerder() -> bool {
        unsafe { IsUserAnAdmin() != 0 }
    }

    /// Start dit programma opnieuw met beheerdersrechten (Windows vraagt om toestemming).
    fn opnieuw_als_beheerder(extra: &str) -> bool {
        let exe = std::env::current_exe().unwrap_or_default();
        let r = unsafe {
            ShellExecuteW(null_mut(), w("runas").as_ptr(), w(&exe.to_string_lossy()).as_ptr(), w(extra).as_ptr(), null(), 1)
        };
        r > 32
    }

    unsafe fn methode(obj: *mut c_void, nummer: usize) -> usize {
        let vtabel = *(obj as *const *const usize);
        *vtabel.add(nummer)
    }

    /// Maakt een snelkoppeling (.lnk) met de Windows-onderdelen ShellLink en PersistFile.
    fn snelkoppeling(lnk: &Path, doel: &Path, map: &Path) -> bool {
        let clsid_shelllink = GUID { d1: 0x0002_1401, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
        let iid_ishelllinkw = GUID { d1: 0x0002_14F9, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
        let iid_ipersistfile = GUID { d1: 0x0000_010B, d2: 0, d3: 0, d4: [0xC0, 0, 0, 0, 0, 0, 0, 0x46] };
        unsafe {
            CoInitializeEx(null_mut(), 2);
            let mut link: *mut c_void = null_mut();
            if CoCreateInstance(&clsid_shelllink, null_mut(), 1, &iid_ishelllinkw, &mut link) != 0 || link.is_null() {
                return false;
            }
            type ZetTekst = unsafe extern "system" fn(*mut c_void, *const u16) -> i32;
            type Vraag = unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32;
            type Bewaar = unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32;
            type Los = unsafe extern "system" fn(*mut c_void) -> u32;
            let zet_pad: ZetTekst = std::mem::transmute(methode(link, 20));
            let zet_map: ZetTekst = std::mem::transmute(methode(link, 9));
            let zet_omschrijving: ZetTekst = std::mem::transmute(methode(link, 7));
            zet_pad(link, w(&doel.to_string_lossy()).as_ptr());
            zet_map(link, w(&map.to_string_lossy()).as_ptr());
            zet_omschrijving(link, w(NAAM).as_ptr());
            let ico = map.join("pictogram.ico");
            if ico.is_file() {
                type ZetIcoon = unsafe extern "system" fn(*mut c_void, *const u16, i32) -> i32;
                let zet_icoon: ZetIcoon = std::mem::transmute(methode(link, 17));
                zet_icoon(link, w(&ico.to_string_lossy()).as_ptr(), 0);
            }
            let qi: Vraag = std::mem::transmute(methode(link, 0));
            let mut bestand: *mut c_void = null_mut();
            let mut gelukt = false;
            if qi(link, &iid_ipersistfile, &mut bestand) == 0 && !bestand.is_null() {
                let bewaar: Bewaar = std::mem::transmute(methode(bestand, 6));
                gelukt = bewaar(bestand, w(&lnk.to_string_lossy()).as_ptr(), 1) == 0;
                let los: Los = std::mem::transmute(methode(bestand, 2));
                los(bestand);
            }
            let los: Los = std::mem::transmute(methode(link, 2));
            los(link);
            gelukt
        }
    }

    fn registersleutel() -> String {
        let schoon: String = NAAM.chars().map(|c| if c.is_alphanumeric() { c } else { '_' }).collect();
        format!("Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\JaapsProgrammeertaal_{}", schoon)
    }

    fn zet_register(iedereen: bool, map: &Path) {
        let wortel = if iedereen { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER };
        let exe = map.join(PROGRAMMA_BESTAND);
        let verwijder = format!("\"{}\" --verwijder", map.join("Verwijderen.exe").display());
        let mut waarden: Vec<(&str, String)> = vec![
            ("DisplayName", NAAM.to_string()),
            ("DisplayIcon", if map.join("pictogram.ico").is_file() { map.join("pictogram.ico") } else { exe.clone() }.to_string_lossy().to_string()),
            ("InstallLocation", map.to_string_lossy().to_string()),
            ("UninstallString", verwijder),
        ];
        if !VERSIE.is_empty() {
            waarden.push(("DisplayVersion", VERSIE.to_string()));
        }
        if !MAKER.is_empty() {
            waarden.push(("Publisher", MAKER.to_string()));
        }
        unsafe {
            let mut sleutel: HANDLE = null_mut();
            if RegCreateKeyExW(wortel, w(&registersleutel()).as_ptr(), 0, null(), 0, KEY_WRITE, null_mut(), &mut sleutel, null_mut()) != 0 {
                return;
            }
            for (naam, waarde) in waarden {
                let data = w(&waarde);
                RegSetValueExW(sleutel, w(naam).as_ptr(), 0, REG_SZ, data.as_ptr() as *const u8, (data.len() * 2) as u32);
            }
            for naam in ["NoModify", "NoRepair"] {
                let een: u32 = 1;
                RegSetValueExW(sleutel, w(naam).as_ptr(), 0, REG_DWORD, &een as *const u32 as *const u8, 4);
            }
            RegCloseKey(sleutel);
        }
    }

    fn snelkoppelingen(iedereen: bool) -> Vec<PathBuf> {
        let mut uit = Vec::new();
        let naam = format!("{}.lnk", PROGRAMMA_NAAM_ZONDER_EXE);
        if STARTMENU {
            if let Some(m) = systeemmap(if iedereen { CSIDL_COMMON_PROGRAMS } else { CSIDL_PROGRAMS }) {
                uit.push(m.join(&naam));
            }
        }
        if BUREAUBLAD {
            if let Some(m) = systeemmap(if iedereen { CSIDL_COMMON_DESKTOPDIRECTORY } else { CSIDL_DESKTOPDIRECTORY }) {
                uit.push(m.join(&naam));
            }
        }
        uit
    }

    fn installeer(k: &Keuzes) -> Result<(), String> {
        let vragen = !k.stil && !k.al_gevraagd;
        if vragen && vraag(&format!("Wil je {} installeren?{}", NAAM, if MAKER.is_empty() { String::new() } else { format!(" Gemaakt door {}.", MAKER) }), MB_YESNO | MB_ICONQUESTION) != IDYES {
            return Ok(());
        }
        if !PRIVACY.is_empty() && vragen {
            let tekst = format!("{}\n\nWil je verder met installeren?", PRIVACY);
            if vraag(&tekst, MB_YESNO | MB_ICONINFORMATION) != IDYES {
                zeg(k, "Ik heb niets geïnstalleerd.", false);
                return Ok(());
            }
        }
        if !LICENTIE.is_empty() && vragen {
            let tekst = format!("{}\n\nGa je akkoord met deze licentie? Alleen als je Ja kiest, ga ik verder.", LICENTIE);
            if vraag(&tekst, MB_YESNO | MB_ICONQUESTION) != IDYES {
                zeg(k, "Je ging niet akkoord met de licentie. Ik heb niets geïnstalleerd.", false);
                return Ok(());
            }
        }
        let iedereen = match (RECHTEN, k.voor_iedereen) {
            (1, _) => false,
            (2, _) => true,
            (_, Some(keuze)) => keuze,
            _ if k.stil => false,
            _ => match vraag(
                &format!(
                    "Voor wie wil je {} installeren?\n\nJa: alleen voor jou. Daarvoor zijn geen beheerdersrechten nodig.\nNee: voor alle gebruikers van deze computer. Daarvoor zijn beheerdersrechten nodig.\nAnnuleren: niet installeren.",
                    NAAM
                ),
                MB_YESNOCANCEL | MB_ICONQUESTION,
            ) {
                IDYES => false,
                IDNO => true,
                _ => return Ok(()),
            },
        };
        if iedereen && !is_beheerder() && k.map.is_none() {
            let extra = if k.stil { "--voor-iedereen --stil" } else { "--voor-iedereen --al-gevraagd" };
            if !opnieuw_als_beheerder(extra) {
                return Err("Zonder toestemming van een beheerder kan ik niet voor alle gebruikers installeren. Je kunt het wel alleen voor jezelf installeren.".into());
            }
            return Ok(());
        }
        let map = match &k.map {
            Some(m) => m.clone(),
            None => {
                let basis = if iedereen {
                    systeemmap(CSIDL_PROGRAM_FILES)
                } else {
                    systeemmap(CSIDL_LOCAL_APPDATA).map(|m| m.join("Programs"))
                };
                basis.ok_or("Ik kan de map voor programma's niet vinden.")?.join(MAP_NAAM)
            }
        };
        std::fs::create_dir_all(&map).map_err(|e| format!("Ik kan de map {} niet maken: {}", map.display(), e))?;
        let schrijf = |naam: &str, data: &[u8]| -> Result<(), String> {
            let pad = map.join(naam);
            if let Some(ouder) = pad.parent() {
                let _ = std::fs::create_dir_all(ouder);
            }
            std::fs::write(&pad, data).map_err(|e| {
                format!("Ik kan {} niet schrijven. Staat {} misschien nog open? Sluit het dan eerst. ({})", pad.display(), NAAM, e)
            })
        };
        schrijf(PROGRAMMA_BESTAND, PROGRAMMA)?;
        for (naam, data) in BESTANDEN {
            schrijf(naam, data)?;
        }
        if !LICENTIE.is_empty() {
            let mut bytes = vec![0xEF, 0xBB, 0xBF];
            bytes.extend_from_slice(LICENTIE.replace('\n', "\r\n").as_bytes());
            schrijf("Licentie.txt", &bytes)?;
        }
        if !PRIVACY.is_empty() {
            let mut bytes = vec![0xEF, 0xBB, 0xBF];
            bytes.extend_from_slice(PRIVACY.replace('\n', "\r\n").as_bytes());
            schrijf("Privacyverklaring.txt", &bytes)?;
        }
        let zelf = std::env::current_exe().map_err(|e| e.to_string())?;
        std::fs::copy(&zelf, map.join("Verwijderen.exe")).map_err(|e| format!("Ik kan het verwijderprogramma niet maken: {}", e))?;
        schrijf("installatie.txt", if iedereen { b"iedereen" } else { b"ik" })?;
        let exe = map.join(PROGRAMMA_BESTAND);
        for lnk in snelkoppelingen(iedereen) {
            if !snelkoppeling(&lnk, &exe, &map) {
                zeg(k, &format!("Let op: de snelkoppeling {} is niet gelukt.", lnk.display()), true);
            }
        }
        zet_register(iedereen, &map);
        if k.stil {
            println!("Geïnstalleerd in {}", map.display());
        } else if vraag(&format!("{} is geïnstalleerd in {}.\n\nWil je het nu starten?", NAAM, map.display()), MB_YESNO | MB_ICONINFORMATION) == IDYES {
            let _ = std::process::Command::new(&exe).current_dir(&map).spawn();
        }
        Ok(())
    }

    fn verwijder(k: &Keuzes) -> Result<(), String> {
        let zelf = std::env::current_exe().map_err(|e| e.to_string())?;
        let map = zelf.parent().ok_or("Ik weet niet waar het programma staat.")?.to_path_buf();
        let iedereen = std::fs::read_to_string(map.join("installatie.txt")).map(|s| s.trim() == "iedereen").unwrap_or(false);
        if !k.stil && !k.al_gevraagd && vraag(&format!("Wil je {} verwijderen van deze computer?", NAAM), MB_YESNO | MB_ICONQUESTION) != IDYES {
            return Ok(());
        }
        if iedereen && !is_beheerder() {
            let extra = if k.stil { "--verwijder --stil" } else { "--verwijder --al-gevraagd" };
            if !opnieuw_als_beheerder(extra) {
                return Err("Zonder toestemming van een beheerder kan ik dit programma niet verwijderen.".into());
            }
            return Ok(());
        }
        for lnk in snelkoppelingen(iedereen) {
            let _ = std::fs::remove_file(lnk);
        }
        unsafe {
            RegDeleteTreeW(if iedereen { HKEY_LOCAL_MACHINE } else { HKEY_CURRENT_USER }, w(&registersleutel()).as_ptr());
        }
        let mut namen: Vec<&str> = vec![PROGRAMMA_BESTAND, "Licentie.txt", "Privacyverklaring.txt", "installatie.txt"];
        namen.extend(BESTANDEN.iter().map(|b| b.0));
        for naam in namen {
            let _ = std::fs::remove_file(map.join(naam));
        }
        // Dit verwijderprogramma draait nog, dus het kan zichzelf niet weghalen.
        // Een kopie in de map voor tijdelijke bestanden ruimt de map op zodra dit programma gestopt is.
        let hulp = std::env::temp_dir().join(format!("{} opruimen.exe", MAP_NAAM));
        if std::fs::copy(&zelf, &hulp).is_ok() {
            let _ = std::process::Command::new(&hulp).arg("--opruimen").arg(&map).spawn();
        }
        zeg(k, &format!("{} is verwijderd.", NAAM), false);
        Ok(())
    }

    /// Wacht tot het verwijderprogramma gestopt is en haalt dan de hele programmamap weg.
    fn opruimen(map: &Path) {
        for _ in 0..60 {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = std::fs::remove_file(map.join("Verwijderen.exe"));
            if std::fs::remove_dir_all(map).is_ok() || !map.exists() {
                break;
            }
        }
    }

    pub fn main() {
        let args: Vec<String> = std::env::args().collect();
        if args.len() >= 3 && args[1] == "--opruimen" {
            opruimen(Path::new(&args[2]));
            return;
        }
        let k = keuzes();
        let verwijderen = k.verwijder;
        let uitkomst = if verwijderen { verwijder(&k) } else { installeer(&k) };
        if let Err(fout) = uitkomst {
            zeg(&k, &fout, true);
            std::process::exit(1);
        }
    }
}
// =================== Einde van het installatieprogramma ===================
