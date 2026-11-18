//! Opdrachten in gewone zinnen voor de editor: "maak er een exe van", "open mijn programma Begroeting",
//! "ga maar verder". Geen AI: elke functie heeft vaste woorden. Een zin past bij een functie als
//! er uit elk groepje woorden minstens één in staat. Past hij bij meer functies, of bij geen,
//! dan wordt er gevraagd. Een tikfout wordt alleen verbeterd na "ja".

use crate::tekst;

/// Wat de editor kan doen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Functie {
    Nieuw,
    Openen,
    OpenExe,
    Onlangs,
    Verder,
    Opslaan,
    OpslaanAls,
    Controleren,
    Uitvoeren,
    ProbeerRegel,
    Beschrijf,
    Scherm,
    RustCode,
    Engels,
    BouwExe,
    Installatie,
    Certificaat,
    WelkeRegel,
    NaarRegel,
    Meldingen,
    Ongedaan,
    Handleiding,
    EigenWoorden,
    EigenZinnen,
    EigenOpdrachten,
    Over,
    Omschrijving,
    VersieOmhoog,
    Wensenlijst,
    WatKanIkZeggen,
    Afsluiten,
}

pub struct Actie {
    pub functie: Functie,
    /// Hoe de functie heet, zoals in het menu.
    pub naam: &'static str,
    /// Sneltoets, of leeg.
    pub toets: &'static str,
    /// Voorbeelden van wat je kunt zeggen.
    pub voorbeelden: &'static [&'static str],
    /// Uit elk groepje moet een woord in de zin staan. Een woord met * aan het eind: alles wat zo begint.
    nodig: &'static [&'static [&'static str]],
    /// Als een van deze woorden erin staat, is het deze functie niet.
    niet: &'static [&'static str],
}

const fn a(
    functie: Functie,
    naam: &'static str,
    toets: &'static str,
    voorbeelden: &'static [&'static str],
    nodig: &'static [&'static [&'static str]],
    niet: &'static [&'static str],
) -> Actie {
    Actie { functie, naam, toets, voorbeelden, nodig, niet }
}

use Functie::*;


