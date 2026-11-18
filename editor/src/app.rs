//! De editor van Jaap's Programmeertaal: het venster waarin je je zinnen schrijft.
//! Alles werkt met het toetsenbord en met een schermlezer (gewone Windows-onderdelen).

use std::cell::{Cell, RefCell};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::{null, null_mut};
use std::sync::Mutex;

use motor::bouwen::{self, Stand};
use motor::{Melding, MeldingSoort};

use crate::win::*;

const TITEL: &str = "Jaap's Programmeertaal";
const HANDLEIDING: &str = include_str!("../../docs/Handleiding.txt");

const ID_NIEUW: u16 = 101;
const ID_OPENEN: u16 = 102;
const ID_OPSLAAN: u16 = 103;
const ID_OPSLAAN_ALS: u16 = 104;
const ID_AFSLUITEN: u16 = 105;
const ID_CONTROLEER: u16 = 201;
const ID_UITVOEREN: u16 = 202;
const ID_BESCHRIJF: u16 = 203;
const ID_RUSTCODE: u16 = 204;
const ID_SCHERM: u16 = 208;
const ID_PROBEER: u16 = 205;
const ID_BOUW_EXE: u16 = 206;
const ID_INSTALLATIE: u16 = 207;
const ID_VERTROUW: u16 = 209;
const ID_ENGELS: u16 = 210;
const ID_EIGEN_WOORDEN: u16 = 211;
const ID_EIGEN_ZINNEN: u16 = 212;
const ID_OPDRACHT: u16 = 213;
const ID_ZOEK_OPDRACHT: u16 = 214;
const ID_EIGEN_OPDRACHTEN: u16 = 215;
const ID_WENSEN: u16 = 216;
const ID_VERSIE_OMHOOG: u16 = 217;
const ID_MOTOREN: u16 = 218;
const ID_OMSCHRIJVING: u16 = 219;
const ID_GESPREK: u16 = 220;
const ID_EIGEN_CERTIFICAAT: u16 = 221;
/// Onlangs geopend: 121 tot en met 129.
const ID_ONLANGS: u16 = 121;
const ID_WELKE_REGEL: u16 = 301;
const ID_NAAR_MELDINGEN: u16 = 302;
const ID_HANDLEIDING: u16 = 401;
const ID_OVER: u16 = 402;

const WM_KLAAR: u32 = WM_APP + 1;

#[derive(Clone, Copy)]
struct Vensters {
    hoofd: HWND,
    label1: HWND,
    editor: HWND,
    label2: HWND,
    meldingen: HWND,
    accel: HANDLE,
    letter: HANDLE,
    vaste_letter: HANDLE,
}

thread_local! {
    static V: Cell<Option<Vensters>> = const { Cell::new(None) };
    static PAD: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static BEZIG: Cell<bool> = const { Cell::new(false) };
    static LAATSTE_FOCUS: Cell<HWND> = const { Cell::new(null_mut()) };
    static TEKSTVENSTERS: RefCell<Vec<HWND>> = const { RefCell::new(Vec::new()) };
    static ONLANGS_MENU: Cell<HANDLE> = const { Cell::new(null_mut()) };
    static OPDRACHTVENSTER: Cell<Option<Opdrachtvenster>> = const { Cell::new(None) };
    static KEUZES: RefCell<Vec<Keuze>> = const { RefCell::new(Vec::new()) };
    static PRAATVENSTER: Cell<Option<Praatvenster>> = const { Cell::new(None) };
    static GESPREK: RefCell<Option<motor::gesprek::Gesprek>> = const { RefCell::new(None) };
    /// Het programma dat het gesprek het laatst in de editor zette.
    static GESPREK_PROGRAMMA: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Wat een achtergrondklus (bouwen) teruggeeft.
enum Klaar {
    Uitvoeren(Result<PathBuf, String>),
    Probeer(Result<String, String>),
    Exe(Result<(PathBuf, bouwen::Verslag), String>),
    Installatie(Result<(PathBuf, bouwen::Verslag), String>),
}

static KLAAR: Mutex<Option<Klaar>> = Mutex::new(None);

/// Waar jouw eigen woorden staan (zoals "laat zien = toon").
fn eigen_woorden_pad() -> PathBuf {
    let basis = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let map = basis.join("Jaaps Programmeertaal");
    let _ = std::fs::create_dir_all(&map);
    map.join("Mijn eigen woorden.txt")
}

/// Waar jouw eigen zinnen staan die in al je programma's werken (Nieuwe zin: ...).
fn eigen_zinnen_pad() -> PathBuf {
    eigen_woorden_pad().with_file_name("Mijn eigen zinnen.txt")
}

fn eigen_zinnen() -> String {
    motor::bestand::lees_txt(&eigen_zinnen_pad()).unwrap_or_default()
}

/// De lijst met programma's die je onlangs hebt geopend of opgeslagen (hooguit 9).
fn onlangs_pad() -> PathBuf {
    eigen_woorden_pad().with_file_name("Onlangs geopend.txt")
}

fn onlangs() -> Vec<PathBuf> {
    motor::bestand::lees_txt(&onlangs_pad())
        .map(|t| t.lines().filter(|l| !l.trim().is_empty()).map(PathBuf::from).take(9).collect())
        .unwrap_or_default()
}

fn onthoud_onlangs(pad: &Path) {
    let mut lijst = onlangs();
    lijst.retain(|p| p.to_string_lossy().to_lowercase() != pad.to_string_lossy().to_lowercase());
    lijst.insert(0, pad.to_path_buf());
    lijst.truncate(9);
    let tekst: Vec<String> = lijst.iter().map(|p| p.display().to_string()).collect();
    let _ = motor::bestand::schrijf_txt(&onlangs_pad(), &tekst.join("\n"));
    vul_onlangs_menu();
}

fn vul_onlangs_menu() {
    let menu = ONLANGS_MENU.with(|m| m.get());
    if menu.is_null() {
        return;
    }
    unsafe {
        while GetMenuItemCount(menu) > 0 {
            DeleteMenu(menu, 0, MF_BYPOSITION);
        }
        let lijst = onlangs();
        if lijst.is_empty() {
            AppendMenuW(menu, MF_STRING | MF_GRAYED, 0, w("(nog niets geopend)").as_ptr());
        }
        for (i, p) in lijst.iter().enumerate() {
            let naam = p.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let map = p.parent().map(|m| m.display().to_string()).unwrap_or_default();
            let tekst = format!("&{} {} (in {})", i + 1, naam.replace('&', "&&"), map.replace('&', "&&"));
            AppendMenuW(menu, MF_STRING, (ID_ONLANGS + i as u16) as usize, w(&tekst).as_ptr());
        }
    }
}

/// Je zinnen uit een gebouwd programma (.exe) halen en in het vak Opdrachten zetten.
fn open_programma_exe(pad: &Path) {
    let inhoud = match std::fs::read(pad) {
        Ok(i) => i,
        Err(e) => {
            melding(&format!("Ik kan {} niet lezen. Reden: {}", pad.display(), e), MB_ICONERROR);
            return;
        }
    };
    let Some(zinnen) = motor::zinnen_uit_programma(&inhoud) else {
        melding(
            &format!(
                "In {} staan geen zinnen die ik kan openen. Misschien is het niet met Jaap's Programmeertaal gebouwd, of met een oudere versie (van voor 17 november 2026). Zoek dan het .txt-bestand met je zinnen.",
                pad.display()
            ),
            MB_ICONINFORMATION,
        );
        return;
    };
    // Staat het .txt-bestand er nog naast? Dan dat voorstellen.
    let txt = pad.with_extension("txt");
    if txt.is_file() {
        let vraag = format!(
            "Naast dit programma staat ook het bestand {}. Wil je dat openen?\n\nJa: dat bestand openen.\nNee: de zinnen uit het programma zelf halen.",
            txt.display()
        );
        match melding(&vraag, MB_YESNOCANCEL | MB_ICONQUESTION) {
            IDYES => return open_gewoon(&txt),
            IDNO => {}
            _ => return,
        }
    }
    zet_tekst(v().editor, &zinnen);
    unsafe {
        SendMessageW(v().editor, EM_SETMODIFY, 1, 0);
        SendMessageW(v().editor, EM_EMPTYUNDOBUFFER, 0, 0);
    }
    PAD.with(|p| *p.borrow_mut() = Some(mijn_programmas().join(format!("{}.txt", pad.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()))));
    zet_titel();
    zet_meldingen(&format!(
        "Dit zijn de zinnen uit het programma {}. Er staan {} regels in. Ze zijn nog niet opgeslagen: druk Control S om ze te bewaren in Documenten, Jaaps Programmeertaal.",
        pad.display(),
        zinnen.lines().count()
    ));
    ga_naar_regel(1);
}

fn eigen_woorden() -> Vec<(String, String)> {
    motor::bestand::lees_txt(&eigen_woorden_pad()).map(|t| motor::bestand::eigen_woorden(&t)).unwrap_or_default()
}

fn lees(bron: &str) -> motor::Uitslag {
    motor::lees_eigen(bron, &eigen_woorden(), &eigen_zinnen())
}

fn rust_code(bron: &str) -> Result<String, Vec<Melding>> {
    motor::rust_code_eigen(bron, &eigen_woorden(), &eigen_zinnen())
}

fn v() -> Vensters {
    V.with(|v| v.get()).expect("vensters")
}

fn melding(tekst: &str, soort: u32) -> i32 {
    unsafe { MessageBoxW(v().hoofd, w(tekst).as_ptr(), w(TITEL).as_ptr(), soort) }
}

fn lees_tekst(h: HWND) -> String {
    unsafe {
        let lengte = GetWindowTextLengthW(h);
        let mut buf = vec![0u16; lengte as usize + 1];
        let n = GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
        String::from_utf16_lossy(&buf[..n.max(0) as usize])
    }
}

fn zet_tekst(h: HWND, tekst: &str) {
    let t = tekst.replace("\r\n", "\n").replace('\n', "\r\n");
    unsafe {
        SetWindowTextW(h, w(&t).as_ptr());
    }
}

fn editor_tekst() -> String {
    lees_tekst(v().editor)
}

/// Vervangt alle tekst in de editor zo dat Ctrl+Z het weer ongedaan maakt.
fn vervang_editor_tekst(nieuw: &str) {
    let t = nieuw.replace("\r\n", "\n").replace('\n', "\r\n");
    unsafe {
        let e = v().editor;
        SendMessageW(e, EM_SETSEL, 0, -1);
        SendMessageW(e, EM_REPLACESEL, 1, w(&t).as_ptr() as isize);
        SendMessageW(e, EM_SETMODIFY, 1, 0);
    }
}

fn ga_naar_regel(regel: usize) {
    unsafe {
        let e = v().editor;
        let begin = SendMessageW(e, EM_LINEINDEX, regel.saturating_sub(1), 0);
        if begin >= 0 {
            SendMessageW(e, EM_SETSEL, begin as usize, begin);
            SendMessageW(e, EM_SCROLLCARET, 0, 0);
        }
        SetFocus(e);
    }
}

fn huidige_regel() -> (usize, usize) {
    unsafe {
        let e = v().editor;
        let regel = SendMessageW(e, EM_LINEFROMCHAR, usize::MAX, 0) as usize + 1;
        let totaal = SendMessageW(e, EM_GETLINECOUNT, 0, 0) as usize;
        (regel, totaal)
    }
}

/// De geselecteerde tekst, of anders de regel waar de cursor staat. Met het regelnummer waar het begint.
fn selectie_of_regel() -> (String, usize) {
    let tekst: Vec<u16> = editor_tekst().encode_utf16().collect();
    let (mut begin, mut eind) = (0u32, 0u32);
    let eerste_regel;
    unsafe {
        SendMessageW(v().editor, EM_GETSEL, &mut begin as *mut u32 as usize, &mut eind as *mut u32 as isize);
        eerste_regel = SendMessageW(v().editor, EM_LINEFROMCHAR, begin as usize, 0) as usize + 1;
    }
    let (b, e) = (begin as usize, eind as usize);
    if e > b && e <= tekst.len() {
        return (String::from_utf16_lossy(&tekst[b..e]), eerste_regel);
    }
    let (regel, _) = huidige_regel();
    (editor_tekst().lines().nth(regel - 1).unwrap_or("").to_string(), regel)
}

fn is_gewijzigd() -> bool {
    unsafe { SendMessageW(v().editor, EM_GETMODIFY, 0, 0) != 0 }
}

fn zet_titel() {
    let naam = PAD.with(|p| {
        p.borrow().as_ref().and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
    });
    let titel = format!("{} - {}", naam.unwrap_or_else(|| "Nieuw programma".into()), TITEL);
    unsafe {
        SetWindowTextW(v().hoofd, w(&titel).as_ptr());
    }
}

fn zet_meldingen(tekst: &str) {
    zet_tekst(v().meldingen, tekst);
}

fn programmamap() -> Option<PathBuf> {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()))
}

