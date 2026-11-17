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
    OpenVoorElk(String, Uitdrukking),
    OpenTel(String, Uitdrukking, Uitdrukking),
    OpenStukje(String, Vec<String>),
    Einde(Blok),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Blok {
    Elk,
    Als,
    Herhaal,
    Knop,
    Stukje,
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
    "bij", "van", "in", "naar", "met", "toon", "maak", "onthoud", "verander", "herhaal", "einde", "zolang", "bevat", "tot",
    "elk", "elke", "uit",
];

/// Woorden uit de zinnen die de motor kent. Hiermee zoeken we naar tikfouten.
const WOORDENLIJST: &[&str] = &[
    "maak", "venster", "knop", "met", "als", "titel", "naam", "toon", "tekst", "onthoud", "dat", "is", "verander",
    "naar", "zet", "op", "tel", "bij", "trek", "af", "van", "verhoog", "verlaag", "anders", "herhaal", "keer", "maal",
    "zolang", "hoogstens", "einde", "opmerking", "druk", "klik", "plus", "min", "gedeeld", "door", "groter", "kleiner",
    "dan", "gelijk", "aan", "minstens", "waar", "niet", "herhaling", "keuze", "waarde", "uitkomst", "een", "de", "het",
    "ik", "in", "installatiebestand", "installatie", "snelkoppeling", "bureaublad", "startmenu", "beheerdersrechten",
    "versie", "maker", "voeg", "toe", "bestand", "licentie", "staat", "tekstvak", "label", "selectievakje", "keuzelijst",
    "uitklaplijst", "vink", "aan", "uit", "focus", "groot", "leeg", "aangevinkt", "keuze", "getal", "lijst", "stukje",
    "doe", "geef", "uitkomst", "invoer", "elk", "elke", "haal", "weg", "bevat", "tot", "aantal", "eerste", "laatste",
    "schrijf", "logboek", "map", "inhoud", "regels", "bestaat", "willekeurig", "tijd", "datum", "speel", "piep",
    "geluid", "stop", "sneltoets", "kleur", "achtergrond", "handleiding", "hertz", "milliseconden", "wacht",
];

