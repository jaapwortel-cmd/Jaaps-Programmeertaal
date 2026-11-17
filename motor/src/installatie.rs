//! Een installatiebestand maken voor je eigen programma.
//!
//! Je beschrijft de installatie met zinnen (zie de handleiding). De motor bouwt eerst
//! je programma, en daarna een installatieprogramma waar je programma in zit.
//! Dat installatieprogramma wordt ook met de ingebouwde Rust-compiler gebouwd.

use std::path::{Path, PathBuf};

use crate::bouwen::{self, Stand, Verslag};
use crate::taal::{InstallatieZin, Programma, Rechten, Soort};

/// Het sjabloon van het installatieprogramma.
pub const SJABLOON: &str = include_str!("../looptijd/installatie.rs");

/// Wat je over de installatie hebt beschreven.
#[derive(Clone, Debug, PartialEq)]
pub struct Installatie {
    pub naam: String,
    pub versie: String,
    pub maker: String,
    pub bureaublad: bool,
    pub startmenu: bool,
    pub rechten: Rechten,
    pub bestanden: Vec<String>,
    pub licentie_regels: Vec<String>,
    pub licentie_bestand: Option<String>,
    pub privacy_regels: Vec<String>,
    pub privacy_bestand: Option<String>,
    /// Ondertekenen met een eigen certificaat op deze naam.
    pub onderteken: Option<String>,
}

/// Haalt de beschrijving van de installatie uit het programma. `None` als er geen naam is.
pub fn uit_programma(p: &Programma) -> Option<Installatie> {
    let mut i = Installatie {
        naam: String::new(),
        versie: String::new(),
        maker: String::new(),
        bureaublad: false,
        startmenu: false,
        rechten: Rechten::MetEnZonder,
        bestanden: Vec::new(),
        licentie_regels: Vec::new(),
        licentie_bestand: None,
        privacy_regels: Vec::new(),
        privacy_bestand: None,
        onderteken: None,
    };
    for o in &p.opdrachten {
        if let Soort::Installatie(z) = &o.soort {
            match z {
                InstallatieZin::Naam(n) => i.naam = n.clone(),
                InstallatieZin::Versie(v) => i.versie = v.clone(),
                InstallatieZin::Maker(m) => i.maker = m.clone(),
                InstallatieZin::Bureaublad => i.bureaublad = true,
                InstallatieZin::Startmenu => i.startmenu = true,
                InstallatieZin::Rechten(r) => i.rechten = *r,
                InstallatieZin::Bestand(b) => i.bestanden.push(b.clone()),
                InstallatieZin::LicentieRegel(t) => i.licentie_regels.push(t.clone()),
                InstallatieZin::LicentieBestand(b) => i.licentie_bestand = Some(b.clone()),
                InstallatieZin::PrivacyRegel(t) => i.privacy_regels.push(t.clone()),
                InstallatieZin::PrivacyBestand(b) => i.privacy_bestand = Some(b.clone()),
                InstallatieZin::Onderteken(n) => i.onderteken = Some(n.clone()),
            }
        }
    }
    (!i.naam.is_empty()).then_some(i)
}

/// Een naam die als bestandsnaam kan: zonder tekens die Windows niet toestaat.
pub fn bestandsnaam(naam: &str) -> String {
    let schoon: String = naam.chars().filter(|c| !"\\/:*?\"<>|".contains(*c) && !c.is_control()).collect();
    let schoon = schoon.trim().trim_end_matches('.').to_string();
    if schoon.is_empty() {
        "Mijn programma".into()
    } else {
        schoon
    }
}

fn zoek_bestand(naam: &str, bronmap: Option<&Path>) -> Result<PathBuf, String> {
    let pad = PathBuf::from(naam);
    let kandidaat = if pad.is_absolute() { pad } else { bronmap.map(|m| m.join(naam)).unwrap_or(pad) };
    if kandidaat.is_file() {
        Ok(kandidaat)
    } else {
        Err(format!(
            "Ik kan het bestand {} niet vinden. Zet het in dezelfde map als je programma, en sla je programma eerst op. Of schrijf het hele pad, zoals C:\\Mappen\\{}.",
            naam, naam
        ))
    }
}

/// De tekst van de privacyverklaring, met een kopje erboven. Leeg als er geen is.
pub fn privacytekst(i: &Installatie, bronmap: Option<&Path>) -> Result<String, String> {
    let inhoud = if let Some(b) = &i.privacy_bestand {
        let pad = zoek_bestand(b, bronmap)?;
        crate::bestand::lees_txt(&pad).map_err(|e| format!("Ik kan de privacyverklaring {} niet lezen: {}", pad.display(), e))?
    } else if !i.privacy_regels.is_empty() {
        i.privacy_regels.join("\n")
    } else {
        return Ok(String::new());
    };
    let mut kop = format!("Privacyverklaring van {}", i.naam);
    if !i.maker.is_empty() {
        kop.push_str(&format!(". Gemaakt door {}", i.maker));
    }
    kop.push('.');
    Ok(format!("{}\n\n{}", kop, inhoud.replace("\r\n", "\n").trim_end()))
}