pub const ACTIES: &[Actie] = &[
    a(Nieuw, "Nieuw programma beginnen", "Control N", &["nieuw programma", "begin een nieuw programma", "maak een nieuw programma"], &[&["nieuw", "nieuwe", "leeg", "lege"]], &["installatie*", "exe", "zin", "zinnen", "woord*"]),
    a(Openen, "Een programma openen", "Control O", &["open mijn programma Begroeting", "open de code van mijn programma", "laat mijn programma Rekenhulp zien"], &[&["open*", "laad", "pak", "haal"]], &["exe", "onlangs", "laatst*", "recent*", "handleiding", "woorden", "zinnen", "meldingen", "opdrachten", "wens*"]),
    a(OpenExe, "De zinnen uit een gebouwd programma (.exe) halen", "", &["open de code uit een exe", "haal mijn zinnen uit het programma"], &[&["open*", "haal", "pak", "terug"], &["exe"]], &[]),
    a(Onlangs, "Onlangs geopende programma's", "", &["wat heb ik onlangs geopend", "mijn laatste programma's"], &[&["onlangs", "laatst*", "recent*", "vorige"]], &["regel", "fout*", "melding*"]),
    a(Verder, "Verder gaan waar je gebleven was", "", &["ga maar verder", "ga verder waar ik gebleven was", "doe maar verder"], &[&["verder", "gebleven", "doorgaan", "ga door"]], &[]),
    a(Opslaan, "Opslaan", "Control S", &["sla op", "bewaar mijn programma", "opslaan"], &[&["sla", "opslaan", "bewaar*", "save"]], &["andere", "als"]),
    a(OpslaanAls, "Opslaan met een andere naam", "Control Shift S", &["sla op als", "bewaar met een andere naam"], &[&["sla", "opslaan", "bewaar*"], &["als", "andere", "nieuwe"]], &[]),
    a(Controleren, "Je zinnen controleren", "F7", &["controleer mijn zinnen", "kijk of het klopt", "zijn er fouten", "test mijn programma"], &[&["controleer*", "controle", "klopt", "kloppen", "fout*", "test*", "check*", "nakijken", "kijk"]], &["regel", "exe", "installatie*", "certificaat"]),
    a(Uitvoeren, "Je programma uitvoeren", "F5", &["voer het uit", "start mijn programma", "laat het programma werken", "probeer het"], &[&["uitvoeren", "voer", "start*", "draai", "run", "werken", "probeer", "proberen"]], &["regel", "zin", "nieuw*", "exe", "installatie*"]),
    a(ProbeerRegel, "Alleen deze regel proberen", "F9", &["probeer deze regel", "test alleen deze zin"], &[&["probeer", "test*", "uitvoeren", "voer"], &["regel", "zin"]], &["welke", "naar"]),
    a(Beschrijf, "Laten zien wat er van je zinnen gemaakt wordt", "F6", &["snap je wat ik bedoel", "heb je begrepen wat ik wil", "wat maak je van mijn zinnen", "beschrijf mijn programma"], &[&["snap*", "begrijp*", "begrepen", "beschrijf", "beschrijf je", "bedoel", "uitleg"]], &["scherm", "venster", "handleiding"]),
    a(Scherm, "Vertellen wat er op het scherm komt", "Control F6", &["wat komt er op het scherm", "vertel wat er in het venster staat"], &[&["scherm", "venster", "beeld"]], &[]),
    a(RustCode, "De Rust-code laten zien", "Shift F6", &["laat de rust code zien", "welke rust code maak je"], &[&["rust", "rustcode", "broncode"]], &["stukje*", "exe", "installatie*"]),
    a(Engels, "Je programma omzetten naar het Engels", "", &["zet mijn programma om naar het Engels", "vertaal naar Engels"], &[&["engels", "english", "vertaal*"]], &[]),
    a(BouwExe, "Een exe-bestand bouwen", "F8", &["maak er een exe van", "bouw het programma", "compileer het", "ik wil een echt programma"], &[&["exe", "exe-bestand", "compileer*", "compileren", "bouw*", "echt"]], &["installatie*", "open*", "haal", "terug"]),
    a(Installatie, "Een installatiebestand maken", "Control F8", &["maak het installatiebestand", "geef mij het installatiebestand", "maak een installatie", "maak een setup"], &[&["installatie*", "installeer*", "installer", "setup"]], &[]),
    a(Certificaat, "Mijn certificaat vertrouwen op deze computer", "", &["vertrouw mijn certificaat", "onderteken"], &[&["certificaat", "vertrouw*", "onderteken*", "signeer*", "handtekening"]], &[]),
    a(WelkeRegel, "Zeggen op welke regel je staat", "Control L", &["op welke regel sta ik", "waar ben ik"], &[&["welke", "waar"], &["regel", "ben", "sta"]], &[]),
    a(NaarRegel, "Naar een regel gaan", "", &["ga naar regel 12", "spring naar regel 3"], &[&["ga", "naar", "spring"], &["regel"]], &["welke"]),
    a(Meldingen, "Naar de meldingen gaan", "F4", &["ga naar de meldingen", "wat is er fout", "lees de fouten voor"], &[&["melding*", "fout*"]], &["controleer*", "zijn"]),
    a(Ongedaan, "Ongedaan maken", "Control Z", &["maak ongedaan", "zet het terug"], &[&["ongedaan", "terugzetten", "undo", "herstel"]], &[]),
    a(Handleiding, "De handleiding openen", "F1", &["open de handleiding", "help", "hoe werkt het"], &[&["handleiding", "help", "hulp", "hoe werkt"]], &[]),
    a(EigenWoorden, "Mijn eigen woorden openen", "", &["open mijn eigen woorden"], &[&["eigen"], &["woord", "woorden"]], &[]),
    a(EigenZinnen, "Mijn eigen zinnen openen", "", &["open mijn eigen zinnen", "ik wil een eigen zin maken"], &[&["eigen", "nieuwe"], &["zin", "zinnen"]], &[]),
    a(EigenOpdrachten, "Mijn eigen opdrachten openen (andere manieren om iets te zeggen)", "", &["open mijn eigen opdrachten", "ik wil een nieuwe opdracht leren"], &[&["eigen", "leren", "leer", "nieuwe"], &["opdracht", "opdrachten", "commando*"]], &[]),
    a(Omschrijving, "Een programma maken van een omschrijving in gewone taal", "Control Shift E", &["maak een programma van mijn omschrijving", "zet mijn omschrijving om in een programma"], &[&["omschrijving", "beschrijving", "omschreven", "beschreven"]], &["installatie*", "website"]),
    a(Over, "Over Jaap's Programmeertaal (versie)", "", &["welke versie is dit", "over het programma"], &[&["versie", "over"]], &["installatie*", "omhoog", "ophogen", "hoger", "verhoog*", "nieuwe", "volgende", "moet", "worden", "wordt", "versienummer"]),
    a(VersieOmhoog, "Het versienummer van je programma ophogen", "", &["verhoog het versienummer", "maak een nieuwe versie", "versie omhoog"], &[&["versie", "versienummer"], &["omhoog", "ophogen", "hoger", "verhoog*", "nieuwe", "volgende"]], &[]),
    a(Wensenlijst, "Mijn wensenlijst openen (wat de taal nog niet kan)", "", &["open mijn wensenlijst", "wat staat er op mijn wensenlijst", "wat moet er nog gebeuren"], &[&["wensenlijst", "wensen", "wens", "lijstje", "nog gebeuren", "rest"]], &[]),
    a(WatKanIkZeggen, "Zoeken in wat je kunt zeggen", "Control F", &["wat kan ik zeggen", "zoek een opdracht", "welke opdrachten zijn er"], &[&["zeggen", "opdrachten", "commando*", "zoek*", "lijst"]], &[]),
    a(Afsluiten, "Afsluiten", "Alt F4", &["sluit het programma af", "ik ben klaar", "stop maar"], &[&["afsluiten", "sluit", "ben klaar", "stop maar", "stoppen", "einde"]], &["herhaal*", "geluid", "opname", "voorlezen"]),
];

