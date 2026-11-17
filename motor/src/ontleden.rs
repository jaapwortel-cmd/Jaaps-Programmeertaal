//! Stap 2 van het lezen: elke regel ontleden en de blokken (Als, Herhaal, knop) opbouwen.
//!
//! Elke zin die werkt staat hier met de hand ingebouwd. Er wordt niets geraden.
//! Kent de motor een zin niet, dan komt er een vraag. Lijkt het op een tikfout,
//! dan komt er een voorstel dat je met ja of nee beantwoordt.

use crate::getallen;
use crate::melding::{Melding, Wijziging};
use crate::taal::*;
use crate::tekst::{self, Stuk};

pub(crate) enum Vorm {
    Leeg,
    Los(Soort),
    OpenAls(Voorwaarde),
    Anders,
    OpenHerhaal(Uitdrukking),
    OpenZolang(Voorwaarde, Uitdrukking),
    OpenKnop(String),
    Einde(Blok),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Blok {
    Elk,
    Als,
    Herhaal,
    Knop,
}

#[derive(Debug)]
pub(crate) struct RegelFout {
    pub tekst: String,
    /// De zin is helemaal niet herkend (dan zoeken we naar een tikfout).
    pub onbekend: bool,
    /// Een voorstel: (vraag, nieuwe regel).
    pub voorstel: Option<(String, String)>,
}

fn fout(tekst: impl Into<String>) -> RegelFout {
    RegelFout { tekst: tekst.into(), onbekend: false, voorstel: None }
}

fn onbekend() -> RegelFout {
    RegelFout { tekst: String::new(), onbekend: true, voorstel: None }
}

/// Woorden die de motor zelf gebruikt. Die mogen geen naam zijn.
const GERESERVEERD: &[&str] = &[
    "plus", "min", "keer", "maal", "gedeeld", "door", "en", "of", "dan", "is", "niet", "waar", "onwaar", "groter",
    "kleiner", "gelijk", "aan", "minstens", "hoogstens", "meer", "minder", "hoger", "lager", "als", "anders", "op",
    "bij", "van", "in", "naar", "met", "toon", "maak", "onthoud", "verander", "herhaal", "einde", "zolang",
];

/// Woorden uit de zinnen die de motor kent. Hiermee zoeken we naar tikfouten.
const WOORDENLIJST: &[&str] = &[
    "maak", "venster", "knop", "met", "als", "titel", "naam", "toon", "tekst", "onthoud", "dat", "is", "verander",
    "naar", "zet", "op", "tel", "bij", "trek", "af", "van", "verhoog", "verlaag", "anders", "herhaal", "keer", "maal",
    "zolang", "hoogstens", "einde", "opmerking", "druk", "klik", "plus", "min", "gedeeld", "door", "groter", "kleiner",
    "dan", "gelijk", "aan", "minstens", "waar", "niet", "herhaling", "keuze", "waarde", "uitkomst", "een", "de", "het",
    "ik", "in",
];

/// Onderdelen die op de wensenlijst staan maar nog niet gebouwd zijn.
const NOG_NIET: &[&str] = &[
    "tekstvak", "label", "lijst", "keuzelijst", "combobox", "selectievakje", "vinkje", "tabel", "menu", "werkbalk",
    "afbeelding", "plaatje", "foto", "geluid", "map", "bestand", "tekstverwerker", "qr-code", "streepjescode",
];

fn klein(s: &[Stuk], i: usize) -> String {
    s.get(i).and_then(|x| x.klein()).unwrap_or_default()
}

fn begint(s: &[Stuk], woorden: &[&str]) -> bool {
    woorden.len() <= s.len() && woorden.iter().enumerate().all(|(i, w)| klein(s, i) == *w)
}

fn zoek(s: &[Stuk], woorden: &[&str], vanaf: usize) -> Option<usize> {
    (vanaf..s.len()).find(|&i| begint(&s[i..], woorden))
}

/// De tekst zoals hij er staat, zonder aanhalingstekens. Voor titels en namen van knoppen.
fn rest_tekst(s: &[Stuk]) -> String {
    if let [Stuk::Tekst(t)] = s {
        return t.clone();
    }
    s.iter()
        .map(|x| match x {
            Stuk::Woord(w) => w.clone(),
            Stuk::Tekst(t) => t.clone(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// De stukjes zoals je ze schreef, met aanhalingstekens.
pub(crate) fn bron_tekst(s: &[Stuk]) -> String {
    s.iter().map(|x| x.als_bron()).collect::<Vec<_>>().join(" ")
}

fn is_lidwoord(w: &str) -> bool {
    matches!(w, "de" | "het" | "een")
}

/// Leest een naam, bijvoorbeeld "teller" of "mijn leeftijd". Altijd kleine letters.
pub(crate) fn naam(s: &[Stuk]) -> Result<String, String> {
    let mut s = s;
    while s.len() > 1 && is_lidwoord(&klein(s, 0)) {
        s = &s[1..];
    }
    if s.is_empty() {
        return Err("Ik mis hier een naam.".into());
    }
    let mut delen = Vec::new();
    for (i, stuk) in s.iter().enumerate() {
        let w = match stuk {
            Stuk::Woord(w) => w.to_lowercase(),
            Stuk::Tekst(t) => {
                return Err(format!("Een naam staat niet tussen aanhalingstekens. Haal de aanhalingstekens weg bij \"{}\".", t))
            }
        };
        if GERESERVEERD.contains(&w.as_str()) {
            return Err(format!(
                "Ik snap \"{}\" niet. Het woord \"{}\" gebruik ik zelf in mijn zinnen, dus het kan geen naam zijn.",
                bron_tekst(s),
                w
            ));
        }
        if !w.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Een naam mag alleen letters en cijfers hebben. \"{}\" kan dus geen naam zijn.", w));
        }
        if i == 0 && w.chars().next().map_or(true, |c| c.is_ascii_digit()) {
            return Err(format!("Een naam moet met een letter beginnen. \"{}\" kan dus geen naam zijn.", bron_tekst(s)));
        }
        delen.push(w);
    }
    Ok(delen.join(" "))
}

fn operand(s: &[Stuk], negatief: bool) -> Result<Uitdrukking, String> {
    let u = if let [Stuk::Tekst(t)] = s {
        Uitdrukking::Vast(Waarde::Tekst(t.clone()))
    } else if s.iter().all(|x| matches!(x, Stuk::Woord(_))) {
        let woorden: Vec<String> = s.iter().filter_map(|x| x.klein()).collect();
        let w: Vec<&str> = woorden.iter().map(|x| x.as_str()).collect();
        match w.as_slice() {
            [een] if getallen::lees(een).is_some() => Uitdrukking::Vast(Waarde::Getal(getallen::lees(een).unwrap())),
            ["waar"] => Uitdrukking::Vast(Waarde::Waarheid(true)),
            ["onwaar"] | ["niet", "waar"] => Uitdrukking::Vast(Waarde::Waarheid(false)),
            _ => Uitdrukking::Naam(naam(s)?),
        }
    } else {
        return Err(format!(
            "Ik snap niet hoe \"{}\" bij elkaar hoort. Zet tekst tussen aanhalingstekens, en plak stukken aan elkaar met \"en dan\".",
            bron_tekst(s)
        ));
    };
    Ok(if negatief { Uitdrukking::Negatief(Box::new(u)) } else { u })
}

/// Leest een waarde of een som, bijvoorbeeld: 3 plus 4 keer 2, of: "Je bent" en dan leeftijd.
pub(crate) fn uitdrukking(s: &[Stuk]) -> Result<Uitdrukking, String> {
    if s.is_empty() {
        return Err("Ik mis hier een waarde: een getal, een tekst of een naam.".into());
    }
    let mut operanden: Vec<Uitdrukking> = Vec::new();
    let mut tekens: Vec<Teken> = Vec::new();
    let mut huidig: Vec<Stuk> = Vec::new();
    let mut negatief = false;
    let mut i = 0;
    while i < s.len() {
        let k = klein(s, i);
        let volgende = klein(s, i + 1);
        let teken = match k.as_str() {
            "plus" | "+" => Some((Teken::Plus, 1)),
            "min" | "-" | "−" => Some((Teken::Min, 1)),
            "keer" | "maal" | "*" | "×" => Some((Teken::Keer, 1)),
            "gedeeld" if volgende == "door" => Some((Teken::GedeeldDoor, 2)),
            "/" => Some((Teken::GedeeldDoor, 1)),
            "en" if volgende == "dan" => Some((Teken::EnDan, 2)),
            _ => None,
        };
        match teken {
            Some((t, lengte)) => {
                if huidig.is_empty() {
                    if t == Teken::Min && !negatief {
                        negatief = true;
                        i += lengte;
                        continue;
                    }
                    return Err(format!("Vóór \"{}\" verwacht ik een getal, een tekst of een naam.", t.woord()));
                }
                operanden.push(operand(&huidig, negatief)?);
                huidig.clear();
                negatief = false;
                tekens.push(t);
                i += lengte;
            }
            None => {
                huidig.push(s[i].clone());
                i += 1;
            }
        }
    }
    if huidig.is_empty() {
        let laatste = tekens.last().map(|t| t.woord()).unwrap_or("min");
        return Err(format!("Na \"{}\" verwacht ik nog een getal, een tekst of een naam.", laatste));
    }
    operanden.push(operand(&huidig, negatief)?);

    // Eerst keer en gedeeld door, dan plus en min, dan "en dan".
    for niveau in [&[Teken::Keer, Teken::GedeeldDoor][..], &[Teken::Plus, Teken::Min][..], &[Teken::EnDan][..]] {
        let mut nieuw_op = vec![operanden.remove(0)];
        let mut nieuw_tekens = Vec::new();
        for (t, o) in tekens.iter().zip(operanden.drain(..)) {
            if niveau.contains(t) {
                let links = nieuw_op.pop().unwrap();
                nieuw_op.push(Uitdrukking::Bewerking(Box::new(links), *t, Box::new(o)));
            } else {
                nieuw_tekens.push(*t);
                nieuw_op.push(o);
            }
        }
        operanden = nieuw_op;
        tekens = nieuw_tekens;
    }
    Ok(operanden.remove(0))
}

const VERGELIJKINGEN: &[(&[&str], Vergelijking)] = &[
    (&["groter", "dan", "of", "gelijk", "aan"], Vergelijking::Minstens),
    (&["kleiner", "dan", "of", "gelijk", "aan"], Vergelijking::Hoogstens),
    (&["niet", "gelijk", "aan"], Vergelijking::NietGelijk),
    (&["groter", "dan"], Vergelijking::Groter),
    (&["meer", "dan"], Vergelijking::Groter),
    (&["hoger", "dan"], Vergelijking::Groter),
    (&["kleiner", "dan"], Vergelijking::Kleiner),
    (&["minder", "dan"], Vergelijking::Kleiner),
    (&["lager", "dan"], Vergelijking::Kleiner),
    (&["gelijk", "aan"], Vergelijking::Gelijk),
    (&["hetzelfde", "als"], Vergelijking::Gelijk),
    (&["ten", "minste"], Vergelijking::Minstens),
    (&["ten", "hoogste"], Vergelijking::Hoogstens),
    (&["minstens"], Vergelijking::Minstens),
    (&["hoogstens"], Vergelijking::Hoogstens),
    (&[">="], Vergelijking::Minstens),
    (&["<="], Vergelijking::Hoogstens),
    (&["!="], Vergelijking::NietGelijk),
    (&["<>"], Vergelijking::NietGelijk),
    (&["≠"], Vergelijking::NietGelijk),
    (&[">"], Vergelijking::Groter),
    (&["<"], Vergelijking::Kleiner),
    (&["="], Vergelijking::Gelijk),
];

fn heeft_en_of(s: &[Stuk]) -> bool {
    (0..s.len()).any(|i| {
        let k = klein(s, i);
        k == "of" || (k == "en" && klein(s, i + 1) != "dan")
    })
}

const EN_OF: &str =
    "Twee voorwaarden in één keer, met \"en\" of \"of\", kan ik nog niet. Zet twee keuzes in elkaar: eerst de ene Als, en daarbinnen de andere.";

/// Leest een voorwaarde, bijvoorbeeld: teller is groter dan 5.
pub(crate) fn voorwaarde(s: &[Stuk]) -> Result<Voorwaarde, String> {
    if s.is_empty() {
        return Err("Waar moet ik op letten? Schrijf bijvoorbeeld: Als teller is groter dan 5.".into());
    }
    // Zoek eerst een vergelijking met woorden. Het woord "is" mag overal staan:
    // "x is groter dan 5", "x groter is dan 5" en "x groter dan 5 is" betekenen hetzelfde.
    let zonder_is: Vec<Stuk> = s.iter().filter(|x| x.klein().as_deref() != Some("is")).cloned().collect();
    for (patroon, verg) in VERGELIJKINGEN {
        if let Some(p) = zoek(&zonder_is, patroon, 0) {
            let links = &zonder_is[..p];
            let rechts = &zonder_is[p + patroon.len()..];
            if links.is_empty() || rechts.is_empty() {
                return Err(format!("Bij \"{}\" mis ik wat er links of rechts van staat.", patroon.join(" ")));
            }
            if heeft_en_of(links) || heeft_en_of(rechts) {
                return Err(EN_OF.into());
            }
            return Ok(Voorwaarde::Vergelijk(uitdrukking(links)?, *verg, uitdrukking(rechts)?));
        }
    }
    if heeft_en_of(s) {
        return Err(EN_OF.into());
    }
    // Dan "x is 5" of "x is niet 5".
    if let Some(p) = zoek(s, &["is"], 0) {
        let links = &s[..p];
        let mut rechts = &s[p + 1..];
        let mut verg = Vergelijking::Gelijk;
        if klein(rechts, 0) == "niet" && rechts.len() > 1 && !(rechts.len() == 2 && klein(rechts, 1) == "waar") {
            verg = Vergelijking::NietGelijk;
            rechts = &rechts[1..];
        }
        if links.is_empty() || rechts.is_empty() {
            return Err("Bij \"is\" mis ik wat er links of rechts van staat.".into());
        }
        return Ok(Voorwaarde::Vergelijk(uitdrukking(links)?, verg, uitdrukking(rechts)?));
    }
    Ok(Voorwaarde::IsWaar(uitdrukking(s)?))
}

fn met_als_titel(s: &[Stuk], i: usize, woorden: &[&str], wat: &str, voorbeeld: &str) -> Result<String, RegelFout> {
    let vraag = || {
        fout(format!(
            "Welke {} moet {} krijgen? Schrijf bijvoorbeeld: {}.",
            woorden[0],
            if wat == "venster" { "het venster" } else { "de knop" },
            voorbeeld
        ))
    };
    if i >= s.len() {
        return Err(vraag());
    }
    let mut j = i;
    if klein(s, j) != "met" {
        // "Maak een knop Welkom": ik gok niet, ik stel voor.
        let rest = rest_tekst(&s[i..]);
        let nieuw = format!(
            "{} met als {} {}",
            bron_tekst(&s[..i]),
            woorden[0],
            bron_tekst(&s[i..])
        );
        return Err(RegelFout {
            tekst: format!("Ik weet niet zeker wat \"{}\" hier betekent.", rest),
            onbekend: false,
            voorstel: Some((format!("Bedoel je: {}?", nieuw), nieuw)),
        });
    }
    j += 1;
    if matches!(klein(s, j).as_str(), "als" | "de" | "het") {
        j += 1;
    }
    if !woorden.contains(&klein(s, j).as_str()) {
        return Err(vraag());
    }
    j += 1;
    if j >= s.len() {
        return Err(vraag());
    }
    Ok(rest_tekst(&s[j..]))
}

fn is_knop_regel(s: &[Stuk]) -> Option<usize> {
    let onderwerp = klein(s, 1);
    if !matches!(onderwerp.as_str(), "ik" | "je" | "jij" | "iemand" | "men" | "u") {
        return None;
    }
    let knop = zoek(s, &["knop"], 2)?;
    if !s[2..knop].iter().any(|x| x.klein().as_deref() == Some("op")) {
        return None;
    }
    Some(knop)
}

/// Ontleedt één (genormaliseerde) regel.
pub(crate) fn ontleed_regel(norm: &str) -> Result<Vorm, RegelFout> {
    let laag = norm.to_lowercase();
    if norm.is_empty() {
        return Ok(Vorm::Leeg);
    }
    for w in ["opmerking", "commentaar", "notitie", "uitleg"] {
        if laag.starts_with(w) && laag[w.len()..].chars().next().map_or(true, |c| !c.is_alphanumeric()) {
            let rest = norm[w.len()..].trim_start_matches([':', ' ']).to_string();
            return Ok(Vorm::Los(Soort::Opmerking(rest)));
        }
    }
    if laag.starts_with("//") || laag.starts_with('#') {
        return Ok(Vorm::Los(Soort::Opmerking(norm.trim_start_matches(['/', '#', ' ']).to_string())));
    }

    let s = tekst::splits(norm).map_err(fout)?;
    if s.is_empty() {
        return Ok(Vorm::Leeg);
    }
    let w0 = klein(&s, 0);
    let u = |r: &[Stuk]| uitdrukking(r).map_err(fout);
    let n = |r: &[Stuk]| naam(r).map_err(fout);

    match w0.as_str() {
        "maak" => {
            let mut i = 1;
            if is_lidwoord(&klein(&s, i)) {
                i += 1;
            }
            match klein(&s, i).as_str() {
                "venster" | "scherm" => Ok(Vorm::Los(Soort::MaakVenster {
                    titel: met_als_titel(&s, i + 1, &["titel", "naam"], "venster", "Maak een venster met als titel Begroeting")?,
                })),
                "knop" => Ok(Vorm::Los(Soort::MaakKnop {
                    naam: met_als_titel(&s, i + 1, &["naam", "tekst", "titel"], "knop", "Maak een knop met als naam Welkom")?,
                })),
                x if NOG_NIET.contains(&x) => Err(fout(format!(
                    "Een {} maken kan ik nog niet. Dat zit nog niet in deze versie. In de handleiding (F1) staat wat wel kan.",
                    x
                ))),
                _ => Err(onbekend()),
            }
        }
        "toon" => {
            let r = &s[1..];
            if r.is_empty() {
                return Err(fout("Wat moet ik tonen? Schrijf bijvoorbeeld: Toon de tekst Hallo."));
            }
            let tekst_na = if begint(r, &["de", "tekst"]) {
                Some(&r[2..])
            } else if begint(r, &["tekst"]) {
                Some(&r[1..])
            } else {
                None
            };
            if let Some(t) = tekst_na {
                if t.is_empty() {
                    return Err(fout("Welke tekst moet ik tonen? Schrijf bijvoorbeeld: Toon de tekst Hallo."));
                }
                if matches!(t[0], Stuk::Tekst(_)) && t.len() > 1 {
                    return Ok(Vorm::Los(Soort::Toon(u(t)?)));
                }
                return Ok(Vorm::Los(Soort::Toon(Uitdrukking::Vast(Waarde::Tekst(rest_tekst(t))))));
            }
            for voor in [&["de", "waarde", "van"][..], &["de", "uitkomst", "van"][..], &["het", "antwoord", "op"][..]] {
                if begint(r, voor) {
                    return Ok(Vorm::Los(Soort::Toon(u(&r[voor.len()..])?)));
                }
            }
            Ok(Vorm::Los(Soort::Toon(u(r)?)))
        }
        "onthoud" => {
            let mut r = &s[1..];
            let dat = klein(r, 0) == "dat";
            if dat {
                r = &r[1..];
            }
            let is_plekken: Vec<usize> = (0..r.len()).filter(|&i| klein(r, i) == "is").collect();
            let (naam_deel, waarde_deel) = if dat && is_plekken.len() == 1 && is_plekken[0] == r.len() - 1 && r.len() >= 3 {
                // "Onthoud dat teller 0 is"
                let lengte = if is_lidwoord(&klein(r, 0)) { 2 } else { 1 };
                (&r[..lengte], &r[lengte..r.len() - 1])
            } else if let Some(&p) = is_plekken.first() {
                (&r[..p], &r[p + 1..])
            } else {
                return Err(fout("Schrijf het zo: Onthoud leeftijd is 42."));
            };
            let nm = n(naam_deel)?;
            if waarde_deel.is_empty() {
                return Err(fout(format!("Wat moet ik onthouden bij {}? Schrijf bijvoorbeeld: Onthoud {} is 0.", nm, nm)));
            }
            Ok(Vorm::Los(Soort::Onthoud { naam: nm, waarde: u(waarde_deel)?, bron: bron_tekst(waarde_deel) }))
        }
        "verander" | "zet" => {
            let scheiding: &[&str] = if w0 == "zet" { &["op"] } else { &["in", "naar"] };
            let p = (1..s.len()).find(|&i| scheiding.contains(&klein(&s, i).as_str()));
            let voorbeeld = if w0 == "zet" { "Zet leeftijd op 43" } else { "Verander leeftijd in 43" };
            let Some(p) = p else {
                return Err(fout(format!("Schrijf het zo: {}.", voorbeeld)));
            };
            if p + 1 >= s.len() {
                return Err(fout(format!("Wat moet de nieuwe waarde zijn? Schrijf bijvoorbeeld: {}.", voorbeeld)));
            }
            Ok(Vorm::Los(Soort::Verander { naam: n(&s[1..p])?, waarde: u(&s[p + 1..])? }))
        }
        "tel" | "trek" => {
            let (scheiding, voorbeeld): (&[&str], &str) = if w0 == "tel" {
                (&["op", "bij"], "Tel 1 op bij teller")
            } else {
                (&["af", "van"], "Trek 1 af van teller")
            };
            let Some(p) = zoek(&s, scheiding, 1) else {
                return Err(fout(format!("Schrijf het zo: {}.", voorbeeld)));
            };
            let waarde = u(&s[1..p])?;
            let nm = n(&s[p + 2..])?;
            Ok(Vorm::Los(if w0 == "tel" { Soort::TelOp { naam: nm, waarde } } else { Soort::TrekAf { naam: nm, waarde } }))
        }
        "verhoog" | "verlaag" => {
            let Some(p) = zoek(&s, &["met"], 1) else {
                return Err(fout(format!("Schrijf het zo: {} teller met 1.", tekst::zelfde_hoofdletter("V", &w0))));
            };
            let nm = n(&s[1..p])?;
            let waarde = u(&s[p + 1..])?;
            Ok(Vorm::Los(if w0 == "verhoog" { Soort::TelOp { naam: nm, waarde } } else { Soort::TrekAf { naam: nm, waarde } }))
        }
        "als" => {
            if let Some(k) = is_knop_regel(&s) {
                let laatste = klein(&s, s.len() - 1);
                let heeft_werkwoord = matches!(laatste.as_str(), "druk" | "drukt" | "klik" | "klikt" | "kies" | "kiest");
                let einde = if heeft_werkwoord { s.len() - 1 } else { s.len() };
                if einde <= k + 1 {
                    return Err(fout("Op welke knop? Schrijf bijvoorbeeld: Als ik op de knop Welkom druk."));
                }
                let knopnaam = rest_tekst(&s[k + 1..einde]);
                if !heeft_werkwoord {
                    let nieuw = format!("{} druk", norm);
                    return Err(RegelFout {
                        tekst: "Ik mis aan het eind van de zin het woord druk.".into(),
                        onbekend: false,
                        voorstel: Some((format!("Bedoel je: {}?", nieuw), nieuw)),
                    });
                }
                return Ok(Vorm::OpenKnop(knopnaam));
            }
            let mut r = &s[1..];
            if klein(r, r.len().wrapping_sub(1)) == "dan" {
                r = &r[..r.len() - 1];
            }
            Ok(Vorm::OpenAls(voorwaarde(r).map_err(fout)?))
        }
        "anders" => {
            if s.len() == 1 || (s.len() == 2 && klein(&s, 1) == "dan") {
                Ok(Vorm::Anders)
            } else if klein(&s, 1) == "als" {
                Err(fout("\"Anders als\" kan ik nog niet. Schrijf Anders op een eigen regel, en zet daaronder een nieuwe Als."))
            } else {
                Err(fout("Na \"Anders\" mag niets meer op dezelfde regel staan. Zet de opdracht op de volgende regel."))
            }
        }
        "herhaal" => {
            let mut r = &s[1..];
            if matches!(klein(r, 0).as_str(), "dit" | "het") {
                r = &r[1..];
            }
            if klein(r, 0) == "zolang" {
                let grens = (1..r.len()).find(|&i| matches!(klein(r, i).as_str(), "hoogstens" | "maximaal" | "maximum"))
                    .map(|i| (i, 1))
                    .or_else(|| zoek(r, &["ten", "hoogste"], 1).map(|i| (i, 2)));
                let Some((g, lengte)) = grens else {
                    let nieuw = format!("{}, hoogstens 100 keer", norm);
                    return Err(RegelFout {
                        tekst: "Deze herhaling heeft geen grens. Ik maak nooit een herhaling zonder grens, want dan kan het programma vastlopen."
                            .into(),
                        onbekend: false,
                        voorstel: Some(("Zal ik er \"hoogstens 100 keer\" achter zetten?".into(), nieuw)),
                    });
                };
                let mut vw = &r[1..g];
                if klein(vw, vw.len().wrapping_sub(1)) == "maar" {
                    vw = &vw[..vw.len() - 1];
                }
                let mut aantal = &r[g + lengte..];
                if matches!(klein(aantal, aantal.len().wrapping_sub(1)).as_str(), "keer" | "maal") {
                    aantal = &aantal[..aantal.len() - 1];
                }
                if aantal.is_empty() {
                    return Err(fout("Hoe vaak mag het hoogstens? Schrijf bijvoorbeeld: Herhaal zolang teller is kleiner dan 10, hoogstens 100 keer."));
                }
                return Ok(Vorm::OpenZolang(voorwaarde(vw).map_err(fout)?, u(aantal)?));
            }
            let laatste = klein(r, r.len().wrapping_sub(1));
            if !matches!(laatste.as_str(), "keer" | "maal") || r.len() < 2 {
                return Err(fout("Hoe vaak moet ik het herhalen? Schrijf bijvoorbeeld: Herhaal 10 keer."));
            }
            Ok(Vorm::OpenHerhaal(u(&r[..r.len() - 1])?))
        }
        "einde" => {
            let rest: Vec<String> =
                s[1..].iter().filter_map(|x| x.klein()).filter(|w| !matches!(w.as_str(), "van" | "de" | "het")).collect();
            let rest: Vec<&str> = rest.iter().map(|x| x.as_str()).collect();
            if s[1..].iter().any(|x| matches!(x, Stuk::Tekst(_))) {
                return Err(fout("Schrijf alleen: Einde als, Einde herhaal of Einde van de knop."));
            }
            match rest.as_slice() {
                [] => Ok(Vorm::Einde(Blok::Elk)),
                ["als"] | ["keuze"] => Ok(Vorm::Einde(Blok::Als)),
                ["herhaal"] | ["herhaling"] => Ok(Vorm::Einde(Blok::Herhaal)),
                ["knop"] => Ok(Vorm::Einde(Blok::Knop)),
                _ => Err(fout("Ik weet niet welk blok je wilt sluiten. Schrijf: Einde als, Einde herhaal of Einde van de knop.")),
            }
        }
        _ => Err(onbekend()),
    }
}

/// Zoekt één tikfout in een zin die niet herkend werd. Geeft de verbeterde zin,
/// alleen als die verbeterde zin wél herkend wordt.
pub(crate) fn zoek_tikfout(regel: &str) -> Option<String> {
    let stukken: Vec<&str> = regel.split(' ').collect();
    let mut in_tekst = false;
    for (i, stuk) in stukken.iter().enumerate() {
        let begon_in_tekst = in_tekst;
        if stuk.matches('"').count() % 2 == 1 {
            in_tekst = !in_tekst;
        }
        if begon_in_tekst || stuk.contains('"') {
            continue;
        }
        let woord: String = stuk.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if woord.is_empty() || WOORDENLIJST.contains(&woord.as_str()) {
            continue;
        }
        let mut kandidaten: Vec<&str> =
            WOORDENLIJST.iter().copied().filter(|g| tekst::mag_tikfout_zijn(&woord, g)).collect();
        kandidaten.sort_by_key(|g| tekst::afstand(&woord, g));
        for goed in kandidaten {
            let mut nieuw = stukken.clone();
            let vervanging = stuk.replacen(
                stuk.trim_matches(|c: char| !c.is_alphanumeric()),
                &tekst::zelfde_hoofdletter(stuk, goed),
                1,
            );
            nieuw[i] = &vervanging;
            let zin = nieuw.join(" ");
            if ontleed_regel(&tekst::normaliseer(&zin)).is_ok() {
                return Some(zin);
            }
        }
    }
    None
}

enum KaderSoort {
    Hoofd,
    Als(Voorwaarde),
    Herhaal(Uitdrukking),
    Zolang(Voorwaarde, Uitdrukking),
    Knop(String),
}

struct Kader {
    soort: KaderSoort,
    regel: usize,
    dan: Vec<Opdracht>,
    anders: Vec<Opdracht>,
    anders_regel: Option<usize>,
}

impl Kader {
    fn nieuw(soort: KaderSoort, regel: usize) -> Self {
        Kader { soort, regel, dan: Vec::new(), anders: Vec::new(), anders_regel: None }
    }
    fn lijst(&mut self) -> &mut Vec<Opdracht> {
        if self.anders_regel.is_some() {
            &mut self.anders
        } else {
            &mut self.dan
        }
    }
    fn blok(&self) -> Blok {
        match self.soort {
            KaderSoort::Hoofd => Blok::Elk,
            KaderSoort::Als(_) => Blok::Als,
            KaderSoort::Herhaal(_) | KaderSoort::Zolang(..) => Blok::Herhaal,
            KaderSoort::Knop(_) => Blok::Knop,
        }
    }
    fn beschrijving(&self) -> String {
        match &self.soort {
            KaderSoort::Hoofd => "het programma".into(),
            KaderSoort::Als(_) => format!("de keuze (Als) van regel {}", self.regel),
            KaderSoort::Herhaal(_) | KaderSoort::Zolang(..) => format!("de herhaling van regel {}", self.regel),
            KaderSoort::Knop(n) => format!("de knop {} van regel {}", n, self.regel),
        }
    }
    fn sluit(self, einde: usize) -> Opdracht {
        let soort = match self.soort {
            KaderSoort::Hoofd => unreachable!(),
            KaderSoort::Als(voorwaarde) => {
                Soort::Als { voorwaarde, dan: self.dan, anders: self.anders, anders_regel: self.anders_regel, einde }
            }
            KaderSoort::Herhaal(aantal) => Soort::Herhaal { aantal, blok: self.dan, einde },
            KaderSoort::Zolang(voorwaarde, hoogstens) => Soort::HerhaalZolang { voorwaarde, hoogstens, blok: self.dan, einde },
            KaderSoort::Knop(naam) => Soort::BijKnop { naam, blok: self.dan, einde },
        };
        Opdracht { regel: self.regel, soort }
    }
}

pub(crate) fn sluitzin(b: Blok) -> &'static str {
    match b {
        Blok::Als => "Einde als",
        Blok::Herhaal => "Einde herhaal",
        Blok::Knop => "Einde van de knop",
        Blok::Elk => "Einde",
    }
}

/// Leest het hele programma. Geeft de opdrachten en alle meldingen.
pub(crate) fn lees_programma(bron: &str) -> (Programma, Vec<Melding>) {
    let mut meldingen = Vec::new();
    let mut stapel = vec![Kader::nieuw(KaderSoort::Hoofd, 0)];
    let regels: Vec<&str> = bron.lines().collect();

    for (i, ruw) in regels.iter().enumerate() {
        let nr = i + 1;
        let norm = tekst::normaliseer(ruw);
        match ontleed_regel(&norm) {
            Ok(Vorm::Leeg) => {}
            Ok(Vorm::Los(soort)) => stapel.last_mut().unwrap().lijst().push(Opdracht { regel: nr, soort }),
            Ok(Vorm::OpenAls(v)) => stapel.push(Kader::nieuw(KaderSoort::Als(v), nr)),
            Ok(Vorm::OpenHerhaal(a)) => stapel.push(Kader::nieuw(KaderSoort::Herhaal(a), nr)),
            Ok(Vorm::OpenZolang(v, a)) => stapel.push(Kader::nieuw(KaderSoort::Zolang(v, a), nr)),
            Ok(Vorm::OpenKnop(naam)) => {
                if stapel.len() > 1 {
                    meldingen.push(Melding::fout(
                        nr,
                        format!(
                            "Wat er bij een knop gebeurt, moet los staan, niet binnen een ander blok. Sluit eerst {}.",
                            stapel.last().unwrap().beschrijving()
                        ),
                    ));
                }
                stapel.push(Kader::nieuw(KaderSoort::Knop(naam), nr));
            }
            Ok(Vorm::Anders) => {
                let top = stapel.last_mut().unwrap();
                match (&top.soort, top.anders_regel) {
                    (KaderSoort::Als(_), None) => top.anders_regel = Some(nr),
                    (KaderSoort::Als(_), Some(eerder)) => meldingen.push(Melding::fout(
                        nr,
                        format!("Deze keuze heeft al een Anders, op regel {}. Een keuze kan maar één Anders hebben.", eerder),
                    )),
                    _ => meldingen.push(Melding::fout(
                        nr,
                        "Anders hoort bij een Als. Maar hier staat geen Als open.".to_string(),
                    )),
                }
            }
            Ok(Vorm::Einde(b)) => {
                if stapel.len() == 1 {
                    meldingen.push(
                        Melding::fout(nr, "Hier sluit je een blok, maar er staat geen blok open.")
                            .met_voorstel("Zal ik deze regel weghalen?", Wijziging::HaalWeg { regel: nr }),
                    );
                    continue;
                }
                let top = stapel.pop().unwrap();
                if b != Blok::Elk && b != top.blok() {
                    let goed = sluitzin(top.blok());
                    meldingen.push(
                        Melding::vraag(
                            nr,
                            format!(
                                "Deze regel sluit een ander soort blok dan er open staat. Open staat {}.",
                                top.beschrijving()
                            ),
                        )
                        .met_voorstel(
                            format!("Bedoel je: {}?", goed),
                            Wijziging::VervangRegel { regel: nr, nieuw: goed.into() },
                        ),
                    );
                }
                let opdracht = top.sluit(nr);
                stapel.last_mut().unwrap().lijst().push(opdracht);
            }
            Err(rf) if rf.onbekend => {
                let getoond = ruw.trim();
                let m = match zoek_tikfout(&norm) {
                    Some(verbeterd) => Melding::vraag(
                        nr,
                        format!("Deze zin ken ik niet: \"{}\". Misschien staat er een tikfout in.", getoond),
                    )
                    .met_voorstel(
                        format!("Bedoel je: {}?", verbeterd),
                        Wijziging::VervangRegel { regel: nr, nieuw: verbeterd },
                    ),
                    None => Melding::vraag(
                        nr,
                        format!(
                            "Deze zin ken ik niet: \"{}\". Ik gok niet wat je bedoelt. Welke zin bedoel je? Een zin begint met Maak, Toon, Onthoud, Verander, Tel, Trek, Als, Anders, Herhaal, Einde of Opmerking. Alle zinnen staan in de handleiding.",
                            getoond
                        ),
                    ),
                };
                meldingen.push(m);
            }
            Err(rf) => {
                let m = match rf.voorstel {
                    Some((vraag, nieuw)) => Melding::vraag(nr, rf.tekst)
                        .met_voorstel(vraag, Wijziging::VervangRegel { regel: nr, nieuw }),
                    None => Melding::fout(nr, rf.tekst),
                };
                meldingen.push(m);
            }
        }
    }

    let laatste = regels.len();
    while stapel.len() > 1 {
        let top = stapel.pop().unwrap();
        let zin = sluitzin(top.blok());
        meldingen.push(
            Melding::fout(top.regel, format!("{} wordt nergens gesloten.", hoofdletter(&top.beschrijving())))
                .met_voorstel(
                    format!("Zal ik \"{}\" onderaan toevoegen?", zin),
                    Wijziging::VoegInVoor { regel: laatste + 1, nieuw: zin.into() },
                ),
        );
        let opdracht = top.sluit(laatste);
        stapel.last_mut().unwrap().lijst().push(opdracht);
    }
    let hoofd = stapel.pop().unwrap();
    (Programma { opdrachten: hoofd.dan }, meldingen)
}

fn hoofdletter(s: &str) -> String {
    tekst::zelfde_hoofdletter("A", s)
}
