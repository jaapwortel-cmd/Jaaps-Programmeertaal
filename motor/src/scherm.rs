//! Vertelt in woorden wat er op het scherm komt, zonder dat je het hoeft te zien.
//! Met vaste regels voor tips over de indeling en de kleuren (geen AI).

use crate::getallen;
use crate::taal::*;

struct Venster {
    titel: String,
    /// Waar het venster vandaan komt: "bij de start" of "als je op de knop X drukt".
    wanneer: String,
    onderdelen: Vec<(String, String)>,
    achtergrond: (String, (u8, u8, u8)),
    tekst: (String, (u8, u8, u8)),
    sneltoetsen: Vec<(String, String)>,
    /// Tabbladen: naam, en de onderdelen (soort, naam) op dat tabblad.
    tabs: Vec<(String, Vec<(String, String)>)>,
    /// Menubalk: menu, en de regels erin.
    menus: Vec<(String, Vec<String>)>,
    /// Een zelf gekozen Tab-volgorde.
    tabvolgorde: Vec<String>,
}

impl Venster {
    /// Een onderdeel komt op het laatste tabblad, of (zonder tabbladen) in het venster zelf.
    fn voeg_toe(&mut self, soort: &str, naam: &str) {
        match self.tabs.last_mut() {
            Some(t) => t.1.push((soort.into(), naam.into())),
            None => self.onderdelen.push((soort.into(), naam.into())),
        }
    }
    fn alle(&self) -> Vec<(String, String)> {
        let mut a = self.onderdelen.clone();
        for t in &self.tabs {
            a.extend(t.1.iter().cloned());
        }
        a
    }
}

fn verzamel(lijst: &[Opdracht], wanneer: &str, vensters: &mut Vec<Venster>) {
    for o in lijst {
        match &o.soort {
            Soort::MaakVenster { titel } => vensters.push(Venster {
                titel: titel.clone(),
                wanneer: wanneer.to_string(),
                onderdelen: Vec::new(),
                achtergrond: ("lichtgrijs".into(), (240, 240, 240)),
                tekst: ("zwart".into(), (0, 0, 0)),
                sneltoetsen: Vec::new(),
                tabs: Vec::new(),
                menus: Vec::new(),
                tabvolgorde: Vec::new(),
            }),
            Soort::Venstergedrag(Gedrag::TabVolgorde(n)) => {
                if let Some(v) = vensters.last_mut() {
                    v.tabvolgorde = n.clone();
                }
            }
            Soort::MaakKnop { naam } => {
                if let Some(v) = vensters.last_mut() {
                    v.voeg_toe("knop", naam);
                }
            }
            Soort::MaakOnderdeel { soort, naam } => {
                if let Some(v) = vensters.last_mut() {
                    v.voeg_toe(soort.woord(), naam);
                }
            }
            Soort::MaakAfbeelding { pad, beschrijving } => {
                if let Some(v) = vensters.last_mut() {
                    v.voeg_toe("afbeelding", &format!("{} (de schermlezer zegt: {})", pad, beschrijving));
                }
            }
            Soort::MaakTabblad { naam } => {
                if let Some(v) = vensters.last_mut() {
                    v.tabs.push((naam.clone(), Vec::new()));
                }
            }
            Soort::MaakMenu { naam } => {
                if let Some(v) = vensters.last_mut() {
                    v.menus.push((naam.clone(), Vec::new()));
                }
            }
            Soort::MaakMenuregel { naam } => {
                if let Some(m) = vensters.last_mut().and_then(|v| v.menus.last_mut()) {
                    m.1.push(naam.clone());
                }
            }
            Soort::Kleur { achtergrond, kleur, rgb } => {
                if let Some(v) = vensters.last_mut() {
                    if *achtergrond {
                        v.achtergrond = (kleur.clone(), *rgb);
                    } else {
                        v.tekst = (kleur.clone(), *rgb);
                    }
                }
            }
            Soort::Sneltoets { knop, toets } => {
                if let Some(v) = vensters.iter_mut().rev().find(|v| {
                    v.alle().iter().any(|o| o.0 == "knop" && o.1.to_lowercase() == knop.to_lowercase())
                        || v.menus.iter().any(|m| m.1.iter().any(|r| r.to_lowercase() == knop.to_lowercase()))
                }) {
                    v.sneltoetsen.push((knop.clone(), toets.naam.clone()));
                }
            }
            Soort::BijKnop { naam, blok, .. } => verzamel(blok, &format!("als je op de knop {} drukt", naam), vensters),
            Soort::Als { dan, anders, .. } => {
                verzamel(dan, wanneer, vensters);
                verzamel(anders, wanneer, vensters);
            }
            Soort::Herhaal { blok, .. } | Soort::HerhaalZolang { blok, .. } | Soort::VoorElk { blok, .. } | Soort::Tel { blok, .. } => {
                verzamel(blok, wanneer, vensters)
            }
            Soort::Stukje { naam, blok, .. } => verzamel(blok, &format!("als het stukje {} wordt gedaan", naam), vensters),
            _ => {}
        }
    }
}