/// Woorden die niets zeggen over wat je wilt.
const VULWOORDEN: &[&str] = &[
    "ik", "je", "jij", "u", "de", "het", "een", "mijn", "m'n", "wil", "wilt", "graag", "even", "maar", "alsjeblieft",
    "alstublieft", "kun", "kunt", "kan", "nu", "eens", "nou", "toch", "dan", "dat", "die", "dit", "deze", "er", "d'r",
    "ook", "meteen", "alvast", "hè", "he", "hoor", "jij", "mij", "me", "voor", "van", "ze", "allemaal", "doe", "zou",
    "hier", "daar", "eerst", "lieve", "please", "oké", "oke", "ok", "ja",
];

/// Woorden die het dicteren vaak verkeerd schrijft, en wat er bedoeld is.
const VERHASPELD: &[(&str, &str)] = &[
    ("stant", "bestand"), ("stand", "bestand"), ("best", "bestand"), ("exxon", "exe"), ("ex", "exe"), ("exe's", "exe"),
    ("ixi", "exe"), ("iksie", "exe"), ("error", "editor"), ("klopblok", "kladblok"), ("setup.exe", "setup"),
    ("programmacode", "code"), ("instellatie", "installatie"), ("installatiestand", "installatiebestand"),
];

/// Wat de editor met je zin moet doen.
#[derive(Debug, PartialEq)]
pub enum Begrip {
    /// Precies één functie. `extra` is bijvoorbeeld de naam van een programma of een regelnummer.
    Doe { functie: Functie, extra: String },
    /// Meer functies passen: kies er een.
    Kies(Vec<Functie>),
    /// Met één verbetering past de zin: vraag eerst of dat klopt.
    BedoelJe { zin: String, functie: Functie },
    /// Geen idee.
    Onbekend,
}

/// Kleine letters, geen leestekens, verhaspelingen verbeterd, en eigen opdrachten toegepast.
fn woorden(zin: &str, eigen: &[(String, String)]) -> Vec<String> {
    let mut t = tekst::normaliseer(zin).to_lowercase();
    for (zeg, bedoel) in eigen {
        let z = zeg.trim().to_lowercase();
        if !z.is_empty() && t.contains(&z) {
            t = t.replace(&z, &bedoel.trim().to_lowercase());
        }
    }
    let schoon: String = t.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '\'' || c == '.' { c } else { ' ' }).collect();
    let mut uit: Vec<String> = Vec::new();
    for w in schoon.split_whitespace() {
        let w = w.trim_matches(|c: char| c == '.' || c == '-' || c == '\'');
        if w.is_empty() {
            continue;
        }
        let w = VERHASPELD.iter().find(|(fout, _)| *fout == w).map(|(_, goed)| goed.to_string()).unwrap_or_else(|| w.to_string());
        // "installatie bestand" en "exe bestand" als één woord.
        if w == "bestand" && uit.last().map_or(false, |v| v == "installatie" || v == "exe") {
            let v = uit.pop().unwrap();
            uit.push(format!("{}bestand", if v == "exe" { "exe-" } else { "installatie" }));
            continue;
        }
        uit.push(w);
    }
    uit
}