/// Documenten\Jaaps Programmeertaal: hier staan je programma's en notities.
fn mijn_programmas() -> PathBuf {
    let mut buf = [0u16; 520];
    let documenten = unsafe {
        if SHGetFolderPathW(null_mut(), 0x05, null_mut(), 0, buf.as_mut_ptr()) == 0 {
            let n = buf.iter().position(|&c| c == 0).unwrap_or(0);
            PathBuf::from(String::from_utf16_lossy(&buf[..n]))
        } else {
            std::env::var_os("USERPROFILE").map(PathBuf::from).unwrap_or_default().join("Documents")
        }
    };
    let map = documenten.join("Jaaps Programmeertaal");
    let _ = std::fs::create_dir_all(&map);
    map
}

fn werkmap() -> PathBuf {
    let basis = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    basis.join("Jaaps Programmeertaal").join("bouw")
}

// ------------------------------------------------------------------ bestanden

fn bestandskeuze(opslaan: bool, standaardnaam: &str, filter: &str, extensie: &str, titel: &str) -> Option<PathBuf> {
    let mut buf = vec![0u16; 1024];
    for (i, c) in standaardnaam.encode_utf16().take(1000).enumerate() {
        buf[i] = c;
    }
    let filter: Vec<u16> = filter.encode_utf16().chain([0, 0]).map(|c| if c == '|' as u16 { 0 } else { c }).collect();
    let ext = w(extensie);
    let titel = w(titel);
    unsafe {
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = v().hoofd;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.nFilterIndex = 1;
        ofn.lpstrFile = buf.as_mut_ptr();
        ofn.nMaxFile = buf.len() as u32;
        ofn.lpstrDefExt = ext.as_ptr();
        ofn.lpstrTitle = titel.as_ptr();
        let begin = w(&mijn_programmas().to_string_lossy());
        ofn.lpstrInitialDir = begin.as_ptr();
        ofn.Flags = OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_NOCHANGEDIR;
        let gelukt = if opslaan {
            ofn.Flags |= OFN_OVERWRITEPROMPT;
            GetSaveFileNameW(&mut ofn)
        } else {
            ofn.Flags |= OFN_FILEMUSTEXIST;
            GetOpenFileNameW(&mut ofn)
        };
        if gelukt == 0 {
            return None;
        }
    }
    let lengte = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(PathBuf::from(String::from_utf16_lossy(&buf[..lengte])))
}

const TXT_FILTER: &str = "Tekstbestanden (*.txt)|*.txt|Alle bestanden (*.*)|*.*";
const OPEN_FILTER: &str = "Mijn programma's (*.txt en *.exe)|*.txt;*.exe|Tekstbestanden (*.txt)|*.txt|Gebouwde programma's (*.exe)|*.exe|Alle bestanden (*.*)|*.*";

fn opslaan_als() -> bool {
    let standaard = PAD.with(|p| p.borrow().as_ref().and_then(|p| p.file_name()).map(|n| n.to_string_lossy().to_string()));
    let Some(pad) = bestandskeuze(true, &standaard.unwrap_or_else(|| "Mijn programma.txt".into()), TXT_FILTER, "txt", "Programma opslaan als")
    else {
        return false;
    };
    PAD.with(|p| *p.borrow_mut() = Some(pad));
    opslaan()
}

fn opslaan() -> bool {
    let Some(pad) = PAD.with(|p| p.borrow().clone()) else {
        return opslaan_als();
    };
    match motor::bestand::schrijf_txt(&pad, &editor_tekst()) {
        Ok(()) => {
            unsafe {
                SendMessageW(v().editor, EM_SETMODIFY, 0, 0);
            }
            zet_titel();
            onthoud_onlangs(&pad);
            zet_meldingen(&format!("Opgeslagen in {}.", pad.display()));
            true
        }
        Err(e) => {
            melding(&format!("Opslaan is niet gelukt. Bestand: {}. Reden: {}", pad.display(), e), MB_ICONERROR);
            false
        }
    }
}

/// Vraagt of je wilt opslaan als er iets is veranderd. Geeft false als je Annuleren kiest.
fn mag_doorgaan() -> bool {
    if !is_gewijzigd() {
        return true;
    }
    match melding("Je programma is veranderd. Wil je het eerst opslaan?", MB_YESNOCANCEL | MB_ICONQUESTION) {
        IDYES => opslaan(),
        IDNO => true,
        _ => false,
    }
}

/// Een bestand van elders (bijvoorbeeld van je telefoon) dat bij een programma hoort.
/// Eerst voorlezen, dan vragen wat ermee moet. Geeft true als het bestand zo is afgehandeld.
fn behandel_van_elders(pad: &Path, tekst: &str) -> bool {
    let Some((code, naam)) = motor::bestand::hoort_bij(tekst) else { return false };
    let bestandsnaam = motor::installatie::bestandsnaam(&naam);
    if pad.file_stem().map_or(false, |s| s.to_string_lossy().eq_ignore_ascii_case(&bestandsnaam)) {
        return false; // Dit is het programma zelf.
    }
    let inhoud: Vec<&str> = tekst.lines().skip_while(|l| l.trim().is_empty()).skip(1).collect();
    let mut voorbeeld = inhoud.join("\n").trim().to_string();
    if voorbeeld.chars().count() > 1500 {
        voorbeeld = voorbeeld.chars().take(1500).collect::<String>() + " ... (en meer)";
    }
    let vraag = format!(
        "Dit bestand is {} voor het programma {}. Dit staat erin:\n\n{}\n\nWat zal ik doen?\nJa: toevoegen als code aan het programma {}.\nNee: bewaren als notitie bij het programma {}.\nAnnuleren: gewoon openen in de editor.",
        if code { "code" } else { "een omschrijving" },
        naam,
        voorbeeld,
        naam,
        naam
    );
    match melding(&vraag, MB_YESNOCANCEL | MB_ICONQUESTION) {
        IDYES => {
            // Het programma zoeken: eerst in dezelfde map, dan in Documenten\Jaaps Programmeertaal.
            let kandidaten = [
                pad.parent().map(|m| m.join(format!("{}.txt", bestandsnaam))),
                Some(mijn_programmas().join(format!("{}.txt", bestandsnaam))),
            ];
            let doel = kandidaten.iter().flatten().find(|p| p.is_file()).cloned();
            let nieuw_stuk = inhoud.join("\n").trim().to_string();
            match doel {
                Some(doel) => {
                    open_gewoon(&doel);
                    let mut alles = editor_tekst().trim_end().to_string();
                    alles.push_str(&format!("\n\nOpmerking: toegevoegd uit {}\n", pad.display()));
                    alles.push_str(&nieuw_stuk);
                    alles.push('\n');
                    vervang_editor_tekst(&alles);
                    zet_meldingen(&format!(
                        "De regels staan onderaan het programma {}. Ik heb nog niet opgeslagen. Ik controleer ze nu. Ctrl+Z maakt het ongedaan.",
                        naam
                    ));
                    controleer_met_vragen(false);
                }
                None => {
                    let nieuw = mijn_programmas().join(format!("{}.txt", bestandsnaam));
                    if melding(
                        &format!("Er is nog geen programma {}. Zal ik het maken, in {}?", naam, nieuw.display()),
                        MB_YESNO | MB_ICONQUESTION,
                    ) != IDYES
                    {
                        open_gewoon(pad);
                        return true;
                    }
                    zet_tekst(v().editor, &format!("{}\n", nieuw_stuk));
                    PAD.with(|p| *p.borrow_mut() = Some(nieuw.clone()));
                    opslaan();
                    controleer_met_vragen(false);
                }
            }
        }
        IDNO => {
            let notities = mijn_programmas().join(format!("{} notities.txt", bestandsnaam));
            let mut oud = motor::bestand::lees_txt(&notities).unwrap_or_default();
            if !oud.is_empty() && !oud.ends_with('\n') {
                oud.push('\n');
            }
            let nieuw = format!(
                "{}\nUit {}:\n{}\n",
                oud,
                pad.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
                inhoud.join("\n").trim()
            );
            match motor::bestand::schrijf_txt(&notities, &nieuw) {
                Ok(()) => {
                    let t = format!("Bewaard als notitie bij het programma {}, in {}.", naam, notities.display());
                    zet_meldingen(&t);
                    melding(&t, MB_ICONINFORMATION);
                }
                Err(e) => {
                    melding(&format!("Bewaren is niet gelukt: {}", e), MB_ICONERROR);
                }
            }
        }
        _ => open_gewoon(pad),
    }
    true
}

fn open_bestand(pad: &Path) {
    if pad.extension().map_or(false, |e| e.eq_ignore_ascii_case("exe")) {
        return open_programma_exe(pad);
    }
    if let Ok(tekst) = motor::bestand::lees_txt(pad) {
        if behandel_van_elders(pad, &tekst) {
            return;
        }
    }
    open_gewoon(pad);
}

/// De omschrijving wordt een programma; de omschrijving zelf blijft bovenaan als opmerking.
fn omschrijving_vervangen(omschrijving: &str, programma: &str) {
    let mut t = String::from("Opmerking: Dit programma is gemaakt van deze omschrijving:\n");
    for r in omschrijving.lines().filter(|r| !r.trim().is_empty()) {
        t.push_str(&format!("Opmerking: {}\n", r.trim().trim_start_matches("Opmerking:").trim()));
    }
    t.push('\n');
    t.push_str(programma);
    vervang_editor_tekst(&t);
    ga_naar_regel(1);
    zet_meldingen("Je omschrijving is een programma geworden. Druk F7 om te controleren en F5 om het te proberen.");
}

fn open_gewoon(pad: &Path) {
    match motor::bestand::lees_txt(pad) {
        Ok(tekst) => {
            // Een omschrijving in gewone taal (bijvoorbeeld uit Kladblok)? Eerst vragen of het een programma moet worden.
            if let Some((programma, uitleg)) = motor::omschrijving_naar_programma(&tekst) {
                let vraag = format!(
                    "{} is een omschrijving in gewone taal, geen programma.\n\n{}\n\nZal ik er een programma van maken? Ja: het programma komt in de editor (je bestand blijft zoals het is). Nee: gewoon openen.",
                    pad.display(),
                    uitleg
                );
                if melding(&vraag, MB_YESNO | MB_ICONQUESTION) == IDYES {
                    zet_tekst(v().editor, "");
                    omschrijving_vervangen(&tekst, &programma);
                    let naam = pad.file_stem().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                    PAD.with(|p| *p.borrow_mut() = Some(mijn_programmas().join(format!("{} programma.txt", naam))));
                    zet_titel();
                    return;
                }
            }
            zet_tekst(v().editor, &tekst);
            unsafe {
                SendMessageW(v().editor, EM_SETMODIFY, 0, 0);
                SendMessageW(v().editor, EM_EMPTYUNDOBUFFER, 0, 0);
            }
            PAD.with(|p| *p.borrow_mut() = Some(pad.to_path_buf()));
            zet_titel();
            onthoud_onlangs(pad);
            let regels = tekst.lines().count();
            zet_meldingen(&format!("Geopend: {}. Er staan {} regels in.", pad.display(), regels));
            ga_naar_regel(1);
        }
        Err(e) => {
            melding(&format!("Openen is niet gelukt. Bestand: {}. Reden: {}", pad.display(), e), MB_ICONERROR);
        }
    }
}

// ------------------------------------------------------------------ controleren

