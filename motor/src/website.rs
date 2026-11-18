//! Websites bouwen met gewone zinnen. Een programma dat begint met "Maak een website" wordt een
//! website: HTML-pagina's met een menu, een stijlblad, en alles toegankelijk (taal, kopjes,
//! beschrijvingen bij afbeeldingen, een link om naar de inhoud te springen, goed zichtbare focus).
//!
//! ```text
//! Maak een website met als titel Jaap Wortel Producties
//! Maak een pagina met als naam Home
//! Zet een kop "Welkom"
//! Zet de tekst "Hier maak ik radio en programma's."
//! Zet een radiospeler met de zender NPO Radio 2
//! Maak een pagina met als naam Contact
//! Zet een e-mailadres info@voorbeeld.nl
//! ```
//!
//! Geen AI: alleen deze vaste zinnen. Een onbekende zin geeft een vraag, geen gok.

use crate::melding::Melding;
use crate::tekst;
use std::path::Path;

#[derive(Clone, Debug, PartialEq)]
pub enum Blok {
    Kop(String),
    Tekst(String),
    Link { naar: String, tekst: String },
    Afbeelding { bestand: String, beschrijving: String },
    Lijst(Vec<String>),
    Radio { naam: String, adres: String },
    Email(String),
    /// Een contactformulier dat een e-mail stuurt naar dit adres (via contact.php).
    Contact(String),
    /// Een link die eruitziet als een knop.
    Knop { naar: String, tekst: String },
    /// Een citaat, met wie het zei.
    Citaat { tekst: String, wie: String },
    /// Een stap in een werkwijze, met kenmerken.
    Stap { titel: String, tekst: String, kenmerken: Vec<String> },
}

#[derive(Clone, Debug, Default)]
pub struct Pagina {
    pub naam: String,
    pub blokken: Vec<Blok>,
}

#[derive(Clone, Debug, Default)]
pub struct Website {
    pub titel: String,
    pub kleur: Option<(String, (u8, u8, u8))>,
    /// Eigen CSS, letterype en grotere letters.
    pub css: Vec<String>,
    pub lettertype: Option<String>,
    pub groter: bool,
    /// Gegevens voor zoekmachines en delen: beschrijving, adres, maker (naam, beroep), plaatje.
    pub beschrijving: Option<String>,
    pub adres: Option<String>,
    pub maker: Option<(String, String)>,
    pub plaatje: Option<String>,
    /// Eigen kleuren als #RRGGBB.
    pub accent: Option<String>,
    pub tekstkleur: Option<String>,
    /// Wat onderaan elke pagina staat.
    pub onderaan: Vec<Blok>,
    /// Een knop in het menu bovenaan (naar, tekst).
    pub menuknop: Option<(String, String)>,
    /// De tekst van de pagina "niet gevonden" (404.html).
    pub foutpagina: Option<String>,
    pub paginas: Vec<Pagina>,
}

/// Een kleur als #RRGGBB: een kleurnaam die het programma kent, of zelf geschreven (#A8521F).
fn kleurcode(t: &str) -> Option<String> {
    let t = t.trim().trim_matches('"').to_lowercase();
    if t.starts_with('#') && t.len() == 7 && t[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Some(t.to_uppercase());
    }
    crate::taal::KLEUREN.iter().find(|k| k.0 == t).map(|(_, (r, g, b))| format!("#{:02X}{:02X}{:02X}", r, g, b))
}

fn rgb_van(code: &str) -> (u8, u8, u8) {
    let d = |i: usize| u8::from_str_radix(&code[i..i + 2], 16).unwrap_or(0);
    (d(1), d(3), d(5))
}

/// "naar de pagina X" of een internetadres.
fn doel(t: &str) -> String {
    let t = t.trim();
    if t.to_lowercase().starts_with("de pagina ") {
        format!("pagina:{}", t[10..].trim())
    } else {
        t.to_string()
    }
}

/// Begint dit programma met "Maak een website"? Dan is het een website.
pub fn is_website(bron: &str) -> bool {
    bron.lines()
        .map(|r| tekst::normaliseer(r).to_lowercase())
        .find(|r| !r.trim().is_empty() && !r.starts_with("opmerking") && !r.starts_with('#') && !r.starts_with("//"))
        .map_or(false, |r| r.starts_with("maak een website"))
}

/// Wat tussen aanhalingstekens staat, of anders de rest van de zin.
fn waarde(rest: &str) -> String {
    let r = rest.trim().trim_end_matches(['.']).trim();
    if let Some(b) = r.find('"') {
        if let Some(e) = r[b + 1..].find('"') {
            return r[b + 1..b + 1 + e].to_string();
        }
    }
    r.to_string()
}

/// Alles tussen aanhalingstekens, of gescheiden door komma's en "en".
fn waarden(rest: &str) -> Vec<String> {
    let delen: Vec<&str> = rest.split('"').collect();
    if delen.len() >= 3 {
        return delen.iter().skip(1).step_by(2).map(|x| x.to_string()).filter(|x| !x.is_empty()).collect();
    }
    rest.split(',').flat_map(|d| d.split(" en ")).map(|x| x.trim().trim_end_matches('.').to_string()).filter(|x| !x.is_empty()).collect()
}

fn na<'a>(norm: &'a str, laag: &str, begin: &str) -> Option<&'a str> {
    laag.starts_with(begin).then(|| norm[begin.len()..].trim())
}