/// Een beschrijving van alle vensters: titel, kleuren, onderdelen van boven naar beneden,
/// de Tab-volgorde, de sneltoetsen, en tips.
pub fn beschrijf_scherm(p: &Programma) -> String {
    let mut vensters = Vec::new();
    verzamel(&p.opdrachten, "bij de start", &mut vensters);
    let mut uit = Vec::new();
    if vensters.is_empty() {
        uit.push("Je programma maakt geen venster. Alles wat je toont, komt aan het eind samen in één venster met tekst.".to_string());
        uit.push("Tip: wil je knoppen of tekstvakken? Begin dan met: Maak een venster met als titel ...".to_string());
        return uit.join("\n");
    }
    uit.push(format!(
        "Je programma heeft {} {}.",
        vensters.len(),
        if vensters.len() == 1 { "venster" } else { "vensters" }
    ));
    let handleiding = p.opdrachten.iter().any(|o| matches!(o.soort, Soort::Handleiding { .. }));
    for (i, v) in vensters.iter().enumerate() {
        uit.push(String::new());
        uit.push(format!("Venster {}: \"{}\". Het verschijnt {}.", i + 1, v.titel, v.wanneer));
        let ct = contrast(v.achtergrond.1, v.tekst.1);
        let oordeel = if ct >= 7.0 {
            "heel goed te lezen"
        } else if ct >= 4.5 {
            "goed te lezen"
        } else if ct >= 3.0 {
            "matig te lezen, alleen voor grote letters"
        } else {
            "slecht te lezen"
        };
        uit.push(format!(
            "Kleuren: de tekst is {}, de achtergrond is {}. Contrast {}: {}.",
            v.tekst.0,
            v.achtergrond.0,
            getallen::schrijf((ct * 10.0).round() / 10.0),
            oordeel
        ));
        let toets_van = |naam: &str| v.sneltoetsen.iter().find(|s| s.0.to_lowercase() == naam.to_lowercase()).map(|s| s.1.clone());
        if !v.menus.is_empty() {
            uit.push(format!(
                "Bovenaan staat een menubalk met {}: {}. Je opent hem met Alt of F10, en loopt erdoor met de pijltjes.",
                if v.menus.len() == 1 { "één menu" } else { "deze menu's" },
                v.menus.iter().map(|m| m.0.clone()).collect::<Vec<_>>().join(", ")
            ));
            for (menu, regels) in &v.menus {
                let r: Vec<String> = regels
                    .iter()
                    .map(|r| match toets_van(r) {
                        Some(t) => format!("{} ({})", r, t),
                        None => r.clone(),
                    })
                    .collect();
                uit.push(format!("  Menu {}: {}.", menu, if r.is_empty() { "nog leeg".to_string() } else { r.join(", ") }));
            }
        }
        let beschrijf_lijst = |lijst: &[(String, String)], uit: &mut Vec<String>| {
            for (j, (soort, naam)) in lijst.iter().enumerate() {
                let extra = match soort.as_str() {
                    "tekstvak" | "groot tekstvak" | "keuzelijst" | "uitklaplijst" | "tabel" => format!(", met erboven de tekst \"{}:\"", naam),
                    _ => String::new(),
                };
                let toets = if soort == "knop" { toets_van(naam).map(|t| format!(", sneltoets {}", t)).unwrap_or_default() } else { String::new() };
                uit.push(format!("  {}. {} {}{}{}", j + 1, lidwoord(soort), naam, extra, toets));
            }
        };
        if !v.tabs.is_empty() {
            if !v.onderdelen.is_empty() {
                uit.push("Boven de tabbladen staat:".into());
                beschrijf_lijst(&v.onderdelen, &mut uit);
            }
            uit.push(format!(
                "{} {} tabbladen: {}. Je wisselt met Ctrl+Tab (en terug met Ctrl+Shift+Tab), of met de pijltjes als je op de tabbladen staat. De schermlezer zegt dan bijvoorbeeld: \"{}, tabblad 1 van {}\". Je ziet altijd maar één tabblad tegelijk; bij de start is dat {}.",
                if v.onderdelen.is_empty() { "Er staan" } else { "Daaronder staan" },
                v.tabs.len(),
                v.tabs.iter().map(|t| t.0.clone()).collect::<Vec<_>>().join(", "),
                v.tabs[0].0,
                v.tabs.len(),
                v.tabs[0].0
            ));
            for (naam, lijst) in &v.tabs {
                if lijst.is_empty() {
                    uit.push(format!("Het tabblad {} is nog leeg.", naam));
                } else {
                    uit.push(format!("Op het tabblad {} staat, van boven naar beneden:", naam));
                    beschrijf_lijst(lijst, &mut uit);
                }
            }
        }
        if v.alle().is_empty() {
            uit.push("Er staat niets in het venster.".into());
        } else if !v.tabs.is_empty() {
            let eerste: Vec<String> = v.onderdelen.iter().chain(v.tabs[0].1.iter()).filter(|o| o.0 != "label").map(|o| o.1.clone()).collect();
            if !eerste.is_empty() {
                uit.push(format!("Bij de start staat de focus op: {}.", eerste[0]));
            }
        } else {
            uit.push("Van boven naar beneden staat erin:".into());
            for (j, (soort, naam)) in v.onderdelen.iter().enumerate() {
                let extra = match soort.as_str() {
                    "tekstvak" | "groot tekstvak" | "keuzelijst" | "uitklaplijst" | "tabel" => format!(", met erboven de tekst \"{}:\"", naam),
                    _ => String::new(),
                };
                let toets = v
                    .sneltoetsen
                    .iter()
                    .find(|s| soort == "knop" && s.0.to_lowercase() == naam.to_lowercase())
                    .map(|s| format!(", sneltoets {}", s.1))
                    .unwrap_or_default();
                uit.push(format!("  {}. {} {}{}{}", j + 1, lidwoord(soort), naam, extra, toets));
            }
            let mut tab: Vec<String> = v.onderdelen.iter().filter(|o| o.0 != "label" && o.0 != "afbeelding").map(|o| o.1.clone()).collect();
            if !v.tabvolgorde.is_empty() {
                // Eerst de zelf gekozen volgorde, dan de rest zoals ze in het venster staan.
                let mut nieuw: Vec<String> = v.tabvolgorde.iter().filter_map(|n| tab.iter().find(|t| t.to_lowercase() == n.to_lowercase()).cloned()).collect();
                let rest: Vec<String> = tab.iter().filter(|t| !nieuw.contains(t)).cloned().collect();
                nieuw.extend(rest);
                tab = nieuw;
            }
            if !tab.is_empty() {
                uit.push(format!("Met Tab ga je in deze volgorde langs: {}.", tab.join(", ")));
                uit.push(format!("Bij de start staat de focus op: {}.", tab[0]));
            }
        }
        let hoogte = 140
            + if v.tabs.is_empty() { 0 } else { 56 }
            + v.onderdelen.iter().map(|o| hoogte_van(&o.0)).sum::<i32>()
            + v.tabs.iter().map(|t| t.1.iter().map(|o| hoogte_van(&o.0)).sum::<i32>()).max().unwrap_or(0);
        uit.push(format!(
            "Grootte: ongeveer 460 beeldpunten breed en {} hoog. Op een klein scherm of tablet {}.",
            hoogte,
            if hoogte > 700 { "moet je dan schuiven: het venster is hoger dan een klein scherm" } else { "past het ook" }
        ));
        for tip in tips(v, handleiding) {
            uit.push(format!("Tip: {}", tip));
        }
    }
    // Braille (uit Bouwmaatje): pijlen, lijnen en emoji zijn onleesbaar op een brailleleesregel.
    let mut teksten = Vec::new();
    for o in p.alle() {
        for u in o.soort.uitdrukkingen() {
            if let crate::taal::Uitdrukking::Vast(crate::taal::Waarde::Tekst(t)) = u {
                teksten.push(t.clone());
            }
        }
        if let Soort::MaakKnop { naam } | Soort::MaakVenster { titel: naam } = &o.soort {
            teksten.push(naam.clone());
        }
    }
    let mut n = 0;
    for t in teksten {
        let slecht: String = t.chars().filter(|c| slecht_voor_braille(*c)).collect();
        let lijn = ["=====", "-----", "*****", "#####", "~~~~~", "_____"].iter().find(|x| t.contains(**x));
        if (!slecht.is_empty() || lijn.is_some()) && n < 3 {
            let wat = if slecht.is_empty() { format!("een lijn van tekens ({})", lijn.unwrap()) } else { format!("de tekens {}", slecht) };
            uit.push(format!("Tip voor braille: in \"{}\" staat {}. Op een brailleleesregel is dat onleesbaar. Gebruik gewone woorden, bijvoorbeeld \"naar\" in plaats van een pijl.", t.chars().take(60).collect::<String>(), wat));
            n += 1;
        }
    }
    uit.push(String::new());
    uit.push("Altijd: Escape sluit een venster. F1 opent de handleiding van je programma (als je die hebt beschreven).".into());
    uit.join("\n")
}