/// Controleert de zinnen en gaat de meldingen één voor één met je langs.
/// Bij een voorstel vraagt het ja of nee. Alleen bij ja verandert je tekst.
/// Geeft het programma terug als alles goed is.
fn controleer_met_vragen(stil_als_goed: bool) -> Option<String> {
    for _ in 0..50 {
        let bron = editor_tekst();
        let uitslag = lees(&bron);
        let lijst: Vec<String> = uitslag.meldingen.iter().map(|m| m.volledig()).collect();
        let kop = motor::samenvatting(&uitslag);
        zet_meldingen(&format!("{}\n{}", kop, lijst.join("\n")));
        let Some(m) = uitslag.meldingen.first().cloned() else {
            if uitslag.programma.opdrachten.is_empty() {
                melding("Er staan nog geen opdrachten in je programma.", MB_ICONINFORMATION);
                return None;
            }
            if !stil_als_goed {
                melding(&kop, MB_ICONINFORMATION);
            }
            return Some(bron);
        };
        if !behandel(&m, uitslag.meldingen.len()) {
            return None;
        }
    }
    None
}

/// Laat één melding horen. Geeft true als de tekst is aangepast (dan opnieuw controleren).
fn behandel(m: &Melding, aantal: usize) -> bool {
    ga_naar_regel(m.regel);
    let nog = if aantal > 1 {
        format!(" Er zijn nog {} meldingen. Je vindt ze allemaal in het vak Meldingen (F4).", aantal - 1)
    } else {
        String::new()
    };
    match &m.voorstel {
        Some(v) => {
            let tekst = format!("{}{}", m.volledig(), nog);
            if melding(&tekst, MB_YESNO | MB_ICONQUESTION) == IDYES {
                let oud_regel = editor_tekst().lines().nth(m.regel.saturating_sub(1)).unwrap_or("").to_string();
                let nieuw = motor::pas_toe(&editor_tekst(), &v.wijziging);
                vervang_editor_tekst(&nieuw);
                ga_naar_regel(m.regel);
                leer_van_verbetering(&oud_regel, &v.wijziging);
                // Een opdracht voor de editor ("maak er een exe van"): nu uitvoeren.
                if let Some(o) = &m.opdracht {
                    let (naam, extra) = o.split_once('|').unwrap_or((o.as_str(), ""));
                    if let Some(f) = motor::opdrachten::functie_van_naam(naam) {
                        voer_functie_uit(f, extra.to_string());
                        return false;
                    }
                }
                true
            } else {
                zet_meldingen(&format!(
                    "Je zei nee. Ik heb niets veranderd. Pas regel {} zelf aan, of vertel met andere woorden wat je bedoelt.\n{}",
                    m.regel,
                    m.volledig()
                ));
                ga_naar_regel(m.regel);
                false
            }
        }
        None => {
            let soort = if m.soort == MeldingSoort::Fout { MB_ICONWARNING } else { MB_ICONQUESTION };
            melding(&format!("{}{}", m.volledig(), nog), soort);
            ga_naar_regel(m.regel);
            false
        }
    }
}

/// Was het een tikfout in één woord? Dan vragen of het programma dat woord mag onthouden.
fn leer_van_verbetering(oud: &str, w: &motor::Wijziging) {
    let motor::Wijziging::VervangRegel { nieuw, .. } = w else { return };
    let a: Vec<String> = motor::tekst::normaliseer(oud).split_whitespace().map(|x| x.to_lowercase()).collect();
    let b: Vec<String> = motor::tekst::normaliseer(nieuw).split_whitespace().map(|x| x.to_lowercase()).collect();
    if a.len() != b.len() {
        return;
    }
    let verschil: Vec<(String, String)> = a.iter().zip(&b).filter(|(x, y)| x != y).map(|(x, y)| (x.clone(), y.clone())).collect();
    let [(van, naar)] = verschil.as_slice() else { return };
    if van.contains('"') || naar.contains('"') {
        return;
    }
    let vraag = format!(
        "Zal ik onthouden dat je met \"{}\" het woord \"{}\" bedoelt? Dan vraag ik het de volgende keer niet meer. Je kunt dit later zien en veranderen in het menu Help, Mijn eigen woorden.",
        van, naar
    );
    if melding(&vraag, MB_YESNO | MB_ICONQUESTION) == IDYES {
        let pad = eigen_woorden_pad();
        let oud = motor::bestand::lees_txt(&pad).unwrap_or_else(|_| "# Mijn eigen woorden: links wat ik zeg, rechts wat ik bedoel.\n".to_string());
        let nieuw = motor::bestand::voeg_eigen_woord_toe(&oud, van, naar);
        if motor::bestand::schrijf_txt(&pad, &nieuw).is_ok() {
            zet_meldingen(&format!("Onthouden: \"{}\" betekent \"{}\". Dit staat in {}.", van, naar, pad.display()));
        }
    }
}

// ------------------------------------------------------------------ bouwen

fn rustc_of_melding() -> Option<PathBuf> {
    let r = bouwen::zoek_rustc(programmamap().as_deref());
    if r.is_none() {
        melding(
            "Ik kan de ingebouwde Rust-compiler niet vinden. Hij hoort in de map rust naast dit programma. Installeer Jaap's Programmeertaal opnieuw.",
            MB_ICONERROR,
        );
    }
    r
}

fn start_klus(klus: impl FnOnce() -> Klaar + Send + 'static) -> bool {
    if BEZIG.with(|b| b.get()) {
        melding("Ik ben nog bezig met bouwen. Wacht even tot ik klaar ben.", MB_ICONINFORMATION);
        return false;
    }
    BEZIG.with(|b| b.set(true));
    let hoofd = v().hoofd as usize;
    std::thread::spawn(move || {
        let uitkomst = klus();
        *KLAAR.lock().unwrap() = Some(uitkomst);
        unsafe {
            PostMessageW(hoofd as HWND, WM_KLAAR, 0, 0);
        }
    });
    unsafe {
        MessageBeep(0xFFFF_FFFF);
    }
    true
}

/// Firefox zoeken (Jaap wil geen Edge). Geeft het pad, of None.
fn firefox() -> Option<PathBuf> {
    let mut plekken = Vec::new();
    for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
        if let Some(m) = std::env::var_os(var) {
            plekken.push(PathBuf::from(&m).join("Mozilla Firefox").join("firefox.exe"));
        }
    }
    plekken.into_iter().find(|p| p.is_file())
}

/// Een website bouwen in een map en openen in Firefox.
fn website_maken(bron: &str, map: &Path) {
    let (w, meldingen) = motor::website::lees(bron);
    if !meldingen.is_empty() {
        return;
    }
    let bronmap = PAD.with(|p| p.borrow().as_ref().and_then(|p| p.parent().map(|m| m.to_path_buf())));
    match motor::website::maak(&w, map, bronmap.as_deref()) {
        Err(e) => {
            melding(&format!("De website maken is niet gelukt: {}", e), MB_ICONERROR);
        }
        Ok(waarschuwingen) => {
            let index = map.join("index.html");
            let mut tekst = format!("De website staat in {}. Hij gaat open in Firefox.", map.display());
            if !waarschuwingen.is_empty() {
                tekst.push_str(&format!(" Let op: {}", waarschuwingen.join(" ")));
            }
            let gestart = match firefox() {
                Some(f) => Command::new(f).arg(&index).spawn().is_ok(),
                None => {
                    tekst = format!("De website staat in {}. Firefox heb ik niet gevonden; open index.html zelf met je browser.", map.display());
                    false
                }
            };
            let _ = gestart;
            zet_meldingen(&tekst);
        }
    }
}

fn uitvoeren() {
    let Some(bron) = controleer_met_vragen(true) else { return };
    if motor::website::is_website(&bron) {
        website_maken(&bron, &werkmap().join("website"));
        return;
    }
    let Some(rustc) = rustc_of_melding() else { return };
    let code = rust_code(&bron).expect("gecontroleerd");
    let bronmap = PAD.with(|p| p.borrow().as_ref().and_then(|p| p.parent().map(|m| m.to_path_buf())));
    let code = motor::installatie::vul_pictogram_in(&code, &lees(&bron).programma, bronmap.as_deref()).unwrap_or(code);
    zet_meldingen("Bezig met bouwen. Je programma start zo meteen vanzelf.");
    start_klus(move || {
        let map = werkmap();
        // Een nieuwe naam per keer: een programma dat nog openstaat kan niet worden overschreven.
        for oud in std::fs::read_dir(&map).into_iter().flatten().flatten() {
            if oud.file_name().to_string_lossy().starts_with("uitproberen-") {
                let _ = std::fs::remove_file(oud.path());
            }
        }
        let tijd = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0);
        let exe = map.join(format!("uitproberen-{}.exe", tijd));
        Klaar::Uitvoeren(bouwen::bouw(&code, &exe, &rustc, Stand::Snel, &map).map(|_| exe))
    });
}

fn probeer() {
    let (stuk, eerste_regel) = selectie_of_regel();
    if stuk.trim().is_empty() {
        melding("Deze regel is leeg. Zet de cursor op een regel met een opdracht, of selecteer een paar regels.", MB_ICONINFORMATION);
        return;
    }
    // Gebruikt de regel namen van eerder? Dan neem ik de Onthoud-regels van daarvoor mee.
    let eerder: Vec<String> = editor_tekst()
        .lines()
        .take(eerste_regel.saturating_sub(1))
        .filter(|r| r.trim_start().to_lowercase().starts_with("onthoud"))
        .map(|r| r.trim().to_string())
        .collect();
    let met_eerder = format!("{}\n{}", eerder.join("\n"), stuk);
    let stuk = if !eerder.is_empty() && !lees(&stuk).is_goed() && lees(&met_eerder).is_goed() {
        met_eerder
    } else {
        stuk
    };
    let uitslag = lees(&stuk);
    if let Some(m) = uitslag.meldingen.first() {
        melding(
            &format!("Ik probeer alleen deze regel (of je selectie), los van de rest. {} ", m.volledig().replace(" Antwoord met ja of nee.", "")),
            MB_ICONINFORMATION,
        );
        return;
    }
    let Some(rustc) = rustc_of_melding() else { return };
    let code = rust_code(&stuk).expect("gecontroleerd");
    start_klus(move || {
        let map = werkmap();
        let exe = map.join("proberen.exe");
        let uit = bouwen::bouw(&code, &exe, &rustc, Stand::Snel, &map).and_then(|_| {
            let mut cmd = Command::new(&exe);
            cmd.arg("--tekst");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0800_0000);
            }
            cmd.output()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .map_err(|e| format!("Ik kan het programma niet starten: {}", e))
        });
        Klaar::Probeer(uit)
    });
}

/// Vraagt of er een eigen certificaat mag komen, als het programma wil ondertekenen en het er nog niet is.
/// Geeft de naam terug als er ondertekend kan worden, of None om zonder te stoppen.
fn certificaat_klaar(bron: &str) -> Result<Option<String>, ()> {
    let zin = motor::installatie::uit_programma(&lees(bron).programma).and_then(|i| i.onderteken);
    // Zonder de zin Onderteken: met je eigen ingelezen certificaat, als je dat hebt.
    let Some(naam) = zin.or_else(|| motor::ondertekenen::eigen().map(|e| e.1)) else {
        return Ok(None);
    };
    match motor::ondertekenen::heeft_certificaat(&naam) {
        Ok(true) => Ok(Some(naam)),
        Ok(false) => {
            let vraag = format!(
                "Er is nog geen eigen certificaat op naam van {}. Zal ik er een maken? Het blijft op deze computer, alleen voor jou, en is 5 jaar geldig.\n\nLet op: op andere computers blijft Windows nog waarschuwen. Alleen een gekocht certificaat haalt die waarschuwing weg.",
                naam
            );
            if melding(&vraag, MB_YESNO | MB_ICONQUESTION) == IDYES {
                Ok(Some(naam))
            } else {
                zet_meldingen("Je zei nee. Ik heb niets gebouwd. Haal de regel Onderteken ... weg als je zonder ondertekenen wilt bouwen.");
                Err(())
            }
        }
        Err(e) => {
            melding(&e, MB_ICONERROR);
            Err(())
        }
    }
}