fn past_woord(patroon: &str, w: &str) -> bool {
    match patroon.strip_suffix('*') {
        Some(begin) => w.starts_with(begin),
        None => patroon == w,
    }
}

fn bevat(ws: &[String], patroon: &str) -> bool {
    if patroon.contains(' ') {
        let delen: Vec<&str> = patroon.split(' ').collect();
        return ws.windows(delen.len()).any(|r| r.iter().zip(&delen).all(|(w, p)| past_woord(p, w)));
    }
    ws.iter().any(|w| past_woord(patroon, w))
}

fn passend(ws: &[String]) -> Vec<(Functie, usize)> {
    let mut uit = Vec::new();
    for actie in ACTIES {
        if actie.niet.iter().any(|n| bevat(ws, n)) {
            continue;
        }
        if actie.nodig.iter().all(|groep| groep.iter().any(|p| bevat(ws, p))) {
            uit.push((actie.functie, actie.nodig.len()));
        }
    }
    // Functies met meer groepjes zijn preciezer: die winnen.
    if let Some(max) = uit.iter().map(|x| x.1).max() {
        uit.retain(|x| x.1 == max);
    }
    uit
}

/// Wat er na het woord "programma" of "regel" staat: een naam of een nummer.
fn extra(functie: Functie, ws: &[String], origineel: &str) -> String {
    match functie {
        NaarRegel => ws.iter().find(|w| w.chars().all(|c| c.is_ascii_digit())).cloned().or_else(|| ws.iter().find_map(|w| crate::getallen::lees(w).map(|g| (g as i64).to_string()))).unwrap_or_default(),
        Openen => {
            // De naam zoals je hem schreef (met hoofdletters), na "programma" of na het werkwoord.
            let norm = tekst::normaliseer(origineel);
            let laag = norm.to_lowercase();
            let mut begin = None;
            for sleutel in ["programma ", "het bestand ", "bestand ", "code van ", "open ", "openen ", "laad "] {
                if let Some(p) = laag.rfind(sleutel) {
                    begin = Some(p + sleutel.len());
                    break;
                }
            }
            let Some(b) = begin else { return String::new() };
            let rest = norm[b..].trim().trim_end_matches(['.', '!', '?']).trim();
            let laag = rest.to_lowercase();
            for weg in ["mijn ", "het ", "de "] {
                if laag.starts_with(weg) {
                    return rest[weg.len()..].trim().to_string();
                }
            }
            if ["programma", "mijn programma", "code", "zien", "maar", "eens"].contains(&laag.as_str()) {
                return String::new();
            }
            rest.to_string()
        }
        _ => String::new(),
    }
}

/// Alle woorden die in de opdrachten voorkomen (voor tikfouten).
fn alle_woorden() -> Vec<&'static str> {
    let mut uit: Vec<&str> = Vec::new();
    for actie in ACTIES {
        for groep in actie.nodig {
            for p in groep.iter() {
                let p = p.trim_end_matches('*');
                if !p.contains(' ') && p.len() >= 4 && !uit.contains(&p) {
                    uit.push(p);
                }
            }
        }
    }
    uit
}