fn lidwoord(soort: &str) -> String {
    match soort {
        "knop" | "keuzelijst" | "uitklaplijst" | "tabel" | "afbeelding" => format!("de {}", soort),
        _ => format!("het {}", soort),
    }
}

fn hoogte_van(soort: &str) -> i32 {
    match soort {
        "knop" => 64,
        "tekstvak" => 92,
        "groot tekstvak" => 254,
        "label" => 46,
        "selectievakje" => 48,
        "keuzelijst" => 214,
        "tabel" => 274,
        "afbeelding" => 300,
        _ => 94,
    }
}

/// Pijlen, lijnen, vormen en emoji: slecht op een brailleleesregel (uit Bouwmaatje).
pub fn slecht_voor_braille(c: char) -> bool {
    matches!(c as u32, 0x2190..=0x21FF | 0x2500..=0x259F | 0x25A0..=0x25FF | 0x2600..=0x27BF | 0x2B00..=0x2BFF | 0x1F000..=0x1FAFF | 0xFE0F)
}

/// Vaste regels voor een betere indeling. Geen gok: alleen bekende vuistregels.
fn tips(v: &Venster, handleiding: bool) -> Vec<String> {
    let mut t = Vec::new();
    let invoer: Vec<usize> = v
        .onderdelen
        .iter()
        .enumerate()
        .filter(|(_, o)| matches!(o.0.as_str(), "tekstvak" | "groot tekstvak" | "keuzelijst" | "uitklaplijst" | "selectievakje"))
        .map(|(i, _)| i)
        .collect();
    let knoppen: Vec<usize> = v.onderdelen.iter().enumerate().filter(|(_, o)| o.0 == "knop").map(|(i, _)| i).collect();
    if let (Some(&eerste_knop), Some(&laatste_invoer)) = (knoppen.first(), invoer.last()) {
        if eerste_knop < laatste_invoer {
            t.push(format!(
                "de knop {} staat boven een vak om in te vullen. Meestal vul je eerst in en druk je daarna op de knop. Zet de knop dan onder de vakken.",
                v.onderdelen[eerste_knop].1
            ));
        }
    }
    if v.tabs.is_empty() && v.onderdelen.len() > 10 {
        t.push("er staan veel onderdelen in dit venster. Verdeel ze over tabbladen, bijvoorbeeld: Maak een tabblad met als naam Algemeen.".into());
    }
    for (naam, lijst) in &v.tabs {
        if lijst.is_empty() {
            t.push(format!("het tabblad {} is leeg. Zet er iets op, of haal het weg.", naam));
        }
    }
    let ct = contrast(v.achtergrond.1, v.tekst.1);
    if ct < 4.5 {
        let beter = if contrast(v.achtergrond.1, (0, 0, 0)) >= contrast(v.achtergrond.1, (255, 255, 255)) { "zwart" } else { "wit" };
        t.push(format!("kies tekst in het {} voor deze achtergrond; dat leest beter.", beter));
    }
    if v.achtergrond.0 != "lichtgrijs" && v.tekst.0 == "zwart" && contrast(v.achtergrond.1, (0, 0, 0)) < 7.0 {
        t.push("met witte tekst of een lichtere achtergrond wordt het nog beter leesbaar.".into());
    }
    if !knoppen.is_empty() && v.sneltoetsen.is_empty() && knoppen.len() > 1 {
        t.push("geef de belangrijkste knop een sneltoets, bijvoorbeeld: Geef de knop ... de sneltoets Ctrl+Enter.".into());
    }
    if !handleiding {
        t.push("je programma heeft nog geen handleiding voor F1. Schrijf bijvoorbeeld: In de handleiding staat: ...".into());
    }
    t
}
