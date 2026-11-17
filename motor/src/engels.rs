//! Engelse opdrachten lezen, en een programma omzetten naar het Engels.
//! Alles met vaste regels en een vaste woordenlijst. Geen AI, geen vertaalmodel.

use crate::taal::*;

/// (Engels begin van de zin, Nederlands begin, soort van de rest).
/// Soort: 'L' = letterlijke tekst (titel, naam), 'U' = waarde of som, 'V' = voorwaarde, '-' = niets.
const ZINNEN: &[(&str, &str, char)] = &[
    ("make a window with title", "Maak een venster met als titel", 'L'),
    ("make a window titled", "Maak een venster met als titel", 'L'),
    ("create a window with title", "Maak een venster met als titel", 'L'),
    ("make a button named", "Maak een knop met als naam", 'L'),
    ("make a button with name", "Maak een knop met als naam", 'L'),
    ("create a button named", "Maak een knop met als naam", 'L'),
    ("make a text box named", "Maak een tekstvak met als naam", 'L'),
    ("make a large text box named", "Maak een groot tekstvak met als naam", 'L'),
    ("make a label named", "Maak een label met als naam", 'L'),
    ("make a checkbox named", "Maak een selectievakje met als naam", 'L'),
    ("make a list box named", "Maak een keuzelijst met als naam", 'L'),
    ("make a combo box named", "Maak een uitklaplijst met als naam", 'L'),
    ("make a list named", "Maak een lijst met als naam", 'U'),
    ("when i press the button", "Als ik op de knop", 'K'),
    ("when i click the button", "Als ik op de knop", 'K'),
    ("when i click on the button", "Als ik op de knop", 'K'),
    ("show the text", "Toon de tekst", 'L'),
    ("show", "Toon", 'U'),
    ("say", "Toon", 'U'),
    ("remember", "Onthoud", 'U'),
    ("set", "Zet", 'S'),
    ("change", "Verander", 'C'),
    ("add", "Tel", 'A'),
    ("subtract", "Trek", 'B'),
    ("if", "Als", 'V'),
    ("else", "Anders", '-'),
    ("otherwise", "Anders", '-'),
    ("repeat while", "Herhaal zolang", 'V'),
    ("repeat", "Herhaal", 'U'),
    ("end if", "Einde als", '-'),
    ("end repeat", "Einde herhaal", '-'),
    ("end loop", "Einde herhaal", '-'),
    ("end of the button", "Einde van de knop", '-'),
    ("end button", "Einde van de knop", '-'),
    ("end function", "Einde van het stukje", '-'),
    ("end", "Einde", '-'),
    ("comment", "Opmerking", 'L'),
    ("note", "Opmerking", 'L'),
    ("beep", "Piep", '-'),
    ("play the sound", "Speel het geluid", 'L'),
    ("stop the sound", "Stop het geluid", '-'),
    ("write in the log", "Schrijf in het logboek", 'U'),
    ("write to the log", "Schrijf in het logboek", 'U'),
];

/// Engelse woorden in een waarde of voorwaarde, met de Nederlandse woorden ervoor in de plaats.
const WOORDEN: &[(&str, &str)] = &[
    ("is not equal to", "is niet gelijk aan"),
    ("is greater than", "is groter dan"),
    ("is more than", "is groter dan"),
    ("is less than", "is kleiner dan"),
    ("is smaller than", "is kleiner dan"),
    ("is equal to", "is gelijk aan"),
    ("is at least", "is minstens"),
    ("is at most", "is hoogstens"),
    ("is not", "is niet"),
    ("divided by", "gedeeld door"),
    ("and then", "en dan"),
    ("at most", "hoogstens"),
    ("minus", "min"),
    ("times", "keer"),
    ("true", "waar"),
    ("false", "niet waar"),
    ("contains", "bevat"),
    ("then", "dan"),
];