/// Begrijpt een zin die je tegen de editor zegt. `eigen` zijn je eigen opdrachten ("zeg = bedoel").
pub fn begrijp(zin: &str, eigen: &[(String, String)]) -> Begrip {
    let ws = woorden(zin, eigen);
    let inhoud: Vec<String> = ws.iter().filter(|w| !VULWOORDEN.contains(&w.as_str())).cloned().collect();
    if inhoud.is_empty() {
        return Begrip::Onbekend;
    }
    let p = passend(&ws);
    match p.len() {
        1 => return Begrip::Doe { functie: p[0].0, extra: extra(p[0].0, &ws, zin) },
        n if n > 1 => return Begrip::Kies(p.into_iter().map(|x| x.0).collect()),
        _ => {}
    }
    // Eén tikfout? Alleen voorstellen, niet zelf doen.
    let lijst = alle_woorden();
    for (i, w) in ws.iter().enumerate() {
        if VULWOORDEN.contains(&w.as_str()) || w.len() < 4 {
            continue;
        }
        let mut kandidaten: Vec<&str> = lijst
            .iter()
            .copied()
            .filter(|g| {
                let d = tekst::afstand(w, g);
                d > 0 && d <= if w.chars().count() >= 7 { 2 } else { 1 }
            })
            .collect();
        kandidaten.sort_by_key(|g| tekst::afstand(w, g));
        for goed in kandidaten {
            let mut nieuw = ws.clone();
            nieuw[i] = goed.to_string();
            let p = passend(&nieuw);
            if p.len() == 1 {
                return Begrip::BedoelJe { zin: nieuw.join(" "), functie: p[0].0 };
            }
        }
    }
    Begrip::Onbekend
}

/// De functie bij een naam zoals "BouwExe" (voor een opdracht in een melding).
pub fn functie_van_naam(naam: &str) -> Option<Functie> {
    ACTIES.iter().map(|a| a.functie).find(|f| format!("{:?}", f) == naam)
}

/// Een regel in je programma die geen zin van de taal is, maar een verzoek of een opdracht
/// ("zet er een knop Opslaan bij", "maak er een exe van"). Geeft een melding met een voorstel.
pub fn verzoek_in_programma(regel: usize, zin: &str) -> Option<crate::melding::Melding> {
    use crate::melding::{Melding, Wijziging};
    use crate::wensen::Voorstel;
    let antwoord = match begrijp_alles(zin, &[]) {
        Antwoord::Lijkt { dan, .. } => *dan,
        a => a,
    };
    Some(match antwoord {
        Antwoord::Taal(Voorstel::Regels { regels, uitleg }) => Melding::vraag(regel, format!("Dit is een verzoek, geen zin van de taal. Ik begrijp: {}", uitleg)).met_voorstel(
            format!("Zal ik deze regel vervangen door: {}?", regels.join(" / ")),
            Wijziging::VervangRegel { regel, nieuw: regels.join("\n") },
        ),
        Antwoord::Taal(Voorstel::Vraag(v)) => Melding::vraag(regel, format!("Dit is een verzoek, maar ik mis nog iets. {}", v)),
        Antwoord::Taal(Voorstel::Uitleg(u)) => Melding::vraag(regel, format!("Dit is een vraag aan het programma, geen zin van de taal. Antwoord: {}", u))
            .met_voorstel("Zal ik deze vraag uit je programma weghalen?", Wijziging::HaalWeg { regel }),
        Antwoord::Taal(Voorstel::NogNiet(n)) => Melding::vraag(regel, n),
        Antwoord::Editor(Begrip::Doe { functie, extra }) => {
            let a = actie(functie);
            let opdracht = if extra.is_empty() { format!("{:?}", functie) } else { format!("{:?}|{}", functie, extra) };
            Melding::vraag(regel, format!("Dit is een opdracht voor de editor: {}.", a.naam))
                .met_voorstel("Zal ik deze regel weghalen en het nu doen?", Wijziging::HaalWeg { regel })
                .met_opdracht(opdracht)
        }
        Antwoord::Editor(Begrip::BedoelJe { zin: beter, functie }) => Melding::vraag(regel, format!("Bedoel je de opdracht: {}? ({})", beter, actie(functie).naam))
            .met_voorstel("Zal ik deze regel weghalen en het nu doen?", Wijziging::HaalWeg { regel })
            .met_opdracht(format!("{:?}", functie)),
        _ => return None,
    })
}

pub fn actie(f: Functie) -> &'static Actie {
    ACTIES.iter().find(|a| a.functie == f).expect("elke functie staat in de lijst")
}

/// Een regel voor de lijst "Wat je kunt zeggen".
pub fn regel(a: &Actie) -> String {
    let toets = if a.toets.is_empty() { String::new() } else { format!(" ({})", a.toets) };
    format!("{}{}. Zeg bijvoorbeeld: {}", a.naam, toets, a.voorbeelden.join(", of: "))
}

