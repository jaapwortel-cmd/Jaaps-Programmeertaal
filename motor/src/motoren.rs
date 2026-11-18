//! Motoren: je eigen bouwblokken. Een motor is een tekstbestand met eigen zinnen (Nieuwe zin: ...).
//! Met één regel gebruik je hem in elk programma: `Gebruik de motor Radio`.
//!
//! Drie motoren zitten erin als voorbeeld: Radio, Brief en Quiz. Je eigen motoren zet je in
//! Documenten\Jaaps Programmeertaal\Motoren (één .txt-bestand per motor). Geen AI: een motor is
//! precies wat erin staat.

use std::path::PathBuf;

/// De drie voorbeeldmotoren.
pub const INGEBOUWD: &[(&str, &str)] = &[
    ("Radio", include_str!("../motoren/Radio.txt")),
    ("Brief", include_str!("../motoren/Brief.txt")),
    ("Quiz", include_str!("../motoren/Quiz.txt")),
];

/// De map met je eigen motoren.
pub fn map() -> PathBuf {
    if let Ok(m) = std::env::var("JAAPTAAL_MOTOREN") {
        return PathBuf::from(m);
    }
    let thuis = std::env::var_os("USERPROFILE").or_else(|| std::env::var_os("HOME")).map(PathBuf::from).unwrap_or_default();
    thuis.join("Documents").join("Jaaps Programmeertaal").join("Motoren")
}

/// De tekst van een motor: eerst je eigen (in de map Motoren), dan de ingebouwde.
pub fn laad(naam: &str) -> Option<String> {
    let n = naam.trim().trim_matches('"').trim();
    if let Ok(lijst) = std::fs::read_dir(map()) {
        for e in lijst.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x.eq_ignore_ascii_case("txt"))
                && p.file_stem().map_or(false, |s| s.to_string_lossy().eq_ignore_ascii_case(n))
            {
                if let Ok(t) = crate::bestand::lees_txt(&p) {
                    return Some(t);
                }
            }
        }
    }
    INGEBOUWD.iter().find(|(m, _)| m.eq_ignore_ascii_case(n)).map(|(_, t)| t.to_string())
}

/// Alle motoren die je kunt gebruiken (namen).
pub fn alle() -> Vec<String> {
    let mut uit: Vec<String> = INGEBOUWD.iter().map(|(m, _)| m.to_string()).collect();
    if let Ok(lijst) = std::fs::read_dir(map()) {
        for e in lijst.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |x| x.eq_ignore_ascii_case("txt")) {
                if let Some(s) = p.file_stem().map(|s| s.to_string_lossy().to_string()) {
                    if !uit.iter().any(|u| u.eq_ignore_ascii_case(&s)) {
                        uit.push(s);
                    }
                }
            }
        }
    }
    uit
}

/// Zoekt "Gebruik de motor X" in een programma. Geeft de tekst van alle motoren, de regels die
/// daarover gaan, en meldingen voor motoren die niet bestaan.
pub fn verzamel(bron: &str) -> (String, Vec<usize>, Vec<crate::melding::Melding>) {
    let mut tekst = String::new();
    let mut regels = Vec::new();
    let mut meldingen = Vec::new();
    for (i, r) in bron.lines().enumerate() {
        let norm = crate::tekst::normaliseer(r);
        let laag = norm.to_lowercase();
        let Some(b) = ["gebruik de motor ", "gebruik motor "].iter().find(|b| laag.starts_with(**b)) else { continue };
        let naam = norm[b.len()..].trim().trim_end_matches('.').to_string();
        regels.push(i + 1);
        match laad(&naam) {
            Some(t) => {
                tekst.push_str(&t);
                tekst.push('\n');
            }
            None => meldingen.push(crate::melding::Melding::vraag(
                i + 1,
                format!(
                    "De motor {} vind ik niet. Ik ken: {}. Een eigen motor zet je in de map {}.",
                    naam,
                    alle().join(", "),
                    map().display()
                ),
            )),
        }
    }
    (tekst, regels, meldingen)
}