/// Vervangt hele woordgroepen buiten aanhalingstekens.
pub(crate) fn vervang_woorden(tekst: &str, paren: &[(&str, &str)]) -> String {
    let mut uit = String::new();
    let mut in_tekst = false;
    let mut stuk = String::new();
    let klaar = |stuk: &mut String, uit: &mut String| {
        let mut s = format!(" {} ", stuk);
        for (en, nl) in paren {
            let zoek = format!(" {} ", en);
            loop {
                let laag = s.to_ascii_lowercase();
                match laag.find(&zoek) {
                    Some(p) => s = format!("{} {} {}", &s[..p], nl, &s[p + zoek.len()..]),
                    None => break,
                }
            }
        }
        uit.push_str(s.trim_matches(' '));
        stuk.clear();
    };
    for c in tekst.chars() {
        if c == '"' {
            if !in_tekst {
                klaar(&mut stuk, &mut uit);
                if !uit.is_empty() && !uit.ends_with(' ') {
                    uit.push(' ');
                }
            }
            uit.push(c);
            in_tekst = !in_tekst;
            continue;
        }
        if in_tekst {
            uit.push(c);
        } else {
            stuk.push(c);
        }
    }
    if !stuk.trim().is_empty() {
        if uit.ends_with('"') {
            uit.push(' ');
        }
        klaar(&mut stuk, &mut uit);
    }
    uit
}

fn vervang_los_woord(tekst: &str, en: &str, nl: &str) -> String {
    vervang_woorden(tekst, &[(en, nl)])
}

/// Zet een Engelse zin om in de Nederlandse zin. `None` als het geen bekende Engelse zin is.
pub fn naar_nederlands(zin: &str) -> Option<String> {
    let laag = zin.to_ascii_lowercase();
    let (en, nl, soort) = ZINNEN.iter().find(|(en, _, _)| {
        laag == *en || (laag.starts_with(en) && (laag[en.len()..].starts_with(' ') || laag[en.len()..].starts_with(':')))
    })?;
    let rest = zin[en.len()..].trim_start_matches(':').trim();
    let rest = match soort {
        'L' | '-' => rest.to_string(),
        'U' => vervang_woorden(&vervang_los_woord(rest, "times", "keer"), WOORDEN),
        'V' => vervang_woorden(rest, WOORDEN),
        'K' => {
            let r = rest.trim_end();
            let r = r.strip_suffix(" is pressed").unwrap_or(r);
            format!("{} druk", r)
        }
        'S' => vervang_woorden(&vervang_los_woord(rest, "to", "op"), WOORDEN),
        'C' => vervang_woorden(&vervang_los_woord(rest, "to", "in"), WOORDEN),
        'A' => vervang_woorden(&vervang_los_woord(rest, "to", "op bij"), WOORDEN),
        'B' => vervang_woorden(&vervang_los_woord(rest, "from", "af van"), WOORDEN),
        _ => rest.to_string(),
    };
    if *soort == '-' && !rest.is_empty() {
        return None;
    }
    Some(format!("{} {}", nl, rest).trim().to_string())
}

// ----------------------------------------------------------------- naar het Engels

fn waarde(u: &Uitdrukking, niet_vertaald: &mut Vec<String>) -> String {
    match u {
        Uitdrukking::Vast(Waarde::Getal(g)) => crate::getallen::schrijf(*g).replace(',', "."),
        Uitdrukking::Vast(Waarde::Tekst(t)) => {
            niet_vertaald.push(t.clone());
            format!("\"{}\"", t)
        }
        Uitdrukking::Vast(Waarde::Waarheid(b)) => if *b { "true".into() } else { "false".into() },
        Uitdrukking::Naam(n) => n.clone(),
        Uitdrukking::Negatief(a) => format!("minus {}", waarde(a, niet_vertaald)),
        Uitdrukking::Bewerking(a, t, b) => {
            let w = match t {
                Teken::Plus => "plus",
                Teken::Min => "minus",
                Teken::Keer => "times",
                Teken::GedeeldDoor => "divided by",
                Teken::EnDan => "and then",
            };
            format!("{} {} {}", waarde(a, niet_vertaald), w, waarde(b, niet_vertaald))
        }
        _ => {
            niet_vertaald.push("(een waarde die ik nog niet in het Engels kan schrijven)".into());
            "?".into()
        }
    }
}

