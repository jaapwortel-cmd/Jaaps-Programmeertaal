//! Verzoeken in gewone taal omzetten in zinnen van Jaap's Programmeertaal.
//!
//! Zo praat Jaap tegen Claude Code: "zet er een knop Opslaan bij", "het moet een menu Bestand
//! hebben", "de versie moet 1.2 worden", "geef de knop Opslaan de sneltoets Ctrl+S". Deze module
//! herkent zulke verzoeken met vaste regels (geen AI) en stelt de juiste zin voor. De editor
//! vraagt altijd eerst "ja of nee" voordat er iets in je programma komt.
//!
//! Wat de taal nog niet kan, wordt niet nagemaakt: dan zegt het programma dat eerlijk, en kan het
//! verzoek op je wensenlijst.

use crate::tekst;

#[derive(Debug, PartialEq)]
pub enum Voorstel {
    /// Deze zinnen zou je programma erbij krijgen, met een korte uitleg.
    Regels { regels: Vec<String>, uitleg: String },
    /// Er mist iets; dit is de ene vraag die nodig is.
    Vraag(String),
    /// De taal kan dit nog niet. De tekst zegt wat er wel kan.
    NogNiet(String),
    /// Uitleg: zo doe je dat (of: dat kan al).
    Uitleg(String),
}

/// Onderdelen: het woord dat je zegt, en hoe de zin het noemt.
const ONDERDELEN: &[(&[&str], &str)] = &[
    (&["knop", "knoppen", "button"], "knop"),
    (&["groot tekstvak", "grote tekstvak", "kladblok", "tekstveld met meer regels"], "groot tekstvak"),
    (&["tekstvak", "tekstveld", "invulvak", "invoervak", "vak", "editveld"], "tekstvak"),
    (&["label", "tekstje", "opschrift"], "label"),
    (&["selectievakje", "vinkje", "vakje", "checkbox", "aankruisvakje"], "selectievakje"),
    (&["keuzelijst", "lijstje om te kiezen", "listbox"], "keuzelijst"),
    (&["uitklaplijst", "combobox", "dropdown"], "uitklaplijst"),
    (&["tabel", "tabelletje"], "tabel"),
    (&["menuregel", "menu-item", "menuitem", "menuonderdeel"], "menuregel"),
    (&["menu", "menubalk"], "menu"),
    (&["tabblad", "tabbladen", "tab"], "tabblad"),
    (&["venster", "scherm", "window"], "venster"),
];

/// Woorden waarmee je vraagt iets te maken.
const MAAK: &[&str] = &["maak", "maken", "inbouwen", "bouwen", "toevoegen", "hebben", "krijgen", "erin", "ook", "zet", "voeg", "bouw", "wil", "moet", "komen", "komt", "graag", "erbij", "toe", "nodig", "plaats", "doe"];

/// Wat de taal nog niet kan (staat op de wensenlijst), met een eerlijke uitleg.
const NOG_NIET: &[(&[&str], &str)] = &[
    (&["donkere stand", "donker thema", "hoog contrast", "dark mode"], "Een donkere stand of hoog contrast (wel kan: Geef het venster de kleur donkerblauw)"),
    (&["spraakherkenning", "dicteren", "whisper", "uitschrijven"], "Spraakherkenning in je eigen programma (dicteren in Jaap's Programmeertaal zelf kan wel, met je eigen dicteerprogramma)"),
    (&["taalmodel", "kunstmatige intelligentie", " ai ", "chatgpt", "claude", "llm"], "AI of een taalmodel: dat zit bewust niet in Jaap's Programmeertaal"),
    (&["3d", "driedimensionaal"], "3D-geluid of 3D-beeld"),
    (&["audio editor", "audio-editor", "geluid bewerken", "knippen in geluid"], "Geluid bewerken (knippen, plakken, effecten)"),
    (&["mailen", "e-mail versturen", "mail versturen", "smtp", "outlook"], "E-mail versturen vanuit je programma"),
    (&["netwerk", "andere computer", "op afstand"], "Verbinding tussen computers"),
    (&["rar"], "RAR-bestanden (zip kan wel)"),
    (&["braille", "brailleleesregel"], "Braille rechtstreeks aansturen (je schermlezer doet dat al)"),
    (&["edge", "webview"], "Edge of WebView2: dat wil je niet, en het zit er niet in. Webpagina's openen gaat met je eigen browser"),
];