fn bouw_exe() {
    let Some(bron) = controleer_met_vragen(true) else { return };
    if motor::website::is_website(&bron) {
        // Een website wordt geen exe, maar een map met pagina's.
        let (w, _) = motor::website::lees(&bron);
        let map = mijn_programmas().join(format!("{} website", w.titel));
        website_maken(&bron, &map);
        return;
    }
    let Some(rustc) = rustc_of_melding() else { return };
    let standaard = PAD
        .with(|p| p.borrow().as_ref().and_then(|p| p.file_stem()).map(|n| format!("{}.exe", n.to_string_lossy())))
        .unwrap_or_else(|| "Mijn programma.exe".into());
    let Some(uit) = bestandskeuze(true, &standaard, "Programma (*.exe)|*.exe", "exe", "Waar moet het programma komen?") else {
        return;
    };
    let Ok(onderteken) = certificaat_klaar(&bron) else { return };
    let code = rust_code(&bron).expect("gecontroleerd");
    let bronmap = PAD.with(|p| p.borrow().as_ref().and_then(|p| p.parent().map(|m| m.to_path_buf())));
    let code = match motor::installatie::vul_pictogram_in(&code, &lees(&bron).programma, bronmap.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            melding(&e, MB_ICONERROR);
            return;
        }
    };
    zet_meldingen("Bezig met het bouwen van het .exe-bestand. Dit kan even duren.");
    start_klus(move || {
        Klaar::Exe(
            bouwen::bouw(&code, &uit, &rustc, Stand::Klein, &werkmap())
                .and_then(|v| match &onderteken {
                    Some(naam) => motor::ondertekenen::onderteken(&uit, naam).map(|_| v),
                    None => Ok(v),
                })
                .map(|v| (uit, v)),
        )
    });
}

fn bouw_installatie() {
    let Some(mut bron) = controleer_met_vragen(true) else { return };
    let mut uitslag = lees(&bron);
    if motor::installatie::uit_programma(&uitslag.programma).is_none() {
        let naam = PAD
            .with(|p| p.borrow().as_ref().and_then(|p| p.file_stem()).map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| "Mijn programma".into());
        let regels = format!(
            "Maak een installatiebestand met als naam {}\nZet een snelkoppeling in het startmenu\nDe installatie mag met en zonder beheerdersrechten",
            naam
        );
        let vraag = format!(
            "Je programma heeft nog geen beschrijving van het installatiebestand. Zal ik deze drie regels onderaan toevoegen?\n\n{}\n\nJe kunt ze daarna zelf aanpassen. Ook een licentie kun je beschrijven, zie de handleiding (F1).",
            regels
        );
        if melding(&vraag, MB_YESNO | MB_ICONQUESTION) != IDYES {
            return;
        }
        let mut nieuw = editor_tekst().trim_end().to_string();
        nieuw.push_str("\n\n");
        nieuw.push_str(&regels);
        nieuw.push('\n');
        vervang_editor_tekst(&nieuw);
        let Some(b) = controleer_met_vragen(true) else { return };
        bron = b;
        uitslag = lees(&bron);
    }
    let Some(inst) = motor::installatie::uit_programma(&uitslag.programma) else { return };
    let Some(rustc) = rustc_of_melding() else { return };
    let bronmap = PAD.with(|p| p.borrow().as_ref().and_then(|p| p.parent().map(|m| m.to_path_buf())));
    if bronmap.is_none() && (!inst.bestanden.is_empty() || inst.licentie_bestand.is_some()) {
        melding("Je installatie gebruikt een eigen bestand. Sla je programma eerst op (Ctrl+S) in de map waar dat bestand staat.", MB_ICONINFORMATION);
        return;
    }
    if certificaat_klaar(&bron).is_err() {
        return;
    }
    let standaard = format!("{} installatie.exe", motor::installatie::bestandsnaam(&inst.naam));
    let Some(uit) = bestandskeuze(true, &standaard, "Installatiebestand (*.exe)|*.exe", "exe", "Waar moet het installatiebestand komen?") else {
        return;
    };
    zet_meldingen("Bezig met het bouwen van je programma en het installatiebestand. Dit kan even duren.");
    start_klus(move || {
        let werk = werkmap().join("installatie");
        Klaar::Installatie(motor::installatie::bouw(&bron, bronmap.as_deref(), &uit, &rustc, &werk).map(|v| (uit, v)))
    });
}

fn klus_klaar() {
    BEZIG.with(|b| b.set(false));
    let Some(k) = KLAAR.lock().unwrap().take() else { return };
    match k {
        Klaar::Uitvoeren(Ok(exe)) => {
            zet_meldingen("Je programma is gestart. Sluit het met Escape of Alt+F4. Daarna ben je weer hier.");
            if let Err(e) = Command::new(&exe).spawn() {
                melding(&format!("Ik kan je programma niet starten: {}", e), MB_ICONERROR);
            }
        }
        Klaar::Probeer(Ok(uit)) => {
            let uit = uit.trim_end();
            let tekst = if uit.is_empty() { "Gedaan. Er kwam niets uit om te tonen.".to_string() } else { format!("Uitkomst:\n{}", uit) };
            zet_meldingen(&tekst);
            melding(&tekst, MB_ICONINFORMATION);
            unsafe {
                SetFocus(v().editor);
            }
        }
        Klaar::Exe(Ok((pad, verslag))) => {
            let tekst = format!(
                "Klaar. Je programma staat in {}. Het is {} groot. Het bouwen duurde {} seconden.",
                pad.display(),
                bouwen::grootte_in_woorden(verslag.grootte),
                verslag.duur.as_secs()
            );
            zet_meldingen(&tekst);
            melding(&tekst, MB_ICONINFORMATION);
        }
        Klaar::Installatie(Ok((pad, verslag))) => {
            let tekst = format!(
                "Klaar. Het installatiebestand staat in {}. Het is {} groot. Geef dit bestand aan iemand, of start het zelf om je programma te installeren.",
                pad.display(),
                bouwen::grootte_in_woorden(verslag.grootte)
            );
            zet_meldingen(&tekst);
            melding(&tekst, MB_ICONINFORMATION);
        }
        Klaar::Installatie(Err(e)) => {
            zet_meldingen(&e);
            melding(&format!("Het installatiebestand is niet gemaakt. {}", e.lines().next().unwrap_or("")), MB_ICONERROR);
        }
        Klaar::Uitvoeren(Err(e)) | Klaar::Probeer(Err(e)) | Klaar::Exe(Err(e)) => {
            zet_meldingen(&e);
            melding(
                "Het bouwen is mislukt. Dit is een fout in Jaap's Programmeertaal zelf, niet in jouw zinnen. De details staan in het vak Meldingen (F4).",
                MB_ICONERROR,
            );
        }
    }
}

// ------------------------------------------------------------------ tekstvensters

fn tekstvenster(titel: &str, tekst: &str, vaste_breedte: bool) {
    let vv = v();
    unsafe {
        let h = CreateWindowExW(
            0,
            w("JaapsTekstVenster").as_ptr(),
            w(&format!("{} - druk op Escape om te sluiten", titel)).as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            900,
            700,
            vv.hoofd,
            null_mut(),
            GetModuleHandleW(null()),
            null_mut(),
        );
        let mut stijl = WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY;
        if vaste_breedte {
            stijl |= WS_HSCROLL | ES_AUTOHSCROLL;
        }
        let vak = CreateWindowExW(WS_EX_CLIENTEDGE, w("EDIT").as_ptr(), null(), stijl, 0, 0, 100, 100, h, null_mut(), GetModuleHandleW(null()), null_mut());
        SendMessageW(vak, EM_SETLIMITTEXT, 0, 0);
        SendMessageW(vak, WM_SETFONT, (if vaste_breedte { vv.vaste_letter } else { vv.letter }) as usize, 1);
        zet_tekst(vak, tekst);
        let mut r = RECT::default();
        GetClientRect(h, &mut r);
        MoveWindow(vak, 0, 0, r.right, r.bottom, 1);
        ShowWindow(h, SW_SHOWMAXIMIZED);
        SetFocus(vak);
        SendMessageW(vak, EM_SETSEL, 0, 0);
        TEKSTVENSTERS.with(|t| t.borrow_mut().push(h));
    }
}

unsafe extern "system" fn tekstvenster_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
    match m {
        WM_SIZE => {
            let vak = crate::win_kind(h);
            if !vak.is_null() {
                MoveWindow(vak, 0, 0, (lp & 0xFFFF) as i32, ((lp >> 16) & 0xFFFF) as i32, 1);
            }
            0
        }
        WM_DESTROY => {
            TEKSTVENSTERS.with(|t| t.borrow_mut().retain(|x| *x != h));
            SetFocus(v().editor);
            0
        }
        _ => DefWindowProcW(h, m, wp, lp),
    }
}

// ------------------------------------------------------------------ opdrachten

