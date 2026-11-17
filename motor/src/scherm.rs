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
            }),
            Soort::MaakKnop { naam } => {
                if let Some(v) = vensters.last_mut() {
                    v.onderdelen.push(("knop".into(), naam.clone()));
                }
            }
            Soort::MaakOnderdeel { soort, naam } => {
                if let Some(v) = vensters.last_mut() {
                    v.onderdelen.push((soort.woord().into(), naam.clone()));
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
                if let Some(v) = vensters.iter_mut().rev().find(|v| v.onderdelen.iter().any(|o| o.0 == "knop" && o.1.to_lowercase() == knop.to_lowercase())) {
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
        if v.onderdelen.is_empty() {
            uit.push("Er staat niets in het venster.".into());
        } else {
            uit.push("Van boven naar beneden staat erin:".into());
            for (j, (soort, naam)) in v.onderdelen.iter().enumerate() {
                let extra = match soort.as_str() {
                    "tekstvak" | "groot tekstvak" | "keuzelijst" | "uitklaplijst" => format!(", met erboven de tekst \"{}:\"", naam),
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
            let tab: Vec<String> = v.onderdelen.iter().filter(|o| o.0 != "label").map(|o| o.1.clone()).collect();
            if !tab.is_empty() {
                uit.push(format!("Met Tab ga je in deze volgorde langs: {}.", tab.join(", ")));
                uit.push(format!("Bij de start staat de focus op: {}.", tab[0]));
            }
        }
        let hoogte = 140 + v.onderdelen.iter().map(|o| hoogte_van(&o.0)).sum::<i32>();
        uit.push(format!(
            "Grootte: ongeveer 460 beeldpunten breed en {} hoog. Op een klein scherm of tablet {}.",
            hoogte,
            if hoogte > 700 { "moet je dan schuiven: het venster is hoger dan een klein scherm" } else { "past het ook" }
        ));
        for tip in tips(v, handleiding) {
            uit.push(format!("Tip: {}", tip));
        }
    }
    uit.push(String::new());
    uit.push("Altijd: Escape sluit een venster. F1 opent de handleiding van je programma (als je die hebt beschreven).".into());
    uit.join("\n")
}

fn lidwoord(soort: &str) -> String {
    match soort {
        "knop" | "keuzelijst" | "uitklaplijst" => format!("de {}", soort),
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
        _ => 94,
    }
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
    if v.onderdelen.len() > 10 {
        t.push("er staan veel onderdelen in dit venster. Overweeg om het te verdelen over twee vensters.".into());
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
