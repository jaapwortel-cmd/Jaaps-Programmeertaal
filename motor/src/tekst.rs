//! Stap 1 van het lezen: een regel netjes maken en in stukjes knippen.
//!
//! - Gesproken leestekens ("aanhalingsteken openen") worden echte tekens.
//! - Krullende aanhalingstekens worden gewone aanhalingstekens.
//! - Een punt aan het eind van de regel valt weg (dicteren zet daar vaak een punt).
//! - Tekst tussen aanhalingstekens blijft precies zoals hij is.

/// Een stukje van een regel: een los woord, of tekst tussen aanhalingstekens.
#[derive(Clone, Debug, PartialEq)]
pub enum Stuk {
    Woord(String),
    Tekst(String),
}

impl Stuk {
    /// Het woord in kleine letters. Tekst tussen aanhalingstekens geeft `None`.
    pub fn klein(&self) -> Option<String> {
        match self {
            Stuk::Woord(w) => Some(w.to_lowercase()),
            Stuk::Tekst(_) => None,
        }
    }

    /// Het stukje zoals het in de regel stond (tekst weer met aanhalingstekens).
    pub fn als_bron(&self) -> String {
        match self {
            Stuk::Woord(w) => w.clone(),
            Stuk::Tekst(t) => format!("\"{}\"", t),
        }
    }
}

const OPENEN: &[&str] = &[
    "aanhalingstekens openen",
    "aanhalingsteken openen",
    "open aanhalingstekens",
    "open aanhalingsteken",
];
const SLUITEN: &[&str] = &[
    "aanhalingstekens sluiten",
    "aanhalingsteken sluiten",
    "sluit aanhalingstekens",
    "sluit aanhalingsteken",
];
const LOS: &[&str] = &["aanhalingstekens", "aanhalingsteken"];

fn is_woordgrens(tekst: &str, begin: usize, eind: usize) -> bool {
    let ervoor = tekst[..begin].chars().last();
    let erna = tekst[eind..].chars().next();
    let goed = |c: Option<char>| c.map_or(true, |c| !c.is_alphanumeric());
    goed(ervoor) && goed(erna)
}

/// Zet gesproken aanhalingstekens om in echte aanhalingstekens.
fn gesproken_leestekens(regel: &str) -> String {
    // Alleen ASCII-kleine letters: dan blijven de posities in de tekst gelijk.
    let klein = regel.to_ascii_lowercase();
    let mut uit = String::with_capacity(regel.len());
    let mut i = 0;
    'buiten: while i < regel.len() {
        for (lijst, soort) in [(OPENEN, 0u8), (SLUITEN, 1u8), (LOS, 2u8)] {
            for zin in lijst {
                if klein[i..].starts_with(zin) && is_woordgrens(&klein, i, i + zin.len()) {
                    let open = match soort {
                        0 => true,
                        1 => false,
                        _ => uit.matches('"').count() % 2 == 0,
                    };
                    if open {
                        uit.push('"');
                        i += zin.len();
                        // De spatie na "aanhalingsteken openen" hoort niet bij de tekst.
                        while i < regel.len() && regel[i..].starts_with(' ') {
                            i += 1;
                        }
                    } else {
                        while uit.ends_with(' ') {
                            uit.pop();
                        }
                        uit.push('"');
                        i += zin.len();
                    }
                    continue 'buiten;
                }
            }
        }
        let c = regel[i..].chars().next().unwrap();
        uit.push(c);
        i += c.len_utf8();
    }
    uit
}

/// Maakt een regel klaar om te lezen.
pub fn normaliseer(regel: &str) -> String {
    let mut s: String = regel
        .chars()
        .filter(|c| *c != '\u{FEFF}')
        .map(|c| match c {
            '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' | '\u{00AB}' | '\u{00BB}' => '"',
            '\t' => ' ',
            c => c,
        })
        .collect();
    s = gesproken_leestekens(&s);
    let mut s = s.trim().to_string();
    // Een punt (of dubbele punt) aan het eind valt weg, maar niet binnen aanhalingstekens.
    while (s.ends_with('.') || s.ends_with(':')) && s.matches('"').count() % 2 == 0 {
        s.pop();
        s = s.trim_end().to_string();
    }
    s
}