/// De tekst van de licentie, met een kopje erboven.
pub fn licentietekst(i: &Installatie, bronmap: Option<&Path>) -> Result<String, String> {
    let inhoud = if let Some(b) = &i.licentie_bestand {
        let pad = zoek_bestand(b, bronmap)?;
        crate::bestand::lees_txt(&pad).map_err(|e| format!("Ik kan de licentie {} niet lezen: {}", pad.display(), e))?
    } else if !i.licentie_regels.is_empty() {
        i.licentie_regels.join("\n")
    } else {
        return Ok(String::new());
    };
    let mut kop = format!("Licentie van {}", i.naam);
    if !i.versie.is_empty() {
        kop.push_str(&format!(", versie {}", i.versie));
    }
    if !i.maker.is_empty() {
        kop.push_str(&format!(". Gemaakt door {}", i.maker));
    }
    kop.push('.');
    Ok(format!("{}\n\n{}", kop, inhoud.replace("\r\n", "\n").trim_end()))
}

fn rs(s: &str) -> String {
    format!("{:?}", s)
}

/// Maakt de Rust-code van het installatieprogramma.
pub fn maak_code(i: &Installatie, programma_exe: &Path, bestanden: &[(String, PathBuf)], licentie: &str, privacy: &str) -> String {
    let naam = bestandsnaam(&i.naam);
    let rechten = match i.rechten {
        Rechten::MetEnZonder => 0,
        Rechten::AlleenZonder => 1,
        Rechten::AlleenMet => 2,
    };
    let mut c = String::from(SJABLOON);
    c.push_str("\n// ==================== Jouw installatie ====================\n");
    c.push_str(&format!("const NAAM: &str = {};\n", rs(&i.naam)));
    c.push_str(&format!("const VERSIE: &str = {};\n", rs(&i.versie)));
    c.push_str(&format!("const MAKER: &str = {};\n", rs(&i.maker)));
    c.push_str(&format!("const BUREAUBLAD: bool = {};\n", i.bureaublad));
    c.push_str(&format!("const STARTMENU: bool = {};\n", i.startmenu));
    c.push_str(&format!("const RECHTEN: u8 = {};\n", rechten));
    c.push_str(&format!("const MAP_NAAM: &str = {};\n", rs(&naam)));
    c.push_str(&format!("const PROGRAMMA_NAAM_ZONDER_EXE: &str = {};\n", rs(&naam)));
    c.push_str(&format!("const PROGRAMMA_BESTAND: &str = {};\n", rs(&format!("{}.exe", naam))));
    c.push_str(&format!("const LICENTIE: &str = {};\n", rs(licentie)));
    c.push_str(&format!("const PRIVACY: &str = {};\n", rs(privacy)));
    c.push_str(&format!("static PROGRAMMA: &[u8] = include_bytes!({});\n", rs(&programma_exe.to_string_lossy())));
    c.push_str("static BESTANDEN: &[(&str, &[u8])] = &[\n");
    for (doelnaam, pad) in bestanden {
        c.push_str(&format!("    ({}, include_bytes!({})),\n", rs(doelnaam), rs(&pad.to_string_lossy())));
    }
    c.push_str("];\n");
    c
}

/// Bouwt het installatiebestand: eerst je programma, dan het installatieprogramma.
/// `bronmap` is de map waar je programma (het TXT-bestand) staat; daar worden extra bestanden gezocht.
pub fn bouw(bron: &str, bronmap: Option<&Path>, uit: &Path, rustc: &Path, werkmap: &Path) -> Result<Verslag, String> {
    let begin = std::time::Instant::now();
    let uitslag = crate::lees(bron);
    if let Some(m) = uitslag.meldingen.first() {
        return Err(format!("Eerst moet alles kloppen. {}", m.volledig()));
    }
    let Some(inst) = uit_programma(&uitslag.programma) else {
        return Err("Je programma heeft nog geen beschrijving van de installatie. Schrijf bijvoorbeeld: Maak een installatiebestand met als naam Begroeting.".into());
    };
    let mut extra = Vec::new();
    for b in &inst.bestanden {
        let pad = zoek_bestand(b, bronmap)?;
        let doelnaam = pad.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| b.clone());
        extra.push((doelnaam, std::fs::canonicalize(&pad).unwrap_or(pad)));
    }
    let licentie = licentietekst(&inst, bronmap)?;
    let privacy = privacytekst(&inst, bronmap)?;

    let regels: Vec<&str> = bron.lines().collect();
    let code = crate::rustcode::maak(&uitslag.programma, &regels);
    let programmamap = werkmap.join("programma");
    let exe = programmamap.join(format!("{}.exe", bestandsnaam(&inst.naam)));
    bouwen::bouw(&code, &exe, rustc, Stand::Klein, &programmamap)?;
    let exe = std::fs::canonicalize(&exe).unwrap_or(exe);
    if let Some(naam) = &inst.onderteken {
        crate::ondertekenen::onderteken(&exe, naam)?;
    }

    let installcode = maak_code(&inst, &exe, &extra, &licentie, &privacy);
    let installmap = werkmap.join("installatie");
    let v = bouwen::bouw(&installcode, uit, rustc, Stand::Klein, &installmap)?;
    if let Some(naam) = &inst.onderteken {
        let volledig = if uit.is_absolute() { uit.to_path_buf() } else { std::env::current_dir().unwrap_or_default().join(uit) };
        crate::ondertekenen::onderteken(&volledig, naam)?;
    }
    Ok(Verslag { duur: begin.elapsed(), grootte: v.grootte })
}