/// Onderdelen die op de wensenlijst staan maar nog niet gebouwd zijn.
const NOG_NIET: &[&str] = &[
    "tabel", "menu", "werkbalk",
    "afbeelding", "plaatje", "foto", "tekstverwerker", "qr-code", "streepjescode",
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

pub(crate) fn is_onderdeelwoord(w: &str) -> bool {
    matches!(w, "tekstvak" | "invulvak" | "label" | "selectievakje" | "vinkje" | "keuzevakje" | "keuzelijst" | "uitklaplijst" | "combobox" | "keuzemenu")
}

/// "de tekst in het tekstvak Naam", "het getal in het tekstvak Leeftijd", "het selectievakje Geluid", ...
fn onderdeel_operand(s: &[Stuk]) -> Option<Uitdrukking> {
    let mut s = s;
    if klein(s, s.len().wrapping_sub(1)) == "staat" && begint(s, &["wat", "er"]) {
        s = &s[..s.len() - 1];
    }
    let voorvoegsels: &[(&[&str], bool)] = &[
        (&["de", "tekst", "in"], false),
        (&["de", "keuze", "in"], false),
        (&["het", "getal", "in"], true),
        (&["wat", "er", "in"], false),
        (&[], false),
    ];
    for (voor, als_getal) in voorvoegsels {
        if !begint(s, voor) {
            continue;
        }
        let mut r = &s[voor.len()..];
        if matches!(klein(r, 0).as_str(), "het" | "de") {
            r = &r[1..];
        }
        if matches!(klein(r, 0).as_str(), "grote" | "groot") {
            r = &r[1..];
        }
        if r.len() >= 2 && is_onderdeelwoord(&klein(r, 0)) {
            return Some(Uitdrukking::Onderdeel { naam: rest_tekst(&r[1..]), als_getal: *als_getal });
        }
    }
    None
}

/// Lijsten en eigen stukjes in een waarde.
fn lijst_of_stukje(s: &[Stuk]) -> Result<Option<Uitdrukking>, String> {
    let lijst_na = |s: &[Stuk], vanaf: usize| -> Result<Uitdrukking, String> {
        let Some(p) = zoek(s, &["lijst"], vanaf) else {
            return Err(format!("Welke lijst bedoel je bij \"{}\"? Schrijf bijvoorbeeld: het aantal in de lijst boodschappen.", bron_tekst(s)));
        };
        Ok(Uitdrukking::Naam(naam(&s[p + 1..])?))
    };
    if begint(s, &["de", "lijst"]) && s.len() > 2 {
        return Ok(Some(Uitdrukking::Naam(naam(&s[2..])?)));
    }
    if begint(s, &["het", "aantal"]) && zoek(s, &["lijst"], 2).is_some() {
        return Ok(Some(Uitdrukking::Aantal(Box::new(lijst_na(s, 2)?))));
    }
    if (begint(s, &["het", "eerste"]) || begint(s, &["het", "laatste"])) && zoek(s, &["lijst"], 2).is_some() {
        let plek = if klein(s, 1) == "eerste" { 1.0 } else { -1.0 };
        return Ok(Some(Uitdrukking::Element(Box::new(lijst_na(s, 2)?), Box::new(Uitdrukking::Vast(Waarde::Getal(plek))))));
    }
    if begint(s, &["nummer"]) {
        if let Some(p) = (2..s.len()).find(|&i| matches!(klein(s, i).as_str(), "uit" | "van") && zoek(s, &["lijst"], i).is_some()) {
            let nummer = operand(&s[1..p], false)?;
            return Ok(Some(Uitdrukking::Element(Box::new(lijst_na(s, p)?), Box::new(nummer))));
        }
    }
    let mut r = s;
    if begint(r, &["de", "uitkomst", "van"]) && r.len() > 3 {
        r = &r[3..];
        if zoek(r, &["met"], 1).is_none() {
            return Ok(Some(operand(r, false)?));
        }
    }
    if let Some(m) = zoek(r, &["met"], 1) {
        if r[..m].iter().all(|x| matches!(x, Stuk::Woord(_))) {
            return Ok(Some(aanroep(r)?));
        }
    }
    Ok(None)
}

/// Een bestandsnaam: letterlijk, of met aanhalingstekens en "en dan" als uitdrukking.
fn pad(s: &[Stuk]) -> Result<Uitdrukking, String> {
    if s.is_empty() {
        return Err("Welk bestand? Schrijf bijvoorbeeld: het bestand notities.txt.".into());
    }
    if matches!(s[0], Stuk::Tekst(_)) && s.len() > 1 {
        return uitdrukking(s);
    }
    Ok(Uitdrukking::Vast(Waarde::Tekst(rest_tekst(s))))
}

/// Bestanden, tijd, datum en willekeurige getallen.
fn ingebouwd(s: &[Stuk]) -> Result<Option<Uitdrukking>, String> {
    let ib = |wat, invoer| Ok(Some(Uitdrukking::Ingebouwd { wat, invoer }));
    for (voor, wat) in [
        (&["de", "inhoud", "van", "het", "bestand"][..], Ingebouwd::Inhoud),
        (&["de", "regels", "van", "het", "bestand"][..], Ingebouwd::Regels),
    ] {
        if begint(s, voor) {
            return ib(wat, vec![pad(&s[voor.len()..])?]);
        }
    }
    if begint(s, &["de", "plek", "van", "het", "logboek"]) && s.len() == 5 {
        return ib(Ingebouwd::LogboekPlek, vec![]);
    }
    if begint(s, &["de", "tijd"]) && s.len() == 2 || begint(s, &["het", "tijdstip"]) && s.len() == 2 {
        return ib(Ingebouwd::Tijd, vec![]);
    }
    if begint(s, &["de", "datum"]) && s.len() == 2 || begint(s, &["de", "datum", "van", "vandaag"]) && s.len() == 4 {
        return ib(Ingebouwd::Datum, vec![]);
    }
    if (begint(s, &["een", "willekeurig", "getal"]) || begint(s, &["een", "toevallig", "getal"])) && zoek(s, &["tot"], 3).is_some() {
        let v = zoek(s, &["van"], 3);
        let t = zoek(s, &["tot"], 3).unwrap();
        let mut na = t + 1;
        if begint(&s[na..], &["en", "met"]) {
            na += 2;
        }
        let Some(v) = v.filter(|v| *v < t) else {
            return Err("Schrijf het zo: een willekeurig getal van 1 tot 6.".into());
        };
        if na >= s.len() || v + 1 >= t {
            return Err("Schrijf het zo: een willekeurig getal van 1 tot 6.".into());
        }
        return ib(Ingebouwd::Willekeurig, vec![operand(&s[v + 1..t], false)?, operand(&s[na..], false)?]);
    }
    Ok(None)
}

/// Leest een sneltoets, zoals "Ctrl+W", "Control Shift S", "Alt 1" of "F5".
pub(crate) fn lees_toets(tekst: &str) -> Result<Toets, String> {
    let mut t = Toets { code: 0, ctrl: false, alt: false, shift: false, naam: String::new() };
    let mut sleutel = String::new();
    for deel in tekst.split(['+', ' ', '-']).filter(|d| !d.is_empty()) {
        let laag = deel.to_lowercase();
        match laag.as_str() {
            "ctrl" | "control" | "controle" | "besturing" => t.ctrl = true,
            "alt" => t.alt = true,
            "shift" | "hoofdletter" => t.shift = true,
            "en" | "plus" => {}
            _ => {
                if !sleutel.is_empty() {
                    return Err(format!("Een sneltoets heeft één gewone toets, maar ik lees er meer: {}.", tekst));
                }
                sleutel = laag;
            }
        }
    }
    let code = match sleutel.as_str() {
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_alphanumeric() => s.to_ascii_uppercase().as_bytes()[0] as u16,
        s if s.starts_with('f') && s[1..].parse::<u16>().map_or(false, |n| (2..=12).contains(&n)) => 0x6F + s[1..].parse::<u16>().unwrap(),
        "f1" => return Err("F1 is altijd de handleiding. Kies een andere sneltoets.".into()),
        "enter" => 0x0D,
        "spatie" => 0x20,
        "escape" | "esc" => return Err("Escape sluit altijd het venster. Kies een andere sneltoets.".into()),
        "" => return Err(format!("Ik mis de gewone toets in de sneltoets {}. Schrijf bijvoorbeeld: Ctrl+W.", tekst)),
        ander => return Err(format!("De toets \"{}\" ken ik niet. Gebruik een letter, een cijfer, F2 tot en met F12, Enter of Spatie.", ander)),
    };
    if !t.ctrl && !t.alt && code < 0x70 && code != 0x0D && code != 0x20 && !(0x70..=0x7B).contains(&code) {
        return Err("Een letter of cijfer als sneltoets heeft Ctrl of Alt nodig, anders kun je niet meer typen. Schrijf bijvoorbeeld: Ctrl+W.".into());
    }
    if !t.ctrl && !t.alt && matches!(code, 0x0D | 0x20) {
        return Err("Enter en Spatie zonder Ctrl of Alt drukken al op de knop waar je staat. Kies bijvoorbeeld Ctrl+Enter.".into());
    }
    t.code = code;
    let mut naam = Vec::new();
    if t.ctrl {
        naam.push("Ctrl".to_string());
    }
    if t.alt {
        naam.push("Alt".to_string());
    }
    if t.shift {
        naam.push("Shift".to_string());
    }
    naam.push(match code {
        0x0D => "Enter".into(),
        0x20 => "Spatie".into(),
        c if (0x70..=0x7B).contains(&c) => format!("F{}", c - 0x6F),
        c => (c as u8 as char).to_string(),
    });
    t.naam = naam.join("+");
    Ok(t)
}

/// "Verdubbel met 5", "Optellen met 3 en 4": de naam van een stukje en de invoer.
pub(crate) fn aanroep(s: &[Stuk]) -> Result<Uitdrukking, String> {
    let m = zoek(s, &["met"], 1).unwrap_or(s.len());
    let naam_deel = &s[..m];
    if naam_deel.is_empty() {
        return Err("Welk stukje bedoel je? Schrijf bijvoorbeeld: Doe Verdubbel met 5.".into());
    }
    let naam = naam_deel.iter().filter_map(|x| x.klein()).collect::<Vec<_>>().join(" ");
    let mut invoer = Vec::new();
    if m < s.len() {
        let rest = &s[m + 1..];
        if rest.is_empty() {
            return Err(format!("Na \"met\" mis ik de invoer voor {}.", naam));
        }
        let mut begin = 0;
        for i in 0..=rest.len() {
            if i == rest.len() || (klein(rest, i) == "en" && klein(rest, i + 1) != "dan") {
                if i == begin {
                    return Err(format!("Bij de invoer voor {} mis ik een waarde rond \"en\".", naam));
                }
                invoer.push(uitdrukking(&rest[begin..i])?);
                begin = i + 1;
            }
        }
    }
    Ok(Uitdrukking::Aanroep { naam, invoer })
}

fn operand(s: &[Stuk], negatief: bool) -> Result<Uitdrukking, String> {
    let u = if let Some(o) = onderdeel_operand(s) {
        o
    } else if let Some(i) = ingebouwd(s)? {
        i
    } else if let Some(l) = lijst_of_stukje(s)? {
        l
    } else if let [Stuk::Tekst(t)] = s {
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

/// Letterlijke tekst (zonder aanhalingstekens), of een uitdrukking als er aanhalingstekens of "en dan" in staan.
fn tekst_of_uitdrukking(s: &[Stuk]) -> Result<Uitdrukking, String> {
    if s.iter().any(|x| matches!(x, Stuk::Tekst(_))) || s.len() == 1 && getallen::lees(&klein(s, 0)).is_some() {
        return uitdrukking(s);
    }
    if let Ok(u) = uitdrukking(s) {
        if !matches!(u, Uitdrukking::Naam(_)) || s.len() == 1 {
            return Ok(u);
        }
    }
    Ok(Uitdrukking::Vast(Waarde::Tekst(rest_tekst(s))))
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
    // "het bestand notities.txt bestaat" (of: bestaat niet)
    if begint(s, &["het", "bestand"]) || begint(s, &["de", "map"]) {
        let mut eind = s.len();
        let mut niet = false;
        if klein(s, eind - 1) == "niet" {
            niet = true;
            eind -= 1;
        }
        if klein(s, eind - 1) == "bestaat" {
            eind -= 1;
            if klein(s, eind - 1) == "niet" {
                niet = true;
                eind -= 1;
            }
            let b = Uitdrukking::Ingebouwd { wat: Ingebouwd::Bestaat, invoer: vec![pad(&s[2..eind])?] };
            let verg = if niet { Vergelijking::NietGelijk } else { Vergelijking::Gelijk };
            return Ok(Voorwaarde::Vergelijk(b, verg, Uitdrukking::Vast(Waarde::Waarheid(true))));
        }
    }
    // "de lijst boodschappen bevat "melk""
    if let Some(p) = zoek(s, &["bevat"], 1) {
        if p + 1 >= s.len() {
            return Err("Wat moet er in de lijst staan? Schrijf bijvoorbeeld: Als de lijst boodschappen bevat \"melk\".".into());
        }
        return Ok(Voorwaarde::Bevat(uitdrukking(&s[..p])?, uitdrukking(&s[p + 1..])?));
    }
    // "het selectievakje Geluid is aangevinkt", "het tekstvak Naam is leeg" (ook met "niet").
    let laatste = klein(s, s.len() - 1);
    if matches!(laatste.as_str(), "aangevinkt" | "leeg") {
        let mut r: Vec<Stuk> = s[..s.len() - 1].iter().filter(|x| x.klein().as_deref() != Some("is")).cloned().collect();
        let niet = r.last().and_then(|x| x.klein()).as_deref() == Some("niet");
        if niet {
            r.pop();
        }
        if let Some(o) = onderdeel_operand(&r) {
            let doel = if laatste == "leeg" { Waarde::Tekst(String::new()) } else { Waarde::Waarheid(true) };
            let verg = if niet { Vergelijking::NietGelijk } else { Vergelijking::Gelijk };
            return Ok(Voorwaarde::Vergelijk(o, verg, Uitdrukking::Vast(doel)));
        }
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
            match wat {
                "venster" => "het venster",
                "installatie" => "het installatiebestand",
                "knop" => "de knop",
                ander => ander,
            },
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

fn installatie_zin(s: &[Stuk]) -> Result<Vorm, RegelFout> {
    let tweede = klein(s, 1);
    let na_is = || zoek(s, &["is"], 2).map(|p| rest_tekst(&s[p + 1..])).filter(|t| !t.is_empty());
    let zin = match tweede.as_str() {
        "versie" => InstallatieZin::Versie(na_is().ok_or_else(|| fout("Welke versie? Schrijf bijvoorbeeld: De versie is 1.0."))?),
        "maker" => InstallatieZin::Maker(na_is().ok_or_else(|| fout("Wie is de maker? Schrijf bijvoorbeeld: De maker is Jaap Wortel."))?),
        "programma" if klein(s, 2) == "heeft" && klein(s, 3) == "versie" && s.len() > 4 => {
            InstallatieZin::Versie(rest_tekst(&s[4..]))
        }
        "licentie" => {
            let Some(p) = zoek(s, &["bestand"], 2) else {
                return Err(fout("Schrijf het zo: De licentie staat in het bestand Licentie.txt. Of: In de licentie staat: (jouw tekst)."));
            };
            if p + 1 >= s.len() {
                return Err(fout("Welk bestand? Schrijf bijvoorbeeld: De licentie staat in het bestand Licentie.txt."));
            }
            InstallatieZin::LicentieBestand(rest_tekst(&s[p + 1..]))
        }
        "installatie" => {
            let heeft = |w: &str| zoek(s, &[w], 2).is_some();
            if !heeft("beheerdersrechten") {
                return Err(fout("Schrijf het zo: De installatie mag met en zonder beheerdersrechten. Of: alleen zonder, of: alleen met beheerdersrechten."));
            }
            let r = if heeft("met") && heeft("zonder") {
                Rechten::MetEnZonder
            } else if heeft("zonder") {
                Rechten::AlleenZonder
            } else {
                Rechten::AlleenMet
            };
            InstallatieZin::Rechten(r)
        }
        _ => return Err(onbekend()),
    };
    Ok(Vorm::Los(Soort::Installatie(zin)))
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

    if laag.starts_with("in de handleiding staat") {
        let mut tekst = norm["in de handleiding staat".len()..].trim_start_matches([':', ' ']).trim().to_string();
        if tekst.len() >= 2 && tekst.starts_with('"') && tekst.ends_with('"') {
            tekst = tekst[1..tekst.len() - 1].to_string();
        }
        if tekst.is_empty() {
            return Err(fout("Wat staat er in de handleiding? Schrijf bijvoorbeeld: In de handleiding staat: Druk op Welkom om gegroet te worden."));
        }
        if !tekst.ends_with(['.', '!', '?', ':']) {
            tekst.push('.');
        }
        return Ok(Vorm::Los(Soort::Handleiding { regel: Some(tekst), bestand: None }));
    }
    if laag.starts_with("in de licentie staat") {
        let mut tekst = norm["in de licentie staat".len()..].trim_start_matches([':', ' ']).trim().to_string();
        if tekst.len() >= 2 && tekst.starts_with('"') && tekst.ends_with('"') {
            tekst = tekst[1..tekst.len() - 1].to_string();
        }
        if tekst.is_empty() {
            return Err(fout("Wat staat er in de licentie? Schrijf bijvoorbeeld: In de licentie staat: Je mag dit programma gratis gebruiken."));
        }
        if !tekst.ends_with(['.', '!', '?', ':']) {
            tekst.push('.');
        }
        return Ok(Vorm::Los(Soort::Installatie(InstallatieZin::LicentieRegel(tekst))));
    }

    let s = tekst::splits(norm).map_err(fout)?;
    if s.is_empty() {
        return Ok(Vorm::Leeg);
    }
    let w0 = klein(&s, 0);
    let u = |r: &[Stuk]| uitdrukking(r).map_err(fout);
    let n = |r: &[Stuk]| naam(r).map_err(fout);

    match w0.as_str() {
        "maak" if begint(&s, &["maak", "de", "lijst"]) && klein(&s, s.len() - 1) == "leeg" && s.len() >= 5 => {
            Ok(Vorm::Los(Soort::Verander { naam: n(&s[3..s.len() - 1])?, waarde: Uitdrukking::LegeLijst }))
        }
        "maak" if matches!(klein(&s, 1).as_str(), "een" | "nieuwe" | "lege") && matches!(klein(&s, 2).as_str(), "lijst" | "stukje") => {
            let wat = klein(&s, 2);
            let mut i = 3;
            if klein(&s, i) == "met" {
                i += 1;
            }
            if matches!(klein(&s, i).as_str(), "als" | "de") {
                i += 1;
            }
            if klein(&s, i) == "naam" {
                i += 1;
            }
            let voorbeeld = if wat == "lijst" {
                "Maak een lijst met als naam boodschappen"
            } else {
                "Maak een stukje met als naam Verdubbel, met als invoer getal"
            };
            let eind_naam = zoek(&s, &["met"], i).unwrap_or(s.len());
            if i >= eind_naam {
                return Err(fout(format!("Welke naam moet de {} krijgen? Schrijf bijvoorbeeld: {}.", wat, voorbeeld)));
            }
            if wat == "lijst" {
                if eind_naam != s.len() {
                    return Err(fout("Een lijst begint altijd leeg. Voeg daarna dingen toe met: Voeg \"melk\" toe aan de lijst boodschappen."));
                }
                return Ok(Vorm::Los(Soort::Onthoud {
                    naam: n(&s[i..])?,
                    waarde: Uitdrukking::LegeLijst,
                    bron: "een lege lijst".into(),
                }));
            }
            let stukjenaam = s[i..eind_naam].iter().filter_map(|x| x.klein()).collect::<Vec<_>>().join(" ");
            let mut invoer = Vec::new();
            if eind_naam < s.len() {
                let Some(p) = zoek(&s, &["invoer"], eind_naam) else {
                    return Err(fout(format!("Schrijf het zo: {}.", voorbeeld)));
                };
                let rest = &s[p + 1..];
                let mut begin = 0;
                for j in 0..=rest.len() {
                    if j == rest.len() || klein(rest, j) == "en" {
                        if j == begin {
                            return Err(fout("Bij de invoer mis ik een naam rond \"en\"."));
                        }
                        invoer.push(n(&rest[begin..j])?);
                        begin = j + 1;
                    }
                }
            }
            Ok(Vorm::OpenStukje(stukjenaam, invoer))
        }
        "maak" if matches!(klein(&s, 1).as_str(), "het" | "de")
            && klein(&s, s.len() - 1) == "leeg"
            && s.len() >= 5
            && is_onderdeelwoord(&klein(&s, 2)) =>
        {
            Ok(Vorm::Los(Soort::ZetIn { waarde: Uitdrukking::Vast(Waarde::Tekst(String::new())), naam: rest_tekst(&s[3..s.len() - 1]) }))
        }
        "maak" => {
            let mut i = 1;
            if is_lidwoord(&klein(&s, i)) {
                i += 1;
            }
            match klein(&s, i).as_str() {
                "venster" | "scherm" => Ok(Vorm::Los(Soort::MaakVenster {
                    titel: met_als_titel(&s, i + 1, &["titel", "naam"], "venster", "Maak een venster met als titel Begroeting")?,
                })),
                "installatiebestand" | "installatie" | "installatieprogramma" => Ok(Vorm::Los(Soort::Installatie(
                    InstallatieZin::Naam(met_als_titel(&s, i + 1, &["naam", "titel"], "installatie", "Maak een installatiebestand met als naam Begroeting")?),
                ))),
                "groot" | "grote" if klein(&s, i + 1) == "tekstvak" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::GrootTekstvak,
                    naam: met_als_titel(&s, i + 2, &["naam", "titel"], "het tekstvak", "Maak een groot tekstvak met als naam Verhaal")?,
                })),
                "map" | "folder" => {
                    let naam = met_als_titel(&s, i + 1, &["naam", "titel"], "de map", "Maak een map met als naam Mijn notities")?;
                    Ok(Vorm::Los(Soort::MaakMap(Uitdrukking::Vast(Waarde::Tekst(naam)))))
                }
                "tekstvak" | "invulvak" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Tekstvak,
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "het tekstvak", "Maak een tekstvak met als naam Naam")?,
                })),
                "label" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Label,
                    naam: met_als_titel(&s, i + 1, &["naam", "tekst", "titel"], "het label", "Maak een label met als naam Antwoord")?,
                })),
                "selectievakje" | "vinkje" | "keuzevakje" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Selectievakje,
                    naam: met_als_titel(&s, i + 1, &["naam", "tekst", "titel"], "het selectievakje", "Maak een selectievakje met als naam Geluid aan")?,
                })),
                "keuzelijst" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Keuzelijst,
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "de keuzelijst", "Maak een keuzelijst met als naam Kleur")?,
                })),
                "uitklaplijst" | "combobox" | "keuzemenu" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Uitklaplijst,
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "de uitklaplijst", "Maak een uitklaplijst met als naam Kleur")?,
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
            if onderdeel_operand(r).is_some() {
                return Ok(Vorm::Los(Soort::Toon(u(r)?)));
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
        "zet" if zoek(&s, &["snelkoppeling"], 1).is_some() => {
            if zoek(&s, &["bureaublad"], 1).is_some() {
                Ok(Vorm::Los(Soort::Installatie(InstallatieZin::Bureaublad)))
            } else if zoek(&s, &["startmenu"], 1).is_some() {
                Ok(Vorm::Los(Soort::Installatie(InstallatieZin::Startmenu)))
            } else {
                Err(fout("Waar moet de snelkoppeling komen? Schrijf: Zet een snelkoppeling op het bureaublad, of: Zet een snelkoppeling in het startmenu."))
            }
        }
        "de" if klein(&s, 1) == "handleiding" => {
            let Some(p) = zoek(&s, &["bestand"], 2) else {
                return Err(fout("Schrijf het zo: De handleiding staat in het bestand Handleiding.txt. Of: In de handleiding staat: (jouw tekst)."));
            };
            if p + 1 >= s.len() {
                return Err(fout("Welk bestand? Schrijf bijvoorbeeld: De handleiding staat in het bestand Handleiding.txt."));
            }
            Ok(Vorm::Los(Soort::Handleiding { regel: None, bestand: Some(rest_tekst(&s[p + 1..])) }))
        }
        "de" | "het" if matches!(klein(&s, 1).as_str(), "installatie" | "versie" | "maker" | "programma" | "licentie") => {
            installatie_zin(&s)
        }
        "voeg" if (1..s.len()).any(|i| matches!(klein(&s, i).as_str(), "keuzelijst" | "uitklaplijst" | "combobox" | "keuzemenu")) => {
            let Some(t) = zoek(&s, &["toe", "aan"], 1) else {
                return Err(fout("Schrijf het zo: Voeg de keuze Rood toe aan de keuzelijst Kleur."));
            };
            let mut b = 1;
            while b < t && matches!(klein(&s, b).as_str(), "de" | "het" | "keuze") {
                b += 1;
            }
            let w = &s[b..t];
            if w.is_empty() {
                return Err(fout("Welke keuze? Schrijf bijvoorbeeld: Voeg de keuze Rood toe aan de keuzelijst Kleur."));
            }
            let keuze = if matches!(w[0], Stuk::Tekst(_)) && w.len() > 1 {
                u(w)?
            } else {
                Uitdrukking::Vast(Waarde::Tekst(rest_tekst(w)))
            };
            let Some(l) = (t + 2..s.len()).find(|&i| matches!(klein(&s, i).as_str(), "keuzelijst" | "uitklaplijst" | "combobox" | "keuzemenu")) else {
                return Err(fout("Aan welke keuzelijst? Schrijf bijvoorbeeld: ... toe aan de keuzelijst Kleur."));
            };
            if l + 1 >= s.len() {
                return Err(fout("Aan welke keuzelijst? Schrijf bijvoorbeeld: ... toe aan de keuzelijst Kleur."));
            }
            Ok(Vorm::Los(Soort::VoegKeuzeToe { keuze, lijst: rest_tekst(&s[l + 1..]) }))
        }
        "voeg" if zoek(&s, &["toe", "aan", "het", "bestand"], 1).is_some() => {
            let t = zoek(&s, &["toe", "aan", "het", "bestand"], 1).unwrap();
            if t == 1 {
                return Err(fout("Wat moet erbij? Schrijf bijvoorbeeld: Voeg \"Nieuwe regel\" toe aan het bestand notities.txt."));
            }
            Ok(Vorm::Los(Soort::SchrijfBestand { waarde: u(&s[1..t])?, pad: pad(&s[t + 4..]).map_err(fout)?, toevoegen: true }))
        }
        "voeg" if zoek(&s, &["toe", "aan"], 1).map_or(false, |t| zoek(&s, &["lijst"], t).is_some()) => {
            let t = zoek(&s, &["toe", "aan"], 1).unwrap();
            let l = zoek(&s, &["lijst"], t).unwrap();
            if t == 1 || l + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Voeg \"melk\" toe aan de lijst boodschappen."));
            }
            Ok(Vorm::Los(Soort::VoegToeAanLijst { waarde: u(&s[1..t])?, lijst: n(&s[l + 1..])? }))
        }
        "haal" => {
            let (Some(w), Some(l)) = (zoek(&s, &["weg"], 1), zoek(&s, &["lijst"], 1)) else {
                return Err(fout("Schrijf het zo: Haal \"melk\" weg uit de lijst boodschappen."));
            };
            if w == 1 || l + 1 >= s.len() || l < w {
                return Err(fout("Schrijf het zo: Haal \"melk\" weg uit de lijst boodschappen."));
            }
            Ok(Vorm::Los(Soort::HaalUitLijst { waarde: u(&s[1..w])?, lijst: n(&s[l + 1..])? }))
        }
        "schrijf" => {
            if let Some(l) = zoek(&s, &["logboek"], 1) {
                // "Schrijf in het logboek X" of "Schrijf X in het logboek"
                let waarde = if l + 1 < s.len() {
                    &s[l + 1..]
                } else {
                    let tot = (1..l).rev().find(|&i| klein(&s, i) == "in").unwrap_or(l);
                    &s[1..tot]
                };
                if waarde.is_empty() {
                    return Err(fout("Wat moet er in het logboek? Schrijf bijvoorbeeld: Schrijf in het logboek \"Het programma is gestart\"."));
                }
                return Ok(Vorm::Los(Soort::Logboek(tekst_of_uitdrukking(waarde).map_err(fout)?)));
            }
            let Some(b) = zoek(&s, &["bestand"], 2) else {
                return Err(fout("Schrijf het zo: Schrijf \"Hallo\" in het bestand notities.txt. Of: Schrijf in het logboek \"Gestart\"."));
            };
            let mut tot = b;
            while tot > 1 && matches!(klein(&s, tot - 1).as_str(), "het" | "in" | "naar") {
                tot -= 1;
            }
            if tot <= 1 {
                return Err(fout("Wat moet er in het bestand? Schrijf bijvoorbeeld: Schrijf \"Hallo\" in het bestand notities.txt."));
            }
            Ok(Vorm::Los(Soort::SchrijfBestand { waarde: u(&s[1..tot])?, pad: pad(&s[b + 1..]).map_err(fout)?, toevoegen: false }))
        }
        "piep" if s.len() == 1 => Ok(Vorm::Los(Soort::Piep {
            hoogte: Uitdrukking::Vast(Waarde::Getal(880.0)),
            duur: Uitdrukking::Vast(Waarde::Getal(200.0)),
        })),
        "speel" => {
            if zoek(&s, &["piep"], 1).is_some() || zoek(&s, &["toon"], 1).is_some() {
                // Speel een piep [van 440 hertz] [van 300 milliseconden]
                let getal_voor = |eenheid: &[&str]| -> Result<Option<Uitdrukking>, RegelFout> {
                    for e in eenheid {
                        if let Some(p) = zoek(&s, &[e], 2) {
                            if p >= 1 {
                                return Ok(Some(operand(&s[p - 1..p], false).map_err(fout)?));
                            }
                        }
                    }
                    Ok(None)
                };
                let hoogte = getal_voor(&["hertz", "hz"])?.unwrap_or(Uitdrukking::Vast(Waarde::Getal(880.0)));
                let duur = getal_voor(&["milliseconden", "ms"])?.unwrap_or(Uitdrukking::Vast(Waarde::Getal(200.0)));
                return Ok(Vorm::Los(Soort::Piep { hoogte, duur }));
            }
            let Some(g) = zoek(&s, &["geluid"], 1) else {
                return Err(fout("Schrijf het zo: Speel het geluid deurbel.wav. Of: Speel een piep."));
            };
            let mut eind = s.len();
            let mut wachten = false;
            if begint(&s[eind.saturating_sub(2)..], &["en", "wacht"]) {
                wachten = true;
                eind -= 2;
            }
            if klein(&s, eind - 1) == "af" {
                eind -= 1;
            }
            if g + 1 >= eind {
                return Err(fout("Welk geluid? Schrijf bijvoorbeeld: Speel het geluid deurbel.wav."));
            }
            Ok(Vorm::Los(Soort::SpeelGeluid { pad: pad(&s[g + 1..eind]).map_err(fout)?, wachten }))
        }
        "stop" if zoek(&s, &["geluid"], 1).is_some() => Ok(Vorm::Los(Soort::StopGeluid)),
        "geef" if zoek(&s, &["sneltoets"], 2).is_some() => {
            let k = zoek(&s, &["knop"], 1).ok_or_else(|| fout("Schrijf het zo: Geef de knop Welkom de sneltoets Ctrl+W."))?;
            let t = zoek(&s, &["sneltoets"], k).unwrap();
            let mut eind_naam = t;
            if matches!(klein(&s, eind_naam - 1).as_str(), "de" | "als") {
                eind_naam -= 1;
            }
            if eind_naam <= k + 1 || t + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Geef de knop Welkom de sneltoets Ctrl+W."));
            }
            let toets = lees_toets(&rest_tekst(&s[t + 1..])).map_err(fout)?;
            Ok(Vorm::Los(Soort::Sneltoets { knop: rest_tekst(&s[k + 1..eind_naam]), toets }))
        }
        "geef" if zoek(&s, &["kleur"], 2).is_some() => {
            let k = zoek(&s, &["kleur"], 2).unwrap();
            let doel: Vec<String> = s[1..k].iter().filter_map(|x| x.klein()).filter(|w| !matches!(w.as_str(), "de" | "het")).collect();
            let achtergrond = match doel.iter().map(|x| x.as_str()).collect::<Vec<_>>().as_slice() {
                ["venster"] | ["achtergrond"] | ["scherm"] => true,
                ["tekst"] | ["letters"] => false,
                _ => return Err(fout("Schrijf het zo: Geef het venster de kleur donkerblauw. Of: Geef de tekst de kleur wit.")),
            };
            let naam = s[k + 1..].iter().filter_map(|x| x.klein()).collect::<Vec<_>>().join("");
            let Some((kleur, rgb)) = KLEUREN.iter().find(|(n, _)| *n == naam) else {
                let lijst: Vec<&str> = KLEUREN.iter().map(|k| k.0).collect();
                return Err(fout(format!("De kleur \"{}\" ken ik niet. Deze kleuren ken ik: {}.", naam, lijst.join(", "))));
            };
            Ok(Vorm::Los(Soort::Kleur { achtergrond, kleur: kleur.to_string(), rgb: *rgb }))
        }
        "doe" => {
            if s.len() < 2 {
                return Err(fout("Welk stukje? Schrijf bijvoorbeeld: Doe Verdubbel met 5."));
            }
            Ok(Vorm::Los(Soort::Doe(aanroep(&s[1..]).map_err(fout)?)))
        }
        "geef" => {
            let mut r = &s[1..];
            if begint(r, &["als", "uitkomst"]) {
                r = &r[2..];
            } else if begint(r, &["de", "uitkomst"]) {
                r = &r[2..];
            } else if klein(r, r.len().wrapping_sub(1)) == "terug" {
                r = &r[..r.len() - 1];
            } else {
                return Err(fout("Schrijf het zo: Geef als uitkomst getal keer 2."));
            }
            Ok(Vorm::Los(Soort::GeefUitkomst(u(r)?)))
        }
        "voeg" => {
            // Voeg het bestand X toe aan de installatie
            let Some(t) = zoek(&s, &["toe"], 1) else {
                return Err(fout("Schrijf het zo: Voeg het bestand Handleiding.txt toe aan de installatie."));
            };
            let mut b = 1;
            while b < t && matches!(klein(&s, b).as_str(), "het" | "de" | "een" | "bestand") {
                b += 1;
            }
            if b >= t {
                return Err(fout("Welk bestand? Schrijf bijvoorbeeld: Voeg het bestand Handleiding.txt toe aan de installatie."));
            }
            if zoek(&s, &["installatie"], t).is_none() && zoek(&s, &["installatiebestand"], t).is_none() {
                return Err(fout("Waaraan moet ik het bestand toevoegen? Schrijf: ... toe aan de installatie."));
            }
            Ok(Vorm::Los(Soort::Installatie(InstallatieZin::Bestand(rest_tekst(&s[b..t])))))
        }
        "zet" if zoek(&s, &["focus"], 1).is_some() => {
            let Some(p) = (2..s.len()).find(|&i| is_onderdeelwoord(&klein(&s, i))) else {
                return Err(fout("Waar moet de focus naartoe? Schrijf bijvoorbeeld: Zet de focus op het tekstvak Naam."));
            };
            if p + 1 >= s.len() {
                return Err(fout("Op welk onderdeel? Schrijf bijvoorbeeld: Zet de focus op het tekstvak Naam."));
            }
            Ok(Vorm::Los(Soort::Focus { naam: rest_tekst(&s[p + 1..]) }))
        }
        "zet" if (1..s.len()).any(|i| klein(&s, i) == "in" && is_onderdeelwoord(&klein(&s, i + 2))) => {
            let p = (1..s.len()).find(|&i| klein(&s, i) == "in" && is_onderdeelwoord(&klein(&s, i + 2))).unwrap();
            if p + 3 >= s.len() {
                return Err(fout("In welk onderdeel? Schrijf bijvoorbeeld: Zet de tekst Hallo in het label Antwoord."));
            }
            let naam = rest_tekst(&s[p + 3..]);
            let mut w = &s[1..p];
            let letterlijk = if begint(w, &["de", "tekst"]) {
                w = &w[2..];
                true
            } else if begint(w, &["tekst"]) {
                w = &w[1..];
                true
            } else {
                false
            };
            if w.is_empty() {
                return Err(fout("Wat moet erin komen? Schrijf bijvoorbeeld: Zet de tekst Hallo in het label Antwoord."));
            }
            let waarde = if letterlijk && !(matches!(w[0], Stuk::Tekst(_)) && w.len() > 1) {
                Uitdrukking::Vast(Waarde::Tekst(rest_tekst(w)))
            } else {
                u(w)?
            };
            Ok(Vorm::Los(Soort::ZetIn { waarde, naam }))
        }
        "vink" => {
            let laatste = klein(&s, s.len() - 1);
            let aan = match laatste.as_str() {
                "aan" => true,
                "uit" => false,
                _ => return Err(fout("Schrijf het zo: Vink het selectievakje Geluid aan. Of: ... uit.")),
            };
            let mut b = 1;
            while b < s.len() - 1 && matches!(klein(&s, b).as_str(), "het" | "de" | "selectievakje" | "vinkje" | "keuzevakje") {
                b += 1;
            }
            if b >= s.len() - 1 {
                return Err(fout("Welk selectievakje? Schrijf bijvoorbeeld: Vink het selectievakje Geluid aan."));
            }
            Ok(Vorm::Los(Soort::Vink { naam: rest_tekst(&s[b..s.len() - 1]), aan }))
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
            if klein(r, 0) == "voor" && matches!(klein(r, 1).as_str(), "elk" | "elke" | "ieder" | "iedere") {
                let r = &r[2..];
                if let Some(p) = (1..r.len()).find(|&i| matches!(klein(r, i).as_str(), "in" | "uit") && zoek(r, &["lijst"], i).is_some()) {
                    let l = zoek(r, &["lijst"], p).unwrap();
                    if l + 1 >= r.len() {
                        return Err(fout("Welke lijst? Schrijf bijvoorbeeld: Herhaal voor elk ding in de lijst boodschappen."));
                    }
                    return Ok(Vorm::OpenVoorElk(n(&r[..p])?, Uitdrukking::Naam(n(&r[l + 1..])?)));
                }
                if let (Some(v), Some(t)) = (zoek(r, &["van"], 1), zoek(r, &["tot"], 2)) {
                    let mut tot = t + 1;
                    if begint(&r[tot..], &["en", "met"]) {
                        tot += 2;
                    }
                    if v < t && tot < r.len() {
                        return Ok(Vorm::OpenTel(n(&r[..v])?, u(&r[v + 1..t])?, u(&r[tot..])?));
                    }
                }
                return Err(fout("Schrijf het zo: Herhaal voor elk ding in de lijst boodschappen. Of: Herhaal voor elk getal van 1 tot 10."));
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
                ["stukje"] => Ok(Vorm::Einde(Blok::Stukje)),
                _ => Err(fout("Ik weet niet welk blok je wilt sluiten. Schrijf: Einde als, Einde herhaal, Einde van de knop of Einde van het stukje.")),
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
    VoorElk(String, Uitdrukking),
    Tel(String, Uitdrukking, Uitdrukking),
    Stukje(String, Vec<String>),
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
            KaderSoort::Herhaal(_) | KaderSoort::Zolang(..) | KaderSoort::VoorElk(..) | KaderSoort::Tel(..) => Blok::Herhaal,
            KaderSoort::Knop(_) => Blok::Knop,
            KaderSoort::Stukje(..) => Blok::Stukje,
        }
    }
    fn beschrijving(&self) -> String {
        match &self.soort {
            KaderSoort::Hoofd => "het programma".into(),
            KaderSoort::Als(_) => format!("de keuze (Als) van regel {}", self.regel),
            KaderSoort::Herhaal(_) | KaderSoort::Zolang(..) | KaderSoort::VoorElk(..) | KaderSoort::Tel(..) => {
                format!("de herhaling van regel {}", self.regel)
            }
            KaderSoort::Knop(n) => format!("de knop {} van regel {}", n, self.regel),
            KaderSoort::Stukje(n, _) => format!("het stukje {} van regel {}", n, self.regel),
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
            KaderSoort::VoorElk(naam, lijst) => Soort::VoorElk { naam, lijst, blok: self.dan, einde },
            KaderSoort::Tel(naam, van, tot) => Soort::Tel { naam, van, tot, blok: self.dan, einde },
            KaderSoort::Stukje(naam, invoer) => Soort::Stukje { naam, invoer, blok: self.dan, einde },
        };
        Opdracht { regel: self.regel, soort }
    }
}