fn opdracht(id: u16) {
    match id {
        ID_NIEUW => {
            if mag_doorgaan() {
                zet_tekst(v().editor, "");
                unsafe {
                    SendMessageW(v().editor, EM_SETMODIFY, 0, 0);
                    SendMessageW(v().editor, EM_EMPTYUNDOBUFFER, 0, 0);
                }
                PAD.with(|p| *p.borrow_mut() = None);
                zet_titel();
                zet_meldingen("Nieuw programma. Schrijf of dicteer je zinnen, één opdracht per regel.");
                unsafe {
                    SetFocus(v().editor);
                }
            }
        }
        ID_OPENEN => {
            if mag_doorgaan() {
                if let Some(pad) = bestandskeuze(false, "", OPEN_FILTER, "txt", "Programma openen") {
                    open_bestand(&pad);
                }
            }
        }
        ID_OPSLAAN => {
            opslaan();
        }
        ID_OPSLAAN_ALS => {
            opslaan_als();
        }
        ID_AFSLUITEN => unsafe {
            PostMessageW(v().hoofd, WM_CLOSE, 0, 0);
        },
        ID_CONTROLEER => {
            controleer_met_vragen(false);
        }
        ID_UITVOEREN => uitvoeren(),
        ID_PROBEER => probeer(),
        ID_BOUW_EXE => bouw_exe(),
        ID_INSTALLATIE => bouw_installatie(),
        ID_ENGELS => {
            let u = lees(&editor_tekst());
            if !u.is_goed() {
                melding("Er zijn nog meldingen. Eerst moet alles kloppen, dan zet ik je programma om naar het Engels.", MB_ICONINFORMATION);
                controleer_met_vragen(false);
                return;
            }
            let (engels, niet, overgeslagen) = motor::engels::naar_engels(&u.programma);
            let mut tekst = String::from("Je programma in het Engels. Selecteer alles met Ctrl+A en kopieer het met Ctrl+C.\n\n");
            tekst.push_str(&engels);
            if !niet.is_empty() {
                tekst.push_str("\n\nDeze teksten heb ik niet vertaald (titels, namen en tekst tussen aanhalingstekens). Mijn vaste woordenlijst kan dat niet; vertaal ze zelf als dat nodig is:\n");
                tekst.push_str(&niet.join("\n"));
            }
            if !overgeslagen.is_empty() {
                tekst.push_str(&format!(
                    "\n\nDeze regels kan ik nog niet in het Engels schrijven: {}.",
                    overgeslagen.iter().map(|r| r.to_string()).collect::<Vec<_>>().join(", ")
                ));
            }
            tekstvenster("Je programma in het Engels", &tekst, false);
        }
        ID_ONLANGS..=129 => {
            let lijst = onlangs();
            if let Some(pad) = lijst.get((id - ID_ONLANGS) as usize).cloned() {
                if !pad.exists() {
                    melding(&format!("Het bestand {} is er niet meer. Is het verplaatst of weggegooid?", pad.display()), MB_ICONERROR);
                } else if mag_doorgaan() {
                    open_bestand(&pad);
                }
            }
        }
        ID_OPDRACHT => open_opdrachtvenster(false),
        ID_GESPREK => open_praatvenster(),
        ID_ZOEK_OPDRACHT => open_opdrachtvenster(true),
        ID_EIGEN_OPDRACHTEN => open_in_kladblok(
            &eigen_opdrachten_pad(),
            "# Mijn eigen opdrachten: links wat ik zeg, rechts wat het programma al kent. Eén per regel.\n# Voorbeeld:\nknal hem eruit = bouw een exe\n",
        ),
        ID_WENSEN => open_in_kladblok(&wensen_pad(), "Mijn wensen voor Jaap's Programmeertaal (wat de taal nog niet kan):\n"),
        ID_VERSIE_OMHOOG => versie_omhoog(),
        ID_OMSCHRIJVING => {
            let tekst = editor_tekst();
            match motor::samengesteld::omschrijving(&tekst) {
                Some((regels, uitleg)) => {
                    let vraag = format!("{}\n\nZal ik je omschrijving vervangen door dit programma? Je omschrijving blijft bovenaan staan als opmerking.\n\n{}", uitleg, regels.join("\n"));
                    if melding(&vraag, MB_YESNO | MB_ICONQUESTION) == IDYES {
                        omschrijving_vervangen(&tekst, &regels.join("\n"));
                    }
                }
                None => {
                    melding("In deze tekst vind ik geen programma om te maken. Schrijf bijvoorbeeld: Ik wil een programma dat Adressen heet. Er moet een tekstvak Naam in en een knop Opslaan. Als ik op Opslaan druk, moet de naam in adressen.txt worden bewaard.", MB_ICONINFORMATION);
                }
            }
        }
        ID_MOTOREN => {
            let map = motor::motoren::map();
            let _ = std::fs::create_dir_all(&map);
            // De drie voorbeelden erin zetten, zodat je ze kunt bekijken en nadoen.
            for (naam, tekst) in motor::motoren::INGEBOUWD {
                let p = map.join(format!("{} (voorbeeld).txt", naam));
                if !p.exists() {
                    let _ = motor::bestand::schrijf_txt(&p, tekst);
                }
            }
            if Command::new("explorer.exe").arg(&map).spawn().is_err() {
                melding(&format!("Je motoren staan in {}.", map.display()), MB_ICONINFORMATION);
            }
            zet_meldingen(&format!(
                "De map Motoren is open: {}. Een motor is een tekstbestand met eigen zinnen (Nieuwe zin: ...). Gebruik hem in een programma met: Gebruik de motor Naam. Ingebouwd: {}.",
                map.display(),
                motor::motoren::alle().join(", ")
            ));
        }
        ID_EIGEN_ZINNEN => {
            let pad = eigen_zinnen_pad();
            if !pad.exists() {
                let _ = motor::bestand::schrijf_txt(
                    &pad,
                    "Opmerking: Mijn eigen zinnen. Deze zinnen werken in al mijn programma's.\nOpmerking: Wat tussen [haken] staat, vul je in als je de zin gebruikt.\nNieuwe zin: Begroet [wie]\nToon \"Hallo\" en dan [wie]\nEinde van de zin\n",
                );
            }
            if Command::new("notepad.exe").arg(&pad).spawn().is_err() {
                melding(&format!("Ik kan Kladblok niet openen. Het bestand staat in {}.", pad.display()), MB_ICONERROR);
            }
        }
        ID_EIGEN_WOORDEN => {
            let pad = eigen_woorden_pad();
            if !pad.exists() {
                let _ = motor::bestand::schrijf_txt(
                    &pad,
                    "# Mijn eigen woorden: links wat ik zeg, rechts wat ik bedoel. Een woord per regel.\n# Voorbeeld:\nlaat zien = toon\n",
                );
            }
            if Command::new("notepad.exe").arg(&pad).spawn().is_err() {
                melding(&format!("Ik kan Kladblok niet openen. Het bestand staat in {}.", pad.display()), MB_ICONERROR);
            }
        }
        ID_EIGEN_CERTIFICAAT => {
            let uitleg = "Kies je eigen certificaat: een .pfx-bestand met de geheime sleutel (zonder wachtwoord), of datzelfde certificaat als tekstbestand (.txt). Het komt in Windows, alleen voor jou op deze computer. Daarna onderteken ik al je programma's en installatiebestanden daarmee.";
            if melding(uitleg, MB_OKCANCEL | MB_ICONINFORMATION) != IDOK {
                return;
            }
            let Some(pad) = bestandskeuze(false, "", "Certificaat (*.pfx;*.p12;*.txt)|*.pfx;*.p12;*.txt", "pfx", "Kies je eigen certificaat") else { return };
            match motor::ondertekenen::importeer(&pad, "") {
                Ok(naam) => {
                    let tekst = format!("Klaar. Je certificaat op naam van {} is ingelezen. Vanaf nu onderteken ik je programma's (F8) en installatiebestanden (Ctrl+F8) daarmee.\n\nEerlijk: op andere computers blijft Windows SmartScreen waarschuwen, want het is een zelfgemaakt certificaat.", naam);
                    zet_meldingen(&tekst);
                    melding(&tekst, MB_ICONINFORMATION);
                }
                Err(e) => {
                    melding(&format!("{}\n\nHeeft het bestand een wachtwoord? Dat kan ik nog niet. Maak dan een .pfx zonder wachtwoord.", e), MB_ICONERROR);
                }
            }
        }
        ID_VERTROUW => {
            let naam = motor::installatie::uit_programma(&lees(&editor_tekst()).programma).and_then(|i| i.onderteken).or_else(|| motor::ondertekenen::eigen().map(|e| e.1));
            match naam {
                None => {
                    melding(
                        "Je programma heeft nog geen regel om te ondertekenen. Schrijf bijvoorbeeld: Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel.",
                        MB_ICONINFORMATION,
                    );
                }
                Some(naam) => {
                    if melding(
                        &format!(
                            "Zal ik deze computer jouw certificaat op naam van {} laten vertrouwen? Windows vraagt daarna zelf nog om toestemming, in een eigen venster. Dit geldt alleen voor deze computer.",
                            naam
                        ),
                        MB_YESNO | MB_ICONQUESTION,
                    ) == IDYES
                    {
                        match motor::ondertekenen::vertrouw(&naam) {
                            Ok(()) => melding("Klaar. Deze computer vertrouwt nu programma's met jouw certificaat.", MB_ICONINFORMATION),
                            Err(e) => melding(&e, MB_ICONERROR),
                        };
                    }
                }
            }
        }
        ID_BESCHRIJF if motor::website::is_website(&editor_tekst()) => {
            let (w, _) = motor::website::lees(&editor_tekst());
            tekstvenster("Wat er op je website komt", &motor::website::beschrijf(&w), false);
        }
        ID_BESCHRIJF => {
            let bron = editor_tekst();
            let u = lees(&bron);
            let mut tekst = motor::beschrijf(&u);
            if !u.is_goed() {
                tekst.push_str("\n\nLet op, er zijn nog meldingen:\n");
                tekst.push_str(&u.meldingen.iter().map(|m| m.volledig()).collect::<Vec<_>>().join("\n"));
            }
            tekstvenster("Wat ik van je zinnen maak", &tekst, false);
        }
        ID_SCHERM => {
            let u = lees(&editor_tekst());
            tekstvenster("Wat er op het scherm komt", &motor::scherm::beschrijf_scherm(&u.programma), false);
        }
        ID_RUSTCODE => match rust_code(&editor_tekst()) {
            Ok(code) => tekstvenster("De Rust-code van je programma", motor::rustcode::alleen_programma(&code), true),
            Err(_) => {
                melding("Er zijn nog meldingen. Eerst moet alles kloppen, dan kan ik Rust-code maken.", MB_ICONINFORMATION);
                controleer_met_vragen(false);
            }
        },
        ID_WELKE_REGEL => {
            let (regel, totaal) = huidige_regel();
            let tekst = editor_tekst();
            let inhoud = tekst.lines().nth(regel - 1).unwrap_or("").trim().to_string();
            let inhoud = if inhoud.is_empty() { "een lege regel".to_string() } else { inhoud };
            melding(&format!("Je staat op regel {} van {}. Daar staat: {}", regel, totaal, inhoud), MB_ICONINFORMATION);
        }
        ID_NAAR_MELDINGEN => unsafe {
            SetFocus(v().meldingen);
            SendMessageW(v().meldingen, EM_SETSEL, 0, 0);
        },
        ID_HANDLEIDING => tekstvenster("Handleiding", HANDLEIDING, false),
        ID_OVER => {
            melding(
                &format!(
                    "Jaap's Programmeertaal, versie {}.\nProgrammeren met gewone Nederlandse zinnen.\nZonder AI: het programma werkt alleen met vaste, ingebouwde regels.\nDe Rust-compiler zit in de map rust naast het programma.",
                    motor::VERSIE
                ),
                MB_ICONINFORMATION,
            );
        }
        _ => {}
    }
}

fn indeling(h: HWND) {
    let Some(vv) = V.with(|v| v.get()) else { return };
    let mut r = RECT::default();
    unsafe {
        GetClientRect(h, &mut r);
        let breedte = r.right;
        let hoogte = r.bottom;
        let label = 36;
        let boven = (hoogte - 2 * label) * 2 / 3;
        MoveWindow(vv.label1, 8, 2, breedte - 16, label - 2, 1);
        MoveWindow(vv.editor, 0, label, breedte, boven, 1);
        MoveWindow(vv.label2, 8, label + boven + 2, breedte - 16, label - 2, 1);
        MoveWindow(vv.meldingen, 0, 2 * label + boven, breedte, hoogte - 2 * label - boven, 1);
    }
}

unsafe extern "system" fn hoofd_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
    match m {
        WM_SIZE => {
            indeling(h);
            0
        }
        WM_COMMAND => {
            if lp == 0 {
                opdracht((wp & 0xFFFF) as u16);
            }
            0
        }
        WM_ACTIVATE => {
            if V.with(|v| v.get()).is_some() {
                if wp & 0xFFFF == 0 {
                    LAATSTE_FOCUS.with(|f| f.set(GetFocus()));
                } else {
                    let f = LAATSTE_FOCUS.with(|f| f.get());
                    SetFocus(if f.is_null() || IsWindow(f) == 0 { v().editor } else { f });
                }
            }
            0
        }
        WM_SETFOCUS => {
            if V.with(|v| v.get()).is_some() {
                SetFocus(v().editor);
            }
            0
        }
        WM_KLAAR => {
            klus_klaar();
            0
        }
        WM_CLOSE => {
            if mag_doorgaan() {
                DestroyWindow(h);
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(h, m, wp, lp),
    }
}

fn maak_menu() -> HANDLE {
    unsafe {
        let balk = CreateMenu();
        let voeg = |menu: HANDLE, id: u16, tekst: &str| {
            AppendMenuW(menu, MF_STRING, id as usize, w(tekst).as_ptr());
        };
        let streep = |menu: HANDLE| {
            AppendMenuW(menu, MF_SEPARATOR, 0, null());
        };

        let bestand = CreatePopupMenu();
        voeg(bestand, ID_NIEUW, "&Nieuw\tCtrl+N");
        voeg(bestand, ID_OPENEN, "&Openen...\tCtrl+O");
        let onlangs = CreatePopupMenu();
        ONLANGS_MENU.with(|m| m.set(onlangs));
        AppendMenuW(bestand, MF_POPUP, onlangs as usize, w("Onlangs &geopend").as_ptr());
        voeg(bestand, ID_OPSLAAN, "Op&slaan\tCtrl+S");
        voeg(bestand, ID_OPSLAAN_ALS, "Opslaan &als...\tCtrl+Shift+S");
        streep(bestand);
        voeg(bestand, ID_AFSLUITEN, "&Afsluiten\tAlt+F4");
        AppendMenuW(balk, MF_POPUP, bestand as usize, w("&Bestand").as_ptr());

        let prog = CreatePopupMenu();
        voeg(prog, ID_OPDRACHT, "&Opdracht geven in gewone zinnen...\tCtrl+E");
        voeg(prog, ID_ZOEK_OPDRACHT, "&Zoeken in wat je kunt zeggen...\tCtrl+F");
        voeg(prog, ID_GESPREK, "&Gesprek met de assistent...\tCtrl+G");
        voeg(prog, ID_OMSCHRIJVING, "Maak een programma van deze &omschrijving\tCtrl+Shift+E");
        streep(prog);
        voeg(prog, ID_CONTROLEER, "&Controleren (testen)\tF7");
        voeg(prog, ID_UITVOEREN, "&Uitvoeren\tF5");
        voeg(prog, ID_PROBEER, "&Probeer alleen deze regel\tF9");
        streep(prog);
        voeg(prog, ID_BESCHRIJF, "&Laat zien wat ik van je zinnen maak\tF6");
        voeg(prog, ID_SCHERM, "Vertel wat er op het &scherm komt\tCtrl+F6");
        voeg(prog, ID_RUSTCODE, "Laat de &Rust-code zien\tShift+F6");
        voeg(prog, ID_ENGELS, "Zet mijn programma om naar het &Engels");
        streep(prog);
        voeg(prog, ID_BOUW_EXE, "Bouw een .&exe-bestand...\tF8");
        voeg(prog, ID_INSTALLATIE, "Bouw een &installatiebestand...\tCtrl+F8");
        voeg(prog, ID_EIGEN_CERTIFICAAT, "Mijn eigen certificaat in&lezen (.pfx)...");
        voeg(prog, ID_VERTROUW, "Mijn certificaat &vertrouwen op deze computer...");
        AppendMenuW(balk, MF_POPUP, prog as usize, w("&Programma").as_ptr());

        let ga = CreatePopupMenu();
        voeg(ga, ID_WELKE_REGEL, "&Op welke regel sta ik?\tCtrl+L");
        voeg(ga, ID_NAAR_MELDINGEN, "Naar het vak &Meldingen\tF4");
        AppendMenuW(balk, MF_POPUP, ga as usize, w("&Ga naar").as_ptr());

        let help = CreatePopupMenu();
        voeg(help, ID_HANDLEIDING, "&Handleiding\tF1");
        voeg(help, ID_EIGEN_WOORDEN, "Mijn &eigen woorden (openen in Kladblok)");
        voeg(help, ID_EIGEN_ZINNEN, "Mijn eigen &zinnen (openen in Kladblok)");
        voeg(help, ID_EIGEN_OPDRACHTEN, "Mijn eigen o&pdrachten (openen in Kladblok)");
        voeg(help, ID_WENSEN, "Mijn &wensenlijst (openen in Kladblok)");
        voeg(help, ID_MOTOREN, "Mijn &motoren (map openen)");
        voeg(help, ID_OVER, "&Over Jaap's Programmeertaal");
        AppendMenuW(balk, MF_POPUP, help as usize, w("&Help").as_ptr());
        balk
    }
}

fn maak_sneltoetsen() -> HANDLE {
    let toets = |extra: u8, key: u16, cmd: u16| ACCEL { fVirt: FVIRTKEY | extra, key, cmd };
    let lijst = [
        toets(FCONTROL, b'N' as u16, ID_NIEUW),
        toets(FCONTROL, b'O' as u16, ID_OPENEN),
        toets(FCONTROL, b'S' as u16, ID_OPSLAAN),
        toets(FCONTROL | FSHIFT, b'S' as u16, ID_OPSLAAN_ALS),
        toets(FCONTROL, b'L' as u16, ID_WELKE_REGEL),
        toets(FCONTROL, b'E' as u16, ID_OPDRACHT),
        toets(FCONTROL, b'G' as u16, ID_GESPREK),
        toets(FCONTROL | FSHIFT, b'E' as u16, ID_OMSCHRIJVING),
        toets(FCONTROL, b'F' as u16, ID_ZOEK_OPDRACHT),
        toets(0, VK_F1, ID_HANDLEIDING),
        toets(0, VK_F1 + 3, ID_NAAR_MELDINGEN),
        toets(0, VK_F1 + 4, ID_UITVOEREN),
        toets(0, VK_F1 + 5, ID_BESCHRIJF),
        toets(FSHIFT, VK_F1 + 5, ID_RUSTCODE),
        toets(FCONTROL, VK_F1 + 5, ID_SCHERM),
        toets(0, VK_F1 + 6, ID_CONTROLEER),
        toets(0, VK_F1 + 7, ID_BOUW_EXE),
        toets(FCONTROL, VK_F1 + 7, ID_INSTALLATIE),
        toets(0, VK_F1 + 8, ID_PROBEER),
    ];
    unsafe { CreateAcceleratorTableW(lijst.as_ptr(), lijst.len() as i32) }
}

fn registreer(naam: &str, proc_: WndProc) {
    let naam = w(naam);
    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(proc_),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleW(null()),
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), 32512 as *const u16),
            hbrBackground: 16 as HANDLE,
            lpszMenuName: null(),
            lpszClassName: naam.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);
    }
}