/// Leest een website-programma. Geeft de website en de meldingen (met regelnummer).
pub fn lees(bron: &str) -> (Website, Vec<Melding>) {
    let mut w = Website::default();
    let mut meldingen = Vec::new();
    for (i, ruw) in bron.lines().enumerate() {
        let r = i + 1;
        let norm = tekst::normaliseer(ruw);
        let norm = norm.trim();
        let laag = norm.to_lowercase();
        if norm.is_empty() || laag.starts_with("opmerking") || laag.starts_with('#') || laag.starts_with("//") {
            continue;
        }
        // Een pagina is nodig voor alles behalve de titel en de kleur.
        fn pagina(w: &mut Website) -> &mut Pagina {
            if w.paginas.is_empty() {
                w.paginas.push(Pagina { naam: "Home".into(), blokken: Vec::new() });
            }
            w.paginas.last_mut().unwrap()
        }
        if let Some(t) = na(norm, &laag, "maak een website met als titel").or_else(|| na(norm, &laag, "maak een website met de titel")) {
            if !w.titel.is_empty() {
                meldingen.push(Melding::vraag(r, format!("De website heeft al de titel {}. Welke titel bedoel je?", w.titel)));
            }
            w.titel = waarde(t);
        } else if laag.starts_with("maak een website") {
            meldingen.push(Melding::fout(r, "Hoe heet je website? Schrijf bijvoorbeeld: Maak een website met als titel Jaap Wortel Producties"));
        } else if let Some(t) = na(norm, &laag, "maak een pagina met als naam").or_else(|| na(norm, &laag, "maak een pagina met de naam")) {
            let naam = waarde(t);
            if let Some(eerder) = w.paginas.iter().position(|p| p.naam.to_lowercase() == naam.to_lowercase()) {
                meldingen.push(Melding::vraag(r, format!("Er is al een pagina {} (pagina {}). Twee pagina's met dezelfde naam spreken elkaar tegen. Welke naam moet deze krijgen?", naam, eerder + 1)));
            }
            w.paginas.push(Pagina { naam, blokken: Vec::new() });
        } else if let Some(t) = na(norm, &laag, "zet een kop").or_else(|| na(norm, &laag, "zet een kopje")) {
            pagina(&mut w).blokken.push(Blok::Kop(waarde(t)));
        } else if let Some(t) = na(norm, &laag, "zet de tekst").or_else(|| na(norm, &laag, "zet een alinea")).or_else(|| na(norm, &laag, "zet tekst")) {
            pagina(&mut w).blokken.push(Blok::Tekst(waarde(t)));
        } else if let Some(t) = na(norm, &laag, "zet een link naar") {
            let l = t.to_lowercase();
            let Some(p) = l.find(" met de tekst ") else {
                meldingen.push(Melding::fout(r, "Wat moet er op de link staan? Schrijf bijvoorbeeld: Zet een link naar https://www.nu.nl met de tekst \"Het nieuws\""));
                continue;
            };
            let mut naar = t[..p].trim().to_string();
            if naar.to_lowercase().starts_with("de pagina ") {
                naar = format!("pagina:{}", naar[10..].trim());
            }
            pagina(&mut w).blokken.push(Blok::Link { naar, tekst: waarde(&t[p + 14..]) });
        } else if let Some(t) = na(norm, &laag, "zet een afbeelding") {
            let l = t.to_lowercase();
            let Some(p) = l.find(" met de beschrijving ") else {
                meldingen.push(Melding::fout(r, "Een afbeelding heeft een beschrijving nodig voor de schermlezer. Schrijf bijvoorbeeld: Zet een afbeelding foto.jpg met de beschrijving \"Jaap in de studio\""));
                continue;
            };
            pagina(&mut w).blokken.push(Blok::Afbeelding { bestand: t[..p].trim().to_string(), beschrijving: waarde(&t[p + 21..]) });
        } else if let Some(t) = na(norm, &laag, "zet een lijst met") {
            let items = waarden(t);
            if items.is_empty() {
                meldingen.push(Melding::fout(r, "Wat moet er in de lijst? Schrijf bijvoorbeeld: Zet een lijst met \"Radio\", \"Programma's\" en \"Muziek\""));
                continue;
            }
            pagina(&mut w).blokken.push(Blok::Lijst(items));
        } else if let Some(t) = na(norm, &laag, "zet een radiospeler met de zender").or_else(|| na(norm, &laag, "zet een radiospeler met het adres")) {
            let z = waarde(t);
            let (naam, adres) = if z.starts_with("http") {
                ("Radio".to_string(), z.clone())
            } else {
                match crate::taal::zender(&z) {
                    Some((n, a)) => (n.to_string(), a.to_string()),
                    None => {
                        meldingen.push(Melding::vraag(r, format!("De zender {} ken ik niet. Op een website heb ik het adres van de stream nodig: Zet een radiospeler met het adres https://...", z)));
                        continue;
                    }
                }
            };
            pagina(&mut w).blokken.push(Blok::Radio { naam, adres });
        } else if let Some(t) = na(norm, &laag, "zet een e-mailadres").or_else(|| na(norm, &laag, "zet het e-mailadres")) {
            let a = waarde(t);
            if !a.contains('@') {
                meldingen.push(Melding::fout(r, format!("{} is geen e-mailadres.", a)));
                continue;
            }
            pagina(&mut w).blokken.push(Blok::Email(a));
        } else if let Some(t) = na(norm, &laag, "zet een contactformulier met het e-mailadres").or_else(|| na(norm, &laag, "zet een contactformulier naar")) {
            let a = waarde(t);
            if !a.contains('@') {
                meldingen.push(Melding::fout(r, format!("{} is geen e-mailadres.", a)));
                continue;
            }
            pagina(&mut w).blokken.push(Blok::Contact(a));
        } else if laag.starts_with("zet een contactformulier") {
            meldingen.push(Melding::fout(r, "Naar welk e-mailadres moeten de berichten? Schrijf bijvoorbeeld: Zet een contactformulier met het e-mailadres info@voorbeeld.nl"));
        } else if let Some(t) = na(norm, &laag, "voeg css toe:").or_else(|| na(norm, &laag, "voeg css toe")).or_else(|| na(norm, &laag, "gebruik de css")) {
            w.css.push(waarde(t));
        } else if let Some(t) = na(norm, &laag, "gebruik het lettertype") {
            w.lettertype = Some(waarde(t));
        } else if laag.starts_with("maak de letters groter") || laag.starts_with("maak de letters van de website groter") {
            w.groter = true;
        } else if let Some(t) = na(norm, &laag, "de beschrijving van de website is") {
            w.beschrijving = Some(waarde(t));
        } else if let Some(t) = na(norm, &laag, "het adres van de website is") {
            w.adres = Some(waarde(t).trim_end_matches('/').to_string());
        } else if let Some(t) = na(norm, &laag, "de maker van de website is") {
            let v = waarde(t);
            let (naam, beroep) = v.split_once(',').map(|(a, b)| (a.trim().to_string(), b.trim().to_string())).unwrap_or((v.clone(), String::new()));
            w.maker = Some((naam, beroep));
        } else if let Some(t) = na(norm, &laag, "het plaatje voor delen is") {
            w.plaatje = Some(waarde(t));
        } else if let Some(t) = na(norm, &laag, "geef de website de accentkleur").or_else(|| na(norm, &laag, "geef de knoppen de kleur")) {
            match kleurcode(&waarde(t)) {
                Some(k) => w.accent = Some(k),
                None => meldingen.push(Melding::fout(r, "Die kleur ken ik niet. Schrijf een kleurnaam of een code zoals #A8521F.")),
            }
        } else if let Some(t) = na(norm, &laag, "geef de website de tekstkleur").or_else(|| na(norm, &laag, "geef de tekst de kleur")) {
            match kleurcode(&waarde(t)) {
                Some(k) => w.tekstkleur = Some(k),
                None => meldingen.push(Melding::fout(r, "Die kleur ken ik niet. Schrijf een kleurnaam of een code zoals #2C2416.")),
            }
        } else if let Some(t) = na(norm, &laag, "maak een foutpagina met de tekst").or_else(|| na(norm, &laag, "maak een pagina niet gevonden met de tekst")) {
            w.foutpagina = Some(waarde(t));
        } else if laag.starts_with("maak een foutpagina") {
            w.foutpagina = Some("Deze pagina bestaat niet (meer). Misschien is hij verhuisd.".into());
        } else if let Some(t) = na(norm, &laag, "zet in het menu een knop naar").or_else(|| na(norm, &laag, "zet bovenaan een knop naar")) {
            let l = t.to_lowercase();
            let Some(p) = l.find(" met de tekst ") else {
                meldingen.push(Melding::fout(r, "Wat moet er op de knop staan? Schrijf bijvoorbeeld: Zet in het menu een knop naar de pagina Contact met de tekst \"Vertel jouw verhaal\""));
                continue;
            };
            w.menuknop = Some((doel(&t[..p]), waarde(&t[p + 14..])));
        } else if let Some(t) = na(norm, &laag, "zet een knop naar") {
            let l = t.to_lowercase();
            let Some(p) = l.find(" met de tekst ") else {
                meldingen.push(Melding::fout(r, "Wat moet er op de knop staan? Schrijf bijvoorbeeld: Zet een knop naar de pagina Over mij met de tekst \"Lees meer\""));
                continue;
            };
            pagina(&mut w).blokken.push(Blok::Knop { naar: doel(&t[..p]), tekst: waarde(&t[p + 14..]) });
        } else if let Some(t) = na(norm, &laag, "zet een citaat") {
            let l = t.to_lowercase();
            let (tekst, wie) = match l.rfind(" van ") {
                Some(p) if t[..p].contains('"') => (waarde(&t[..p]), t[p + 5..].trim().trim_end_matches('.').to_string()),
                _ => (waarde(t), String::new()),
            };
            pagina(&mut w).blokken.push(Blok::Citaat { tekst, wie });
        } else if let Some(t) = na(norm, &laag, "zet een stap") {
            // Zet een stap "Luisteren" met de tekst "..." en de kenmerken "Intake", "Op jouw tempo"
            let l = t.to_lowercase();
            let Some(p) = l.find(" met de tekst ") else {
                meldingen.push(Melding::fout(r, "Schrijf het zo: Zet een stap \"Luisteren\" met de tekst \"Eerst praten we.\" en de kenmerken \"Gesprek\" en \"Op jouw tempo\""));
                continue;
            };
            let rest = &t[p + 14..];
            let (tekst, kenmerken) = match rest.to_lowercase().find(" en de kenmerken ") {
                Some(k) => (waarde(&rest[..k]), waarden(&rest[k + 17..])),
                None => (waarde(rest), Vec::new()),
            };
            pagina(&mut w).blokken.push(Blok::Stap { titel: waarde(&t[..p]), tekst, kenmerken });
        } else if let Some(t) = na(norm, &laag, "zet onderaan een link naar") {
            let l = t.to_lowercase();
            let Some(p) = l.find(" met de tekst ") else {
                meldingen.push(Melding::fout(r, "Wat moet er op de link staan? Schrijf bijvoorbeeld: Zet onderaan een link naar de pagina Privacy met de tekst \"Privacy\""));
                continue;
            };
            w.onderaan.push(Blok::Link { naar: doel(&t[..p]), tekst: waarde(&t[p + 14..]) });
        } else if let Some(t) = na(norm, &laag, "zet onderaan een e-mailadres") {
            w.onderaan.push(Blok::Email(waarde(t)));
        } else if let Some(t) = na(norm, &laag, "zet onderaan de tekst").or_else(|| na(norm, &laag, "zet onderaan")) {
            w.onderaan.push(Blok::Tekst(waarde(t)));
        } else if let Some(t) = na(norm, &laag, "geef de website de kleur") {
            let k = waarde(t).to_lowercase();
            match crate::taal::KLEUREN.iter().find(|x| x.0 == k) {
                Some((n, rgb)) => w.kleur = Some((n.to_string(), *rgb)),
                None => meldingen.push(Melding::fout(r, format!("De kleur {} ken ik niet.", k))),
            }
        } else {
            meldingen.push(Melding::vraag(
                r,
                format!(
                    "Deze zin ken ik niet voor een website: \"{}\". Ik gok niet. Ik ken: Maak een pagina, Zet een kop, Zet de tekst, Zet een link naar, Zet een afbeelding, Zet een lijst met, Zet een radiospeler, Zet een e-mailadres, en Geef de website de kleur.",
                    norm
                ),
            ));
        }
    }
    if w.titel.is_empty() && meldingen.is_empty() {
        meldingen.push(Melding::fout(1, "Een website begint met: Maak een website met als titel ..."));
    }
    // Links naar pagina's moeten kloppen.
    let namen: Vec<String> = w.paginas.iter().map(|p| p.naam.to_lowercase()).collect();
    let menu_link = w.menuknop.as_ref().map(|(n, t)| Blok::Link { naar: n.clone(), tekst: t.clone() });
    for b in w.paginas.iter().flat_map(|p| p.blokken.iter()).chain(w.onderaan.iter()).chain(menu_link.iter()) {
        {
            if let Blok::Link { naar, .. } | Blok::Knop { naar, .. } = b {
                if let Some(n) = naar.strip_prefix("pagina:") {
                    if !namen.contains(&n.to_lowercase()) {
                        meldingen.push(Melding::vraag(1, format!("Er is een link naar de pagina {}, maar die pagina bestaat niet. Moet ik die pagina maken, of bedoel je een andere?", n)));
                    }
                }
            }
        }
    }
    meldingen.sort_by_key(|m| m.regel);
    (w, meldingen)
}