pub(crate) fn sluitzin(b: Blok) -> &'static str {
    match b {
        Blok::Als => "Einde als",
        Blok::Herhaal => "Einde herhaal",
        Blok::Knop => "Einde van de knop",
        Blok::Stukje => "Einde van het stukje",
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
            Ok(Vorm::Los(soort)) => {
                if matches!(soort, Soort::GeefUitkomst(_)) && !stapel.iter().any(|k| matches!(k.soort, KaderSoort::Stukje(..))) {
                    meldingen.push(Melding::fout(nr, "\"Geef als uitkomst\" kan alleen binnen een eigen stukje (tussen Maak een stukje en Einde van het stukje)."));
                }
                stapel.last_mut().unwrap().lijst().push(Opdracht { regel: nr, soort })
            }
            Ok(Vorm::OpenVoorElk(naam, l)) => stapel.push(Kader::nieuw(KaderSoort::VoorElk(naam, l), nr)),
            Ok(Vorm::OpenTel(naam, van, tot)) => stapel.push(Kader::nieuw(KaderSoort::Tel(naam, van, tot), nr)),
            Ok(Vorm::OpenStukje(naam, invoer)) => {
                if stapel.len() > 1 {
                    meldingen.push(Melding::fout(
                        nr,
                        format!("Een eigen stukje moet los staan, niet binnen een ander blok. Sluit eerst {}.", stapel.last().unwrap().beschrijving()),
                    ));
                }
                stapel.push(Kader::nieuw(KaderSoort::Stukje(naam, invoer), nr));
            }
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
    let mut programma = Programma { opdrachten: hoofd.dan };
    verbind_stukjes(&mut programma);
    (programma, meldingen)
}

/// Een naam van een eigen stukje zonder invoer ("Toon de uitkomst van Dobbelsteen") wordt een aanroep.
fn verbind_stukjes(p: &mut Programma) {
    let namen: Vec<String> = p
        .opdrachten
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::Stukje { naam, .. } => Some(naam.clone()),
            _ => None,
        })
        .collect();
    if namen.is_empty() {
        return;
    }
    fn lijst(l: &mut [Opdracht], namen: &[String]) {
        for o in l {
            for u in o.soort.uitdrukkingen_mut() {
                u.verbind_stukjes(namen);
            }
            for blok in o.soort.blokken_mut() {
                lijst(blok, namen);
            }
        }
    }
    lijst(&mut p.opdrachten, &namen);
}

fn hoofdletter(s: &str) -> String {
    tekst::zelfde_hoofdletter("A", s)
}