pub fn start() {
    unsafe {
        SetProcessDPIAware();
    }
    registreer("JaapsProgrammeertaalEditor", hoofd_proc);
    registreer("JaapsTekstVenster", tekstvenster_proc);
    registreer("JaapsOpdrachtVenster", opdrachtvenster_proc);
    registreer("JaapsPraatVenster", praatvenster_proc);

    unsafe {
        let inst = GetModuleHandleW(null());
        let dc = GetDC(null_mut());
        let dpi = GetDeviceCaps(dc, LOGPIXELSY).max(96);
        ReleaseDC(null_mut(), dc);
        let letter = CreateFontW(-(24 * dpi / 96), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w("Segoe UI").as_ptr());
        let vaste_letter = CreateFontW(-(22 * dpi / 96), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0x31, w("Consolas").as_ptr());

        let hoofd = CreateWindowExW(
            0,
            w("JaapsProgrammeertaalEditor").as_ptr(),
            w(TITEL).as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1000,
            750,
            null_mut(),
            maak_menu(),
            inst,
            null_mut(),
        );
        let kind = |klasse: &str, tekst: &str, stijl: u32, ex: u32| {
            let h = CreateWindowExW(ex, w(klasse).as_ptr(), w(tekst).as_ptr(), WS_CHILD | WS_VISIBLE | stijl, 0, 0, 10, 10, hoofd, null_mut(), inst, null_mut());
            SendMessageW(h, WM_SETFONT, letter as usize, 1);
            h
        };
        // Het label staat vlak voor het vak: zo leest de schermlezer het als naam van het vak.
        let label1 = kind("STATIC", "Opdrachten (je programma, één opdracht per regel):", 0, 0);
        let editor = kind(
            "EDIT",
            "",
            WS_TABSTOP | WS_VSCROLL | WS_HSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_AUTOHSCROLL | ES_WANTRETURN | ES_NOHIDESEL,
            WS_EX_CLIENTEDGE,
        );
        SendMessageW(editor, EM_SETLIMITTEXT, 0, 0);
        let label2 = kind("STATIC", "Meldingen en uitvoer:", 0, 0);
        let meldingen =
            kind("EDIT", "", WS_TABSTOP | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY, WS_EX_CLIENTEDGE);

        V.with(|v| {
            v.set(Some(Vensters { hoofd, label1, editor, label2, meldingen, accel: maak_sneltoetsen(), letter, vaste_letter }))
        });
        indeling(hoofd);
        vul_onlangs_menu();
        zet_titel();
        zet_meldingen("Welkom bij Jaap's Programmeertaal. Schrijf of dicteer je zinnen in het vak Opdrachten. F1 opent de handleiding.");
        ShowWindow(hoofd, SW_SHOWMAXIMIZED);
        SetForegroundWindow(hoofd);
        SetFocus(editor);

        if let Some(pad) = std::env::args().nth(1) {
            open_bestand(Path::new(&pad));
        }

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            if opdrachtvenster_toets(&msg) || praatvenster_toets(&msg) {
                continue;
            }
            let vv = v();
            let wortel = GetAncestor(msg.hwnd, GA_ROOT);
            if wortel == vv.hoofd {
                if TranslateAcceleratorW(vv.hoofd, vv.accel, &msg) != 0 {
                    continue;
                }
                if msg.message == WM_KEYDOWN && msg.wParam == VK_TAB && GetKeyState(VK_CONTROL) >= 0 {
                    // Tab wisselt tussen het vak Opdrachten en het vak Meldingen.
                    let naar = if GetFocus() == vv.editor { vv.meldingen } else { vv.editor };
                    SetFocus(naar);
                    continue;
                }
                if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE && GetFocus() == vv.meldingen {
                    SetFocus(vv.editor);
                    continue;
                }
            } else if !wortel.is_null()
                && msg.message == WM_KEYDOWN
                && msg.wParam == VK_ESCAPE
                && TEKSTVENSTERS.with(|t| t.borrow().contains(&wortel))
            {
                DestroyWindow(wortel);
                continue;
            }
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

// ------------------------------------------------------------------ opdrachten in gewone zinnen

#[derive(Clone, Copy)]
struct Opdrachtvenster {
    venster: HWND,
    vak: HWND,
    lijst: HWND,
    zoeken: bool,
}

#[derive(Clone)]
enum Keuze {
    Functie(motor::opdrachten::Functie),
    Bestand(PathBuf),
}

fn eigen_opdrachten_pad() -> PathBuf {
    eigen_woorden_pad().with_file_name("Mijn eigen opdrachten.txt")
}

fn wensen_pad() -> PathBuf {
    eigen_woorden_pad().with_file_name("Mijn wensen.txt")
}

fn eigen_opdrachten() -> Vec<(String, String)> {
    motor::bestand::lees_txt(&eigen_opdrachten_pad()).map(|t| motor::opdrachten::eigen_opdrachten(&t)).unwrap_or_default()
}

fn open_in_kladblok(pad: &Path, begin: &str) {
    if !pad.exists() {
        let _ = motor::bestand::schrijf_txt(pad, begin);
    }
    if Command::new("notepad.exe").arg(pad).spawn().is_err() {
        melding(&format!("Ik kan Kladblok niet openen. Het bestand staat in {}.", pad.display()), MB_ICONERROR);
    }
}

fn open_opdrachtvenster(zoeken: bool) {
    let titel = if zoeken {
        "Zoeken in wat je kunt zeggen - Escape sluit"
    } else {
        "Opdracht geven in gewone zinnen - Escape sluit"
    };
    if let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) {
        unsafe {
            DestroyWindow(o.venster);
        }
    }
    let vv = v();
    unsafe {
        let inst = GetModuleHandleW(null());
        let h = CreateWindowExW(0, w("JaapsOpdrachtVenster").as_ptr(), w(titel).as_ptr(), WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN, CW_USEDEFAULT, CW_USEDEFAULT, 900, 600, vv.hoofd, null_mut(), inst, null_mut());
        let kind = |klasse: &str, tekst: &str, stijl: u32, ex: u32| {
            let k = CreateWindowExW(ex, w(klasse).as_ptr(), w(tekst).as_ptr(), WS_CHILD | WS_VISIBLE | stijl, 0, 0, 10, 10, h, null_mut(), inst, null_mut());
            SendMessageW(k, WM_SETFONT, vv.letter as usize, 1);
            k
        };
        let vraag = if zoeken { "Zoek (typ een paar woorden, Enter gaat naar de lijst):" } else { "Wat wil je doen? Typ of dicteer het, en druk Enter:" };
        let _label1 = kind("STATIC", vraag, 0, 0);
        let vak = kind("EDIT", "", WS_TABSTOP | ES_AUTOHSCROLL, WS_EX_CLIENTEDGE);
        let _label2 = kind("STATIC", "Wat je kunt zeggen (Enter voert uit):", 0, 0);
        let lijst = kind("LISTBOX", "", WS_TABSTOP | WS_VSCROLL | LBS_NOTIFY, WS_EX_CLIENTEDGE);
        OPDRACHTVENSTER.with(|o| o.set(Some(Opdrachtvenster { venster: h, vak, lijst, zoeken })));
        indeling_opdrachtvenster(h);
        vul_lijst_zoek("");
        SetFocus(vak);
    }
}

fn indeling_opdrachtvenster(h: HWND) {
    unsafe {
        let mut r = RECT::default();
        GetClientRect(h, &mut r);
        let kinderen = kinderen_van(h);
        if kinderen.len() < 4 {
            return;
        }
        let breed = r.right - 20;
        MoveWindow(kinderen[0], 10, 8, breed, 34, 1);
        MoveWindow(kinderen[1], 10, 44, breed, 40, 1);
        MoveWindow(kinderen[2], 10, 94, breed, 34, 1);
        MoveWindow(kinderen[3], 10, 130, breed, (r.bottom - 140).max(60), 1);
    }
}

fn kinderen_van(h: HWND) -> Vec<HWND> {
    #[link(name = "user32")]
    extern "system" {
        fn GetWindow(h: HWND, cmd: u32) -> HWND;
    }
    let mut uit = Vec::new();
    unsafe {
        let mut k = GetWindow(h, 5);
        while !k.is_null() {
            uit.push(k);
            k = GetWindow(k, 2);
        }
    }
    uit
}

fn zet_keuzes(lijst: Vec<(String, Keuze)>) {
    let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) else { return };
    unsafe {
        SendMessageW(o.lijst, LB_RESETCONTENT, 0, 0);
        for (tekst, _) in &lijst {
            SendMessageW(o.lijst, LB_ADDSTRING, 0, w(tekst).as_ptr() as isize);
        }
        SendMessageW(o.lijst, LB_SETCURSEL, 0, 0);
    }
    KEUZES.with(|k| *k.borrow_mut() = lijst.into_iter().map(|x| x.1).collect());
}

