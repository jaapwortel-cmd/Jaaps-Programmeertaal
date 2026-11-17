//! Je eigen zinnen: zelf een nieuwe zin bedenken en zeggen wat hij betekent.
//!
//! ```text
//! Nieuwe zin: Begroet [wie]
//! Toon "Hallo" en dan [wie]
//! Einde van de zin
//!
//! Nieuwe zin: Zeg dag tegen [wie] betekent Toon "Dag" en dan [wie]
//! ```
//!
//! Wat tussen [haken] staat, is een open plek. Als je de zin gebruikt, komt daar te staan wat jij
//! op die plek schreef. Geen AI: de zin wordt precies vervangen door wat jij beschreef.

use crate::melding::Melding;
use crate::tekst::{self, Stuk};

#[derive(Clone, Debug)]
pub struct Sjabloon {
    /// De zin zoals je hem schreef.
    pub zin: String,
    /// Stukjes van de zin: een woord (kleine letters), een tekst, of een open plek.
    delen: Vec<Deel>,
    /// Wat de zin betekent: een of meer regels.
    pub betekenis: Vec<String>,
    pub regel: usize,
}

#[derive(Clone, Debug, PartialEq)]
enum Deel {
    Woord(String),
    Tekst(String),
    Plek(String),
}

fn is_begin(laag: &str) -> Option<usize> {
    for b in ["nieuwe zin:", "nieuwe zin "] {
        if laag.starts_with(b) {
            return Some(b.len());
        }
    }
    None
}

fn is_einde(laag: &str) -> bool {
    matches!(laag.trim(), "einde van de zin" | "einde zin" | "einde van de nieuwe zin")
}

fn maak_delen(zin: &str) -> Result<Vec<Deel>, String> {
    let stukken = tekst::splits(&tekst::normaliseer(zin))?;
    Ok(stukken
        .into_iter()
        .map(|s| match s {
            Stuk::Tekst(t) => Deel::Tekst(t),
            Stuk::Woord(w) if w.starts_with('[') && w.ends_with(']') && w.len() > 2 => Deel::Plek(w.to_lowercase()),
            Stuk::Woord(w) => Deel::Woord(w.to_lowercase()),
        })
        .collect())
}

/// Leest alle eigen zinnen uit een tekst. Geeft de zinnen, de meldingen, en welke regels bij een
/// beschrijving van een eigen zin horen (die doen zelf niets).
pub fn verzamel(bron: &str) -> (Vec<Sjabloon>, Vec<Melding>, Vec<usize>) {
    let regels: Vec<&str> = bron.lines().collect();
    let mut uit = Vec::new();
    let mut meldingen = Vec::new();
    let mut bezet = Vec::new();
    let mut i = 0;
    while i < regels.len() {
        let norm = tekst::normaliseer(regels[i]);
        let laag = norm.to_lowercase();
        let Some(b) = is_begin(&laag) else {
            i += 1;
            continue;
        };
        let nr = i + 1;
        bezet.push(nr);
        let rest = norm[b..].trim().to_string();
        let (zin, mut betekenis) = match laag.find(" betekent ") {
            Some(p) => (norm[b..p].trim().to_string(), vec![norm[p + " betekent ".len()..].trim().to_string()]),
            None => (rest, Vec::new()),
        };
        if betekenis.is_empty() {
            let mut j = i + 1;
            let mut gesloten = false;
            while j < regels.len() {
                bezet.push(j + 1);
                let r = tekst::normaliseer(regels[j]);
                if is_einde(&r.to_lowercase()) {
                    gesloten = true;
                    break;
                }
                if !r.trim().is_empty() {
                    betekenis.push(regels[j].trim().to_string());
                }
                j += 1;
            }
            if !gesloten {
                meldingen.push(Melding::fout(nr, "Deze nieuwe zin wordt niet gesloten. Zet onder de betekenis een regel: Einde van de zin."));
            }
            i = j + 1;
        } else {
            i += 1;
        }
        match maak_delen(&zin) {
            Err(e) => meldingen.push(Melding::fout(nr, e)),
            Ok(delen) if delen.is_empty() => meldingen.push(Melding::fout(nr, "Welke zin wil je maken? Schrijf bijvoorbeeld: Nieuwe zin: Begroet [wie]")),
            Ok(delen) if !delen.iter().any(|d| matches!(d, Deel::Woord(_))) => meldingen.push(Melding::fout(
                nr,
                "Een eigen zin heeft minstens één vast woord nodig, niet alleen open plekken tussen haken.",
            )),
            Ok(_) if betekenis.is_empty() => meldingen.push(Melding::fout(nr, format!("Wat betekent de zin \"{}\"? Zet de betekenis eronder.", zin))),
            Ok(delen) => {
                if let Some(eerder) = uit.iter().find(|s: &&Sjabloon| s.delen == delen) {
                    meldingen.push(Melding::vraag(
                        nr,
                        format!("De zin \"{}\" heb je al gemaakt op regel {}. Dat spreekt elkaar tegen. Welke van de twee bedoel je?", zin, eerder.regel),
                    ));
                }
                uit.push(Sjabloon { zin, delen, betekenis, regel: nr });
            }
        }
    }
    (uit, meldingen, bezet)
}

/// Past een regel op een eigen zin? Geeft dan de ingevulde open plekken.
fn past(delen: &[Deel], stukken: &[Stuk]) -> Option<Vec<(String, String)>> {
    fn zoek(d: &[Deel], s: &[Stuk], gevonden: &mut Vec<(String, String)>) -> bool {
        match d.first() {
            None => s.is_empty(),
            Some(Deel::Woord(w)) => s.first().and_then(|x| x.klein()).as_deref() == Some(w.as_str()) && zoek(&d[1..], &s[1..], gevonden),
            Some(Deel::Tekst(t)) => matches!(s.first(), Some(Stuk::Tekst(x)) if x == t) && zoek(&d[1..], &s[1..], gevonden),
            Some(Deel::Plek(naam)) => {
                for n in 1..=s.len() {
                    gevonden.push((naam.clone(), s[..n].iter().map(|x| x.als_bron()).collect::<Vec<_>>().join(" ")));
                    if zoek(&d[1..], &s[n..], gevonden) {
                        return true;
                    }
                    gevonden.pop();
                }
                false
            }
        }
    }
    let mut gevonden = Vec::new();
    zoek(delen, stukken, &mut gevonden).then_some(gevonden)
}

fn vul_in(regel: &str, plekken: &[(String, String)]) -> String {
    let mut uit = regel.to_string();
    for (naam, waarde) in plekken {
        loop {
            let laag = uit.to_lowercase();
            match laag.find(naam.as_str()) {
                Some(p) => uit = format!("{}{}{}", &uit[..p], waarde, &uit[p + naam.len()..]),
                None => break,
            }
        }
    }
    uit
}

/// Probeert een regel als eigen zin te lezen. Geeft de regels die ervoor in de plaats komen.
pub fn vervang(regel: &str, zinnen: &[Sjabloon]) -> Option<Vec<String>> {
    let stukken = tekst::splits(&tekst::normaliseer(regel)).ok()?;
    for z in zinnen {
        if let Some(plekken) = past(&z.delen, &stukken) {
            return Some(z.betekenis.iter().map(|b| vul_in(b, &plekken)).collect());
        }
    }
    None
}