/// Zoeken in wat je kunt zeggen: alle acties waar elk zoekwoord in de naam of voorbeelden staat.
pub fn zoek(zoekwoorden: &str) -> Vec<&'static Actie> {
    let ws: Vec<String> = woorden(zoekwoorden, &[]).into_iter().filter(|w| !VULWOORDEN.contains(&w.as_str())).collect();
    let mut uit: Vec<&Actie> = ACTIES.iter().filter(|a| {
        let t = regel(a).to_lowercase();
        ws.iter().all(|w| t.contains(w.as_str()))
    }).collect();
    // Niets met alle woorden? Dan wat de meeste woorden heeft.
    if uit.is_empty() && !ws.is_empty() {
        let mut telling: Vec<(&Actie, usize)> = ACTIES
            .iter()
            .map(|a| {
                let t = regel(a).to_lowercase();
                (a, ws.iter().filter(|w| w.len() >= 3 && t.contains(w.as_str())).count())
            })
            .filter(|x| x.1 > 0)
            .collect();
        telling.sort_by(|x, y| y.1.cmp(&x.1));
        uit = telling.into_iter().map(|x| x.0).collect();
    }
    // Past de zin precies bij een functie? Die bovenaan.
    if let Begrip::Doe { functie, .. } = begrijp(zoekwoorden, &[]) {
        uit.retain(|a| a.functie != functie);
        uit.insert(0, actie(functie));
    }
    uit
}

/// Wat een zin betekent: iets voor de editor, of iets voor in je programma.
#[derive(Debug, PartialEq)]
pub enum Antwoord {
    Editor(Begrip),
    Taal(crate::wensen::Voorstel),
    /// Je zin lijkt op iets wat Jaap eerder zei: eerst vragen of dat bedoeld is.
    Lijkt { jaaps_zin: String, dan: Box<Antwoord> },
}

/// Wat een betekenis uit Jaaps zinnen inhoudt.
fn betekenis(b: &str, eigen: &[(String, String)]) -> Antwoord {
    use crate::wensen::Voorstel;
    if let Some(u) = b.strip_prefix("uitleg:") {
        return Antwoord::Taal(Voorstel::Uitleg(u.trim().to_string()));
    }
    if let Some(n) = b.strip_prefix("kan nog niet:") {
        let mut n = n.trim().to_string();
        if let Some(c) = n.get(..1) {
            n = c.to_uppercase() + &n[1..];
        }
        return Antwoord::Taal(Voorstel::NogNiet(format!("{}: dat kan Jaap's Programmeertaal nog niet. Ik kan het op je wensenlijst zetten.", n)));
    }
    begrijp_regels(b, eigen)
}

/// Begrijpt alles wat je zegt. Eerst: lijkt het sterk op iets wat Jaap eerder zei? Dan de vaste
/// regels: iets voor in je programma ("zet er een knop Opslaan bij") of voor de editor ("maak er
/// een exe van"). Lijkt het een beetje op een zin van Jaap, dan wordt eerst gevraagd of dat bedoeld is.
pub fn begrijp_alles(zin: &str, eigen: &[(String, String)]) -> Antwoord {
    let lijkt = crate::jaaps_zinnen::lijkt_op(zin);
    let r = begrijp_regels(zin, eigen);
    // Een concrete zin voor je programma (met naam of bestand) gaat altijd voor.
    if matches!(r, Antwoord::Taal(crate::wensen::Voorstel::Regels { .. })) {
        return r;
    }
    // "Kan ik ook ...?" is een vraag of iets kan: dan telt het voorbeeld eerder.
    let laag = zin.trim().to_lowercase();
    let vraag_of_het_kan = ["kan ", "kun ", "kunnen ", "is het mogelijk", "lukt het", "werkt "].iter().any(|b| laag.starts_with(b));
    if let Some((s, _, b)) = lijkt {
        if s >= 0.75 || (vraag_of_het_kan && s >= 0.45) {
            return betekenis(b, eigen);
        }
    }
    if !matches!(r, Antwoord::Editor(Begrip::Onbekend) | Antwoord::Editor(Begrip::Kies(_))) {
        return r;
    }
    match lijkt {
        Some((s, jaap, b)) if s >= 0.3 => Antwoord::Lijkt { jaaps_zin: jaap.to_string(), dan: Box::new(betekenis(b, eigen)) },
        _ => r,
    }
}