fn vul_lijst_zoek(zoek: &str) {
    let lijst = motor::opdrachten::zoek(zoek).into_iter().map(|a| (motor::opdrachten::regel(a), Keuze::Functie(a.functie))).collect();
    zet_keuzes(lijst);
}

fn sluit_opdrachtvenster() {
    if let Some(o) = OPDRACHTVENSTER.with(|o| o.take()) {
        unsafe {
            DestroyWindow(o.venster);
            SetFocus(v().editor);
        }
    }
}

/// Enter in het vak of in de lijst.
fn opdrachtvenster_enter(in_lijst: bool) {
    let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) else { return };
    if in_lijst {
        let i = unsafe { SendMessageW(o.lijst, LB_GETCURSEL, 0, 0) };
        let keuze = KEUZES.with(|k| k.borrow().get(i.max(0) as usize).cloned());
        match keuze {
            Some(Keuze::Functie(f)) => {
                sluit_opdrachtvenster();
                voer_functie_uit(f, String::new());
            }
            Some(Keuze::Bestand(p)) => {
                sluit_opdrachtvenster();
                if mag_doorgaan() {
                    open_bestand(&p);
                }
            }
            None => {}
        }
        return;
    }
    let zin = lees_tekst(o.vak);
    if o.zoeken {
        unsafe {
            SetFocus(o.lijst);
        }
        return;
    }
    if zin.trim().is_empty() {
        melding("Typ of dicteer eerst wat je wilt. Bijvoorbeeld: maak er een exe van. Met Tab ga je naar de lijst met wat je kunt zeggen.", MB_ICONINFORMATION);
        return;
    }
    behandel_zin(&zin);
}

fn behandel_zin(zin: &str) {
    behandel_antwoord(zin, motor::opdrachten::begrijp_alles(zin, &eigen_opdrachten()));
}

fn behandel_antwoord(zin: &str, antwoord: motor::opdrachten::Antwoord) {
    use motor::opdrachten::{Antwoord, Begrip};
    use motor::wensen::Voorstel;
    let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) else { return };
    match antwoord {
        Antwoord::Lijkt { jaaps_zin, dan } => {
            if melding(&format!("Bedoel je zoiets als: {}?", jaaps_zin), MB_YESNO | MB_ICONQUESTION) == IDYES {
                behandel_antwoord(zin, *dan);
            } else {
                melding("Zeg het dan op een andere manier, of kies met Tab uit de lijst met wat je kunt zeggen.", MB_ICONINFORMATION);
                unsafe {
                    SetFocus(o.vak);
                    SendMessageW(o.vak, EM_SETSEL, 0, -1);
                }
            }
        }
        Antwoord::Taal(Voorstel::Uitleg(u)) => {
            melding(&u, MB_ICONINFORMATION);
            unsafe {
                SetFocus(o.vak);
                SendMessageW(o.vak, EM_SETSEL, 0, -1);
            }
        }
        Antwoord::Editor(Begrip::Doe { functie, extra }) => {
            sluit_opdrachtvenster();
            voer_functie_uit(functie, extra);
        }
        Antwoord::Editor(Begrip::BedoelJe { zin: beter, functie }) => {
            let a = motor::opdrachten::actie(functie);
            if melding(&format!("Bedoel je: {}?\n\nDan doe ik: {}.", beter, a.naam), MB_YESNO | MB_ICONQUESTION) == IDYES {
                sluit_opdrachtvenster();
                voer_functie_uit(functie, String::new());
            }
        }
        Antwoord::Editor(Begrip::Kies(lijst)) => {
            let namen: Vec<String> = lijst.iter().map(|f| motor::opdrachten::actie(*f).naam.to_string()).collect();
            zet_keuzes(lijst.iter().map(|f| (motor::opdrachten::regel(motor::opdrachten::actie(*f)), Keuze::Functie(*f))).collect());
            melding(&format!("Je zin past bij meer dingen: {}. Kies er een uit de lijst en druk Enter.", namen.join(", of ")), MB_ICONQUESTION);
            unsafe {
                SetFocus(o.lijst);
            }
        }
        Antwoord::Editor(Begrip::Onbekend) => {
            vul_lijst_zoek("");
            melding(
                &format!(
                    "Dat snap ik niet: \"{}\". Ik gok niet. Zeg het anders, of kies met Tab uit de lijst met wat je kunt zeggen. Je kunt me ook een eigen opdracht leren: menu Help, Mijn eigen opdrachten.",
                    zin.trim()
                ),
                MB_ICONINFORMATION,
            );
            unsafe {
                SetFocus(o.vak);
                SendMessageW(o.vak, EM_SETSEL, 0, -1);
            }
        }
        Antwoord::Taal(Voorstel::Regels { regels, uitleg }) => {
            let (regel, _) = huidige_regel();
            let vraag = format!("{}\n\nZal ik dit in je programma zetten, onder regel {}?\n\n{}", uitleg, regel, regels.join("\n"));
            if melding(&vraag, MB_YESNO | MB_ICONQUESTION) == IDYES {
                sluit_opdrachtvenster();
                voeg_regels_toe(&regels);
            }
        }
        Antwoord::Taal(Voorstel::Vraag(v)) => {
            melding(&v, MB_ICONQUESTION);
            unsafe {
                SetFocus(o.vak);
                SendMessageW(o.vak, EM_SETSEL, 0, -1);
            }
        }
        Antwoord::Taal(Voorstel::NogNiet(uitleg)) => {
            if melding(&format!("{}\n\nZal ik het op je wensenlijst zetten?", uitleg), MB_YESNO | MB_ICONQUESTION) == IDYES {
                zet_op_wensenlijst(zin);
            }
        }
    }
}

fn zet_op_wensenlijst(zin: &str) {
    let pad = wensen_pad();
    let mut tekst = motor::bestand::lees_txt(&pad).unwrap_or_else(|_| "Mijn wensen voor Jaap's Programmeertaal (wat de taal nog niet kan):\n".into());
    if !tekst.ends_with('\n') {
        tekst.push('\n');
    }
    tekst.push_str(&format!("- {}\n", zin.trim()));
    match motor::bestand::schrijf_txt(&pad, &tekst) {
        Ok(()) => melding(&format!("Staat op je wensenlijst: {}. Je vindt de lijst in het menu Help, Mijn wensenlijst.", zin.trim()), MB_ICONINFORMATION),
        Err(e) => melding(&format!("Het lukte niet om je wens te bewaren. Reden: {}", e), MB_ICONERROR),
    };
}

/// Zet regels onder de regel waar je staat.
fn voeg_regels_toe(regels: &[String]) {
    let ed = v().editor;
    let (regel, _) = huidige_regel();
    unsafe {
        let begin = SendMessageW(ed, EM_LINEINDEX, regel - 1, 0);
        let lengte = SendMessageW(ed, EM_LINELENGTH, begin as usize, 0);
        let eind = begin + lengte;
        SendMessageW(ed, EM_SETSEL, eind as usize, eind);
        let leeg = editor_tekst().trim().is_empty();
        let tekst = if leeg { regels.join("\r\n") } else { format!("\r\n{}", regels.join("\r\n")) };
        SendMessageW(ed, EM_REPLACESEL, 1, w(&tekst).as_ptr() as isize);
        SetFocus(ed);
    }
    let nieuw = if editor_tekst().lines().count() <= regels.len() { 1 } else { regel + 1 };
    zet_meldingen(&format!("Toegevoegd op regel {}: {}. Druk F7 om te controleren.", nieuw, regels.join(" / ")));
}

/// Zoekt een programma op naam: eerst bij onlangs geopend, dan in Documenten\Jaaps Programmeertaal.
fn zoek_programma(naam: &str) -> Vec<PathBuf> {
    let n = naam.to_lowercase();
    let mut alle: Vec<PathBuf> = onlangs();
    if let Ok(lijst) = std::fs::read_dir(mijn_programmas()) {
        for e in lijst.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x.eq_ignore_ascii_case("txt")) && !alle.contains(&p) {
                alle.push(p);
            }
        }
    }
    let stam = |p: &PathBuf| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
    let precies: Vec<PathBuf> = alle.iter().filter(|p| stam(p) == n).cloned().collect();
    if !precies.is_empty() {
        return precies;
    }
    alle.into_iter().filter(|p| stam(p).contains(&n)).collect()
}

fn voer_functie_uit(f: motor::opdrachten::Functie, extra: String) {
    use motor::opdrachten::Functie::*;
    let id = match f {
        Nieuw => ID_NIEUW,
        OpenExe => ID_OPENEN,
        Opslaan => ID_OPSLAAN,
        OpslaanAls => ID_OPSLAAN_ALS,
        Controleren => ID_CONTROLEER,
        Uitvoeren => ID_UITVOEREN,
        ProbeerRegel => ID_PROBEER,
        Beschrijf => ID_BESCHRIJF,
        Scherm => ID_SCHERM,
        RustCode => ID_RUSTCODE,
        Engels => ID_ENGELS,
        BouwExe => ID_BOUW_EXE,
        Installatie => ID_INSTALLATIE,
        Certificaat => ID_VERTROUW,
        WelkeRegel => ID_WELKE_REGEL,
        Meldingen => ID_NAAR_MELDINGEN,
        Handleiding => ID_HANDLEIDING,
        EigenWoorden => ID_EIGEN_WOORDEN,
        EigenZinnen => ID_EIGEN_ZINNEN,
        EigenOpdrachten => ID_EIGEN_OPDRACHTEN,
        Wensenlijst => ID_WENSEN,
        VersieOmhoog => ID_VERSIE_OMHOOG,
        Over => ID_OVER,
        Omschrijving => ID_OMSCHRIJVING,
        WatKanIkZeggen => ID_ZOEK_OPDRACHT,
        Openen => {
            if extra.is_empty() {
                ID_OPENEN
            } else {
                let gevonden = zoek_programma(&extra);
                match gevonden.len() {
                    0 => {
                        melding(
                            &format!("Ik vind geen programma met de naam {} bij onlangs geopend of in Documenten, Jaaps Programmeertaal. Kies het zelf.", extra),
                            MB_ICONINFORMATION,
                        );
                        ID_OPENEN
                    }
                    1 => {
                        if mag_doorgaan() {
                            open_bestand(&gevonden[0]);
                        }
                        return;
                    }
                    _ => {
                        open_opdrachtvenster(false);
                        zet_keuzes(gevonden.iter().map(|p| (format!("Open {}", p.display()), Keuze::Bestand(p.clone()))).collect());
                        melding(&format!("Ik vind meer programma's met {} in de naam. Kies er een uit de lijst en druk Enter.", extra), MB_ICONQUESTION);
                        if let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) {
                            unsafe {
                                SetFocus(o.lijst);
                            }
                        }
                        return;
                    }
                }
            }
        }
        Onlangs => {
            let lijst = onlangs();
            if lijst.is_empty() {
                melding("Je hebt nog niets geopend.", MB_ICONINFORMATION);
                return;
            }
            open_opdrachtvenster(false);
            zet_keuzes(lijst.iter().map(|p| (format!("Open {}", p.display()), Keuze::Bestand(p.clone()))).collect());
            if let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) {
                unsafe {
                    SetFocus(o.lijst);
                }
            }
            return;
        }
        Verder => {
            if editor_tekst().trim().is_empty() {
                if let Some(p) = onlangs().into_iter().find(|p| p.exists()) {
                    open_bestand(&p);
                }
            }
            let totaal = editor_tekst().lines().count().max(1);
            ga_naar_regel(totaal);
            let ed = v().editor;
            unsafe {
                let n = GetWindowTextLengthW(ed) as usize;
                SendMessageW(ed, EM_SETSEL, n, n as isize);
                SetFocus(ed);
            }
            zet_meldingen(&format!("Je staat aan het eind, op regel {}. Typ of dicteer verder.", totaal));
            return;
        }
        NaarRegel => {
            match extra.parse::<usize>() {
                Ok(n) if n >= 1 => {
                    let totaal = editor_tekst().lines().count().max(1);
                    ga_naar_regel(n.min(totaal));
                    zet_meldingen(&format!("Je staat op regel {} van {}.", n.min(totaal), totaal));
                }
                _ => {
                    melding("Naar welke regel? Zeg bijvoorbeeld: ga naar regel 12.", MB_ICONQUESTION);
                }
            }
            return;
        }
        Ongedaan => {
            unsafe {
                SendMessageW(v().editor, EM_UNDO, 0, 0);
                SetFocus(v().editor);
            }
            zet_meldingen("Ongedaan gemaakt.");
            return;
        }
        Afsluiten => {
            unsafe {
                PostMessageW(v().hoofd, WM_CLOSE, 0, 0);
            }
            return;
        }
    };
    opdracht(id);
}