/// Knipt een (genormaliseerde) regel in stukjes.
pub fn splits(regel: &str) -> Result<Vec<Stuk>, String> {
    let mut stukken = Vec::new();
    let mut woord = String::new();
    let mut chars = regel.chars().peekable();

    fn klaar(woord: &mut String, stukken: &mut Vec<Stuk>) {
        if woord.is_empty() {
            return;
        }
        // Komma, puntkomma of dubbele punt achter een woord tellen niet mee.
        // Een komma tussen cijfers (3,5) blijft staan.
        let schoon = woord.trim_end_matches([',', ';', ':']).to_string();
        let schoon = schoon.trim_start_matches(['(', ')']).trim_end_matches([')']).to_string();
        if !schoon.is_empty() {
            stukken.push(Stuk::Woord(schoon));
        }
        woord.clear();
    }

    while let Some(c) = chars.next() {
        if c == '"' {
            klaar(&mut woord, &mut stukken);
            let mut tekst = String::new();
            let mut gesloten = false;
            for c2 in chars.by_ref() {
                if c2 == '"' {
                    gesloten = true;
                    break;
                }
                tekst.push(c2);
            }
            if !gesloten {
                return Err(
                    "Er staat een aanhalingsteken dat niet wordt gesloten. Zet aan het eind van de tekst nog een aanhalingsteken, of zeg: aanhalingsteken sluiten."
                        .to_string(),
                );
            }
            stukken.push(Stuk::Tekst(tekst));
        } else if c.is_whitespace() {
            klaar(&mut woord, &mut stukken);
        } else {
            woord.push(c);
        }
    }
    klaar(&mut woord, &mut stukken);
    Ok(stukken)
}

/// Afstand tussen twee woorden: hoeveel letters moet je veranderen,
/// weghalen of toevoegen om van het ene het andere woord te maken.
pub fn afstand(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.to_lowercase().chars().collect();
    let b: Vec<char> = b.to_lowercase().chars().collect();
    let mut vorige: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut huidige = vec![i + 1; b.len() + 1];
        for (j, cb) in b.iter().enumerate() {
            let kosten = if ca == cb { 0 } else { 1 };
            huidige[j + 1] = (vorige[j] + kosten).min(vorige[j + 1] + 1).min(huidige[j] + 1);
        }
        vorige = huidige;
    }
    vorige[b.len()]
}

/// Hoeveel verschil mag er zijn om aan een tikfout te denken.
pub fn mag_tikfout_zijn(woord: &str, goed: &str) -> bool {
    let lengte = woord.chars().count().max(goed.chars().count());
    let grens = if lengte <= 4 { 1 } else { 2 };
    let d = afstand(woord, goed);
    d > 0 && d <= grens
}

/// Zet de eerste letter in hoofdletter als het voorbeeld ook zo begint.
pub fn zelfde_hoofdletter(voorbeeld: &str, woord: &str) -> String {
    if voorbeeld.chars().next().map_or(false, |c| c.is_uppercase()) {
        let mut c = woord.chars();
        match c.next() {
            Some(eerste) => eerste.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    } else {
        woord.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gesproken_aanhalingstekens() {
        assert_eq!(
            normaliseer("Toon de tekst aanhalingsteken openen Hallo daar aanhalingsteken sluiten."),
            "Toon de tekst \"Hallo daar\""
        );
        assert_eq!(normaliseer("Toon aanhalingsteken Hoi aanhalingsteken"), "Toon \"Hoi\"");
    }

    #[test]
    fn punt_aan_het_eind() {
        assert_eq!(normaliseer("Maak een knop met als naam Welkom."), "Maak een knop met als naam Welkom");
        assert_eq!(normaliseer("Toon \"Dag.\""), "Toon \"Dag.\"");
    }

    #[test]
    fn knippen() {
        let s = splits("Toon \"Hallo daar\" en dan x,").unwrap();
        assert_eq!(s.len(), 5);
        assert_eq!(s[1], Stuk::Tekst("Hallo daar".into()));
        assert_eq!(s[4], Stuk::Woord("x".into()));
        assert!(splits("Toon \"open").is_err());
    }

    #[test]
    fn afstanden() {
        assert_eq!(afstand("mak", "maak"), 1);
        assert_eq!(afstand("Toom", "toon"), 1);
        assert!(mag_tikfout_zijn("herhal", "herhaal"));
        assert!(!mag_tikfout_zijn("fiets", "toon"));
    }
}
