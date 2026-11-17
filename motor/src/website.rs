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
    pub paginas: Vec<Pagina>,
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
    for p in &w.paginas {
        for b in &p.blokken {
            if let Blok::Link { naar, .. } = b {
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
    let css = format!(
        "/* Gemaakt met Jaap's Programmeertaal. Toegankelijk: grote letters, goed contrast, zichtbare focus. */\n\
body {{ font-family: Segoe UI, Arial, sans-serif; font-size: 1.15rem; line-height: 1.6; margin: 0; background: {achter}; color: {voor}; }}\n\
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
    std::fs::write(map.join("stijl.css"), css).map_err(|e| e.to_string())?;
    let titels: Vec<(String, String)> = (0..w.paginas.len()).map(|i| (w.paginas[i].naam.clone(), bestandsnaam(w, i))).collect();
    for (i, p) in w.paginas.iter().enumerate() {
        let mut h = String::new();
        h.push_str("<!DOCTYPE html>\n<html lang=\"nl\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        h.push_str(&format!("<title>{} - {}</title>\n<link rel=\"stylesheet\" href=\"stijl.css\">\n</head>\n<body>\n", html(&p.naam), html(&w.titel)));
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
        h.push_str("</header>\n<main id=\"inhoud\">\n");
        let heeft_kop = p.blokken.iter().any(|b| matches!(b, Blok::Kop(_)));
        if !heeft_kop {
            h.push_str(&format!("<h1>{}</h1>\n", html(&p.naam)));
        }
        let mut eerste_kop = true;
        let mut radio = 0;
        for b in &p.blokken {
            match b {
                Blok::Kop(t) => {
                    h.push_str(&format!("<{0}>{1}</{0}>\n", if eerste_kop { "h1" } else { "h2" }, html(t)));
                    eerste_kop = false;
                }
                Blok::Tekst(t) => h.push_str(&format!("<p>{}</p>\n", html(t))),
                Blok::Link { naar, tekst } => {
                    let doel = match naar.strip_prefix("pagina:") {
                        Some(n) => titels.iter().find(|x| x.0.to_lowercase() == n.to_lowercase()).map(|x| x.1.clone()).unwrap_or_default(),
                        None => naar.clone(),
                    };
                    h.push_str(&format!("<p><a href=\"{}\">{}</a></p>\n", html(&doel), html(tekst)));
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
            }
        }
        h.push_str(&format!("</main>\n<footer><p>{}</p></footer>\n</body>\n</html>\n", html(&w.titel)));
        std::fs::write(map.join(bestandsnaam(w, i)), h).map_err(|e| e.to_string())?;
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
