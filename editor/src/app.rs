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

fn eigen_woorden() -> Vec<(String, String)> {
    motor::bestand::lees_txt(&eigen_woorden_pad()).map(|t| motor::bestand::eigen_woorden(&t)).unwrap_or_default()
}

fn lees(bron: &str) -> motor::Uitslag {
    motor::lees_met(bron, &eigen_woorden())
}

fn rust_code(bron: &str) -> Result<String, Vec<Melding>> {
    motor::rust_code_met(bron, &eigen_woorden())
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
    if let Ok(tekst) = motor::bestand::lees_txt(pad) {
        if behandel_van_elders(pad, &tekst) {
            return;
        }
    }
    open_gewoon(pad);
}

fn open_gewoon(pad: &Path) {
    match motor::bestand::lees_txt(pad) {
        Ok(tekst) => {
            zet_tekst(v().editor, &tekst);
            unsafe {
                SendMessageW(v().editor, EM_SETMODIFY, 0, 0);
                SendMessageW(v().editor, EM_EMPTYUNDOBUFFER, 0, 0);
            }
            PAD.with(|p| *p.borrow_mut() = Some(pad.to_path_buf()));
            zet_titel();
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

fn uitvoeren() {
    let Some(bron) = controleer_met_vragen(true) else { return };
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
    let Some(naam) = motor::installatie::uit_programma(&lees(bron).programma).and_then(|i| i.onderteken) else {
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
                if let Some(pad) = bestandskeuze(false, "", TXT_FILTER, "txt", "Programma openen") {
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
        ID_VERTROUW => {
            let naam = motor::installatie::uit_programma(&lees(&editor_tekst()).programma).and_then(|i| i.onderteken);
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
        voeg(bestand, ID_OPSLAAN, "Op&slaan\tCtrl+S");
        voeg(bestand, ID_OPSLAAN_ALS, "Opslaan &als...\tCtrl+Shift+S");
        streep(bestand);
        voeg(bestand, ID_AFSLUITEN, "&Afsluiten\tAlt+F4");
        AppendMenuW(balk, MF_POPUP, bestand as usize, w("&Bestand").as_ptr());

        let prog = CreatePopupMenu();
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
        voeg(prog, ID_VERTROUW, "Mijn certificaat &vertrouwen op deze computer...");
        AppendMenuW(balk, MF_POPUP, prog as usize, w("&Programma").as_ptr());

        let ga = CreatePopupMenu();
        voeg(ga, ID_WELKE_REGEL, "&Op welke regel sta ik?\tCtrl+L");
        voeg(ga, ID_NAAR_MELDINGEN, "Naar het vak &Meldingen\tF4");
        AppendMenuW(balk, MF_POPUP, ga as usize, w("&Ga naar").as_ptr());

        let help = CreatePopupMenu();
        voeg(help, ID_HANDLEIDING, "&Handleiding\tF1");
        voeg(help, ID_EIGEN_WOORDEN, "Mijn &eigen woorden (openen in Kladblok)");
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