fn begrijp_regels(zin: &str, eigen: &[(String, String)]) -> Antwoord {
    use crate::wensen::Voorstel;
    let editor = begrijp(zin, eigen);
    match crate::wensen::begrijp(zin) {
        Some(Voorstel::Vraag(_)) if matches!(editor, Begrip::Doe { .. }) => Antwoord::Editor(editor),
        Some(v) => Antwoord::Taal(v),
        None => Antwoord::Editor(editor),
    }
}

/// Leest het bestand "Mijn eigen opdrachten": regels als "maak er wat van = bouw een exe".
pub fn eigen_opdrachten(tekst: &str) -> Vec<(String, String)> {
    crate::bestand::eigen_woorden(tekst)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doe(zin: &str) -> Functie {
        match begrijp(zin, &[]) {
            Begrip::Doe { functie, .. } => functie,
            ander => panic!("{} gaf {:?}", zin, ander),
        }
    }

    #[test]
    fn zoals_jaap_praat() {
        assert_eq!(doe("Ga maar ook verder"), Verder);
        assert_eq!(doe("Ik zei dat je verder moest gaan."), Verder);
        assert_eq!(doe("Maak alvast maar het installatieprogramma"), Installatie);
        assert_eq!(doe("Geef mij hier het installatie best direct"), Installatie);
        assert_eq!(doe("Het installatie Stant moet er ook in"), Installatie);
        assert_eq!(doe("Maak er een Exxon van"), BouwExe);
        assert_eq!(doe("compileer het maar"), BouwExe);
        assert_eq!(doe("Snap je wat ik bedoel?"), Beschrijf);
        assert_eq!(doe("Heb je nou begrepen wat ik wil?"), Beschrijf);
        assert_eq!(doe("Controleer mijn zinnen"), Controleren);
        assert_eq!(doe("ga naar regel 12"), NaarRegel);
        assert_eq!(doe("op welke regel sta ik"), WelkeRegel);
        assert_eq!(doe("sla op"), Opslaan);
        assert_eq!(doe("bewaar het met een andere naam"), OpslaanAls);
        assert_eq!(doe("open de handleiding"), Handleiding);
        assert_eq!(doe("wat kan ik zeggen"), WatKanIkZeggen);
        assert_eq!(doe("haal mijn zinnen uit de exe"), OpenExe);
        assert_eq!(doe("start mijn programma"), Uitvoeren);
        assert_eq!(doe("laat de rust code zien"), RustCode);
        assert_eq!(doe("verhoog het versienummer"), VersieOmhoog);
        assert_eq!(doe("wat moet er nog gebeuren voor het lijstje"), Wensenlijst);
        assert_eq!(doe("open de handleiding"), Handleiding);
        assert!(matches!(begrijp_alles("open de handleiding", &[]), Antwoord::Editor(_)));
        assert!(matches!(begrijp_alles("zet er een knop Opslaan bij", &[]), Antwoord::Taal(_)));
        assert!(matches!(begrijp_alles("de versie moet 2.0 worden", &[]), Antwoord::Taal(_)));
    }

    #[test]
    fn naam_en_nummer() {
        assert_eq!(begrijp("Open mijn programma Begroeting", &[]), Begrip::Doe { functie: Openen, extra: "Begroeting".into() });
        assert_eq!(begrijp("open Rekenhulp", &[]), Begrip::Doe { functie: Openen, extra: "Rekenhulp".into() });
        assert_eq!(begrijp("ga naar regel twaalf", &[]), Begrip::Doe { functie: NaarRegel, extra: "12".into() });
    }

    #[test]
    fn niet_gokken() {
        assert_eq!(begrijp("bananen zijn geel", &[]), Begrip::Onbekend);
        assert_eq!(begrijp("graag even", &[]), Begrip::Onbekend);
        match begrijp("controleeer mijn programma", &[]) {
            Begrip::BedoelJe { functie, .. } => assert_eq!(functie, Controleren),
            ander => panic!("{:?}", ander),
        }
    }

    #[test]
    fn eigen_opdracht() {
        let eigen = vec![("knal hem eruit".to_string(), "bouw een exe".to_string())];
        assert_eq!(begrijp("knal hem eruit", &eigen), Begrip::Doe { functie: BouwExe, extra: String::new() });
    }

    #[test]
    fn zoeken() {
        assert_eq!(zoek("installatie")[0].functie, Installatie);
        assert!(zoek("regel").len() >= 2);
        assert_eq!(zoek("").len(), ACTIES.len());
    }
}
