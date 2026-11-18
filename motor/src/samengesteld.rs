//! Lange verzoeken in gewone taal omzetten in een heel programma, met vaste regels (geen AI).
//!
//! "Maak een venster met de titel Adressen, met een tekstvak Naam en een knop Opslaan, en als ik op
//! Opslaan druk, schrijf de naam in het bestand adressen.txt en zeg opgeslagen"
//!
//! wordt:
//!
//! Maak een venster met als titel Adressen
//! Maak een tekstvak met als naam Naam
//! Maak een knop met als naam Opslaan
//! Als ik op de knop Opslaan druk
//! Schrijf de tekst in het tekstvak Naam in het bestand adressen.txt
//! Zeg "opgeslagen"
//! Einde van de knop
//!
//! Wat niet begrepen wordt, komt als opmerking in het voorstel en wordt eerlijk genoemd.

use crate::tekst;

/// (woord in je verzoek, soort in de taal, meervoud)
const SOORTEN: &[(&str, &str)] = &[
    ("groot tekstvak", "groot tekstvak"),
    ("grote tekstvakken", "groot tekstvak"),
    ("tekstvakken", "tekstvak"),
    ("tekstvak", "tekstvak"),
    ("invulvakken", "tekstvak"),
    ("invulvak", "tekstvak"),
    ("tekstveld", "tekstvak"),
    ("knoppen", "knop"),
    ("knop", "knop"),
    ("labels", "label"),
    ("label", "label"),
    ("selectievakjes", "selectievakje"),
    ("selectievakje", "selectievakje"),
    ("vinkje", "selectievakje"),
    ("keuzelijst", "keuzelijst"),
    ("uitklaplijst", "uitklaplijst"),
    ("tabel", "tabel"),
    ("tabblad", "tabblad"),
    ("tabbladen", "tabblad"),
];

/// Woorden waar een naam ophoudt.
const STOP: &[&str] = &[
    "en", "met", "een", "de", "het", "als", "dan", "daarna", "erbij", "erin", "eronder", "bovenaan", "onderaan", "voor", "die", "dat",
    "waarin", "waarmee", "om", "in", "op", "naast", "inbouwen", "maken", "toevoegen", "bouwen", "hebben", "komen", "nodig", "graag",
    "erop", "zetten", "ook", "moet", "moeten", "wil", ",", "heet", "heten", "zijn", "is", "staan", "komt", "komen",
];