/// De bestandsnaam van een pagina: de eerste is index.html.
pub fn bestandsnaam(w: &Website, i: usize) -> String {
    if i == 0 {
        return "index.html".into();
    }
    let s: String = w.paginas[i].naam.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    format!("{}.html", s.trim_matches('-'))
}

/// Waar een link heen gaat: een pagina van de website, of een adres.
fn link_doel(naar: &str, titels: &[(String, String)]) -> String {
    match naar.strip_prefix("pagina:") {
        Some(n) => titels.iter().find(|x| x.0.to_lowercase() == n.to_lowercase()).map(|x| x.1.clone()).unwrap_or_default(),
        None => naar.to_string(),
    }
}

/// Gegevens voor zoekmachines en voor delen op Facebook en dergelijke (zoals op jaapwortelproducties.nl).
fn meta(w: &Website, bestand: &str, titel: &str) -> String {
    let mut h = String::new();
    let adres = w.adres.as_ref().map(|a| if bestand == "index.html" { format!("{}/", a) } else { format!("{}/{}", a, bestand) });
    if let Some(b) = &w.beschrijving {
        h.push_str(&format!("<meta name=\"description\" content=\"{}\">\n", html(b)));
    }
    if let Some(a) = &adres {
        h.push_str(&format!("<link rel=\"canonical\" href=\"{}\">\n", html(a)));
    }
    h.push_str(&format!("<meta property=\"og:title\" content=\"{}\">\n<meta property=\"og:type\" content=\"website\">\n<meta property=\"og:locale\" content=\"nl_NL\">\n", html(titel)));
    if let Some(b) = &w.beschrijving {
        h.push_str(&format!("<meta property=\"og:description\" content=\"{}\">\n", html(b)));
    }
    if let Some(a) = &adres {
        h.push_str(&format!("<meta property=\"og:url\" content=\"{}\">\n", html(a)));
    }
    if let Some(p) = &w.plaatje {
        let volledig = match &w.adres {
            Some(a) if !p.starts_with("http") => format!("{}/{}", a, p.trim_start_matches('/')),
            _ => p.clone(),
        };
        h.push_str(&format!("<meta property=\"og:image\" content=\"{}\">\n<meta name=\"twitter:card\" content=\"summary_large_image\">\n", html(&volledig)));
    }
    if let Some((naam, beroep)) = &w.maker {
        let j = |t: &str| t.replace('\\', "\\\\").replace('"', "\\\"");
        h.push_str(&format!(
            "<script type=\"application/ld+json\">\n{{\"@context\": \"https://schema.org\", \"@type\": \"Person\", \"name\": \"{}\", \"jobTitle\": \"{}\", \"worksFor\": {{\"@type\": \"Organization\", \"name\": \"{}\"}}{}{}}}\n</script>\n",
            j(naam),
            j(beroep),
            j(&w.titel),
            w.adres.as_ref().map(|a| format!(", \"url\": \"{}\"", j(a))).unwrap_or_default(),
            w.beschrijving.as_ref().map(|b| format!(", \"description\": \"{}\"", j(b))).unwrap_or_default()
        ));
    }
    h
}