/// Het versienummer ophogen: "De versie is 1.2" wordt "De versie is 1.3". Eerst vragen.
fn versie_omhoog() {
    let tekst = editor_tekst();
    let gevonden = tekst.lines().enumerate().find(|(_, r)| r.trim().to_lowercase().starts_with("de versie is "));
    let Some((i, r)) = gevonden else {
        melding("Je programma heeft nog geen versie. Zeg bijvoorbeeld: de versie moet 1.0 worden.", MB_ICONINFORMATION);
        return;
    };
    let oud = r.trim()["de versie is ".len()..].trim().to_string();
    let mut delen: Vec<String> = oud.split('.').map(|d| d.to_string()).collect();
    let Some(laatste) = delen.last_mut().and_then(|d| d.parse::<u64>().ok()) else {
        melding(&format!("De versie {} kan ik niet ophogen: het laatste stukje is geen getal.", oud), MB_ICONINFORMATION);
        return;
    };
    *delen.last_mut().unwrap() = (laatste + 1).to_string();
    let nieuw = delen.join(".");
    if melding(&format!("Zal ik de versie ophogen van {} naar {} (regel {})?", oud, nieuw, i + 1), MB_YESNO | MB_ICONQUESTION) != IDYES {
        return;
    }
    let regels: Vec<String> = tekst.lines().enumerate().map(|(j, r)| if j == i { format!("De versie is {}", nieuw) } else { r.to_string() }).collect();
    vervang_editor_tekst(&regels.join("\n"));
    zet_meldingen(&format!("De versie is nu {}.", nieuw));
}

unsafe extern "system" fn opdrachtvenster_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
    match m {
        WM_SIZE => {
            indeling_opdrachtvenster(h);
            0
        }
        WM_COMMAND => {
            if let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) {
                let code = (wp >> 16) & 0xFFFF;
                if lp as HWND == o.vak && code == EN_CHANGE {
                    vul_lijst_zoek(&lees_tekst(o.vak));
                } else if lp as HWND == o.lijst && code == LBN_DBLCLK {
                    opdrachtvenster_enter(true);
                }
            }
            0
        }
        WM_DESTROY => {
            OPDRACHTVENSTER.with(|o| {
                if o.get().map_or(false, |x| x.venster == h) {
                    o.set(None);
                }
            });
            0
        }
        _ => DefWindowProcW(h, m, wp, lp),
    }
}

/// Toetsen in het opdrachtvenster: Enter, Escape en Tab. Geeft true als de toets is afgehandeld.
fn opdrachtvenster_toets(msg: &MSG) -> bool {
    let Some(o) = OPDRACHTVENSTER.with(|o| o.get()) else { return false };
    if msg.message != WM_KEYDOWN {
        return false;
    }
    unsafe {
        if GetAncestor(msg.hwnd, GA_ROOT) != o.venster {
            return false;
        }
        match msg.wParam {
            VK_ESCAPE => {
                sluit_opdrachtvenster();
                true
            }
            VK_RETURN => {
                opdrachtvenster_enter(GetFocus() == o.lijst);
                true
            }
            VK_TAB => {
                SetFocus(if GetFocus() == o.vak { o.lijst } else { o.vak });
                true
            }
            _ => false,
        }
    }
}

// ------------------------------------------------------------------ gesprek met de assistent (proefstap 0.2)
//
// Een venster om in gewone zinnen te praten. De motor (motor::gesprek) begrijpt de zinnen met vaste regels,
// zonder AI. Weet de assistent precies wat je wilt, dan zet het venster het programma in de editor en start het
// (F5). Een opdracht voor de editor (zoals een exe bouwen) wordt ook uitgevoerd.

#[derive(Clone, Copy)]
struct Praatvenster {
    venster: HWND,
    vak: HWND,
    gesprek: HWND,
}

fn open_praatvenster() {
    if let Some(p) = PRAATVENSTER.with(|p| p.get()) {
        unsafe {
            SetForegroundWindow(p.venster);
            SetFocus(p.vak);
        }
        return;
    }
    let vv = v();
    unsafe {
        let inst = GetModuleHandleW(null());
        let h = CreateWindowExW(
            0,
            w("JaapsPraatVenster").as_ptr(),
            w("Gesprek met de assistent - Escape sluit").as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            900,
            600,
            vv.hoofd,
            null_mut(),
            inst,
            null_mut(),
        );
        let kind = |klasse: &str, tekst: &str, stijl: u32, ex: u32| {
            let k = CreateWindowExW(ex, w(klasse).as_ptr(), w(tekst).as_ptr(), WS_CHILD | WS_VISIBLE | stijl, 0, 0, 10, 10, h, null_mut(), inst, null_mut());
            SendMessageW(k, WM_SETFONT, vv.letter as usize, 1);
            k
        };
        let _label1 = kind("STATIC", "Wat wil je zeggen? Typ of dicteer het, en druk Enter:", 0, 0);
        let vak = kind("EDIT", "", WS_TABSTOP | ES_AUTOHSCROLL, WS_EX_CLIENTEDGE);
        let _label2 = kind("STATIC", "Het gesprek (het nieuwste staat onderaan):", 0, 0);
        let gesprek = kind("EDIT", "", WS_TABSTOP | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY, WS_EX_CLIENTEDGE);
        PRAATVENSTER.with(|p| p.set(Some(Praatvenster { venster: h, vak, gesprek })));
        indeling_opdrachtvenster(h);
        toon_gesprek();
        SetFocus(vak);
    }
    zeg_hardop("Gesprek met de assistent. Vertel wat voor programma je wilt. Als ik precies weet wat je bedoelt, maak ik het en start ik het. Typ of dicteer, en druk Enter.");
}

/// Zet het hele gesprek in het vak en gaat naar het eind.
fn toon_gesprek() {
    let Some(p) = PRAATVENSTER.with(|p| p.get()) else { return };
    let tekst = GESPREK.with(|g| g.borrow().as_ref().map(|g| g.als_tekst()).unwrap_or_default());
    zet_tekst(p.gesprek, &tekst);
    unsafe {
        SendMessageW(p.gesprek, EM_SETSEL, usize::MAX >> 1, (usize::MAX >> 1) as isize);
        SendMessageW(p.gesprek, EM_SCROLLCARET, 0, 0);
    }
}

fn praatvenster_enter() {
    let Some(p) = PRAATVENSTER.with(|p| p.get()) else { return };
    let zin = lees_tekst(p.vak);
    if zin.trim().is_empty() {
        zeg_hardop("Typ of dicteer eerst wat je wilt zeggen. Bijvoorbeeld: maak een klein programma waar ik tekst in kan typen.");
        return;
    }
    let antwoord = GESPREK.with(|g| g.borrow_mut().get_or_insert_with(motor::gesprek::Gesprek::nieuw).zeg(&zin));
    zet_tekst(p.vak, "");
    toon_gesprek();
    zeg_hardop(&antwoord.tekst);
    if let Some(bron) = antwoord.maak {
        zet_gesprek_programma(&bron);
    }
    if let Some(f) = antwoord.doe {
        voer_functie_uit(f, String::new());
    }
}

/// Zet het programma uit het gesprek in de editor en start het (zoals F5).
/// Staan er andere zinnen in de editor, dan wordt eerst gevraagd of je die wilt opslaan; het wordt een nieuw programma.
fn zet_gesprek_programma(bron: &str) {
    let nu = editor_tekst().replace("\r\n", "\n");
    let van_gesprek = GESPREK_PROGRAMMA.with(|g| g.borrow().as_deref().map(|b| b.replace("\r\n", "\n")) == Some(nu.clone()));
    if !nu.trim().is_empty() && !van_gesprek {
        if !mag_doorgaan() {
            zeg_hardop("Goed, ik heb je zinnen in de editor laten staan. Het programma is niet gemaakt.");
            return;
        }
        PAD.with(|p| *p.borrow_mut() = None);
        zet_titel();
    }
    vervang_editor_tekst(bron);
    GESPREK_PROGRAMMA.with(|g| *g.borrow_mut() = Some(editor_tekst()));
    zet_meldingen("Het programma uit het gesprek staat in het vak Opdrachten. Het wordt nu gebouwd en gestart.");
    uitvoeren();
}

/// Laat de schermlezer een tekst voorlezen (UI Automation-melding, voor JAWS, NVDA en Verteller).
/// De functies worden pas bij gebruik opgezocht: werkt het niet, dan staat de tekst nog in het vak Het gesprek.
fn zeg_hardop(tekst: &str) {
    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(naam: *const u16) -> HANDLE;
        fn GetProcAddress(m: HANDLE, naam: *const u8) -> *const core::ffi::c_void;
    }
    type Gastheer = unsafe extern "system" fn(HWND, *mut *mut core::ffi::c_void) -> i32;
    type Melding = unsafe extern "system" fn(*mut core::ffi::c_void, i32, i32, *mut u16, *mut u16) -> i32;
    type Maak = unsafe extern "system" fn(*const u16) -> *mut u16;
    type Vrij = unsafe extern "system" fn(*mut u16);
    let Some(p) = PRAATVENSTER.with(|p| p.get()) else { return };
    unsafe {
        let uia = LoadLibraryW(w("uiautomationcore.dll").as_ptr());
        let ole = LoadLibraryW(w("oleaut32.dll").as_ptr());
        if uia.is_null() || ole.is_null() {
            return;
        }
        let gastheer = GetProcAddress(uia, b"UiaHostProviderFromHwnd\0".as_ptr());
        let melding = GetProcAddress(uia, b"UiaRaiseNotificationEvent\0".as_ptr());
        let maak = GetProcAddress(ole, b"SysAllocString\0".as_ptr());
        let vrij = GetProcAddress(ole, b"SysFreeString\0".as_ptr());
        if gastheer.is_null() || melding.is_null() || maak.is_null() || vrij.is_null() {
            return;
        }
        let gastheer: Gastheer = std::mem::transmute(gastheer);
        let melding: Melding = std::mem::transmute(melding);
        let maak: Maak = std::mem::transmute(maak);
        let vrij: Vrij = std::mem::transmute(vrij);
        let mut leverancier: *mut core::ffi::c_void = null_mut();
        if gastheer(p.vak, &mut leverancier) < 0 || leverancier.is_null() {
            return;
        }
        let tekst_b = maak(w(tekst).as_ptr());
        let id_b = maak(w("JaapsProgrammeertaalGesprek").as_ptr());
        // Soort: Other (4). Verwerking: ImportantMostRecent (1), zodat een nieuw antwoord het oude onderbreekt.
        melding(leverancier, 4, 1, tekst_b, id_b);
        vrij(tekst_b);
        vrij(id_b);
        // IUnknown::Release (plek 2 in de tabel).
        let tabel = *(leverancier as *const *const usize);
        let release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32 = std::mem::transmute(*tabel.add(2));
        release(leverancier);
    }
}

unsafe extern "system" fn praatvenster_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
    match m {
        WM_SIZE => {
            indeling_opdrachtvenster(h);
            0
        }
        WM_DESTROY => {
            PRAATVENSTER.with(|p| {
                if p.get().map_or(false, |x| x.venster == h) {
                    p.set(None);
                }
            });
            0
        }
        _ => DefWindowProcW(h, m, wp, lp),
    }
}

/// Toetsen in het praatvenster: Enter, Escape en Tab. Geeft true als de toets is afgehandeld.
fn praatvenster_toets(msg: &MSG) -> bool {
    let Some(p) = PRAATVENSTER.with(|p| p.get()) else { return false };
    if msg.message != WM_KEYDOWN {
        return false;
    }
    unsafe {
        if GetAncestor(msg.hwnd, GA_ROOT) != p.venster {
            return false;
        }
        match msg.wParam {
            VK_ESCAPE => {
                DestroyWindow(p.venster);
                SetFocus(v().editor);
                true
            }
            VK_RETURN if GetFocus() == p.vak => {
                praatvenster_enter();
                true
            }
            VK_TAB => {
                SetFocus(if GetFocus() == p.vak { p.gesprek } else { p.vak });
                true
            }
            _ => false,
        }
    }
}