fn hoofdletter(t: &str) -> String {
    let mut c = t.chars();
    c.next().map(|e| e.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default()
}

fn klopt(regel: &str) -> bool {
    crate::ontleden::ontleed_regel(&tekst::normaliseer(regel)).is_ok()
}

/// Woorden met hun plek, zonder leestekens (tekst tussen aanhalingstekens blijft één woord).
fn woorden(t: &str) -> Vec<String> {
    let mut uit = Vec::new();
    let mut huidig = String::new();
    let mut in_tekst = false;
    for c in t.chars() {
        if c == '"' {
            huidig.push(c);
            if in_tekst {
                uit.push(std::mem::take(&mut huidig));
            }
            in_tekst = !in_tekst;
            continue;
        }
        if in_tekst {
            huidig.push(c);
        } else if c.is_whitespace() || c == ',' || c == ';' {
            if !huidig.is_empty() {
                uit.push(std::mem::take(&mut huidig));
            }
            if c == ',' {
                uit.push(",".to_string());
            }
        } else {
            huidig.push(c);
        }
    }
    if !huidig.is_empty() {
        uit.push(huidig);
    }
    uit.into_iter().map(|w| w.trim_end_matches(['.', '!', '?', ':']).to_string()).filter(|w| !w.is_empty()).collect()
}

/// Namen na een soortwoord: "knop Opslaan", "knoppen Opslaan en Stop", "tekstvak voor de naam".
fn namen_na(w: &[String], i: usize, meervoud: bool) -> Vec<String> {
    let mut namen = Vec::new();
    let mut j = i;
    // "een tekstvak voor de naam" / "met de naam X" / "genaamd X"
    let laag = |k: usize| w.get(k).map(|x| x.to_lowercase()).unwrap_or_default();
    if laag(j) == "voor" && matches!(laag(j + 1).as_str(), "de" | "het" | "je" | "jouw") {
        j += 2;
    } else if laag(j) == "met" && matches!(laag(j + 1).as_str(), "de" | "als") && laag(j + 2) == "naam" {
        j += 3;
    } else if matches!(laag(j).as_str(), "genaamd" | "heet") {
        j += 1;
    }
    loop {
        let mut naam = Vec::new();
        while let Some(x) = w.get(j) {
            let l = x.to_lowercase();
            if x.starts_with('"') {
                naam.push(x.trim_matches('"').to_string());
                j += 1;
                break;
            }
            if STOP.contains(&l.as_str()) || SOORTEN.iter().any(|s| s.0 == l) || l == "venster" || l == "menu" {
                break;
            }
            naam.push(x.clone());
            j += 1;
        }
        if naam.is_empty() {
            break;
        }
        namen.push(hoofdletter(&naam.join(" ")));
        if !meervoud || laag(j) != "en" && laag(j) != "," {
            break;
        }
        j += 1;
    }
    namen
}

/// Eén actie (wat er moet gebeuren) in een zin van de taal. Geeft None als het niet lukt.
fn actie(t: &str, onderdelen: &[(String, String)]) -> Option<Vec<String>> {
    let gewoon = gewone_actie(t);
    let t = gewoon.trim().trim_end_matches(['.', '!']).trim();
    if t.is_empty() {
        return Some(Vec::new());
    }
    let laag = t.to_lowercase();
    let zin = hoofdletter(t);
    let rest = |b: &str| t[b.len()..].trim().to_string();
    let aanhalen = |x: &str| if x.starts_with('"') { x.to_string() } else { format!("\"{}\"", x.trim_matches('"')) };
    // Het onderdeel dat met een naam wordt bedoeld ("de naam" = het tekstvak Naam).
    let onderdeel = |naam: &str| -> Option<&(String, String)> {
        let n = naam.trim().trim_start_matches("de ").trim_start_matches("het ").trim_start_matches("tekst in het tekstvak ").trim().to_lowercase();
        onderdelen.iter().find(|(soort, o)| o.to_lowercase() == n && matches!(soort.as_str(), "tekstvak" | "groot tekstvak" | "label" | "keuzelijst" | "uitklaplijst"))
    };
    // Wat er in het onderdeel staat: de tekst, of de keuze in een keuzelijst.
    let inhoud = |soort: &str, naam: &str| -> String {
        if soort == "keuzelijst" || soort == "uitklaplijst" {
            format!("de keuze in de {} {}", soort, naam)
        } else {
            format!("de tekst in het {} {}", soort, naam)
        }
    };
    match laag.as_str() {
        "zeg hoe laat het is" | "zeg de tijd" | "vertel hoe laat het is" => return Some(vec!["Zeg \"Het is\" en dan de tijd".into()]),
        "zeg de datum" | "zeg welke dag het is" | "vertel de datum" => return Some(vec!["Zeg \"Het is\" en dan de datum".into()]),
        "gooi een dobbelsteen" | "zeg een getal van 1 tot 6" => return Some(vec!["Zeg \"Je gooit\" en dan een willekeurig getal van 1 tot 6".into()]),
        "sluit het programma" | "sluit af" | "stop het programma" => return Some(vec!["Opmerking: afsluiten doe je met Escape of Alt+F4".into()]),
        _ => {}
    }
    for b in ["zeg ", "laat de schermlezer zeggen ", "meld "] {
        if laag.starts_with(b) {
            let x = rest(b);
            return Some(vec![match onderdeel(&x) {
                Some((soort, naam)) => format!("Zeg {}", inhoud(soort, naam)),
                None => format!("Zeg {}", aanhalen(&x)),
            }]);
        }
    }
    for b in ["toon ", "laat zien ", "geef de melding "] {
        if laag.starts_with(b) {
            let x = rest(b);
            return Some(vec![match onderdeel(&x) {
                Some((soort, naam)) => format!("Toon {}", inhoud(soort, naam)),
                None => format!("Toon {}", aanhalen(&x)),
            }]);
        }
    }
    if laag.starts_with("lees ") && laag.ends_with(" voor") {
        let x = t[5..t.len() - 5].trim();
        return Some(vec![match onderdeel(x) {
            Some((soort, naam)) => format!("Lees {} voor", inhoud(soort, naam)),
            None => format!("Lees voor {}", aanhalen(x)),
        }]);
    }
    // schrijf/bewaar/sla de naam op in het bestand x.txt
    for b in ["schrijf ", "bewaar ", "sla "] {
        if laag.starts_with(b) {
            if let Some(p) = laag.rfind(" in ").or_else(|| laag.rfind(" naar ")) {
                let wat = t[b.len()..p].trim().trim_end_matches(" op").to_string();
                let mut bestand = t[p..].trim().splitn(2, ' ').nth(1).unwrap_or("").trim().to_string();
                for v in ["het bestand ", "bestand "] {
                    if bestand.to_lowercase().starts_with(v) {
                        bestand = bestand[v.len()..].to_string();
                    }
                }
                if !bestand.is_empty() {
                    let waarde = match onderdeel(&wat) {
                        Some((soort, naam)) => inhoud(soort, naam),
                        None if wat.is_empty() || wat.to_lowercase() == "alles" => continue,
                        None => aanhalen(&wat),
                    };
                    return Some(vec![format!("Schrijf {} in het bestand {}", waarde, bestand)]);
                }
            }
        }
    }
    if laag.starts_with("maak ") && laag.ends_with(" leeg") {
        if let Some((soort, naam)) = onderdeel(&t[5..t.len() - 5]) {
            return Some(vec![format!("Maak het {} {} leeg", soort, naam)]);
        }
    }
    if laag.starts_with("speel ") || laag.starts_with("zet ") && laag.contains("radio") {
        if laag.contains("radio") || laag.contains("zender") {
            let naam = t.split_whitespace().skip_while(|w| !w.to_lowercase().contains("radio") && !w.to_lowercase().contains("zender")).skip(1).collect::<Vec<_>>().join(" ");
            let naam = naam.trim_start_matches("op ").trim().to_string();
            if !naam.is_empty() {
                return Some(vec![format!("Speel de radiozender {}", naam)]);
            }
        }
        if laag.contains("piep") {
            return Some(vec!["Speel een piep".into()]);
        }
    }
    if ["stop de radio", "zet de radio uit"].contains(&laag.as_str()) {
        return Some(vec!["Stop de radio".into()]);
    }
    if matches!(laag.as_str(), "piep" | "geef een piep" | "laat een piep horen") {
        return Some(vec!["Speel een piep".into()]);
    }
    if laag.starts_with("open ") {
        let x = rest("open ");
        return Some(vec![format!("Open het bestand {}", x.trim_start_matches("het bestand ").trim_start_matches("de webpagina "))]);
    }
    if laag.starts_with("start ") {
        return Some(vec![format!("Start het programma {}", rest("start ").trim_start_matches("het programma "))]);
    }
    if laag.starts_with("wacht ") {
        let x = rest("wacht ");
        if klopt(&format!("Wacht {}", x)) {
            return Some(vec![format!("Wacht {}", x)]);
        }
    }
    // Een gewone zin van de taal.
    if klopt(&zin) && !laag.starts_with("zet ") {
        return Some(vec![zin]);
    }
    // Wat de andere regels al kennen ("zet er een knop bij" e.d.).
    if let Some(crate::wensen::Voorstel::Regels { regels, .. }) = crate::wensen::begrijp_enkel(t) {
        return Some(regels);
    }
    None
}

/// Een actie zoals je die in een omschrijving schrijft, omzetten naar een opdracht:
/// "moet hij opgeslagen zeggen" -> "zeg opgeslagen", "moet de naam in adressen.txt worden bewaard"
/// -> "schrijf de naam in adressen.txt".
fn gewone_actie(t: &str) -> String {
    let mut a = t.trim().trim_end_matches(['.', '!', ',']).trim().to_string();
    loop {
        let l = a.to_lowercase();
        let voor = [
            "dan moet het programma ", "dan moet hij ", "dan moet het ", "moet het programma ", "moet hij ", "moet het ", "moet er ", "moet ",
            "wil ik dat het programma ", "wil ik dat hij ", "het programma moet ", "hij moet ", "dan ", "en dan ", "daarna ", "graag ",
            "het programma ", "hij ", "ik wil dat ",
        ];
        match voor.iter().find(|v| l.starts_with(**v)) {
            Some(v) => a = a[v.len()..].trim().to_string(),
            None => break,
        }
    }
    let l = a.to_lowercase();
    // "de naam moet in x worden bewaard" -> zonder "moet"
    let a2 = a.replace(" moet ", " ").replace(" moeten ", " ");
    let l2 = a2.to_lowercase();
    for eind in [" worden bewaard", " worden opgeslagen", " worden geschreven", " bewaren", " opslaan", " schrijven", " wegschrijven", " bewaard", " opgeslagen"] {
        if l2.ends_with(eind) && (l2.contains(" in ") || l2.contains(" naar ")) {
            return format!("schrijf {}", &a2[..a2.len() - eind.len()]);
        }
    }
    let werkwoorden: &[(&str, &str)] = &[(" zeggen", "zeg "), (" melden", "zeg "), (" laten horen", "zeg "), (" tonen", "toon "), (" laten zien", "toon "), (" zien", "toon "),
        (" stoppen", "stop "), (" uitzetten", "stop "), (" afspelen", "speel "), (" spelen", "speel "), (" openen", "open "), (" starten", "start ")];
    for (eind, begin) in werkwoorden {
        if l.ends_with(eind) {
            return format!("{}{}", begin, &a[..a.len() - eind.len()]);
        }
    }
    if l.ends_with(" voorlezen") {
        return format!("lees {} voor", &a[..a.len() - " voorlezen".len()]);
    }
    if l.ends_with(" leegmaken") {
        return format!("maak {} leeg", &a[..a.len() - " leegmaken".len()]);
    }
    a
}

/// Een omschrijving van een programma, met meer zinnen (bijvoorbeeld uit Kladblok), omzetten in een
/// programma. Eerst wat erin staat, dan wat er gebeurt.
pub fn omschrijving(tekst: &str) -> Option<(Vec<String>, String)> {
    let mut zinnen: Vec<String> = Vec::new();
    let mut huidig = String::new();
    let mut in_tekst = false;
    let tekens: Vec<char> = tekst.chars().collect();
    for (i, &c) in tekens.iter().enumerate() {
        if c == '"' {
            in_tekst = !in_tekst;
        }
        // Een punt is alleen het einde van een zin als er een spatie of nieuwe regel achter staat (niet in adressen.txt).
        let einde_zin = matches!(c, '.' | '!' | '?') && tekens.get(i + 1).map_or(true, |n| n.is_whitespace());
        if !in_tekst && (c == '\n' || c == '\r' || einde_zin) {
            if !huidig.trim().is_empty() {
                zinnen.push(std::mem::take(&mut huidig));
            } else {
                huidig.clear();
            }
            continue;
        }
        huidig.push(c);
    }
    if !huidig.trim().is_empty() {
        zinnen.push(huidig);
    }
    // Opmerkingen en de kop "Dit is een omschrijving voor ..." overslaan.
    let zinnen: Vec<String> = zinnen
        .into_iter()
        .map(|z| z.trim().to_string())
        .filter(|z| {
            let l = z.to_lowercase();
            !l.starts_with("opmerking") && !l.starts_with("dit is een omschrijving") && !l.starts_with("dit is code")
        })
        .collect();
    let (handelingen, opbouw): (Vec<String>, Vec<String>) = zinnen.into_iter().partition(|z| z.to_lowercase().contains("als ik "));
    let samen = format!("{} {}", opbouw.join(", "), handelingen.join(" "));
    // Zonder "als ik op ... druk": misschien zeg je wat het programma moet kunnen.
    if handelingen.is_empty() {
        if let Some(p) = programma_dat_kan(&samen) {
            let eigen_namen = vertaal(&samen).map_or(0, |(r, _)| r.iter().filter(|x| x.starts_with("Maak een ") && !x.starts_with("Maak een venster")).count());
            if eigen_namen < 2 {
                return Some(p);
            }
        }
    }
    vertaal(&samen)
}

/// Knipt de acties: "schrijf ... en zeg opgeslagen" -> ["schrijf ...", "zeg opgeslagen"].
fn acties_los(t: &str) -> Vec<String> {
    let mut stukken = vec![t.to_string()];
    for scheiding in [", en dan ", " en daarna ", ", daarna ", " daarna ", " en dan ", ", en ", "; ", ", ", " en "] {
        stukken = stukken
            .into_iter()
            .flat_map(|s| {
                // Niet knippen binnen aanhalingstekens.
                let mut uit = Vec::new();
                let mut rest = s.as_str();
                loop {
                    let Some(p) = rest.to_lowercase().find(scheiding) else { break };
                    let voor = &rest[..p];
                    if voor.matches('"').count() % 2 == 1 {
                        break;
                    }
                    uit.push(voor.to_string());
                    rest = &rest[p + scheiding.len()..];
                }
                uit.push(rest.to_string());
                uit
            })
            .collect();
    }
    stukken.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Zet een lang verzoek om in zinnen van de taal. Geeft de regels en een uitleg, of None als er
/// geen venster of onderdeel in het verzoek staat.
pub fn vertaal(zin: &str) -> Option<(Vec<String>, String)> {
    let norm = tekst::normaliseer(zin);
    let laag = norm.to_lowercase();
    // Knippen bij "als ik op ... druk" en "als ik de menuregel ... kies".
    let mut grenzen: Vec<usize> = Vec::new();
    let mut zoek = 0;
    while let Some(p) = laag[zoek..].find("als ik ") {
        grenzen.push(zoek + p);
        zoek += p + 7;
    }
    let opbouw = &norm[..grenzen.first().copied().unwrap_or(norm.len())];
    let w = woorden(opbouw);
    let wl: Vec<String> = w.iter().map(|x| x.to_lowercase()).collect();
    let mut regels: Vec<String> = Vec::new();
    let mut onderdelen: Vec<(String, String)> = Vec::new();
    let mut niet = Vec::new();
    // Het venster.
    if let Some(v) = wl.iter().position(|x| x == "venster" || x == "programma" || x == "scherm") {
        let mut j = v + 1;
        if matches!(wl.get(j).map(|s| s.as_str()), Some("met")) && matches!(wl.get(j + 1).map(|s| s.as_str()), Some("de" | "als")) && wl.get(j + 2).map_or(false, |s| s == "titel" || s == "naam") {
            j += 3;
        } else if matches!(wl.get(j).map(|s| s.as_str()), Some("genaamd" | "dat" | "die" | "met")) && !matches!(wl.get(j + 1).map(|s| s.as_str()), Some("een" | "de" | "het")) || matches!(wl.get(j).map(|s| s.as_str()), Some("dat" | "die")) {
            j += 1;
        } else {
            j = usize::MAX;
        }
        if j != usize::MAX {
            let titel = namen_na(&w, j, false);
            if let Some(t) = titel.first() {
                regels.push(format!("Maak een venster met als titel {}", t));
            }
        }
    }
    // Het menu: "een menu Bestand met (de menuregels) Openen en Opslaan".
    if let Some(m) = wl.iter().position(|x| x == "menu") {
        let namen = namen_na(&w, m + 1, false);
        if let Some(menu) = namen.first() {
            regels.push(format!("Maak een menu met als naam {}", menu));
            let mut j = m + 2;
            if wl.get(j).map_or(false, |s| s == "met") {
                j += 1;
                while matches!(wl.get(j).map(|s| s.as_str()), Some("de" | "menuregels" | "menuregel" | "regels" | "keuzes")) {
                    j += 1;
                }
                for r in namen_na(&w, j, true) {
                    regels.push(format!("Maak een menuregel met als naam {}", r));
                    onderdelen.push(("menuregel".into(), r));
                }
            }
        }
    }
    // De onderdelen, in de volgorde waarin je ze noemt.
    let mut i = 0;
    while i < wl.len() {
        let twee = format!("{} {}", wl[i], wl.get(i + 1).cloned().unwrap_or_default());
        let gevonden = SOORTEN.iter().find(|s| s.0 == twee).map(|s| (s, 2)).or_else(|| SOORTEN.iter().find(|s| s.0 == wl[i]).map(|s| (s, 1)));
        if let Some((s, n)) = gevonden {
            let meervoud = s.0.ends_with("en") && !s.0.ends_with("tekstvak");
            let namen = namen_na(&w, i + n, meervoud);
            for naam in &namen {
                regels.push(format!("Maak een {} met als naam {}", s.1, naam));
                onderdelen.push((s.1.to_string(), naam.clone()));
            }
            if namen.is_empty() {
                niet.push(format!("de naam van {} {}", if s.1 == "knop" || s.1 == "keuzelijst" || s.1 == "tabel" { "de" } else { "het" }, s.1));
            }
            i += n;
            continue;
        }
        i += 1;
    }
    if onderdelen.is_empty() && regels.is_empty() {
        return None;
    }
    if !regels.iter().any(|r| r.starts_with("Maak een venster")) {
        regels.insert(0, "Maak een venster met als titel Mijn programma".into());
    }
    // Wat er gebeurt: "als ik op Opslaan druk, ..."
    let mut stukken_tekst = Vec::new();
    for (k, &g) in grenzen.iter().enumerate() {
        let eind = grenzen.get(k + 1).copied().unwrap_or(norm.len());
        stukken_tekst.push(norm[g..eind].trim().trim_end_matches([',', '.']).trim().trim_end_matches(" en").to_string());
    }
    for stuk in stukken_tekst {
        let sl = stuk.to_lowercase();
        let (kop, rest) = if sl.starts_with("als ik op ") {
            let Some(d) = sl.find(" druk").or_else(|| sl.find(" klik")) else {
                niet.push(stuk.clone());
                continue;
            };
            let mut naam = stuk[10..d].trim().to_string();
            for v in ["de knop ", "knop "] {
                if naam.to_lowercase().starts_with(v) {
                    naam = naam[v.len()..].to_string();
                }
            }
            let rest = stuk[d..].splitn(2, ' ').nth(1).unwrap_or("").trim_start_matches(|c: char| c.is_alphabetic()).trim_start_matches([',', ' ']);
            let naam = onderdelen.iter().find(|(_, o)| o.to_lowercase() == naam.to_lowercase()).map(|(_, o)| o.clone()).unwrap_or_else(|| hoofdletter(&naam));
            let kop = if onderdelen.iter().any(|(s, o)| s == "menuregel" && *o == naam) {
                format!("Als ik de menuregel {} kies", naam)
            } else {
                format!("Als ik op de knop {} druk", naam)
            };
            (kop, rest.to_string())
        } else if sl.starts_with("als ik de menuregel ") || sl.starts_with("als ik ") && sl.contains(" kies") {
            let Some(d) = sl.find(" kies") else {
                niet.push(stuk.clone());
                continue;
            };
            let naam = stuk[..d].split_whitespace().skip(2).filter(|x| !matches!(x.to_lowercase().as_str(), "menuregel" | "de" | "het")).collect::<Vec<_>>().join(" ");
            (format!("Als ik de menuregel {} kies", hoofdletter(&naam)), stuk[d + 5..].trim_start_matches([',', ' ']).to_string())
        } else {
            niet.push(stuk.clone());
            continue;
        };
        regels.push(kop);
        for a in acties_los(&rest) {
            match actie(&a, &onderdelen) {
                Some(r) => regels.extend(r),
                None => {
                    regels.push(format!("Opmerking: dit begreep ik niet: {}", a));
                    niet.push(a);
                }
            }
        }
        regels.push(if regels.last().map_or(false, |r| r.starts_with("Als ik de menuregel")) || regels.iter().rev().find(|r| r.starts_with("Als ik")).map_or(false, |r| r.contains("menuregel")) {
            "Einde van de knop".into()
        } else {
            "Einde van de knop".into()
        });
    }
    // Wat niet klopt, wordt een opmerking (zodat je programma toch te controleren is).
    let regels: Vec<String> = regels
        .into_iter()
        .map(|r| if r.starts_with("Opmerking") || klopt(&r) { r } else { format!("Opmerking: dit begreep ik niet: {}", r) })
        .collect();
    let aantal = onderdelen.len();
    let mut uitleg = format!("Ik maak een venster met {} onderdelen{}.", aantal, if grenzen.is_empty() { String::new() } else { " en wat er gebeurt als je erop drukt".into() });
    if !niet.is_empty() {
        uitleg.push_str(&format!(" Dit begreep ik niet, dat staat als opmerking in je programma: {}.", niet.join("; ")));
    }
    Some((regels, uitleg))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn programma(zin: &str) -> String {
        let (regels, _) = vertaal(zin).unwrap_or_else(|| panic!("niets voor: {}", zin));
        let tekst = regels.join("\n") + "\n";
        let u = crate::lees(&tekst);
        assert!(u.is_goed(), "{}\n{:#?}", tekst, u.meldingen);
        tekst
    }

    #[test]
    fn lang_verzoek_wordt_een_programma() {
        let t = programma("Maak een venster met de titel Adressen, met een tekstvak Naam, een tekstvak Telefoon en een knop Opslaan, en als ik op Opslaan druk, schrijf de naam in het bestand adressen.txt en zeg opgeslagen");
        assert!(t.contains("Maak een venster met als titel Adressen\n"), "{}", t);
        assert!(t.contains("Maak een tekstvak met als naam Telefoon\n"), "{}", t);
        assert!(t.contains("Als ik op de knop Opslaan druk\nSchrijf de tekst in het tekstvak Naam in het bestand adressen.txt\nZeg \"opgeslagen\"\nEinde van de knop"), "{}", t);
    }

    #[test]
    fn knoppen_menu_en_radio() {
        let t = programma("ik wil een programma met knoppen Radio 2 en Stop, als ik op Radio 2 druk speel de radiozender NPO Radio 2 en zeg radio 2, als ik op Stop druk stop de radio");
        assert!(t.contains("Maak een knop met als naam Radio 2\nMaak een knop met als naam Stop\n"), "{}", t);
        assert!(t.contains("Speel de radiozender NPO Radio 2"), "{}", t);
        let t = programma("maak een venster genaamd Notities met een groot tekstvak Tekst en een menu Bestand met Openen en Opslaan, als ik Opslaan kies schrijf de tekst in notities.txt");
        assert!(t.contains("Maak een menuregel met als naam Opslaan\n"), "{}", t);
        assert!(t.contains("Als ik de menuregel Opslaan kies\nSchrijf de tekst in het groot tekstvak Tekst in het bestand notities.txt"), "{}", t);
    }

    #[test]
    fn eerlijk_wat_niet_begrepen_is() {
        let (regels, uitleg) = vertaal("maak een venster met een knop Dansen, als ik op Dansen druk laat de computer dansen").unwrap();
        assert!(regels.iter().any(|r| r.starts_with("Opmerking: dit begreep ik niet")));
        assert!(uitleg.contains("begreep ik niet"));
    }
}

#[cfg(test)]
mod omschrijving_tests {
    use super::*;

    #[test]
    fn omschrijving_uit_kladblok() {
        let tekst = "Ik wil een programma dat Adressen heet.\nEr moet een tekstvak Naam in en een knop Opslaan.\nAls ik op Opslaan druk, moet de naam in adressen.txt worden bewaard en moet hij opgeslagen zeggen.\n";
        let (regels, _) = omschrijving(tekst).expect("programma");
        let t = regels.join("\n") + "\n";
        assert!(crate::lees(&t).is_goed(), "{}\n{:#?}", t, crate::lees(&t).meldingen);
        assert!(t.starts_with("Maak een venster met als titel Adressen\n"), "{}", t);
        assert!(t.contains("Schrijf de tekst in het tekstvak Naam in het bestand adressen.txt\nZeg \"opgeslagen\""), "{}", t);
    }
}

// ---------------------------------------------------------------------------------------------
// "Een programma waarmee ik ... kan": wat het programma moet KUNNEN, met vaste recepten.
// ---------------------------------------------------------------------------------------------

/// Een recept: als een van de woorden in je zin staat, komt dit erbij.
struct Recept {
    woorden: &'static [&'static str],
    uitleg: &'static str,
}

const RECEPTEN: &[(&str, Recept)] = &[
    ("tekst", Recept { woorden: &["invoeren", "typen", "intypen", "invullen", "tekst", "invoerveld", "tekstvak", "notitie", "notities", "schrijven", "opschrijven", "brief"], uitleg: "een vak om tekst in te typen" }),
    ("opslaan", Recept { woorden: &["opslaan", "bewaren", "bewaar", "wegschrijven", "sla", "slaan", "opgeslagen", "bewaard", "tekstbestand"], uitleg: "een knop Opslaan (je kiest zelf waar)" }),
    ("openen", Recept { woorden: &["openen", "inladen", "laden", "inlezen", "terughalen", "terug"], uitleg: "een knop Openen" }),
    ("voorlezen", Recept { woorden: &["voorlezen", "voorlees", "uitspreken", "uitgesproken", "horen"], uitleg: "een knop Voorlezen" }),
    ("wissen", Recept { woorden: &["wissen", "leegmaken", "weghalen", "opnieuw"], uitleg: "een knop Wissen" }),
    ("rekenen", Recept { woorden: &["optellen", "rekenen", "uitrekenen", "rekenmachine", "sommen", "som"], uitleg: "twee vakken voor getallen, een knop Optellen en de uitkomst" }),
    ("tijd", Recept { woorden: &["tijd", "klok", "laat"], uitleg: "een knop Tijd die zegt hoe laat het is" }),
    ("radio", Recept { woorden: &["radio", "radiozender", "radiozenders"], uitleg: "een lijst met radiozenders en de knoppen Luisteren en Stop" }),
    ("lijst", Recept { woorden: &["lijst", "lijstje", "boodschappen", "boodschappenlijst", "takenlijst", "taken"], uitleg: "een vak, een knop Toevoegen en een lijst" }),
    ("dobbelsteen", Recept { woorden: &["dobbelsteen", "dobbelen", "gooien"], uitleg: "een knop Gooi die een getal van 1 tot 6 zegt" }),
];

/// Woorden die laten zien dat je zegt wat een programma moet kunnen.
fn beschrijft_een_programma(l: &str) -> bool {
    let programma = ["programma", "programmaatje", "app", "applicatie", "tool", "iets", "venster"].iter().any(|w| l.split(|c: char| !c.is_alphanumeric()).any(|x| x == *w));
    let kunnen = [" waarmee ", " waarin ", " om ", " dat ", " die ", " zodat ", " voor ", " met ", " wat "].iter().any(|w| l.contains(w));
    programma && kunnen
}

/// "Maak een programmaatje waarmee ik tekst kan invoeren en in een tekstbestand kan opslaan."
pub fn programma_dat_kan(zin: &str) -> Option<(Vec<String>, String)> {
    let l = format!(" {} ", zin.to_lowercase().replace([',', '.', '!', '?', ';'], " "));
    if !beschrijft_een_programma(&l) {
        return None;
    }
    let woorden: Vec<&str> = l.split_whitespace().collect();
    let heeft = |w: &str| woorden.iter().any(|x| *x == w);
    let mut gekozen: Vec<&str> = RECEPTEN.iter().filter(|(_, r)| r.woorden.iter().any(|w| heeft(w))).map(|(n, _)| *n).collect();
    // "laat" alleen bij "hoe laat"; "terug" alleen bij een bestand.
    if gekozen.contains(&"tijd") && !l.contains(" tijd ") && !l.contains(" klok ") && !l.contains(" hoe laat ") {
        gekozen.retain(|g| *g != "tijd");
    }
    if gekozen.is_empty() {
        return None;
    }
    // Opslaan, openen, voorlezen en wissen hebben een vak met tekst nodig.
    if ["opslaan", "openen", "voorlezen", "wissen"].iter().any(|g| gekozen.contains(g)) && !gekozen.contains(&"tekst") && !gekozen.contains(&"lijst") {
        gekozen.insert(0, "tekst");
    }
    // Een bestandsnaam die je zelf noemt (notities.txt).
    let bestand = zin.split_whitespace().map(|w| w.trim_matches(|c: char| matches!(c, ',' | '.' | '!' | '?' | '"'))).find(|w| w.contains('.') && w.len() > 4 && w.rsplit('.').next().map_or(false, |e| (2..=4).contains(&e.len()) && e.chars().all(|c| c.is_ascii_alphabetic())));
    // Een klein vak bij een invoerveld, anders een groot tekstvak.
    let klein = heeft("invoerveld") || heeft("veld") || heeft("tekstvak") || heeft("invullen");
    let vak = if klein { "tekstvak" } else { "groot tekstvak" };
    let titel = if gekozen.contains(&"tekst") && gekozen.contains(&"opslaan") {
        "Mijn teksten"
    } else if gekozen.contains(&"rekenen") {
        "Rekenen"
    } else if gekozen.contains(&"radio") {
        "Mijn radio"
    } else if gekozen.contains(&"lijst") {
        "Mijn lijst"
    } else {
        "Mijn programma"
    };
    let mut r: Vec<String> = vec![format!("Maak een venster met als titel {}", titel)];
    let mut acties: Vec<String> = Vec::new();
    let mut knoppen: Vec<&str> = Vec::new();
    for g in &gekozen {
        match *g {
            "tekst" => r.push(format!("Maak een {} met als naam Tekst", vak)),
            "lijst" => {
                r.push("Maak een tekstvak met als naam Nieuw".into());
                r.push("Maak een keuzelijst met als naam Lijst".into());
                knoppen.push("Toevoegen");
                acties.extend([
                    "Als ik op de knop Toevoegen druk",
                    "Vul de keuzelijst Lijst met de tekst in het tekstvak Nieuw",
                    "Maak het tekstvak Nieuw leeg",
                    "Zeg \"Toegevoegd\"",
                    "Einde van de knop",
                ].map(String::from));
            }
            "rekenen" => {
                r.push("Maak een tekstvak met als naam Getal 1".into());
                r.push("Maak een tekstvak met als naam Getal 2".into());
                r.push("Maak een label met als naam Uitkomst".into());
                knoppen.push("Optellen");
                acties.extend([
                    "Als ik op de knop Optellen druk",
                    "Onthoud uitkomst is het getal in het tekstvak Getal 1 plus het getal in het tekstvak Getal 2",
                    "Zet \"De uitkomst is\" en dan uitkomst in het label Uitkomst",
                    "Zeg \"De uitkomst is\" en dan uitkomst",
                    "Einde van de knop",
                ].map(String::from));
            }
            "radio" => {
                r.push("Maak een keuzelijst met als naam Zenders".into());
                r.push("Vul de keuzelijst Zenders met de radiozenders".into());
                knoppen.push("Luisteren");
                knoppen.push("Stop");
                acties.extend([
                    "Als ik op de knop Luisteren druk",
                    "Speel de radiozender die gekozen is in de keuzelijst Zenders",
                    "Einde van de knop",
                    "Als ik op de knop Stop druk",
                    "Stop de radio",
                    "Einde van de knop",
                ].map(String::from));
            }
            "opslaan" => {
                knoppen.push("Opslaan");
                acties.push("Als ik op de knop Opslaan druk".into());
                match bestand {
                    Some(b) => {
                        acties.push(format!("Schrijf de tekst in het tekstvak Tekst in het bestand {}", b));
                        acties.push(format!("Zeg \"Opgeslagen in {}\"", b));
                    }
                    None => {
                        acties.push("Onthoud plek is de plek die ik kies om op te slaan".into());
                        acties.push("Als plek is niet \"\"".into());
                        acties.push(format!("Schrijf de tekst in het tekstvak Tekst in het bestand met de naam plek"));
                        acties.push("Zeg \"Opgeslagen\"".into());
                        acties.push("Einde als".into());
                    }
                }
                acties.push("Einde van de knop".into());
            }
            "openen" => {
                knoppen.push("Openen");
                acties.extend([
                    "Als ik op de knop Openen druk".to_string(),
                    "Onthoud gekozen is een bestand dat ik kies".into(),
                    "Als gekozen is niet \"\"".into(),
                    format!("Zet de inhoud van het bestand met de naam gekozen in het tekstvak Tekst"),
                    "Zeg \"Geopend\"".into(),
                    "Einde als".into(),
                    "Einde van de knop".into(),
                ]);
            }
            "voorlezen" => {
                knoppen.push("Voorlezen");
                acties.extend([
                    "Als ik op de knop Voorlezen druk".to_string(),
                    "Lees de tekst in het tekstvak Tekst voor".to_string(),
                    "Einde van de knop".into(),
                ]);
            }
            "wissen" => {
                knoppen.push("Wissen");
                acties.extend([
                    "Als ik op de knop Wissen druk".to_string(),
                    "Maak het tekstvak Tekst leeg".to_string(),
                    "Zeg \"Gewist\"".into(),
                    "Einde van de knop".into(),
                ]);
            }
            "tijd" => {
                knoppen.push("Tijd");
                acties.extend(["Als ik op de knop Tijd druk", "Zeg \"Het is\" en dan de tijd", "Einde van de knop"].map(String::from));
            }
            "dobbelsteen" => {
                knoppen.push("Gooi");
                acties.extend(["Als ik op de knop Gooi druk", "Zeg \"Je gooit\" en dan een willekeurig getal van 1 tot 6", "Einde van de knop"].map(String::from));
            }
            _ => {}
        }
    }
    for k in &knoppen {
        r.push(format!("Maak een knop met als naam {}", k));
    }
    r.extend(acties);
    let wat: Vec<&str> = gekozen.iter().filter_map(|g| RECEPTEN.iter().find(|(n, _)| n == g).map(|(_, rc)| rc.uitleg)).collect();
    let uitleg = format!("Ik maak een programma met {}. Mist er iets? Zeg het erbij, dan maak ik het opnieuw.", wat.join(", "));
    Some((r, uitleg))
}

#[cfg(test)]
mod recepten_tests {
    use super::*;

    #[test]
    fn jaaps_eigen_vraag() {
        let (r, uitleg) = programma_dat_kan("maak een programmaatje waarmee ik tekst kan invoeren in een invoerveld en waarmee ik het programma dat dan in een tekstbestand kan opslaan").expect("programma");
        let t = r.join("\n") + "\n";
        assert!(crate::lees(&t).is_goed(), "{}\n{:#?}", t, crate::lees(&t).meldingen);
        assert!(t.contains("Maak een tekstvak met als naam Tekst") && t.contains("Maak een knop met als naam Opslaan"), "{}", t);
        assert!(uitleg.contains("Opslaan"));
        for z in [
            "ik wil een programma om notities te schrijven, op te slaan en weer te openen",
            "maak een app waarmee ik kan optellen",
            "een programma waarmee ik naar de radio kan luisteren",
            "maak een programma voor een boodschappenlijst",
            "maak een programmaatje dat tekst kan voorlezen en wissen",
        ] {
            let (r, _) = programma_dat_kan(z).unwrap_or_else(|| panic!("niets voor {}", z));
            let t = r.join("\n") + "\n";
            assert!(crate::lees(&t).is_goed(), "{}\n{}\n{:#?}", z, t, crate::lees(&t).meldingen);
        }
        assert!(programma_dat_kan("sla mijn programma op").is_none());
    }
}