/// De onderkant van elke pagina: wat je onderaan zet, en anders de titel.
fn onderkant(w: &Website, titels: &[(String, String)]) -> String {
    let mut h = String::from("<footer>\n");
    let links: Vec<&Blok> = w.onderaan.iter().filter(|b| matches!(b, Blok::Link { .. } | Blok::Email(_))).collect();
    for b in &w.onderaan {
        if let Blok::Tekst(t) = b {
            h.push_str(&format!("<p>{}</p>\n", html(t)));
        }
    }
    if !links.is_empty() {
        h.push_str("<ul>\n");
        for b in links {
            match b {
                Blok::Link { naar, tekst } => h.push_str(&format!("<li><a href=\"{}\">{}</a></li>\n", html(&link_doel(naar, titels)), html(tekst))),
                Blok::Email(a) => h.push_str(&format!("<li><a href=\"mailto:{0}\">{0}</a></li>\n", html(a))),
                _ => {}
            }
        }
        h.push_str("</ul>\n");
    }
    if w.onderaan.is_empty() {
        h.push_str(&format!("<p>{}</p>\n", html(&w.titel)));
    }
    h.push_str("</footer>\n");
    h
}

fn html(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Vertelt in gewone taal wat er op de website komt.
pub fn beschrijf(w: &Website) -> String {
    let mut uit = vec![format!("Een website met de titel {}, met {} pagina's. Elke pagina heeft bovenaan een menu met alle pagina's.", w.titel, w.paginas.len())];
    for (i, p) in w.paginas.iter().enumerate() {
        uit.push(format!("Pagina {}: {} (bestand {}).", i + 1, p.naam, bestandsnaam(w, i)));
        for b in &p.blokken {
            uit.push(match b {
                Blok::Kop(t) => format!("  Een kopje: {}", t),
                Blok::Tekst(t) => format!("  Tekst: {}", t),
                Blok::Link { naar, tekst } => format!("  Een link \"{}\" naar {}", tekst, naar.replace("pagina:", "de pagina ")),
                Blok::Afbeelding { bestand, beschrijving } => format!("  Een afbeelding {} met de beschrijving: {}", bestand, beschrijving),
                Blok::Lijst(l) => format!("  Een lijst: {}", l.join(", ")),
                Blok::Radio { naam, .. } => format!("  Een radiospeler voor {} met een knop Afspelen en een knop Stoppen", naam),
                Blok::Email(a) => format!("  Een e-mailadres: {}", a),
                Blok::Knop { naar, tekst } => format!("  Een knop \"{}\" naar {}", tekst, naar.replace("pagina:", "de pagina ")),
                Blok::Citaat { tekst, wie } => format!("  Een citaat: \"{}\"{}", tekst, if wie.is_empty() { String::new() } else { format!(" van {}", wie) }),
                Blok::Stap { titel, tekst, kenmerken } => format!("  Een stap {}: {}{}", titel, tekst, if kenmerken.is_empty() { String::new() } else { format!(" (kenmerken: {})", kenmerken.join(", ")) }),
                Blok::Contact(a) => format!("  Een contactformulier (naam, e-mail, bericht) dat mailt naar {}. Dit werkt alleen op een webruimte met PHP.", a),
            });
        }
    }
    uit.join("\n")
}

/// Maakt de website in een map: index.html, de andere pagina's en stijl.css. Afbeeldingen worden
/// meegekopieerd uit `bronmap` (de map van je programma), als ze daar staan.
pub fn maak(w: &Website, map: &Path, bronmap: Option<&Path>) -> Result<Vec<String>, String> {
    std::fs::create_dir_all(map).map_err(|e| format!("Ik kan de map {} niet maken: {}", map.display(), e))?;
    let mut waarschuwingen = Vec::new();
    let (achter, voor) = match &w.kleur {
        Some((_, k)) => {
            let zwart = crate::taal::contrast(*k, (0, 0, 0)) >= crate::taal::contrast(*k, (255, 255, 255));
            (format!("rgb({},{},{})", k.0, k.1, k.2), if zwart { "#000" } else { "#fff" }.to_string())
        }
        None => ("#ffffff".into(), "#1a1a1a".into()),
    };
    let voor = w.tekstkleur.clone().unwrap_or(voor);
    let accent = w.accent.clone().unwrap_or_else(|| "#1F4E8C".into());
    let op_accent = {
        let k = rgb_van(&accent);
        if crate::taal::contrast(k, (255, 255, 255)) >= 4.5 { "#fff" } else { "#000" }
    };
    if w.accent.is_some() && crate::taal::contrast(rgb_van(&accent), (255, 255, 255)) < 4.5 && crate::taal::contrast(rgb_van(&accent), (0, 0, 0)) < 4.5 {
        waarschuwingen.push(format!("De accentkleur {} geeft weinig verschil met de tekst op de knoppen. Kies een donkerdere of lichtere kleur.", accent));
    }
    let css = format!(
        "/* Gemaakt met Jaap's Programmeertaal. Toegankelijk: grote letters, goed contrast, zichtbare focus. */\n\
body {{ font-family: Segoe UI, Arial, sans-serif; font-size: 1.15rem; line-height: 1.6; margin: 0; background: {achter}; color: {voor}; }}\n\
.knop {{ display: inline-flex; align-items: center; min-height: 44px; padding: .7rem 1.25rem; border-radius: 100px; background: {accent}; color: {op_accent}; text-decoration: none; font-weight: 600; }}\n\
.knop:hover {{ filter: brightness(1.15); }}\n\
blockquote {{ margin: 2rem 0; padding-left: 1.25rem; border-left: 4px solid {accent}; font-size: 1.25rem; font-style: italic; }}\n\
blockquote footer {{ font-size: 1rem; font-style: normal; margin-top: .5rem; }}\n\
.stap {{ margin: 1.5rem 0; padding: 1rem 1.25rem; border: 1px solid currentColor; border-radius: 12px; }}\n\
.stap .nummer {{ font-weight: 700; color: {accent}; margin: 0; }}\n\
.kenmerken {{ list-style: none; padding: 0; display: flex; flex-wrap: wrap; gap: .5rem; }}\n\
.kenmerken li {{ border: 1px solid currentColor; border-radius: 100px; padding: .1rem .75rem; font-size: .95rem; }}\n\
footer ul {{ list-style: none; padding: 0; display: flex; flex-wrap: wrap; gap: .5rem 1.5rem; }}\n\
a {{ color: inherit; text-decoration: underline; }}\n\
.overslaan {{ position: absolute; left: -999px; }}\n\
.overslaan:focus {{ left: 1rem; top: 1rem; background: #ffeb3b; color: #000; padding: .5rem; z-index: 9; }}\n\
header, main, footer {{ max-width: 48rem; margin: 0 auto; padding: 1rem; }}\n\
nav ul {{ list-style: none; padding: 0; display: flex; flex-wrap: wrap; gap: .5rem 1.5rem; }}\n\
nav a[aria-current=page] {{ font-weight: bold; }}\n\
:focus-visible {{ outline: 4px solid #ffbf00; outline-offset: 3px; }}\n\
img {{ max-width: 100%; height: auto; }}\n\
button {{ font-size: 1.1rem; padding: .5rem 1rem; margin-right: .5rem; }}\n\
@media (prefers-reduced-motion: reduce) {{ * {{ transition: none !important; }} }}\n"
    );
    let mut css = css;
    if let Some(l) = &w.lettertype {
        css.push_str(&format!("body {{ font-family: \"{}\", Segoe UI, Arial, sans-serif; }}\n", l.replace('"', "")));
    }
    if w.groter {
        css.push_str("body { font-size: 1.45rem; }\n");
    }
    for c in &w.css {
        css.push_str(&format!("/* Eigen CSS */\n{}\n", c));
    }
    std::fs::write(map.join("stijl.css"), css).map_err(|e| e.to_string())?;
    // Een contactformulier: contact.php verstuurt het bericht (alleen op een webruimte met PHP).
    if let Some(adres) = w.paginas.iter().flat_map(|p| p.blokken.iter()).find_map(|b| if let Blok::Contact(a) = b { Some(a.clone()) } else { None }) {
        let php = format!(
            "<?php\n// Gemaakt met Jaap's Programmeertaal. Stuurt het contactformulier naar {adres}.\n\
$naam = trim(strip_tags($_POST['naam'] ?? ''));\n$van = filter_var(trim($_POST['email'] ?? ''), FILTER_VALIDATE_EMAIL);\n\
$bericht = trim(strip_tags($_POST['bericht'] ?? ''));\n$gelukt = false;\n\
if ($_SERVER['REQUEST_METHOD'] === 'POST' && $naam !== '' && $van && $bericht !== '' && empty($_POST['website'])) {{\n\
    $kop = 'From: {adres}' . \"\\r\\nReply-To: \" . $van . \"\\r\\nContent-Type: text/plain; charset=utf-8\";\n\
    $gelukt = mail('{adres}', 'Bericht via de website van ' . str_replace([\"\\r\", \"\\n\"], ' ', $naam), $bericht, $kop);\n}}\n?>\n\
<!DOCTYPE html>\n<html lang=\"nl\">\n<head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>Bericht verstuurd - {titel}</title><link rel=\"stylesheet\" href=\"stijl.css\"></head>\n<body><main id=\"inhoud\">\n\
<h1><?php echo $gelukt ? 'Bedankt, je bericht is verstuurd.' : 'Je bericht is niet verstuurd. Vul alles in en probeer het opnieuw.'; ?></h1>\n\
<p><a href=\"index.html\">Terug naar de website</a></p>\n</main></body></html>\n",
            adres = adres.replace('\'', ""),
            titel = html(&w.titel)
        );
        std::fs::write(map.join("contact.php"), php).map_err(|e| e.to_string())?;
        waarschuwingen.push("Het contactformulier werkt alleen op een webruimte met PHP (de meeste webhosts hebben dat). In Firefox op je eigen computer kun je het wel zien, maar niet versturen.".into());
    }
    let titels: Vec<(String, String)> = (0..w.paginas.len()).map(|i| (w.paginas[i].naam.clone(), bestandsnaam(w, i))).collect();
    for (i, p) in w.paginas.iter().enumerate() {
        let mut h = String::new();
        h.push_str("<!DOCTYPE html>\n<html lang=\"nl\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        h.push_str(&format!("<title>{} - {}</title>\n", html(&p.naam), html(&w.titel)));
        h.push_str(&meta(w, &bestandsnaam(w, i), &format!("{} - {}", p.naam, w.titel)));
        h.push_str("<link rel=\"stylesheet\" href=\"stijl.css\">\n</head>\n<body>\n");
        h.push_str("<a class=\"overslaan\" href=\"#inhoud\">Naar de inhoud</a>\n<header>\n");
        h.push_str(&format!("<p><strong>{}</strong></p>\n", html(&w.titel)));
        if titels.len() > 1 {
            h.push_str("<nav aria-label=\"Menu\"><ul>\n");
            for (j, (naam, bestand)) in titels.iter().enumerate() {
                let huidig = if j == i { " aria-current=\"page\"" } else { "" };
                h.push_str(&format!("<li><a href=\"{}\"{}>{}</a></li>\n", bestand, huidig, html(naam)));
            }
            h.push_str("</ul></nav>\n");
        }
        if let Some((naar, tekst)) = &w.menuknop {
            h.push_str(&format!("<p><a class=\"knop\" href=\"{}\">{}</a></p>\n", html(&link_doel(naar, &titels)), html(tekst)));
        }
        h.push_str("</header>\n<main id=\"inhoud\">\n");
        let heeft_kop = p.blokken.iter().any(|b| matches!(b, Blok::Kop(_)));
        if !heeft_kop {
            h.push_str(&format!("<h1>{}</h1>\n", html(&p.naam)));
        }
        let mut eerste_kop = true;
        let mut radio = 0;
        let mut stap = 0;
        for b in &p.blokken {
            match b {
                Blok::Kop(t) => {
                    h.push_str(&format!("<{0}>{1}</{0}>\n", if eerste_kop { "h1" } else { "h2" }, html(t)));
                    eerste_kop = false;
                }
                Blok::Tekst(t) => h.push_str(&format!("<p>{}</p>\n", html(t))),
                Blok::Link { naar, tekst } => {
                    h.push_str(&format!("<p><a href=\"{}\">{}</a></p>\n", html(&link_doel(naar, &titels)), html(tekst)));
                }
                Blok::Knop { naar, tekst } => {
                    h.push_str(&format!("<p><a class=\"knop\" href=\"{}\">{}</a></p>\n", html(&link_doel(naar, &titels)), html(tekst)));
                }
                Blok::Citaat { tekst, wie } => {
                    h.push_str(&format!("<blockquote><p>“{}”</p>", html(tekst)));
                    if !wie.is_empty() {
                        h.push_str(&format!("<footer>— {}</footer>", html(wie)));
                    }
                    h.push_str("</blockquote>\n");
                }
                Blok::Stap { titel, tekst, kenmerken } => {
                    stap += 1;
                    h.push_str(&format!("<section class=\"stap\" aria-labelledby=\"stap{0}\">\n<p class=\"nummer\">{0:02}</p>\n<h3 id=\"stap{0}\">{1}</h3>\n<p>{2}</p>\n", stap, html(titel), html(tekst)));
                    if !kenmerken.is_empty() {
                        h.push_str("<ul class=\"kenmerken\">\n");
                        for k in kenmerken {
                            h.push_str(&format!("<li>{}</li>\n", html(k)));
                        }
                        h.push_str("</ul>\n");
                    }
                    h.push_str("</section>\n");
                }
                Blok::Afbeelding { bestand, beschrijving } => {
                    let naam = Path::new(bestand).file_name().map(|x| x.to_string_lossy().to_string()).unwrap_or_else(|| bestand.clone());
                    let bron = bronmap.map(|m| m.join(bestand)).filter(|p| p.is_file()).or_else(|| Some(Path::new(bestand).to_path_buf()).filter(|p| p.is_file()));
                    match bron {
                        Some(b) => {
                            let _ = std::fs::copy(b, map.join(&naam));
                        }
                        None => waarschuwingen.push(format!("De afbeelding {} heb ik niet gevonden. Zet hem in de map van de website.", bestand)),
                    }
                    h.push_str(&format!("<p><img src=\"{}\" alt=\"{}\"></p>\n", html(&naam), html(beschrijving)));
                }
                Blok::Lijst(l) => {
                    h.push_str("<ul>\n");
                    for x in l {
                        h.push_str(&format!("<li>{}</li>\n", html(x)));
                    }
                    h.push_str("</ul>\n");
                }
                Blok::Radio { naam, adres } => {
                    radio += 1;
                    h.push_str(&format!(
                        "<section aria-label=\"Radio {0}\">\n<h2>{0}</h2>\n<audio id=\"radio{1}\" src=\"{2}\" preload=\"none\"></audio>\n\
<button type=\"button\" aria-label=\"Afspelen {0}\" onclick=\"var r=document.getElementById('radio{1}');r.play();document.getElementById('stand{1}').textContent='{0} speelt.';\">Afspelen</button>\n\
<button type=\"button\" aria-label=\"Stoppen {0}\" onclick=\"var r=document.getElementById('radio{1}');r.pause();document.getElementById('stand{1}').textContent='Gestopt.';\">Stoppen</button>\n\
<p id=\"stand{1}\" role=\"status\" aria-live=\"polite\"></p>\n</section>\n",
                        html(naam),
                        radio,
                        html(adres)
                    ));
                }
                Blok::Email(a) => h.push_str(&format!("<p>E-mail: <a href=\"mailto:{0}\">{0}</a></p>\n", html(a))),
                Blok::Contact(_) => h.push_str(
                    "<form action=\"contact.php\" method=\"post\" aria-label=\"Contactformulier\">\n\
<p><label for=\"naam\">Je naam</label><br><input id=\"naam\" name=\"naam\" autocomplete=\"name\" required></p>\n\
<p><label for=\"email\">Je e-mailadres</label><br><input id=\"email\" name=\"email\" type=\"email\" autocomplete=\"email\" required></p>\n\
<p><label for=\"bericht\">Je bericht</label><br><textarea id=\"bericht\" name=\"bericht\" rows=\"6\" required></textarea></p>\n\
<p hidden><label for=\"website\">Laat dit leeg</label><input id=\"website\" name=\"website\" tabindex=\"-1\" autocomplete=\"off\"></p>\n\
<p><button type=\"submit\">Versturen</button></p>\n</form>\n",
                ),
            }
        }
        h.push_str("</main>\n");
        h.push_str(&onderkant(w, &titels));
        h.push_str("</body>\n</html>\n");
        std::fs::write(map.join(bestandsnaam(w, i)), h).map_err(|e| e.to_string())?;
    }
    // De pagina "niet gevonden" (404.html), zoals op jaapwortelproducties.nl.
    if let Some(t) = &w.foutpagina {
        let mut h = String::from("<!DOCTYPE html>\n<html lang=\"nl\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        h.push_str(&format!("<title>Pagina niet gevonden - {}</title>\n<meta name=\"robots\" content=\"noindex\">\n<link rel=\"stylesheet\" href=\"/stijl.css\">\n</head>\n<body>\n", html(&w.titel)));
        h.push_str(&format!("<main id=\"inhoud\">\n<h1>Pagina niet gevonden</h1>\n<p>{}</p>\n<p><a class=\"knop\" href=\"/\">Naar de beginpagina</a></p>\n</main>\n", html(t)));
        h.push_str(&onderkant(w, &titels));
        h.push_str("</body>\n</html>\n");
        std::fs::write(map.join("404.html"), h).map_err(|e| e.to_string())?;
        // Zo laat een webserver (Apache) deze pagina zien als een pagina niet bestaat.
        std::fs::write(map.join(".htaccess"), "ErrorDocument 404 /404.html\n").map_err(|e| e.to_string())?;
    }
    Ok(waarschuwingen)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VOORBEELD: &str = "Maak een website met als titel Jaap Wortel Producties\nMaak een pagina met als naam Home\nZet een kop \"Welkom\"\nZet de tekst \"Ik maak radio & programma's.\"\nZet een radiospeler met de zender NPO Radio 2\nZet een link naar de pagina Contact met de tekst \"Neem contact op\"\nMaak een pagina met als naam Contact\nZet een e-mailadres info@voorbeeld.nl\nZet een lijst met \"Radio\", \"Programma's\" en \"Muziek\"\n";

    #[test]
    fn website_maken() {
        assert!(is_website(VOORBEELD));
        assert!(!is_website("Maak een venster met als titel A\n"));
        let (w, m) = lees(VOORBEELD);
        assert!(m.is_empty(), "{:#?}", m);
        assert_eq!(w.paginas.len(), 2);
        let map = std::env::temp_dir().join(format!("jaaptaal-website-{}", std::process::id()));
        maak(&w, &map, None).unwrap();
        let index = std::fs::read_to_string(map.join("index.html")).unwrap();
        assert!(index.contains("<html lang=\"nl\">"));
        assert!(index.contains("<h1>Welkom</h1>"));
        assert!(index.contains("radio &amp; programma's"));
        assert!(index.contains("https://icecast.omroep.nl/radio2-bb-mp3"));
        assert!(index.contains("<a href=\"contact.html\">Neem contact op</a>"));
        assert!(index.contains("aria-current=\"page\">Home"));
        let contact = std::fs::read_to_string(map.join("contact.html")).unwrap();
        assert!(contact.contains("mailto:info@voorbeeld.nl") && contact.contains("<li>Muziek</li>"));
        assert!(map.join("stijl.css").is_file());
        let _ = std::fs::remove_dir_all(&map);
    }

    #[test]
    fn niet_gokken_op_een_website() {
        let (_, m) = lees("Maak een website met als titel A\nZet een dansende eenhoorn\n");
        assert_eq!(m[0].regel, 2);
        let (_, m) = lees("Maak een website met als titel A\nZet een afbeelding foto.jpg\n");
        assert!(m[0].tekst.contains("beschrijving"));
        let (_, m) = lees("Maak een website met als titel A\nZet een link naar de pagina Weg met de tekst \"x\"\n");
        assert!(m[0].tekst.contains("bestaat niet"));
    }
}