fn voorwaarde(v: &Voorwaarde, n: &mut Vec<String>) -> String {
    match v {
        Voorwaarde::Vergelijk(a, verg, b) => {
            let w = match verg {
                Vergelijking::Gelijk => "is equal to",
                Vergelijking::NietGelijk => "is not equal to",
                Vergelijking::Groter => "is greater than",
                Vergelijking::Kleiner => "is less than",
                Vergelijking::Minstens => "is at least",
                Vergelijking::Hoogstens => "is at most",
            };
            format!("{} {} {}", waarde(a, n), w, waarde(b, n))
        }
        Voorwaarde::IsWaar(a) => waarde(a, n),
        Voorwaarde::Bevat(a, b) => format!("{} contains {}", waarde(a, n), waarde(b, n)),
        Voorwaarde::En(a, b) => format!("{} and {}", voorwaarde(a, n), voorwaarde(b, n)),
        Voorwaarde::Of(a, b) => format!("{} or {}", voorwaarde(a, n), voorwaarde(b, n)),
    }
}

fn lijst(l: &[Opdracht], diepte: usize, uit: &mut Vec<String>, n: &mut Vec<String>, overgeslagen: &mut Vec<usize>) {
    let insp = "    ".repeat(diepte);
    for o in l {
        let zin = match &o.soort {
            Soort::Opmerking(t) => format!("Comment: {}", t),
            Soort::MaakVenster { titel } => {
                n.push(titel.clone());
                format!("Make a window with title {}", titel)
            }
            Soort::MaakKnop { naam } => {
                n.push(naam.clone());
                format!("Make a button named {}", naam)
            }
            Soort::Toon(u) => format!("Show {}", waarde(u, n)),
            Soort::Onthoud { naam, waarde: w, .. } => format!("Remember {} is {}", naam, waarde(w, n)),
            Soort::Verander { naam, waarde: w } => format!("Change {} to {}", naam, waarde(w, n)),
            Soort::TelOp { naam, waarde: w } => format!("Add {} to {}", waarde(w, n), naam),
            Soort::TrekAf { naam, waarde: w } => format!("Subtract {} from {}", waarde(w, n), naam),
            Soort::Piep { .. } => "Beep".into(),
            Soort::BijKnop { naam, blok, .. } => {
                uit.push(format!("{}When I press the button {}", insp, naam));
                lijst(blok, diepte + 1, uit, n, overgeslagen);
                uit.push(format!("{}End of the button", insp));
                continue;
            }
            Soort::Als { voorwaarde: v, dan, anders, anders_regel, .. } => {
                uit.push(format!("{}If {}", insp, voorwaarde(v, n)));
                lijst(dan, diepte + 1, uit, n, overgeslagen);
                if anders_regel.is_some() {
                    uit.push(format!("{}Else", insp));
                    lijst(anders, diepte + 1, uit, n, overgeslagen);
                }
                uit.push(format!("{}End if", insp));
                continue;
            }
            Soort::Herhaal { aantal, blok, .. } => {
                uit.push(format!("{}Repeat {} times", insp, waarde(aantal, n)));
                lijst(blok, diepte + 1, uit, n, overgeslagen);
                uit.push(format!("{}End repeat", insp));
                continue;
            }
            Soort::HerhaalZolang { voorwaarde: v, hoogstens, blok, .. } => {
                uit.push(format!("{}Repeat while {}, at most {} times", insp, voorwaarde(v, n), waarde(hoogstens, n)));
                lijst(blok, diepte + 1, uit, n, overgeslagen);
                uit.push(format!("{}End repeat", insp));
                continue;
            }
            _ => {
                overgeslagen.push(o.regel);
                format!("Comment: (line {} could not be translated yet)", o.regel)
            }
        };
        uit.push(format!("{}{}", insp, zin));
    }
}

/// Zet een programma om naar Engelse zinnen. Geeft de Engelse tekst, de teksten die niet vertaald
/// zijn (titels, namen en tekst tussen aanhalingstekens), en de regels die nog niet om te zetten waren.
pub fn naar_engels(p: &Programma) -> (String, Vec<String>, Vec<usize>) {
    let mut uit = Vec::new();
    let mut niet = Vec::new();
    let mut overgeslagen = Vec::new();
    lijst(&p.opdrachten, 0, &mut uit, &mut niet, &mut overgeslagen);
    niet.dedup();
    (uit.join("\n"), niet, overgeslagen)
}
