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
    "privacyverklaring", "onderteken", "certificaat", "pak", "zipbestand", "zeg", "schermlezer", "pictogram", "lees", "voor", "voorlezen", "volume", "menu", "menuregel",
    "kies", "kopieer", "verplaats", "gooi", "verwijder", "tabblad", "ga", "verstuur", "download", "adres", "webpagina",
    "opdrachtvak", "zeg", "opdracht", "werkbalk", "knoppen",
    "wortel", "macht", "afgerond", "decimalen", "rest", "delen", "absolute", "lengte", "tekens", "hoofdletters", "kleine", "letters",
    "woorden", "gesplitst", "samengevoegd", "vervangen", "waarin", "spaties", "seconden", "programma", "open",
    "updates", "zoek", "opnemen", "opname", "begin", "start", "afbeelding", "beschrijving", "tabel", "kolom", "rij", "rijen", "sorteer", "sla", "gekozen",
];

/// Onderdelen die op de wensenlijst staan maar nog niet gebouwd zijn.
const NOG_NIET: &[&str] = &[
    "tekstverwerker", "streepjescode",
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
    matches!(w, "tabel" | "tekstvak" | "invulvak" | "label" | "selectievakje" | "vinkje" | "keuzevakje" | "keuzelijst" | "uitklaplijst" | "combobox" | "keuzemenu")
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
    for (voor, wat) in [
        (&["de", "inhoud", "van", "de", "webpagina"][..], Ingebouwd::Webpagina),
        (&["de", "tekst", "van", "de", "webpagina"][..], Ingebouwd::WebTekst),
    ] {
        if begint(s, voor) {
            return ib(wat, vec![pad(&s[voor.len()..])?]);
        }
    }
    // De tekst van het Word-bestand brief.docx / van het PDF-bestand brief.pdf
    if begint(s, &["de", "tekst", "van", "het"]) || begint(s, &["de", "tekst", "uit", "het"]) {
        if s.len() > 5 && is_documentwoord(&klein(s, 4)) {
            return ib(Ingebouwd::DocumentTekst, vec![pad(&s[5..])?]);
        }
    }
    // Tabellen: de gekozen rij, het aantal rijen, rij nummer 2.
    let tabelnaam = |s: &[Stuk], vanaf: usize| -> Option<Uitdrukking> {
        let t = zoek(s, &["tabel"], vanaf)?;
        (t + 1 < s.len()).then(|| Uitdrukking::Vast(Waarde::Tekst(rest_tekst(&s[t + 1..]))))
    };
    if begint(s, &["de", "gekozen", "rij"]) {
        if let Some(t) = tabelnaam(s, 3) {
            return ib(Ingebouwd::GekozenRij, vec![t]);
        }
    }
    if begint(s, &["het", "aantal", "rijen"]) {
        if let Some(t) = tabelnaam(s, 3) {
            return ib(Ingebouwd::AantalRijen, vec![t]);
        }
    }
    if begint(s, &["rij", "nummer"]) || begint(s, &["de", "rij", "nummer"]) {
        let b = if klein(s, 0) == "de" { 3 } else { 2 };
        if let Some(v) = (b + 1..s.len()).find(|&i| matches!(klein(s, i).as_str(), "van" | "uit")) {
            if let Some(t) = tabelnaam(s, v) {
                return ib(Ingebouwd::RijNummer, vec![t, operand(&s[b..v], false)?]);
            }
        }
    }
    if let Some(f) = functie(s)? {
        return Ok(Some(f));
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

/// Rekenen en tekst: de wortel van 16, 2 tot de macht 3, de lengte van naam, naam in hoofdletters, ...
fn functie(s: &[Stuk]) -> Result<Option<Uitdrukking>, String> {
    let ib = |wat, invoer| Ok(Some(Uitdrukking::Ingebouwd { wat, invoer }));
    let op = |d: &[Stuk], voorbeeld: &str| -> Result<Uitdrukking, String> {
        if d.is_empty() {
            return Err(format!("Hier mist iets. Schrijf het zo: {}", voorbeeld));
        }
        operand(d, false)
    };
    // Vaste woorden vooraan, daarna één waarde.
    for (voor, wat, vb) in [
        (&["de", "wortel", "van"][..], Ingebouwd::Wortel, "de wortel van 16"),
        (&["de", "wortel", "uit"][..], Ingebouwd::Wortel, "de wortel uit 16"),
        (&["de", "absolute", "waarde", "van"][..], Ingebouwd::Absoluut, "de absolute waarde van getal"),
        (&["de", "lengte", "van"][..], Ingebouwd::Lengte, "de lengte van naam"),
        (&["het", "aantal", "tekens", "van"][..], Ingebouwd::Lengte, "het aantal tekens van naam"),
        (&["het", "aantal", "tekens", "in"][..], Ingebouwd::Lengte, "het aantal tekens in naam"),
        (&["de", "woorden", "van"][..], Ingebouwd::Woorden, "de woorden van zin"),
        (&["de", "woorden", "in"][..], Ingebouwd::Woorden, "de woorden in zin"),
    ] {
        if begint(s, voor) && !(wat == Ingebouwd::Lengte && zoek(s, &["lijst"], voor.len()) == Some(voor.len())) {
            return ib(wat, vec![op(&s[voor.len()..], vb)?]);
        }
    }
    // de rest bij het delen van 7 door 2
    if begint(s, &["de", "rest", "bij", "het", "delen", "van"]) || begint(s, &["de", "rest", "van"]) {
        let b = if klein(s, 2) == "bij" { 6 } else { 3 };
        let vb = "de rest bij het delen van 7 door 2";
        let Some(d) = zoek(s, &["door"], b + 1) else {
            return Err(format!("Waardoor moet ik delen? Schrijf het zo: {}", vb));
        };
        return ib(Ingebouwd::Rest, vec![op(&s[b..d], vb)?, op(&s[d + 1..], vb)?]);
    }
    // de eerste 3 tekens van naam / de laatste 3 tekens van naam
    if (begint(s, &["de", "eerste"]) || begint(s, &["de", "laatste"])) && zoek(s, &["tekens"], 3).is_some() || zoek(s, &["teken"], 2) == Some(2) {
        let t = zoek(s, &["tekens"], 2).or_else(|| zoek(s, &["teken"], 2)).unwrap();
        let wat = if klein(s, 1) == "eerste" { Ingebouwd::EersteTekens } else { Ingebouwd::LaatsteTekens };
        let vb = "de eerste 3 tekens van naam";
        if !matches!(klein(s, t + 1).as_str(), "van" | "uit") {
            return Err(format!("Van welke tekst? Schrijf het zo: {}", vb));
        }
        let n = if t == 2 { Uitdrukking::Vast(Waarde::Getal(1.0)) } else { op(&s[2..t], vb)? };
        return ib(wat, vec![op(&s[t + 2..], vb)?, n]);
    }
    // Eén waarde, daarna vaste woorden.
    let einde = |achter: &[&str]| -> bool {
        s.len() > achter.len() && s[s.len() - achter.len()..].iter().zip(achter).all(|(x, w)| x.klein().as_deref() == Some(*w))
    };
    for (achter, wat, vb) in [
        (&["in", "hoofdletters"][..], Ingebouwd::Hoofdletters, "naam in hoofdletters"),
        (&["in", "kleine", "letters"][..], Ingebouwd::KleineLetters, "naam in kleine letters"),
        (&["als", "getal"][..], Ingebouwd::AlsGetal, "de tekst in het tekstvak Leeftijd als getal"),
        (&["als", "tekst"][..], Ingebouwd::AlsTekst, "getal als tekst"),
        (&["zonder", "spaties"][..], Ingebouwd::ZonderSpaties, "naam zonder spaties"),
        (&["zonder", "min"][..], Ingebouwd::Absoluut, "getal zonder min"),
        (&["afgerond"][..], Ingebouwd::Afgerond, "getal afgerond"),
    ] {
        if einde(achter) {
            let d = &s[..s.len() - achter.len()];
            let mut invoer = vec![op(d, vb)?];
            if wat == Ingebouwd::Afgerond {
                invoer.push(Uitdrukking::Vast(Waarde::Getal(0.0)));
            }
            return ib(wat, invoer);
        }
    }
    // getal afgerond op 2 decimalen / cijfers achter de komma
    if let Some(a) = zoek(s, &["afgerond", "op"], 1) {
        let mut eind = s.len();
        for achter in [&["cijfers", "achter", "de", "komma"][..], &["cijfer", "achter", "de", "komma"][..], &["decimalen"][..], &["decimaal"][..]] {
            if einde(achter) {
                eind = s.len() - achter.len();
            }
        }
        let vb = "getal afgerond op 2 decimalen";
        if eind == s.len() || eind <= a + 2 {
            return Err(format!("Op hoeveel cijfers achter de komma? Schrijf het zo: {}", vb));
        }
        return ib(Ingebouwd::Afgerond, vec![op(&s[..a], vb)?, op(&s[a + 2..eind], vb)?]);
    }
    // 2 tot de macht 3
    if let Some(t) = zoek(s, &["tot", "de", "macht"], 1) {
        let vb = "2 tot de macht 3";
        return ib(Ingebouwd::Macht, vec![op(&s[..t], vb)?, op(&s[t + 3..], vb)?]);
    }
    // zin gesplitst bij ","
    if let Some(g) = zoek(s, &["gesplitst", "bij"], 1).or_else(|| zoek(s, &["gesplitst", "op"], 1)) {
        let vb = "zin gesplitst bij \",\"";
        return ib(Ingebouwd::Gesplitst, vec![op(&s[..g], vb)?, op(&s[g + 2..], vb)?]);
    }
    // de lijst namen samengevoegd met ", "
    if let Some(g) = zoek(s, &["samengevoegd", "met"], 1) {
        let vb = "de lijst namen samengevoegd met \", \"";
        return ib(Ingebouwd::Samengevoegd, vec![op(&s[..g], vb)?, op(&s[g + 2..], vb)?]);
    }
    // zin waarin "a" vervangen is door "b" / zin met "a" vervangen door "b"
    if let Some(v) = zoek(s, &["vervangen"], 2) {
        let vb = "zin waarin \"fiets\" vervangen is door \"auto\"";
        let Some(m) = (1..v).rev().find(|&i| matches!(klein(s, i).as_str(), "waarin" | "met")) else {
            return Err(format!("Schrijf het zo: {}", vb));
        };
        let mut d = v + 1;
        if klein(s, d) == "is" || klein(s, d) == "wordt" {
            d += 1;
        }
        if klein(s, d) != "door" {
            return Err(format!("Waardoor moet het vervangen worden? Schrijf het zo: {}", vb));
        }
        return ib(Ingebouwd::Vervangen, vec![op(&s[..m], vb)?, op(&s[m + 1..v], vb)?, op(&s[d + 1..], vb)?]);
    }
    Ok(None)
}

fn is_documentwoord(w: &str) -> bool {
    matches!(w, "word-bestand" | "wordbestand" | "worddocument" | "word-document" | "pdf-bestand" | "pdfbestand" | "pdf-document")
}

/// Wat de schermlezer moet zeggen of de stem moet voorlezen: letterlijke tekst, of een waarde.
fn spreektekst(r: &[Stuk]) -> Result<Uitdrukking, RegelFout> {
    let letterlijk = !r.iter().any(|x| matches!(x, Stuk::Tekst(_)))
        && uitdrukking(r).map_or(true, |u| matches!(u, Uitdrukking::Naam(_)) && r.len() > 1);
    if letterlijk && !begint(r, &["de", "tekst", "in"]) {
        let t = if begint(r, &["de", "tekst"]) { rest_tekst(&r[2..]) } else { rest_tekst(r) };
        return Ok(Uitdrukking::Vast(Waarde::Tekst(t)));
    }
    match ontleed_regel(&format!("Toon {}", bron_tekst(r)))? {
        Vorm::Los(Soort::Toon(u)) => Ok(u),
        _ => Err(fout("Schrijf het zo: Zeg Het bestand is opgeslagen.")),
    }
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
    // "10 gedeeld door 3 afgerond op 2 decimalen": het afronden hoort bij de hele som.
    if let Some(a) = (1..s.len()).rev().find(|&i| klein(s, i) == "afgerond") {
        let rekent = s[..a].iter().enumerate().any(|(i, x)| {
            matches!(x.klein().as_deref(), Some("plus" | "min" | "keer" | "maal" | "+" | "-" | "*" | "/")) || klein(s, i) == "gedeeld"
        });
        let rest: Vec<String> = s[a + 1..].iter().filter_map(|x| x.klein()).collect();
        let r: Vec<&str> = rest.iter().map(|x| x.as_str()).collect();
        let n = match r.as_slice() {
            [] => Some(Uitdrukking::Vast(Waarde::Getal(0.0))),
            ["op", .., "decimalen" | "decimaal"] | ["op", .., "cijfers" | "cijfer", "achter", "de", "komma"] => {
                let eind = s.len() - if r.last() == Some(&"komma") { 4 } else { 1 };
                (eind > a + 2).then(|| operand(&s[a + 2..eind], false)).transpose()?
            }
            _ => None,
        };
        if let (true, Some(n)) = (rekent, n) {
            return Ok(Uitdrukking::Ingebouwd { wat: Ingebouwd::Afgerond, invoer: vec![uitdrukking(&s[..a])?, n] });
        }
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

/// Waar staat een "en" of "of" dat twee voorwaarden verbindt? (Niet in "en dan", "tot en met",
/// "groter dan of gelijk aan", of tussen de invoer van een stukje na "met".)
fn logisch_woord(s: &[Stuk], woord: &str) -> Option<usize> {
    let mut na_met = false;
    for i in 1..s.len().saturating_sub(1) {
        let k = klein(s, i);
        if k == "met" {
            na_met = true;
        }
        if k == "is" || zoek(&s[i..i + 1], &["bevat"], 0).is_some() {
            na_met = false;
        }
        if k != woord {
            continue;
        }
        let voor = klein(s, i - 1);
        let na = klein(s, i + 1);
        let overslaan = match woord {
            "en" => na == "dan" || (voor == "tot" && na == "met") || na_met,
            _ => voor == "dan" && na == "gelijk",
        };
        if !overslaan {
            return Some(i);
        }
    }
    None
}

/// Leest een voorwaarde, bijvoorbeeld: teller is groter dan 5.
/// Twee voorwaarden kunnen samen met "en" (allebei waar) of "of" (minstens één waar).
pub(crate) fn voorwaarde(s: &[Stuk]) -> Result<Voorwaarde, String> {
    if s.is_empty() {
        return Err("Waar moet ik op letten? Schrijf bijvoorbeeld: Als teller is groter dan 5.".into());
    }
    for (woord, of) in [("of", true), ("en", false)] {
        if let Some(i) = logisch_woord(s, woord) {
            let (a, b) = (voorwaarde(&s[..i])?, voorwaarde(&s[i + 1..])?);
            return Ok(if of { Voorwaarde::Of(Box::new(a), Box::new(b)) } else { Voorwaarde::En(Box::new(a), Box::new(b)) });
        }
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
    if laag.starts_with("in de privacyverklaring staat") {
        let mut tekst = norm["in de privacyverklaring staat".len()..].trim_start_matches([':', ' ']).trim().to_string();
        if tekst.len() >= 2 && tekst.starts_with('"') && tekst.ends_with('"') {
            tekst = tekst[1..tekst.len() - 1].to_string();
        }
        if tekst.is_empty() {
            return Err(fout("Wat staat er in de privacyverklaring? Schrijf bijvoorbeeld: In de privacyverklaring staat: Dit programma verstuurt geen gegevens."));
        }
        if !tekst.ends_with(['.', '!', '?', ':']) {
            tekst.push('.');
        }
        return Ok(Vorm::Los(Soort::Installatie(InstallatieZin::PrivacyRegel(tekst))));
    }
    // Internetradio.
    if laag == "stop de radio" || laag == "zet de radio uit" || laag == "stop de radiozender" {
        return Ok(Vorm::Los(Soort::Radio(None)));
    }
    if laag.starts_with("speel de radiozender") || laag.starts_with("zet de radiozender") || laag.starts_with("speel de radio ") || laag.starts_with("zet de radio aan op") {
        let woorden = ["speel de radiozender met het adres", "speel de radiozender", "zet de radiozender", "speel de radio", "zet de radio aan op"];
        let b = woorden.iter().find(|w| laag.starts_with(**w)).unwrap();
        let rest = norm[b.len()..].trim().trim_end_matches(['.', '!']).trim();
        let rest = if rest.to_lowercase().starts_with("op ") { &rest[3..] } else { rest };
        if rest.is_empty() {
            return Err(fout("Welke zender? Schrijf bijvoorbeeld: Speel de radiozender NPO Radio 1. Of: Speel de radiozender met het adres https://..."));
        }
        if rest.to_lowercase().starts_with("http") {
            return Ok(Vorm::Los(Soort::Radio(Some((rest.to_string(), Uitdrukking::Vast(Waarde::Tekst(rest.to_string())))))));
        }
        let naam = rest.trim_matches('"');
        return Ok(Vorm::Los(Soort::Radio(Some(match zender(naam) {
            Some((naam, adres)) => (naam.to_string(), Uitdrukking::Vast(Waarde::Tekst(adres.into()))),
            // Niet in de lijst: je programma zoekt de zender op internet op (radio-browser.info).
            None => (naam.to_string(), Uitdrukking::Vast(Waarde::Tekst(format!("zoek:{}", naam)))),
        }))));
    }
    // Hoe het venster zich gedraagt.
    let tussen_aanhalingstekens = |t: &str| -> Option<String> {
        let b = t.find('"')?;
        let e = t[b + 1..].find('"')?;
        Some(t[b + 1..b + 1 + e].to_string())
    };
    if (laag.contains("schermvullend") || laag.contains("gemaximaliseerd") || laag.contains("maximaal groot")) && laag.contains("venster") {
        return Ok(Vorm::Los(Soort::Venstergedrag(Gedrag::Schermvullend)));
    }
    if laag.starts_with("vraag bij afsluiten") || laag.starts_with("vraag bij het afsluiten") || laag.starts_with("vraag als ik afsluit") {
        let vraag = tussen_aanhalingstekens(norm).unwrap_or_else(|| "Weet je zeker dat je wilt afsluiten?".to_string());
        return Ok(Vorm::Los(Soort::Venstergedrag(Gedrag::VraagBijAfsluiten(vraag))));
    }
    if ["maar één keer open", "maar een keer open", "maar 1 keer open", "maar één keer tegelijk open", "maar een keer tegelijk open"].iter().any(|z| laag.contains(z)) {
        return Ok(Vorm::Los(Soort::Venstergedrag(Gedrag::MaarEenKeerOpen)));
    }
    if laag.starts_with("geef de knop ") && laag.contains(" de tip ") {
        let p = laag.find(" de tip ").unwrap();
        let knop = norm["geef de knop ".len()..p].trim().to_string();
        let tekst = tussen_aanhalingstekens(&norm[p..]).unwrap_or_else(|| norm[p + " de tip ".len()..].trim().to_string());
        if knop.is_empty() || tekst.is_empty() {
            return Err(fout("Schrijf het zo: Geef de knop Opslaan de tip \"Bewaart je werk\"."));
        }
        return Ok(Vorm::Los(Soort::Venstergedrag(Gedrag::Tip { knop, tekst })));
    }
    if laag.starts_with("maak een reservekopie van") {
        let rest = norm["maak een reservekopie van".len()..].trim();
        let rest = ["het bestand ", "bestand "].iter().find_map(|b| rest.to_lowercase().starts_with(b).then(|| &rest[b.len()..])).unwrap_or(rest);
        if rest.is_empty() {
            return Err(fout("Schrijf het zo: Maak een reservekopie van het bestand notities.txt."));
        }
        let stukken = tekst::splits(rest).map_err(fout)?;
        return Ok(Vorm::Los(Soort::Reservekopie(pad(&stukken).map_err(fout)?)));
    }
    // "Dit is code voor het programma Rekenhulp": zegt bij welk programma dit bestand hoort.
    if laag.starts_with("dit is code voor") || laag.starts_with("dit is een omschrijving voor") || laag.starts_with("dit is de code voor") {
        return Ok(Vorm::Los(Soort::Opmerking(norm.to_string())));
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
        "maak" if begint(&s, &["maak", "de", "tabel"]) && klein(&s, s.len() - 1) == "leeg" && s.len() >= 5 => {
            Ok(Vorm::Los(Soort::Tabel { tabel: rest_tekst(&s[3..s.len() - 1]), actie: TabelActie::Leeg }))
        }
        "voeg" if zoek(&s, &["toe", "aan", "de", "tabel"], 1).is_some() => {
            let t = zoek(&s, &["toe", "aan", "de", "tabel"], 1).unwrap();
            if t + 4 >= s.len() {
                return Err(fout("Aan welke tabel? Schrijf bijvoorbeeld: ... toe aan de tabel Uitgaven."));
            }
            let tabel = rest_tekst(&s[t + 4..]);
            if begint(&s, &["voeg", "de", "kolom"]) || begint(&s, &["voeg", "een", "kolom"]) {
                if t <= 3 {
                    return Err(fout("Welke kolom? Schrijf bijvoorbeeld: Voeg de kolom Bedrag toe aan de tabel Uitgaven."));
                }
                return Ok(Vorm::Los(Soort::Tabel { tabel, actie: TabelActie::Kolom(rest_tekst(&s[3..t])) }));
            }
            let mut b = 1;
            while b < t && matches!(klein(&s, b).as_str(), "de" | "een" | "rij") {
                b += 1;
            }
            if b >= t {
                return Err(fout("Wat moet er in de rij? Schrijf bijvoorbeeld: Voeg de rij \"Brood\" en 2,50 toe aan de tabel Uitgaven."));
            }
            let r = &s[b..t];
            let mut waarden = Vec::new();
            let mut begin = 0;
            for i in 0..=r.len() {
                if i == r.len() || (klein(r, i) == "en" && klein(r, i + 1) != "dan") {
                    if i == begin {
                        return Err(fout("Rond \"en\" mis ik een waarde voor de rij."));
                    }
                    waarden.push(u(&r[begin..i])?);
                    begin = i + 1;
                }
            }
            Ok(Vorm::Los(Soort::Tabel { tabel, actie: TabelActie::Rij(waarden) }))
        }
        "haal" if zoek(&s, &["tabel"], 1).is_some() => {
            let t = zoek(&s, &["tabel"], 1).unwrap();
            if t + 1 >= s.len() || zoek(&s, &["gekozen"], 1).is_none() {
                return Err(fout("Schrijf het zo: Haal de gekozen rij weg uit de tabel Uitgaven."));
            }
            Ok(Vorm::Los(Soort::Tabel { tabel: rest_tekst(&s[t + 1..]), actie: TabelActie::HaalGekozenWeg }))
        }
        "sorteer" => {
            let (Some(t), Some(k)) = (zoek(&s, &["tabel"], 1), zoek(&s, &["kolom"], 2)) else {
                return Err(fout("Schrijf het zo: Sorteer de tabel Uitgaven op de kolom Bedrag."));
            };
            let mut eind_t = k;
            while eind_t > t + 1 && matches!(klein(&s, eind_t - 1).as_str(), "op" | "de") {
                eind_t -= 1;
            }
            if eind_t <= t + 1 || k + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Sorteer de tabel Uitgaven op de kolom Bedrag."));
            }
            Ok(Vorm::Los(Soort::Tabel { tabel: rest_tekst(&s[t + 1..eind_t]), actie: TabelActie::Sorteer(rest_tekst(&s[k + 1..])) }))
        }
        "sla" | "lees" if zoek(&s, &["tabel"], 1).is_some() && zoek(&s, &["bestand"], 2).is_some() => {
            let t = zoek(&s, &["tabel"], 1).unwrap();
            let b = zoek(&s, &["bestand"], t).unwrap();
            let mut eind_t = b;
            while eind_t > t + 1 && matches!(klein(&s, eind_t - 1).as_str(), "op" | "in" | "het" | "uit" | "naar") {
                eind_t -= 1;
            }
            if eind_t <= t + 1 || b + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Sla de tabel Uitgaven op in het bestand uitgaven.csv. Of: Lees de tabel Uitgaven uit het bestand uitgaven.csv."));
            }
            let p = pad(&s[b + 1..]).map_err(fout)?;
            let actie = if w0 == "sla" { TabelActie::Opslaan(p) } else { TabelActie::Inlezen(p) };
            Ok(Vorm::Los(Soort::Tabel { tabel: rest_tekst(&s[t + 1..eind_t]), actie }))
        }
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
        "geef" | "maak" if zoek(&s, &["opdrachtvak"], 1).is_some() => Ok(Vorm::Los(Soort::Opdrachtvak)),
        "maak" if matches!(klein(&s, 2).as_str(), "qr-code" | "qrcode") || (klein(&s, 2) == "qr" && klein(&s, 3) == "code") => {
            // Maak een QR-code van "https://..." in het bestand link.png
            let vb = "Schrijf het zo: Maak een QR-code van \"https://www.voorbeeld.nl\" in het bestand link.png";
            let b = if klein(&s, 2) == "qr" { 4 } else { 3 };
            if !matches!(klein(&s, b).as_str(), "van" | "met") {
                return Err(fout(vb));
            }
            let Some(p) = zoek(&s, &["bestand"], b + 1) else { return Err(fout(vb)) };
            let mut tot = p;
            while tot > b + 1 && matches!(klein(&s, tot - 1).as_str(), "het" | "in" | "naar" | "als") {
                tot -= 1;
            }
            if tot <= b + 1 || p + 1 >= s.len() {
                return Err(fout(vb));
            }
            Ok(Vorm::Los(Soort::QrCode { waarde: tekst_of_uitdrukking(&s[b + 1..tot]).map_err(fout)?, pad: pad(&s[p + 1..]).map_err(fout)? }))
        }
        "maak" if zoek(&s, &["werkbalk"], 1).is_some_and(|p| p <= 2) => {
            // Maak een werkbalk met de knoppen Nieuw, Openen en Opslaan
            let vb = "Schrijf het zo: Maak een werkbalk met de knoppen Nieuw, Openen en Opslaan.";
            let Some(k) = (2..s.len()).find(|&i| matches!(klein(&s, i).as_str(), "knoppen" | "knop")) else {
                return Err(fout(vb));
            };
            let woord = klein(&s, k);
            let laag = norm.to_lowercase();
            let rest = match laag.find(&format!("{} ", woord)) {
                Some(p) => norm[p + woord.len() + 1..].trim_end_matches(['.', '!']).to_string(),
                None => bron_tekst(&s[k + 1..]),
            };
            let namen: Vec<String> = rest
                .split(',')
                .flat_map(|d| d.split(" en "))
                .map(|n| n.trim().trim_matches('"').trim().to_string())
                .filter(|n| !n.is_empty())
                .collect();
            if namen.is_empty() {
                return Err(fout(vb));
            }
            Ok(Vorm::Los(Soort::Werkbalk(namen)))
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
                "tabel" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Tabel,
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "de tabel", "Maak een tabel met als naam Uitgaven")?,
                })),
                "uitklaplijst" | "combobox" | "keuzemenu" => Ok(Vorm::Los(Soort::MaakOnderdeel {
                    soort: OnderdeelSoort::Uitklaplijst,
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "de uitklaplijst", "Maak een uitklaplijst met als naam Kleur")?,
                })),
                "tabblad" | "tab" => Ok(Vorm::Los(Soort::MaakTabblad {
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "het tabblad", "Maak een tabblad met als naam Algemeen")?,
                })),
                "afbeelding" | "plaatje" | "foto" => {
                    let voorbeeld = "Schrijf het zo: Maak een afbeelding van het bestand hond.jpg met de beschrijving Mijn hond in het park.";
                    let b = zoek(&s, &["bestand"], i + 1).ok_or_else(|| fout(voorbeeld))?;
                    let Some(m) = zoek(&s, &["beschrijving"], b + 1) else {
                        return Err(fout(format!(
                            "Welke beschrijving moet de afbeelding krijgen? De schermlezer leest die voor. Schrijf erachter: met de beschrijving ... ({})",
                            voorbeeld
                        )));
                    };
                    let mut eind_pad = m;
                    while eind_pad > b + 1 && matches!(klein(&s, eind_pad - 1).as_str(), "met" | "de" | "als") {
                        eind_pad -= 1;
                    }
                    if eind_pad <= b + 1 || m + 1 >= s.len() {
                        return Err(fout(voorbeeld));
                    }
                    Ok(Vorm::Los(Soort::MaakAfbeelding { pad: rest_tekst(&s[b + 1..eind_pad]), beschrijving: rest_tekst(&s[m + 1..]) }))
                }
                "menu" => Ok(Vorm::Los(Soort::MaakMenu {
                    naam: met_als_titel(&s, i + 1, &["naam", "titel"], "het menu", "Maak een menu met als naam Bestand")?,
                })),
                "menuregel" | "menu-item" | "menuitem" => Ok(Vorm::Los(Soort::MaakMenuregel {
                    naam: met_als_titel(&s, i + 1, &["naam", "titel", "tekst"], "de menuregel", "Maak een menuregel met als naam Opslaan")?,
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
        "zeg" | "laat" if w0 == "zeg" || begint(&s, &["laat", "de", "schermlezer", "zeggen"]) => {
            let begin = if w0 == "zeg" { 1 } else { 4 };
            if begin >= s.len() {
                return Err(fout("Wat moet de schermlezer zeggen? Schrijf bijvoorbeeld: Zeg Het bestand is opgeslagen."));
            }
            Ok(Vorm::Los(Soort::Zeg(spreektekst(&s[begin..])?)))
        }
        "lees" if klein(&s, 1) == "voor" || klein(&s, s.len() - 1) == "voor" => {
            let r = if klein(&s, 1) == "voor" { &s[2..] } else { &s[1..s.len() - 1] };
            if r.is_empty() {
                return Err(fout("Wat moet ik voorlezen? Schrijf bijvoorbeeld: Lees voor \"Goedemorgen\"."));
            }
            Ok(Vorm::Los(Soort::LeesVoor(spreektekst(r)?)))
        }
        "stop" if zoek(&s, &["voorlezen"], 1).is_some() => Ok(Vorm::Los(Soort::StopVoorlezen)),
        "als" if matches!(klein(&s, 1).as_str(), "ik" | "je" | "iemand") && matches!(klein(&s, 2).as_str(), "zeg" | "typ" | "zegt" | "typt") => {
            // Als ik zeg "maak het donker"
            let zin = match s.get(3) {
                Some(Stuk::Tekst(t)) if s.len() == 4 => t.clone(),
                _ => return Err(fout("Zet de zin tussen aanhalingstekens. Schrijf bijvoorbeeld: Als ik zeg \"maak het donker\"")),
            };
            if zin.trim().is_empty() {
                return Err(fout("Welke zin? Schrijf bijvoorbeeld: Als ik zeg \"maak het donker\""));
            }
            Ok(Vorm::OpenKnop(format!("zeg: {}", zin.trim())))
        }
        "wacht" if s.len() >= 2 => {
            let mut eind = s.len();
            if matches!(klein(&s, eind - 1).as_str(), "seconden" | "seconde" | "sec") {
                eind -= 1;
            } else {
                return Err(fout("Schrijf het zo: Wacht 3 seconden."));
            }
            if eind <= 1 {
                return Err(fout("Hoeveel seconden? Schrijf het zo: Wacht 3 seconden."));
            }
            Ok(Vorm::Los(Soort::Wacht(operand(&s[1..eind], false).map_err(fout)?)))
        }
        "start" | "open" if zoek(&s, &["programma"], 1).is_some_and(|p| p <= 2) => {
            let p = zoek(&s, &["programma"], 1).unwrap();
            if p + 1 >= s.len() {
                return Err(fout("Welk programma? Schrijf het zo: Start het programma notepad.exe."));
            }
            Ok(Vorm::Los(Soort::StartProgramma(pad(&s[p + 1..]).map_err(fout)?)))
        }
        "open" if s.len() >= 2 => {
            let mut b = 1;
            if matches!(klein(&s, b).as_str(), "het" | "de") {
                b += 1;
            }
            if matches!(klein(&s, b).as_str(), "bestand" | "map" | "webpagina" | "website") {
                b += 1;
            }
            if b >= s.len() {
                return Err(fout("Wat moet ik openen? Schrijf het zo: Open het bestand brief.docx."));
            }
            Ok(Vorm::Los(Soort::OpenBestand(pad(&s[b..]).map_err(fout)?)))
        }
        "begin" | "start" if zoek(&s, &["opnemen"], 1).is_some() || zoek(&s, &["opname"], 1).is_some() => Ok(Vorm::Los(Soort::BeginOpname)),
        "stop" if zoek(&s, &["opnemen"], 1).is_some() || zoek(&s, &["opname"], 1).is_some() => {
            let p = zoek(&s, &["bestand"], 1).or_else(|| (1..s.len()).rev().find(|&i| klein(&s, i) == "als"));
            let Some(p) = p.filter(|p| p + 1 < s.len()) else {
                return Err(fout("Waar moet de opname komen? Schrijf bijvoorbeeld: Stop met opnemen en sla het op in het bestand opname.wav."));
            };
            Ok(Vorm::Los(Soort::StopOpname(pad(&s[p + 1..]).map_err(fout)?)))
        }
        "ga" | "kies" if zoek(&s, &["tabblad"], 1).is_some() => {
            let t = zoek(&s, &["tabblad"], 1).unwrap();
            if t + 1 >= s.len() {
                return Err(fout("Welk tabblad? Schrijf bijvoorbeeld: Ga naar het tabblad Geluid."));
            }
            Ok(Vorm::Los(Soort::KiesTabblad { naam: rest_tekst(&s[t + 1..]) }))
        }
        "zet" if zoek(&s, &["volume"], 1).map_or(false, |v| s[1..v].iter().all(|x| matches!(x.klein().as_deref(), Some("het" | "de")))) => {
            let Some(p) = zoek(&s, &["op"], 2) else {
                return Err(fout("Schrijf het zo: Zet het volume op 50. (Van 0 tot 100.)"));
            };
            let mut eind = s.len();
            if matches!(klein(&s, eind - 1).as_str(), "procent" | "%") {
                eind -= 1;
            }
            if p + 1 >= eind {
                return Err(fout("Op hoeveel? Schrijf bijvoorbeeld: Zet het volume op 50."));
            }
            Ok(Vorm::Los(Soort::Volume(u(&s[p + 1..eind])?)))
        }
        "kopieer" | "verplaats" | "gooi" | "verwijder" => {
            let soort = match w0.as_str() {
                "kopieer" => 0u8,
                "verplaats" => 1,
                _ => 2,
            };
            let voorbeeld = match soort {
                0 => "Kopieer het bestand brief.txt naar Reserve/brief.txt",
                1 => "Verplaats het bestand brief.txt naar Oud/brief.txt",
                _ => "Gooi het bestand brief.txt weg",
            };
            let mut b = 1;
            while b < s.len() && matches!(klein(&s, b).as_str(), "het" | "de" | "bestand" | "map") {
                b += 1;
            }
            if soort == 2 {
                let mut e = s.len();
                if klein(&s, e - 1) == "weg" {
                    e -= 1;
                }
                if b >= e {
                    return Err(fout(format!("Welk bestand? Schrijf bijvoorbeeld: {}.", voorbeeld)));
                }
                return Ok(Vorm::Los(Soort::BestandActie { soort, van: pad(&s[b..e]).map_err(fout)?, naar: None }));
            }
            let Some(n) = zoek(&s, &["naar"], b + 1) else {
                return Err(fout(format!("Waarheen? Schrijf bijvoorbeeld: {}.", voorbeeld)));
            };
            if n + 1 >= s.len() {
                return Err(fout(format!("Waarheen? Schrijf bijvoorbeeld: {}.", voorbeeld)));
            }
            let mut d = n + 1;
            while d < s.len() - 1 && matches!(klein(&s, d).as_str(), "het" | "de" | "bestand" | "map") {
                d += 1;
            }
            Ok(Vorm::Los(Soort::BestandActie { soort, van: pad(&s[b..n]).map_err(fout)?, naar: Some(pad(&s[d..]).map_err(fout)?) }))
        }
        "het" if klein(&s, 1) == "pictogram" || klein(&s, 1) == "icoon" => {
            let Some(p) = zoek(&s, &["bestand"], 2) else {
                return Err(fout("Schrijf het zo: Het pictogram staat in het bestand logo.ico."));
            };
            if p + 1 >= s.len() {
                return Err(fout("Welk bestand? Schrijf bijvoorbeeld: Het pictogram staat in het bestand logo.ico."));
            }
            Ok(Vorm::Los(Soort::Pictogram(rest_tekst(&s[p + 1..]))))
        }
        "toon" => {
            let r = &s[1..];
            if r.is_empty() {
                return Err(fout("Wat moet ik tonen? Schrijf bijvoorbeeld: Toon de tekst Hallo."));
            }
            if onderdeel_operand(r).is_some() || matches!(ingebouwd(r), Ok(Some(_))) {
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
        "de" if klein(&s, 1) == "privacyverklaring" => {
            let Some(p) = zoek(&s, &["bestand"], 2) else {
                return Err(fout("Schrijf het zo: De privacyverklaring staat in het bestand Privacy.txt. Of: In de privacyverklaring staat: (jouw tekst)."));
            };
            if p + 1 >= s.len() {
                return Err(fout("Welk bestand? Schrijf bijvoorbeeld: De privacyverklaring staat in het bestand Privacy.txt."));
            }
            Ok(Vorm::Los(Soort::Installatie(InstallatieZin::PrivacyBestand(rest_tekst(&s[p + 1..])))))
        }
        "onderteken" | "signeer" => {
            // Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel
            let Some(p) = zoek(&s, &["naam"], 1) else {
                return Err(fout("Schrijf het zo: Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel."));
            };
            let mut b = p + 1;
            if klein(&s, b) == "van" {
                b += 1;
            }
            if b >= s.len() {
                return Err(fout("Op welke naam? Schrijf bijvoorbeeld: Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel."));
            }
            Ok(Vorm::Los(Soort::Installatie(InstallatieZin::Onderteken(rest_tekst(&s[b..])))))
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
        "schrijf" if (1..s.len()).any(|i| is_documentwoord(&klein(&s, i))) => {
            let d = (1..s.len()).find(|&i| is_documentwoord(&klein(&s, i))).unwrap();
            let soort = if klein(&s, d).starts_with("pdf") { 1 } else { 0 };
            let mut tot = d;
            while tot > 1 && matches!(klein(&s, tot - 1).as_str(), "het" | "in" | "naar" | "een") {
                tot -= 1;
            }
            if tot <= 1 || d + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Schrijf \"Beste Piet\" in het Word-bestand brief.docx. Of: ... in het PDF-bestand brief.pdf."));
            }
            Ok(Vorm::Los(Soort::Document { soort, waarde: u(&s[1..tot])?, pad: pad(&s[d + 1..]).map_err(fout)? }))
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
            // Waar het geluid klinkt: links, rechts, achter, voor, boven, onder (ook samen).
            let (mut h, mut d, mut v, mut midden) = (0u8, 0u8, 0u8, false);
            loop {
                if eind <= g + 2 {
                    break;
                }
                match klein(&s, eind - 1).as_str() {
                    "me" | "mij" | "je" | "jou" | "en" | "van" | "naar" | "aan" | "de" | "het" | "in" | "op" | "uit" | "kant" | "," => {}
                    "links" | "linkerkant" => h = 1,
                    "rechts" | "rechterkant" => h = 2,
                    "achter" | "achteren" | "achterkant" => d = 1,
                    "voor" | "voren" | "voorkant" => d = 2,
                    "boven" | "bovenkant" | "omhoog" => v = 1,
                    "onder" | "onderkant" | "beneden" | "omlaag" => v = 2,
                    "midden" => midden = true,
                    "linksachter" => (h, d) = (1, 1),
                    "rechtsachter" => (h, d) = (2, 1),
                    "linksvoor" => (h, d) = (1, 2),
                    "rechtsvoor" => (h, d) = (2, 2),
                    "linksboven" => (h, v) = (1, 1),
                    "rechtsboven" => (h, v) = (2, 1),
                    "linksonder" => (h, v) = (1, 2),
                    "rechtsonder" => (h, v) = (2, 2),
                    _ => break,
                }
                eind -= 1;
            }
            // Alleen kleine woorden weggehaald, maar geen richting? Dan horen ze bij de naam.
            let mut richting = h + 3 * d + 9 * v;
            if richting == 0 && midden {
                richting = 3 * 2; // midden = voor je
            }
            if richting == 0 {
                eind = s.len();
                if wachten {
                    eind -= 2;
                }
            }
            if klein(&s, eind - 1) == "af" {
                eind -= 1;
            }
            if g + 1 >= eind {
                return Err(fout("Welk geluid? Schrijf bijvoorbeeld: Speel het geluid deurbel.wav."));
            }
            Ok(Vorm::Los(Soort::SpeelGeluid { pad: pad(&s[g + 1..eind]).map_err(fout)?, wachten, richting }))
        }
        "stop" if zoek(&s, &["geluid"], 1).is_some() => Ok(Vorm::Los(Soort::StopGeluid)),
        "pak" if zoek(&s, &["zipbestand"], 1).is_some() => {
            let voorbeeld = "Schrijf het zo: Pak de map Foto's in tot het zipbestand Foto's.zip. Of: Pak het zipbestand Foto's.zip uit in de map Foto's.";
            let z = zoek(&s, &["zipbestand"], 1).unwrap();
            let m = zoek(&s, &["map"], 1).ok_or_else(|| fout(voorbeeld))?;
            if m < z {
                // Pak de map X in tot het zipbestand Y
                let i = (m + 1..z).find(|&i| klein(&s, i) == "in").ok_or_else(|| fout(voorbeeld))?;
                if i == m + 1 || z + 1 >= s.len() {
                    return Err(fout(voorbeeld));
                }
                Ok(Vorm::Los(Soort::Zip { inpakken: true, map: pad(&s[m + 1..i]).map_err(fout)?, zip: pad(&s[z + 1..]).map_err(fout)? }))
            } else {
                // Pak het zipbestand Y uit in de map X
                let u = (z + 1..m).find(|&i| klein(&s, i) == "uit").ok_or_else(|| fout(voorbeeld))?;
                if u == z + 1 || m + 1 >= s.len() {
                    return Err(fout(voorbeeld));
                }
                Ok(Vorm::Los(Soort::Zip { inpakken: false, map: pad(&s[m + 1..]).map_err(fout)?, zip: pad(&s[z + 1..u]).map_err(fout)? }))
            }
        }
        "geef" if zoek(&s, &["sneltoets"], 2).is_some() => {
            let k = zoek(&s, &["knop"], 1)
                .or_else(|| zoek(&s, &["menuregel"], 1))
                .ok_or_else(|| fout("Schrijf het zo: Geef de knop Welkom de sneltoets Ctrl+W."))?;
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
        "verstuur" | "stuur" => {
            let Some(a) = zoek(&s, &["adres"], 2) else {
                return Err(fout("Schrijf het zo: Verstuur \"Hallo\" naar het adres https://voorbeeld.nl/ontvang."));
            };
            let mut tot = a;
            while tot > 1 && matches!(klein(&s, tot - 1).as_str(), "het" | "naar" | "internetadres") {
                tot -= 1;
            }
            if tot <= 1 || a + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Verstuur \"Hallo\" naar het adres https://voorbeeld.nl/ontvang."));
            }
            Ok(Vorm::Los(Soort::Verstuur { waarde: u(&s[1..tot])?, adres: pad(&s[a + 1..]).map_err(fout)? }))
        }
        "download" => {
            let (Some(a), Some(n)) = (zoek(&s, &["adres"], 1), (2..s.len()).rev().find(|&i| klein(&s, i) == "naar")) else {
                return Err(fout("Schrijf het zo: Download het bestand van het adres https://voorbeeld.nl/a.mp3 naar muziek.mp3."));
            };
            if n <= a + 1 || n + 1 >= s.len() {
                return Err(fout("Schrijf het zo: Download het bestand van het adres https://voorbeeld.nl/a.mp3 naar muziek.mp3."));
            }
            Ok(Vorm::Los(Soort::Download { adres: pad(&s[a + 1..n]).map_err(fout)?, pad: pad(&s[n + 1..]).map_err(fout)? }))
        }
        "zoek" if zoek(&s, &["updates"], 1).is_some() || zoek(&s, &["update"], 1).is_some() => Ok(Vorm::Los(Soort::ZoekUpdates)),
        "de" if begint(&s, &["de", "updates", "staan", "op"]) => {
            let Some(a) = zoek(&s, &["adres"], 4) else {
                return Err(fout("Schrijf het zo: De updates staan op het adres https://www.dropbox.com/... (het adres van je versiebestand)."));
            };
            if a + 1 >= s.len() {
                return Err(fout("Welk adres? Schrijf bijvoorbeeld: De updates staan op het adres https://www.dropbox.com/..."));
            }
            Ok(Vorm::Los(Soort::Installatie(InstallatieZin::UpdateAdres(rest_tekst(&s[a + 1..])))))
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
        "als" if matches!(klein(&s, 1).as_str(), "ik" | "je" | "iemand") && zoek(&s, &["menuregel"], 2).is_some() => {
            // Als ik de menuregel Opslaan kies
            let m = zoek(&s, &["menuregel"], 2).unwrap();
            let mut eind = s.len();
            if matches!(klein(&s, eind - 1).as_str(), "kies" | "kiest" | "druk" | "drukt" | "klik" | "klikt") {
                eind -= 1;
            }
            if eind <= m + 1 {
                return Err(fout("Welke menuregel? Schrijf bijvoorbeeld: Als ik de menuregel Opslaan kies."));
            }
            Ok(Vorm::OpenKnop(rest_tekst(&s[m + 1..eind])))
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
                ["opdracht"] | ["zin"] => Ok(Vorm::Einde(Blok::Knop)),
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
            KaderSoort::Stukje(naam, invoer) => Soort::Stukje { naam, invoer, blok: self.dan, einde, rust: None },
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
/// Probeert een onbekende zin op een andere manier: met jouw eigen woorden, of als Engelse zin.
fn anders_lezen(norm: &str, eigen: &[(String, String)]) -> Option<Vorm> {
    for (van, naar) in eigen {
        let nieuw = crate::engels::vervang_woorden(norm, &[(van.as_str(), naar.as_str())]);
        if nieuw != norm {
            if let Ok(v) = ontleed_regel(&tekst::normaliseer(&nieuw)) {
                return Some(v);
            }
        }
    }
    let nl = crate::engels::naar_nederlands(norm)?;
    ontleed_regel(&tekst::normaliseer(&nl)).ok()
}

/// Zet eigen zinnen om in wat ze betekenen. Elke regel houdt het regelnummer waar de zin stond.
/// Rust-stukjes: een stukje met eigen Rust-code. Die regels worden niet als zinnen gelezen.
fn rust_stukjes(bron: &str, meldingen: &mut Vec<Melding>) -> (Vec<Opdracht>, Vec<usize>) {
    let regels: Vec<&str> = bron.lines().collect();
    let mut uit = Vec::new();
    let mut bezet = Vec::new();
    let mut i = 0;
    while i < regels.len() {
        let norm = tekst::normaliseer(regels[i]);
        let laag = norm.to_lowercase();
        if !(laag.starts_with("maak een rust-stukje") || laag.starts_with("maak een ruststukje")) {
            i += 1;
            continue;
        }
        let nr = i + 1;
        let kop = format!("Maak een stukje{}", &norm[norm.to_lowercase().find("stukje").unwrap() + "stukje".len()..]);
        let mut code = Vec::new();
        let mut j = i + 1;
        let mut einde = None;
        while j < regels.len() {
            let l = tekst::normaliseer(regels[j]).to_lowercase();
            if matches!(l.as_str(), "einde van het rust-stukje" | "einde rust-stukje" | "einde van het ruststukje") {
                einde = Some(j + 1);
                break;
            }
            code.push(regels[j].to_string());
            j += 1;
        }
        for k in nr..=einde.unwrap_or(regels.len()) {
            bezet.push(k);
        }
        match (ontleed_regel(&kop), einde) {
            (_, None) => meldingen.push(Melding::fout(nr, "Dit Rust-stukje wordt niet gesloten. Zet eronder: Einde van het Rust-stukje.")),
            (Ok(Vorm::OpenStukje(naam, invoer)), Some(e)) => uit.push(Opdracht {
                regel: nr,
                soort: Soort::Stukje { naam, invoer, blok: Vec::new(), einde: e, rust: Some(code.join("\n")) },
            }),
            (Err(rf), _) => meldingen.push(Melding::fout(nr, rf.tekst)),
            _ => meldingen.push(Melding::fout(nr, "Schrijf het zo: Maak een Rust-stukje met als naam Dubbel, met als invoer x")),
        }
        i = einde.unwrap_or(regels.len());
    }
    (uit, bezet)
}

fn breid_uit(bron: &str, eigen: &[(String, String)], bibliotheek: &str, meldingen: &mut Vec<Melding>, rust_bezet: &[usize]) -> Vec<(usize, String)> {
    let (mut zinnen, eigen_meldingen, mut bezet) = crate::eigen_zinnen::verzamel(bron);
    bezet.extend_from_slice(rust_bezet);
    meldingen.extend(eigen_meldingen);
    let (bib, _, _) = crate::eigen_zinnen::verzamel(bibliotheek);
    zinnen.extend(bib);
    let mut uit = Vec::new();
    fn verwerk(nr: usize, regel: &str, zinnen: &[crate::eigen_zinnen::Sjabloon], eigen: &[(String, String)], diepte: usize, uit: &mut Vec<(usize, String)>, meldingen: &mut Vec<Melding>) {
        let norm = tekst::normaliseer(regel);
        // Een eigen zin geldt als de regel geen goede vaste zin is.
        let onbekend = ontleed_regel(&norm).is_err() && anders_lezen(&norm, eigen).is_none();
        if onbekend && !zinnen.is_empty() {
            if let Some(betekenis) = crate::eigen_zinnen::vervang(regel, zinnen) {
                if diepte >= 10 {
                    meldingen.push(Melding::fout(nr, "Deze eigen zin gebruikt zichzelf steeds weer (meer dan tien keer diep). Ik ben gestopt."));
                    return;
                }
                for b in betekenis {
                    verwerk(nr, &b, zinnen, eigen, diepte + 1, uit, meldingen);
                }
                return;
            }
        }
        uit.push((nr, regel.to_string()));
    }
    for (i, ruw) in bron.lines().enumerate() {
        if bezet.contains(&(i + 1)) {
            continue;
        }
        verwerk(i + 1, ruw, &zinnen, eigen, 0, &mut uit, meldingen);
    }
    uit
}

pub(crate) fn lees_programma_eigen(bron: &str, eigen: &[(String, String)], bibliotheek: &str) -> (Programma, Vec<Melding>) {
    let mut meldingen = Vec::new();
    let mut stapel = vec![Kader::nieuw(KaderSoort::Hoofd, 0)];
    let (rust, rust_bezet) = rust_stukjes(bron, &mut meldingen);
    let regels = breid_uit(bron, eigen, bibliotheek, &mut meldingen, &rust_bezet);
    let aantal_regels = bron.lines().count();

    for (nr, ruw) in regels.iter() {
        let nr = *nr;
        let ruw = ruw.as_str();
        let norm = tekst::normaliseer(ruw);
        let gelezen = match ontleed_regel(&norm) {
            Err(rf) if rf.onbekend => anders_lezen(&norm, eigen).ok_or(rf),
            ander => ander,
        };
        match gelezen {
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

    let laatste = aantal_regels;
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
    programma.opdrachten.extend(rust);
    programma.opdrachten.sort_by_key(|o| o.regel);
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