fn klein(zin: &str) -> String {
    let t = tekst::normaliseer(zin).to_lowercase();
    format!(" {} ", t.replace([',', '!', '?', ';', ':'], " ").split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Haalt een naam uit de zin zoals je hem schreef: na "met de naam", "genaamd", "die heet", of
/// direct na het woord van het onderdeel. Tekst tussen aanhalingstekens gaat voor.
fn naam_na(origineel: &str, na: &str) -> Option<String> {
    if let Some(b) = origineel.find('"') {
        if let Some(e) = origineel[b + 1..].find('"') {
            let n = origineel[b + 1..b + 1 + e].trim();
            if !n.is_empty() {
                return Some(n.to_string());
            }
        }
    }
    let norm = tekst::normaliseer(origineel);
    let laag = norm.to_lowercase();
    let mut begin = None;
    for sleutel in ["met als naam ", "met de naam ", "met als titel ", "met de titel ", "genaamd ", "die heet ", "dat heet ", "met naam ", "met titel "] {
        if let Some(p) = laag.find(sleutel) {
            begin = Some(p + sleutel.len());
            break;
        }
    }
    if begin.is_none() {
        let p = laag.find(&format!("{} ", na))?;
        begin = Some(p + na.len() + 1);
    }
    let rest = &norm[begin?..];
    // Stoppen bij woorden die niet bij de naam horen.
    let mut woorden = Vec::new();
    for (i, w) in rest.split_whitespace().enumerate() {
        let l = w.to_lowercase();
        if i == 0 && matches!(l.as_str(), "de" | "het" | "een") {
            continue;
        }
        let l = l.trim_matches(|c: char| !c.is_alphanumeric());
        if matches!(
            l,
            "bij" | "erbij" | "toe" | "toevoegen" | "maken" | "komen" | "in" | "op" | "aan" | "zetten" | "hebben" | "krijgen" | "en" | "met" | "die" | "dat" | "graag" | "alsjeblieft" | "maar" | "hoor" | "he" | "hè" | "de" | "het" | "een" | "moet" | "moeten" | "sneltoets" | "als" | "worden" | "wordt" | "voor"
        ) {
            break;
        }
        woorden.push(w.trim_matches(|c: char| matches!(c, '.' | ',' | '!' | '?')));
    }
    let n = woorden.join(" ");
    let n = n.trim();
    let nl = n.to_lowercase();
    if n.is_empty() || ["een", "de", "het", "er", "ook", "nog"].contains(&nl.as_str()) {
        return None;
    }
    let n = ["een ", "de ", "het "].iter().find_map(|p| nl.starts_with(p).then(|| n[p.len()..].to_string())).unwrap_or(n.to_string());
    Some(n)
}

fn heeft(t: &str, woord: &str) -> bool {
    t.contains(&format!(" {} ", woord.trim()))
}

/// Een sneltoets zoals Ctrl+S, Control S of Alt+F4.
fn sneltoets(origineel: &str) -> Option<String> {
    let woorden: Vec<String> = tekst::normaliseer(origineel).replace('+', " ").split_whitespace().map(|w| w.to_string()).collect();
    for (i, w) in woorden.iter().enumerate() {
        let l = w.to_lowercase();
        let mods = match l.as_str() {
            "ctrl" | "control" | "controle" | "kontrol" => "Ctrl",
            "alt" => "Alt",
            "shift" => "Shift",
            _ => continue,
        };
        let mut delen = vec![mods.to_string()];
        let mut j = i + 1;
        while j < woorden.len() {
            let x = woorden[j].trim_matches(|c: char| matches!(c, '.' | ',' | '!' | '?'));
            match x.to_lowercase().as_str() {
                "ctrl" | "control" => delen.push("Ctrl".into()),
                "alt" => delen.push("Alt".into()),
                "shift" => delen.push("Shift".into()),
                k if k.chars().count() == 1 || (k.starts_with('f') && k[1..].parse::<u8>().is_ok()) => {
                    delen.push(k.to_uppercase());
                    return Some(delen.join("+"));
                }
                _ => return None,
            }
            j += 1;
        }
    }
    None
}

/// Probeert een verzoek om te zetten in zinnen van de taal.
pub fn begrijp(zin: &str) -> Option<Voorstel> {
    let t = klein(zin);

    // Wat de taal nog niet kan: eerlijk zeggen.
    for (woorden, wat) in NOG_NIET {
        if woorden.iter().any(|w| t.contains(&format!(" {} ", w.trim())) || (w.contains(' ') && t.contains(w))) {
            return Some(Voorstel::NogNiet(format!(
                "{}: dat kan Jaap's Programmeertaal nog niet. Ik kan het op je wensenlijst zetten.",
                wat
            )));
        }
    }

    // Dingen die nu ook kunnen.
    let regel = |r: &str, u: &str| Some(Voorstel::Regels { regels: vec![r.to_string()], uitleg: u.to_string() });
    if ["gemaximaliseerd", "schermvullend", "maximaal", "volledig scherm"].iter().any(|w| t.contains(w)) {
        return regel("Het venster start schermvullend", "Het venster start schermvullend.");
    }
    if t.contains(" afsluiten ") && (t.contains(" vraag") || t.contains(" zeker ")) {
        return regel("Vraag bij afsluiten \"Weet je zeker dat je wilt afsluiten?\"", "Bij afsluiten komt eerst een vraag met Ja en Nee.");
    }
    if ["een keer open", "één keer open", "1 keer open", "twee keer open"].iter().any(|w| t.contains(w)) {
        return regel("Mijn programma mag maar één keer open zijn", "Je programma kan maar één keer tegelijk open zijn.");
    }
    if heeft(&t, "reservekopie") || heeft(&t, "back-up") || heeft(&t, "backup") || heeft(&t, "reservekopieën") {
        return Some(match tekst::normaliseer(zin).split_whitespace().find(|w| w.contains('.') && w.len() > 4 && !w.ends_with('.')) {
            Some(b) => Voorstel::Regels { regels: vec![format!("Maak een reservekopie van het bestand {}", b)], uitleg: "Er komt een reservekopie in de map Reservekopieën; de laatste 10 blijven.".into() },
            None => Voorstel::Vraag("Van welk bestand moet ik een reservekopie maken? Zeg bijvoorbeeld: maak een reservekopie van notities.txt.".into()),
        });
    }
    if heeft(&t, "qr-code") || heeft(&t, "qr") || heeft(&t, "qrcode") {
        return Some(Voorstel::Uitleg("Een QR-code maak je zo: Maak een QR-code van \"https://www.voorbeeld.nl\" in het bestand qr.png. Daarna kun je hem tonen: Maak een afbeelding van het bestand qr.png met de beschrijving QR-code naar mijn website. QR-codes lezen kan nog niet.".into()));
    }
    if heeft(&t, "internetradio") || heeft(&t, "radiozender") || heeft(&t, "radio") && !heeft(&t, "radiostudio") {
        if let Some((naam, _, _)) = crate::taal::ZENDERS.iter().find(|(n, _, _)| t.contains(&format!(" {} ", n.to_lowercase()))) {
            return regel(&format!("Speel de radiozender {}", naam), &format!("Je programma speelt {}.", naam));
        }
        return Some(Voorstel::Uitleg("Een internetradio maak je zo: Speel de radiozender NPO Radio 2 (of een andere naam, of: met het adres https://...), en: Stop de radio. Zet ze achter knoppen, zoals in het voorbeeld Mijn radio.txt. Op een website: Zet een radiospeler met de zender NPO Radio 2.".into()));
    }
    if heeft(&t, "website") || heeft(&t, "webpagina") || heeft(&t, "site") || heeft(&t, "homepage") {
        return Some(Voorstel::Uitleg("Een website maak je met een nieuw programma dat begint met: Maak een website met als titel ... Daarna: Maak een pagina met als naam Home, Zet een kop \"...\", Zet de tekst \"...\", Zet een link naar ..., Zet een afbeelding ... met de beschrijving ..., Zet een radiospeler met de zender .... Met F5 gaat hij open in Firefox. Zie het voorbeeld Mijn radiosite (website).txt.".into()));
    }
    if heeft(&t, "tip") || heeft(&t, "tooltip") {
        if let Some(knop) = naam_na(zin, "knop") {
            let tip = zin.split('"').nth(1).unwrap_or("Uitleg over deze knop").to_string();
            return regel(&format!("Geef de knop {} de tip \"{}\"", knop, tip), "De knop krijgt een tip.");
        }
    }

    // Sneltoets: "geef de knop Opslaan de sneltoets Ctrl+S", "Opslaan moet Control S krijgen".
    if heeft(&t, "sneltoets") || t.contains(" ctrl ") || t.contains(" control ") {
        if let Some(toets) = sneltoets(zin) {
            let soort = if heeft(&t, "menuregel") || heeft(&t, "menu") { "menuregel" } else { "knop" };
            let naam = naam_na(zin, soort).or_else(|| naam_na(zin, "de"));
            let naam = naam.filter(|n| !n.to_lowercase().contains("sneltoets") && !n.to_lowercase().starts_with("ctrl"));
            return Some(match naam {
                Some(n) => Voorstel::Regels {
                    regels: vec![format!("Geef de {} {} de sneltoets {}", soort, n, toets)],
                    uitleg: format!("De {} {} krijgt de sneltoets {}.", soort, n, toets),
                },
                None => Voorstel::Vraag(format!("Welke {} moet de sneltoets {} krijgen? Zeg bijvoorbeeld: geef de knop Opslaan de sneltoets {}.", soort, toets, toets)),
            });
        }
    }

    // Versie: "de versie moet 1.2 worden".
    if heeft(&t, "versie") || heeft(&t, "versienummer") {
        let getal = t.split_whitespace().find(|w| w.chars().next().map_or(false, |c| c.is_ascii_digit()) && w.chars().all(|c| c.is_ascii_digit() || c == '.'));
        if let Some(v) = getal {
            return Some(Voorstel::Regels { regels: vec![format!("De versie is {}", v.trim_end_matches('.'))], uitleg: format!("Je programma krijgt versie {}.", v) });
        }
        if ["omhoog", "ophogen", "hoger", "verhoog", "nieuwe"].iter().any(|w| heeft(&t, w)) {
            return None; // Dat doet de editor zelf (versie ophogen).
        }
    }

    // Maker, snelkoppelingen, handleiding, pictogram, licentie, privacy, updates.
    if heeft(&t, "maker") || t.contains(" gemaakt door ") {
        if let Some(n) = naam_na(zin, "maker").or_else(|| naam_na(zin, "door")) {
            let n = n.trim_start_matches("is ").to_string();
            return Some(Voorstel::Regels { regels: vec![format!("De maker is {}", n)], uitleg: format!("{} staat als maker in de installatie.", n) });
        }
    }
    if heeft(&t, "bureaublad") {
        return Some(Voorstel::Regels { regels: vec!["Zet een snelkoppeling op het bureaublad".into()], uitleg: "De installatie zet een snelkoppeling op het bureaublad.".into() });
    }
    if heeft(&t, "startmenu") {
        return Some(Voorstel::Regels { regels: vec!["Zet een snelkoppeling in het startmenu".into()], uitleg: "De installatie zet een snelkoppeling in het startmenu.".into() });
    }
    for (woord, zin_begin, voorbeeld) in [
        ("handleiding", "De handleiding staat in het bestand", "Handleiding.txt"),
        ("pictogram", "Het pictogram staat in het bestand", "logo.ico"),
        ("icoon", "Het pictogram staat in het bestand", "logo.ico"),
        ("licentie", "De licentie staat in het bestand", "Licentie.txt"),
        ("privacyverklaring", "De privacyverklaring staat in het bestand", "Privacy.txt"),
    ] {
        if heeft(&t, woord) {
            let bestand = tekst::normaliseer(zin).split_whitespace().find(|w| w.contains('.') && w.len() > 4 && !w.ends_with('.')).map(|w| w.to_string());
            return Some(match bestand {
                Some(b) => Voorstel::Regels { regels: vec![format!("{} {}", zin_begin, b)], uitleg: format!("{} komt uit {}.", woord, b) },
                None => Voorstel::Vraag(format!("In welk bestand staat de {}? Zeg bijvoorbeeld: de {} staat in {}.", woord, woord, voorbeeld)),
            });
        }
    }
    if heeft(&t, "updates") || heeft(&t, "update") {
        if let Some(adres) = zin.split_whitespace().find(|w| w.starts_with("http")) {
            return Some(Voorstel::Regels { regels: vec![format!("De updates staan op het adres {}", adres)], uitleg: "Je programma zoekt updates op dat adres.".into() });
        }
        if heeft(&t, "zoek") || heeft(&t, "zoeken") || heeft(&t, "kijk") || heeft(&t, "controleer") {
            return Some(Voorstel::Regels { regels: vec!["Zoek naar updates".into()], uitleg: "Je programma kijkt of er een nieuwe versie is.".into() });
        }
        return Some(Voorstel::Vraag("Op welk adres staan de updates? Zeg bijvoorbeeld: de updates staan op https://www.dropbox.com/... (een link naar je Dropbox-map).".into()));
    }
    if heeft(&t, "onderteken") || heeft(&t, "ondertekenen") || heeft(&t, "signeer") || heeft(&t, "signeren") {
        let naam = naam_na(zin, "naam van").unwrap_or_else(|| "Jaap Wortel".into());
        return Some(Voorstel::Regels {
            regels: vec![format!("Onderteken het programma met mijn eigen certificaat op naam van {}", naam)],
            uitleg: "Je programma en je installatie worden ondertekend met je eigen certificaat.".into(),
        });
    }

    // Geluid, voorlezen, schermlezer.
    let noemt_onderdeel = ONDERDELEN.iter().any(|(ws, _)| ws.iter().any(|w| heeft(&t, w)));
    if !noemt_onderdeel && (heeft(&t, "piep") || heeft(&t, "piepje") || heeft(&t, "geluidje")) {
        if let Some(b) = tekst::normaliseer(zin).split_whitespace().find(|w| [".wav", ".mp3", ".wma", ".m4a"].iter().any(|e| w.to_lowercase().ends_with(e))) {
            return Some(Voorstel::Regels { regels: vec![format!("Speel het geluid {}", b)], uitleg: format!("Je programma speelt {} af.", b) });
        }
        return Some(Voorstel::Regels { regels: vec!["Speel een piep".into()], uitleg: "Je programma laat een piepje horen.".into() });
    }
    if let Some(b) = tekst::normaliseer(zin).split_whitespace().find(|w| [".wav", ".mp3", ".wma", ".m4a"].iter().any(|e| w.to_lowercase().ends_with(e))) {
        return Some(Voorstel::Regels { regels: vec![format!("Speel het geluid {}", b)], uitleg: format!("Je programma speelt {} af.", b) });
    }
    if (heeft(&t, "voorlezen") || heeft(&t, "lees") || heeft(&t, "zeg") || heeft(&t, "schermlezer") || heeft(&t, "uitspreken")) && zin.contains('"') {
        let tussen = zin.split('"').nth(1).unwrap_or("").to_string();
        let regel = if heeft(&t, "voorlezen") || heeft(&t, "lees") { format!("Lees voor \"{}\"", tussen) } else { format!("Zeg \"{}\"", tussen) };
        return Some(Voorstel::Regels { regels: vec![regel], uitleg: "Dit wordt hardop gezegd.".into() });
    }

    // Kleur.
    for kleur in crate::taal::KLEUREN {
        if heeft(&t, kleur.0) && (heeft(&t, "kleur") || heeft(&t, "achtergrond") || heeft(&t, "venster")) {
            let wat = if heeft(&t, "tekst") || heeft(&t, "letters") { "de tekst" } else { "het venster" };
            return Some(Voorstel::Regels { regels: vec![format!("Geef {} de kleur {}", wat, kleur.0)], uitleg: format!("{} wordt {}.", wat, kleur.0) });
        }
    }

    // Onderdelen maken: "zet er een knop Opslaan bij", "ik wil een tekstvak voor de naam".
    let wil_maken = MAAK.iter().any(|w| heeft(&t, w)) || t.trim_start().starts_with("een ");
    if wil_maken {
        for (woorden, soort) in ONDERDELEN {
            let Some(gevonden) = woorden.iter().find(|w| heeft(&t, w)) else { continue };
            let lidwoord = if matches!(*soort, "label" | "menu" | "venster" | "tabblad" | "groot tekstvak" | "tekstvak" | "selectievakje") { "het" } else { "de" };
            let naam = naam_na(zin, gevonden).or_else(|| {
                // "een tekstvak voor de naam" -> Naam
                let laag = klein(zin);
                laag.find(" voor de ").map(|p| laag[p + 9..].split_whitespace().next().unwrap_or("").to_string()).filter(|n| !n.is_empty()).map(|n| {
                    let mut c = n.chars();
                    c.next().map(|e| e.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
                })
            });
            let Some(naam) = naam else {
                return Some(Voorstel::Vraag(format!(
                    "Hoe moet {} {} heten? Zeg bijvoorbeeld: maak een {} met als naam Opslaan.",
                    lidwoord,
                    soort,
                    soort
                )));
            };
            let regel = if *soort == "venster" {
                format!("Maak een venster met als titel {}", naam)
            } else {
                format!("Maak een {} met als naam {}", soort, naam)
            };
            return Some(Voorstel::Regels { regels: vec![regel], uitleg: format!("Er komt een {} {} bij.", soort, naam) });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn regel(zin: &str) -> String {
        match begrijp(zin) {
            Some(Voorstel::Regels { regels, .. }) => {
                for r in &regels {
                    assert!(crate::ontleden::ontleed_regel(&tekst::normaliseer(r)).is_ok(), "{} gaf de zin {}, en die kent de taal niet", zin, r);
                }
                regels.join(" | ")
            }
            ander => panic!("{} gaf {:?}", zin, ander),
        }
    }

    #[test]
    fn verzoeken_zoals_jaap_ze_doet() {
        assert_eq!(regel("Zet er een knop Opslaan bij"), "Maak een knop met als naam Opslaan");
        assert_eq!(regel("ik wil een tekstvak met de naam Leeftijd"), "Maak een tekstvak met als naam Leeftijd");
        assert_eq!(regel("Er moet een menu Bestand komen"), "Maak een menu met als naam Bestand");
        assert_eq!(regel("voeg een menuregel Afsluiten toe"), "Maak een menuregel met als naam Afsluiten");
        assert_eq!(regel("Ook tabbladen inbouwen, een tabblad Algemeen"), "Maak een tabblad met als naam Algemeen");
        assert_eq!(regel("maak een venster genaamd Instellingen"), "Maak een venster met als titel Instellingen");
        assert_eq!(regel("geef de knop Opslaan de sneltoets Control S"), "Geef de knop Opslaan de sneltoets Ctrl+S");
        assert_eq!(regel("de menuregel Afsluiten moet Alt+F4 als sneltoets krijgen"), "Geef de menuregel Afsluiten de sneltoets Alt+F4");
        assert_eq!(regel("de versie moet 1.2 worden"), "De versie is 1.2");
        assert_eq!(regel("de maker is Jaap Wortel"), "De maker is Jaap Wortel");
        assert_eq!(regel("ik wil een snelkoppeling op het bureaublad"), "Zet een snelkoppeling op het bureaublad");
        assert_eq!(regel("een eigen pictogram voor mijn programma: logo.ico"), "Het pictogram staat in het bestand logo.ico");
        assert_eq!(regel("de handleiding staat in Handleiding.txt"), "De handleiding staat in het bestand Handleiding.txt");
        assert_eq!(regel("speel een geluidje"), "Speel een piep");
        assert_eq!(regel("speel deurbel.wav af"), "Speel het geluid deurbel.wav");
        assert_eq!(regel("Eigen meldingen die je schermlezer uitspreekt: \"Opgeslagen\""), "Zeg \"Opgeslagen\"");
        assert_eq!(regel("lees \"Goedemorgen\" voor"), "Lees voor \"Goedemorgen\"");
        assert_eq!(regel("geef het venster de kleur donkerblauw"), "Geef het venster de kleur donkerblauw");
        assert_eq!(regel("met mijn eigen certificaat ondertekenen"), "Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel");
        assert_eq!(regel("kijk of er updates zijn"), "Zoek naar updates");
    }

    #[test]
    fn eerst_vragen() {
        assert!(matches!(begrijp("zet er een knop bij"), Some(Voorstel::Vraag(_))));
        assert!(matches!(begrijp("updates via Dropbox"), Some(Voorstel::Vraag(_))));
        assert!(matches!(begrijp("het moet een sneltoets Ctrl+W hebben"), Some(Voorstel::Vraag(_))));
    }

    #[test]
    fn eerlijk_wat_nog_niet_kan() {
        assert!(matches!(begrijp("het venster moet gemaximaliseerd starten"), Some(Voorstel::Regels { .. })));
        assert!(matches!(begrijp("vraag bij afsluiten of ik het zeker weet"), Some(Voorstel::Regels { .. })));
        assert!(matches!(begrijp("ik wil een internetradio bouwen"), Some(Voorstel::Uitleg(_))));
        assert!(matches!(begrijp("zet radio 538 aan"), Some(Voorstel::Regels { .. })));
        assert!(matches!(begrijp("ik wil een website bouwen"), Some(Voorstel::Uitleg(_))));
        assert!(matches!(begrijp("ik wil 3d beelden maken"), Some(Voorstel::NogNiet(_))));
        assert!(matches!(begrijp("geen Edge gebruiken"), Some(Voorstel::NogNiet(_))));
        assert_eq!(begrijp("bananen zijn geel"), None);
    }
}
