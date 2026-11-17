// ======================================================================
// Looptijd van Jaap's Programmeertaal.
// Dit stuk gaat mee in elk programma dat je bouwt. Het zorgt voor rekenen,
// vergelijken, tonen, vensters en knoppen. Het gebruikt alleen de standaard
// Rust-bibliotheek en Windows zelf. Geen AI, geen internet, geen extra pakketten.
//
// Starten met --tekst: alles komt als tekst in de uitvoer (voor proeven).
// Met --druk Naam drukt de proef daarna op de knop Naam.
// ======================================================================
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(dead_code, unused_variables, unused_mut, unreachable_code, unused_imports, non_snake_case, unused_braces, unused_parens)]

#[derive(Clone, Debug)]
enum W {
    G(f64),
    T(String),
    B(bool),
    L(Vec<W>),
}

/// Waar een naam staat: in het hele programma (G) of alleen in een eigen stukje (L).
#[derive(Clone, Copy)]
enum P {
    G(usize),
    L(usize),
}

struct Fout {
    regel: usize,
    tekst: String,
}

type R<T> = Result<T, Fout>;

fn fout<T>(regel: usize, tekst: String) -> R<T> {
    Err(Fout { regel, tekst })
}

fn schrijf_getal(g: f64) -> String {
    if g.fract() == 0.0 && g.abs() < 1e15 {
        return format!("{}", g as i64);
    }
    let mut s = format!("{:.10}", g);
    while s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    s.replace('.', ",")
}

impl W {
    fn tekst(&self) -> String {
        match self {
            W::G(g) => schrijf_getal(*g),
            W::T(t) => t.clone(),
            W::B(true) => "waar".to_string(),
            W::B(false) => "niet waar".to_string(),
            W::L(l) if l.is_empty() => "een lege lijst".to_string(),
            W::L(l) => l.iter().map(|w| w.tekst()).collect::<Vec<_>>().join(", "),
        }
    }
    fn soort(&self) -> &'static str {
        match self {
            W::G(_) => "een getal",
            W::T(_) => "tekst",
            W::B(_) => "waar of niet waar",
            W::L(_) => "een lijst",
        }
    }
}

const MAX_STAPPEN: u64 = 10_000_000;
/// De knop Doe het van het opdrachtvak.
const OPDRACHT_ACTIE: usize = usize::MAX;
const MAX_HERHALINGEN: f64 = 1_000_000.0;

fn reken(a: W, teken: u8, b: W, r: usize) -> R<W> {
    if teken == 4 {
        let (x, y) = (a.tekst(), b.tekst());
        return Ok(W::T(if x.is_empty() || y.is_empty() { format!("{}{}", x, y) } else { format!("{} {}", x, y) }));
    }
    let woord = ["plus", "min", "keer", "gedeeld door"][teken as usize];
    let (x, y) = match (&a, &b) {
        (W::G(x), W::G(y)) => (*x, *y),
        (W::G(_), ander) | (ander, _) => {
            return fout(r, format!("Rekenen ({}) kan alleen met getallen, maar hier staat {}.", woord, ander.soort()))
        }
    };
    let uit = match teken {
        0 => x + y,
        1 => x - y,
        2 => x * y,
        _ => {
            if y == 0.0 {
                return fout(r, "Delen door nul kan niet.".to_string());
            }
            x / y
        }
    };
    if !uit.is_finite() {
        return fout(r, "Het getal wordt te groot om mee te rekenen.".to_string());
    }
    Ok(W::G(uit))
}

fn negatief(a: W, r: usize) -> R<W> {
    match a {
        W::G(g) => Ok(W::G(-g)),
        ander => fout(r, format!("Min ervoor kan alleen bij een getal, maar hier staat {}.", ander.soort())),
    }
}

fn gelijk(a: &W, b: &W) -> bool {
    match (a, b) {
        (W::G(x), W::G(y)) => x == y,
        (W::T(x), W::T(y)) => x.to_lowercase() == y.to_lowercase(),
        (W::B(x), W::B(y)) => x == y,
        (W::L(x), W::L(y)) => x.len() == y.len() && x.iter().zip(y).all(|(p, q)| gelijk(p, q)),
        _ => false,
    }
}

fn lijst_van(a: W, r: usize) -> R<Vec<W>> {
    match a {
        W::L(l) => Ok(l),
        ander => fout(r, format!("Hier verwacht ik een lijst, maar hier staat {}.", ander.soort())),
    }
}

fn aantal_in(a: W, r: usize) -> R<W> {
    Ok(W::G(lijst_van(a, r)?.len() as f64))
}

fn element(a: W, i: W, r: usize) -> R<W> {
    let l = lijst_van(a, r)?;
    let n = match i {
        W::G(g) if g.fract() == 0.0 => g as i64,
        ander => return fout(r, format!("Het nummer in de lijst moet een heel getal zijn, en niet {}.", ander.tekst())),
    };
    let plek = if n < 0 { l.len() as i64 + n } else { n - 1 };
    if l.is_empty() {
        return fout(r, "De lijst is leeg. Er staat nog niets in.".to_string());
    }
    if plek < 0 || plek >= l.len() as i64 {
        return fout(r, format!("De lijst heeft {} dingen. Nummer {} bestaat dus niet.", l.len(), n));
    }
    Ok(l[plek as usize].clone())
}

fn bevat(a: W, b: W, r: usize) -> R<bool> {
    Ok(lijst_van(a, r)?.iter().any(|x| gelijk(x, &b)))
}

/// De getallen van ... tot en met ..., met een grens.
fn telling(a: W, b: W, r: usize) -> R<std::ops::RangeInclusive<i64>> {
    let heel = |w: W| -> R<i64> {
        match w {
            W::G(g) if g.fract() == 0.0 => Ok(g as i64),
            ander => fout(r, format!("Tellen gaat met hele getallen, en niet met {}.", ander.tekst())),
        }
    };
    let (van, tot) = (heel(a)?, heel(b)?);
    if tot - van > MAX_HERHALINGEN as i64 {
        return fout(r, "Zo ver tellen mag niet. Het kan hoogstens een miljoen keer.".to_string());
    }
    Ok(van..=tot)
}

// ---------------------------------------------------------------------
// Bestanden, logboek, tijd en willekeurige getallen.
// ---------------------------------------------------------------------

/// De map waar bestanden zonder volledig pad komen: Documenten (of JAAPTAAL_MAP voor proeven).
fn basismap() -> std::path::PathBuf {
    if let Some(m) = std::env::var_os("JAAPTAAL_MAP") {
        return m.into();
    }
    #[cfg(windows)]
    {
        if let Some(m) = win::documenten() {
            return m;
        }
    }
    std::env::current_dir().unwrap_or_default()
}

fn volledig_pad(p: &W) -> std::path::PathBuf {
    let t = p.tekst();
    let pad = std::path::PathBuf::from(t.trim());
    if pad.is_absolute() {
        pad
    } else {
        basismap().join(pad)
    }
}

fn lees_tekstbestand(pad: &std::path::Path) -> std::io::Result<String> {
    let b = std::fs::read(pad)?;
    let b = b.strip_prefix(&[0xEF, 0xBB, 0xBF][..]).unwrap_or(&b);
    Ok(match String::from_utf8(b.to_vec()) {
        Ok(s) => s,
        Err(_) => b.iter().map(|&c| c as char).collect(),
    })
}

fn schrijf_bestand(p: W, w: W, toevoegen: bool, r: usize) -> R<()> {
    let pad = volledig_pad(&p);
    let mut tekst = w.tekst().replace("\r\n", "\n").replace('\n', "\r\n");
    let resultaat = if toevoegen {
        use std::io::Write;
        let oud = std::fs::read(&pad).unwrap_or_default();
        std::fs::OpenOptions::new().create(true).append(true).open(&pad).and_then(|mut f| {
            if oud.is_empty() {
                f.write_all(&[0xEF, 0xBB, 0xBF])?;
            } else if !oud.ends_with(b"\n") {
                // De laatste regel had nog geen einde: eerst een nieuwe regel.
                f.write_all(b"\r\n")?;
            }
            tekst.push_str("\r\n");
            f.write_all(tekst.as_bytes())
        })
    } else {
        let mut inhoud = vec![0xEF, 0xBB, 0xBF];
        inhoud.extend_from_slice(tekst.as_bytes());
        std::fs::write(&pad, inhoud)
    };
    resultaat.or_else(|e| fout(r, format!("Ik kan niet schrijven in {}. Reden: {}", pad.display(), e)))
}

fn maak_map(p: W, r: usize) -> R<()> {
    let pad = volledig_pad(&p);
    std::fs::create_dir_all(&pad).or_else(|e| fout(r, format!("Ik kan de map {} niet maken. Reden: {}", pad.display(), e)))
}

/// Datum en tijd van nu: (jaar, maand, dag, uur, minuut, seconde). Op Windows in de tijd van de computer.
fn nu() -> (u32, u32, u32, u32, u32, u32) {
    #[cfg(windows)]
    {
        return win::nu();
    }
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let dagen = (s / 86400) as i64;
    let rest = s % 86400;
    // Van dagen sinds 1970 naar een datum (algoritme van Howard Hinnant).
    let z = dagen + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y as u32, m as u32, d as u32, (rest / 3600) as u32, (rest % 3600 / 60) as u32, (rest % 60) as u32)
}

fn logboekpad() -> std::path::PathBuf {
    let naam = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "programma".into());
    if let Some(m) = std::env::var_os("JAAPTAAL_MAP") {
        return std::path::PathBuf::from(m).join(format!("{} logboek.txt", naam));
    }
    let basis = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from).unwrap_or_else(|| basismap());
    basis.join(&naam).join("logboek.txt")
}

/// Schrijft een regel in het logboek, met datum en tijd ervoor.
fn logboek(t: &str) {
    use std::io::Write;
    let (j, mn, d, u, mi, se) = nu();
    let pad = logboekpad();
    if let Some(m) = pad.parent() {
        let _ = std::fs::create_dir_all(m);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&pad) {
        let _ = write!(f, "{:02}-{:02}-{:04} {:02}:{:02}:{:02}  {}\r\n", d, mn, j, u, mi, se, t.replace('\n', " "));
    }
}

thread_local! {
    static TOEVAL: std::cell::Cell<u64> = std::cell::Cell::new(0);
}

/// Een willekeurig getal (xorshift), elke keer anders omdat het met de klok begint.
fn toeval() -> u64 {
    TOEVAL.with(|t| {
        let mut x = t.get();
        if x == 0 {
            x = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos() as u64).unwrap_or(1) | 1;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        t.set(x);
        x
    })
}

fn ingebouwd(wat: u8, invoer: Vec<W>, r: usize) -> R<W> {
    match wat {
        0 | 1 => {
            let pad = volledig_pad(&invoer[0]);
            let tekst = lees_tekstbestand(&pad).or_else(|e| fout(r, format!("Ik kan het bestand {} niet lezen. Reden: {}", pad.display(), e)))?;
            if wat == 0 {
                Ok(W::T(tekst.replace("\r\n", "\n").trim_end_matches('\n').to_string()))
            } else {
                Ok(W::L(tekst.lines().map(|l| W::T(l.to_string())).collect()))
            }
        }
        2 => Ok(W::B(volledig_pad(&invoer[0]).exists())),
        3 => Ok(W::T(logboekpad().display().to_string())),
        4 => {
            let heel = |w: &W| -> R<i64> {
                match w {
                    W::G(g) if g.fract() == 0.0 => Ok(*g as i64),
                    ander => fout(r, format!("Een willekeurig getal kan alleen tussen hele getallen, en niet {}.", ander.tekst())),
                }
            };
            let (a, b) = (heel(&invoer[0])?, heel(&invoer[1])?);
            if b < a {
                return fout(r, format!("Van {} tot {} kan niet: het eerste getal moet het kleinste zijn.", a, b));
            }
            let breedte = (b - a + 1) as u64;
            Ok(W::G((a + (toeval() % breedte) as i64) as f64))
        }
        5 => {
            let (_, _, _, u, m, _) = nu();
            Ok(W::T(format!("{:02}:{:02}", u, m)))
        }
        6 => {
            let (j, m, d, _, _, _) = nu();
            Ok(W::T(format!("{:02}-{:02}-{:04}", d, m, j)))
        }
        _ => {
            let adres = invoer[0].tekst();
            let data = internet("GET", &adres, None).or_else(|e| fout(r, e))?;
            let tekst = String::from_utf8_lossy(&data).to_string();
            Ok(W::T(if wat == 7 { tekst } else { leesbare_tekst(&tekst) }))
        }
    }
}

/// Tekst lezen als getal: 12, 3,5, 1.000 of -2.
fn tekst_als_getal(t: &str) -> Option<f64> {
    let t = t.trim().replace(' ', "");
    let schoon = if t.contains(',') {
        t.replace('.', "").replace(',', ".")
    } else if t.matches('.').count() == 1 && t.split('.').nth(1).map_or(0, |d| d.len()) != 3 {
        t.clone()
    } else {
        t.replace('.', "")
    };
    schoon.parse::<f64>().ok().filter(|g| g.is_finite() && !t.is_empty())
}

/// Rekenen en tekst: wortel, macht, afronden, lengte, hoofdletters, ...
fn functie(wat: u8, invoer: Vec<W>, r: usize) -> R<W> {
    let getal = |i: usize| -> R<f64> {
        match &invoer[i] {
            W::G(g) => Ok(*g),
            ander => fout(r, format!("Hier moet een getal staan, maar het is {}: {}", ander.soort(), ander.tekst())),
        }
    };
    let heel = |i: usize, wat: &str| -> R<usize> {
        let g = getal(i)?;
        if g < 0.0 || g.fract() != 0.0 || g > 1_000_000.0 {
            return fout(r, format!("{} moet een heel getal zijn van 0 of meer, en niet {}.", wat, schrijf_getal(g)));
        }
        Ok(g as usize)
    };
    let tekst = |i: usize| invoer[i].tekst();
    let klaar = |g: f64| -> R<W> {
        if g.is_finite() {
            Ok(W::G(g))
        } else {
            fout(r, "Het getal wordt te groot om mee te rekenen.".to_string())
        }
    };
    match wat {
        12 => {
            let g = getal(0)?;
            if g < 0.0 {
                return fout(r, format!("De wortel van een getal onder nul ({}) kan niet.", schrijf_getal(g)));
            }
            klaar(g.sqrt())
        }
        13 => klaar(getal(0)?.powf(getal(1)?)),
        14 => {
            let n = heel(1, "Het aantal cijfers achter de komma")?.min(10) as i32;
            let f = 10f64.powi(n);
            klaar((getal(0)? * f).round() / f)
        }
        15 => {
            let d = getal(1)?;
            if d == 0.0 {
                return fout(r, "Delen door nul kan niet.".to_string());
            }
            klaar(getal(0)? % d)
        }
        16 => klaar(getal(0)?.abs()),
        17 => match &invoer[0] {
            W::G(g) => Ok(W::G(*g)),
            w => match tekst_als_getal(&w.tekst()) {
                Some(g) => Ok(W::G(g)),
                None if w.tekst().trim().is_empty() => fout(r, "Er staat nog niets, dus ik kan er geen getal van maken.".to_string()),
                None => fout(r, format!("Hier staat geen getal, maar: {}", w.tekst().trim())),
            },
        },
        18 => Ok(W::T(tekst(0))),
        19 => Ok(W::G(tekst(0).chars().count() as f64)),
        20 => Ok(W::T(tekst(0).to_uppercase())),
        21 => Ok(W::T(tekst(0).to_lowercase())),
        22 | 23 => {
            let t: Vec<char> = tekst(0).chars().collect();
            let n = heel(1, "Het aantal tekens")?.min(t.len());
            let deel: String = if wat == 22 { t[..n].iter().collect() } else { t[t.len() - n..].iter().collect() };
            Ok(W::T(deel))
        }
        24 => Ok(W::L(tekst(0).split_whitespace().map(|w| W::T(w.trim_matches(|c: char| c.is_ascii_punctuation()).to_string())).filter(|w| !w.tekst().is_empty()).collect())),
        25 => {
            let bij = tekst(1);
            if bij.is_empty() {
                return fout(r, "Waarbij moet ik de tekst in stukken hakken? Dat teken is leeg.".to_string());
            }
            Ok(W::L(tekst(0).split(bij.as_str()).map(|d| W::T(d.trim().to_string())).collect()))
        }
        26 => match &invoer[0] {
            W::L(l) => Ok(W::T(l.iter().map(|w| w.tekst()).collect::<Vec<_>>().join(&tekst(1)))),
            ander => fout(r, format!("Samenvoegen kan alleen met een lijst, maar dit is {}.", ander.soort())),
        },
        27 => {
            let zoek = tekst(1);
            if zoek.is_empty() {
                return fout(r, "Wat moet ik vervangen? Die tekst is leeg.".to_string());
            }
            Ok(W::T(tekst(0).replace(zoek.as_str(), &tekst(2))))
        }
        29 => {
            let pad = volledig_pad(&invoer[0]);
            let data = std::fs::read(&pad).or_else(|e| fout(r, format!("Ik kan het bestand {} niet lezen. Reden: {}", pad.display(), e)))?;
            let laag = pad.to_string_lossy().to_lowercase();
            let uit = if laag.ends_with(".pdf") { pdf_lezen(&data) } else { docx_lezen(&data) };
            match uit {
                Some(t) => Ok(W::T(t)),
                None if laag.ends_with(".pdf") => fout(r, format!("Uit het PDF-bestand {} kan ik geen tekst halen. Misschien is het een scan (een plaatje van tekst), of gebruikt het een lettertype dat ik niet kan lezen.", pad.display())),
                None => fout(r, format!("{} is geen Word-bestand (DOCX) dat ik kan lezen. Oude .doc-bestanden kan ik niet lezen.", pad.display())),
            }
        }
        _ => Ok(W::T(tekst(0).trim().to_string())),
    }
}

/// Een QR-code maken (byte-stand, foutcorrectie M, versie 1 tot 10: tot 213 tekens).
/// Geeft een rooster: true = zwart vakje. Zelf gemaakt volgens de QR-norm, zonder hulpprogramma's.
fn qr_code(tekst: &[u8]) -> Option<Vec<Vec<bool>>> {
    const EC: [usize; 11] = [0, 10, 16, 26, 18, 24, 16, 18, 22, 22, 26];
    const BLOKKEN: [usize; 11] = [0, 1, 1, 1, 2, 2, 4, 4, 4, 5, 5];
    let ruwe = |v: usize| -> usize {
        let mut r = (16 * v + 128) * v + 64;
        if v >= 2 {
            let n = v / 7 + 2;
            r -= (25 * n - 10) * n - 55;
            if v >= 7 {
                r -= 36;
            }
        }
        r
    };
    let data_cw = |v: usize| ruwe(v) / 8 - EC[v] * BLOKKEN[v];
    let versie = (1..=10).find(|&v| 4 + (if v < 10 { 8 } else { 16 }) + tekst.len() * 8 <= data_cw(v) * 8)?;
    // Bits: stand 0100, lengte, de bytes, eindteken en opvulling.
    let mut bits: Vec<bool> = Vec::new();
    let mut zet = |w: u32, n: u32, bits: &mut Vec<bool>| {
        for i in (0..n).rev() {
            bits.push((w >> i) & 1 == 1);
        }
    };
    zet(4, 4, &mut bits);
    zet(tekst.len() as u32, if versie < 10 { 8 } else { 16 }, &mut bits);
    for &b in tekst {
        zet(b as u32, 8, &mut bits);
    }
    let capaciteit = data_cw(versie) * 8;
    let eind = (capaciteit - bits.len()).min(4) as u32;
    zet(0, eind, &mut bits);
    let rest = (8 - bits.len() % 8) % 8;
    zet(0, rest as u32, &mut bits);
    let mut vul = 0xEC;
    while bits.len() < capaciteit {
        zet(vul, 8, &mut bits);
        vul ^= 0xEC ^ 0x11;
    }
    let data: Vec<u8> = bits.chunks(8).map(|c| c.iter().fold(0u8, |a, &b| (a << 1) | b as u8)).collect();
    // Reed-Solomon in GF(256).
    let maal = |x: u8, y: u8| -> u8 {
        let mut z: u32 = 0;
        for i in (0..8).rev() {
            z = (z << 1) ^ ((z >> 7) * 0x11D);
            z ^= ((y as u32 >> i) & 1) * x as u32;
        }
        z as u8
    };
    let graad = EC[versie];
    let mut deler = vec![0u8; graad];
    deler[graad - 1] = 1;
    let mut wortel = 1u8;
    for _ in 0..graad {
        for j in 0..graad {
            deler[j] = maal(deler[j], wortel);
            if j + 1 < graad {
                deler[j] ^= deler[j + 1];
            }
        }
        wortel = maal(wortel, 0x02);
    }
    let rest_van = |d: &[u8]| -> Vec<u8> {
        let mut r = vec![0u8; graad];
        for &b in d {
            let f = b ^ r[0];
            r.remove(0);
            r.push(0);
            for (x, &y) in r.iter_mut().zip(&deler) {
                *x ^= maal(y, f);
            }
        }
        r
    };
    // Blokken maken en door elkaar zetten.
    let aantal = BLOKKEN[versie];
    let totaal = ruwe(versie) / 8;
    let kort = aantal - totaal % aantal;
    let kort_len = totaal / aantal;
    let mut blokken: Vec<Vec<u8>> = Vec::new();
    let mut k = 0;
    for i in 0..aantal {
        let n = kort_len - graad + if i < kort { 0 } else { 1 };
        let mut blok = data[k..k + n].to_vec();
        k += n;
        let ec = rest_van(&blok);
        if i < kort {
            blok.push(0);
        }
        blok.extend(ec);
        blokken.push(blok);
    }
    let mut alle = Vec::new();
    for i in 0..blokken[0].len() {
        for (j, b) in blokken.iter().enumerate() {
            if i != kort_len - graad || j >= kort {
                alle.push(b[i]);
            }
        }
    }
    // Het rooster.
    let maat = versie * 4 + 17;
    let mut zw = vec![vec![false; maat]; maat];
    let mut vast = vec![vec![false; maat]; maat];
    let mut zetv = |x: usize, y: usize, z: bool, zw: &mut Vec<Vec<bool>>, vast: &mut Vec<Vec<bool>>| {
        zw[y][x] = z;
        vast[y][x] = true;
    };
    for i in 0..maat {
        zetv(6, i, i % 2 == 0, &mut zw, &mut vast);
        zetv(i, 6, i % 2 == 0, &mut zw, &mut vast);
    }
    for &(cx, cy) in &[(3i32, 3i32), (maat as i32 - 4, 3), (3, maat as i32 - 4)] {
        for dy in -4..=4i32 {
            for dx in -4..=4i32 {
                let (x, y) = (cx + dx, cy + dy);
                if x >= 0 && y >= 0 && (x as usize) < maat && (y as usize) < maat {
                    let d = dx.abs().max(dy.abs());
                    zetv(x as usize, y as usize, d != 2 && d != 4, &mut zw, &mut vast);
                }
            }
        }
    }
    if versie >= 2 {
        let n = versie / 7 + 2;
        let stap = if versie == 32 { 26 } else { (versie * 4 + n * 2 + 1) / (n * 2 - 2) * 2 };
        let mut pos = vec![6usize];
        let mut p = maat - 7;
        let mut tussen = Vec::new();
        for _ in 0..n - 1 {
            tussen.push(p);
            p -= stap;
        }
        tussen.reverse();
        pos.extend(tussen);
        for (i, &a) in pos.iter().enumerate() {
            for (j, &b) in pos.iter().enumerate() {
                if (i == 0 && j == 0) || (i == 0 && j == n - 1) || (i == n - 1 && j == 0) {
                    continue;
                }
                for dy in -2..=2i32 {
                    for dx in -2..=2i32 {
                        zetv((a as i32 + dx) as usize, (b as i32 + dy) as usize, dx.abs().max(dy.abs()) != 1, &mut zw, &mut vast);
                    }
                }
            }
        }
    }
    // Opmaak (foutcorrectie M = 00, masker 0) en versie-informatie.
    let masker = 0u32;
    let mut f = masker;
    let mut r = f;
    for _ in 0..10 {
        r = (r << 1) ^ ((r >> 9) * 0x537);
    }
    f = ((f << 10) | r) ^ 0x5412;
    let bit = |w: u32, i: u32| (w >> i) & 1 == 1;
    for i in 0..6 {
        zetv(8, i, bit(f, i as u32), &mut zw, &mut vast);
    }
    zetv(8, 7, bit(f, 6), &mut zw, &mut vast);
    zetv(8, 8, bit(f, 7), &mut zw, &mut vast);
    zetv(7, 8, bit(f, 8), &mut zw, &mut vast);
    for i in 9..15 {
        zetv(14 - i, 8, bit(f, i as u32), &mut zw, &mut vast);
    }
    for i in 0..8 {
        zetv(maat - 1 - i, 8, bit(f, i as u32), &mut zw, &mut vast);
    }
    for i in 8..15 {
        zetv(8, maat - 15 + i, bit(f, i as u32), &mut zw, &mut vast);
    }
    zetv(8, maat - 8, true, &mut zw, &mut vast);
    if versie >= 7 {
        let mut r = versie as u32;
        for _ in 0..12 {
            r = (r << 1) ^ ((r >> 11) * 0x1F25);
        }
        let v = ((versie as u32) << 12) | r;
        for i in 0..18 {
            let (a, b) = (maat - 11 + i % 3, i / 3);
            zetv(a, b, bit(v, i as u32), &mut zw, &mut vast);
            zetv(b, a, bit(v, i as u32), &mut zw, &mut vast);
        }
    }
    // De gegevens in een zigzag, met masker 0 ((x + y) even = omdraaien).
    let mut i = 0usize;
    let mut rechts = maat as i32 - 1;
    while rechts >= 1 {
        if rechts == 6 {
            rechts = 5;
        }
        for vert in 0..maat {
            for j in 0..2 {
                let x = (rechts - j) as usize;
                let omhoog = (rechts + 1) & 2 == 0;
                let y = if omhoog { maat - 1 - vert } else { vert };
                if !vast[y][x] {
                    let b = i < alle.len() * 8 && (alle[i >> 3] >> (7 - (i & 7))) & 1 == 1;
                    zw[y][x] = b ^ ((x + y) % 2 == 0);
                    i += 1;
                }
            }
        }
        rechts -= 2;
    }
    Some(zw)
}

/// Een zwart-wit PNG-bestand van een rooster (zonder compressie, wel een geldig PNG).
fn qr_png(rooster: &[Vec<bool>], vak: usize) -> Vec<u8> {
    let rand = 4;
    let n = (rooster.len() + 2 * rand) * vak;
    let mut ruw = Vec::with_capacity((n + 1) * n);
    for y in 0..n {
        ruw.push(0u8);
        for x in 0..n {
            let (rx, ry) = ((x / vak) as isize - rand as isize, (y / vak) as isize - rand as isize);
            let zwart = rx >= 0 && ry >= 0 && (rx as usize) < rooster.len() && (ry as usize) < rooster.len() && rooster[ry as usize][rx as usize];
            ruw.push(if zwart { 0 } else { 255 });
        }
    }
    let mut z = vec![0x78u8, 0x01];
    for (k, stuk) in ruw.chunks(65535).enumerate() {
        let laatste = (k + 1) * 65535 >= ruw.len();
        z.push(laatste as u8);
        z.extend_from_slice(&(stuk.len() as u16).to_le_bytes());
        z.extend_from_slice(&(!(stuk.len() as u16)).to_le_bytes());
        z.extend_from_slice(stuk);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &x in &ruw {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    z.extend_from_slice(&((b << 16) | a).to_be_bytes());
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut stuk = |soort: &[u8], data: &[u8], png: &mut Vec<u8>| {
        png.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let mut c = soort.to_vec();
        c.extend_from_slice(data);
        png.extend_from_slice(&c);
        png.extend_from_slice(&crc32(&c).to_be_bytes());
    };
    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&(n as u32).to_be_bytes());
    ihdr.extend_from_slice(&(n as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 0, 0, 0, 0]);
    stuk(b"IHDR", &ihdr, &mut png);
    stuk(b"IDAT", &z, &mut png);
    stuk(b"IEND", &[], &mut png);
    png
}

fn maak_qr(waarde: W, pad: W, r: usize) -> R<()> {
    let t = waarde.tekst();
    if t.is_empty() {
        return fout(r, "Een QR-code van lege tekst kan niet.".to_string());
    }
    let Some(rooster) = qr_code(t.as_bytes()) else {
        return fout(r, format!("Deze tekst is te lang voor een QR-code ({} tekens; hoogstens 213).", t.len()));
    };
    let p = volledig_pad(&pad);
    if let Some(m) = p.parent() {
        let _ = std::fs::create_dir_all(m);
    }
    std::fs::write(&p, qr_png(&rooster, 8)).or_else(|e| fout(r, format!("Ik kan {} niet schrijven: {}", p.display(), e)))
}

/// Een radiozender opzoeken bij radio-browser.info (de meest beluisterde met deze naam).
fn zoek_zender(naam: &str) -> Option<String> {
    let codeer = |t: &str| -> String {
        t.bytes().map(|b| if b.is_ascii_alphanumeric() { (b as char).to_string() } else { format!("%{:02X}", b) }).collect()
    };
    for server in ["https://de1.api.radio-browser.info", "https://nl1.api.radio-browser.info", "https://at1.api.radio-browser.info"] {
        for soort in ["bynameexact", "byname"] {
            let adres = format!("{}/json/stations/{}/{}?hidebroken=true&order=clickcount&reverse=true&limit=1", server, soort, codeer(naam));
            let Ok(data) = internet("GET", &adres, None) else { continue };
            let tekst = String::from_utf8_lossy(&data).to_string();
            if let Some(p) = tekst.find("\"url_resolved\":\"") {
                let rest = &tekst[p + 16..];
                if let Some(e) = rest.find('"') {
                    let u = rest[..e].replace("\\/", "/");
                    if u.starts_with("http") {
                        return Some(u);
                    }
                }
            }
        }
    }
    None
}

/// Een reservekopie in de map Reservekopieën naast het bestand, met datum en tijd. De laatste 10 blijven.
fn reservekopie(p: W, r: usize) -> R<()> {
    let pad = volledig_pad(&p);
    if !pad.is_file() {
        return fout(r, format!("Het bestand {} bestaat niet, dus ik kan er geen reservekopie van maken.", pad.display()));
    }
    let map = pad.parent().map(|m| m.join("Reservekopieën")).unwrap_or_else(|| "Reservekopieën".into());
    std::fs::create_dir_all(&map).or_else(|e| fout(r, format!("Ik kan de map {} niet maken: {}", map.display(), e)))?;
    let stam = pad.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let ext = pad.extension().map(|e| format!(".{}", e.to_string_lossy())).unwrap_or_default();
    let (j, m, d, u, mi, se) = nu();
    let mut doel = map.join(format!("{} {:04}-{:02}-{:02} {:02}.{:02}.{:02}{}", stam, j, m, d, u, mi, se, ext));
    let mut n = 2;
    while doel.exists() {
        doel = map.join(format!("{} {:04}-{:02}-{:02} {:02}.{:02}.{:02} ({}){}", stam, j, m, d, u, mi, se, n, ext));
        n += 1;
    }
    std::fs::copy(&pad, &doel).or_else(|e| fout(r, format!("De reservekopie is niet gelukt: {}", e)))?;
    // De oudste weg, zodat er hoogstens 10 zijn.
    let mut kopieen: Vec<std::path::PathBuf> = std::fs::read_dir(&map)
        .map(|l| l.flatten().map(|e| e.path()).filter(|x| x.file_name().map_or(false, |f| { let f = f.to_string_lossy(); f.starts_with(&format!("{} ", stam)) && f.ends_with(&ext) })).collect())
        .unwrap_or_default();
    kopieen.sort();
    while kopieen.len() > 10 {
        let _ = std::fs::remove_file(kopieen.remove(0));
    }
    Ok(())
}

/// Waar een geluid klinkt, in woorden (zelfde als in de motor).
fn richting_naam(r: u8) -> String {
    let (h, d, v) = (r % 3, (r / 3) % 3, r / 9);
    let mut delen = Vec::new();
    match h {
        1 => delen.push("links"),
        2 => delen.push("rechts"),
        _ => {}
    }
    match d {
        1 => delen.push("achter je"),
        2 => delen.push("voor je"),
        _ => {}
    }
    match v {
        1 => delen.push("boven je"),
        2 => delen.push("onder je"),
        _ => {}
    }
    if delen.is_empty() {
        delen.push("in het midden");
    }
    delen.join(", ")
}

/// Een WAV-geluid ruimtelijk maken: links, rechts, achter of voor je. Eenvoudig, zonder HRTF:
/// links/rechts met het volume per oor (en een heel kleine vertraging), achter je wat zachter en
/// doffer. Geeft een nieuw WAV-bestand (16 bit, stereo) in het geheugen.
fn ruimtelijk_geluid(data: &[u8], richting: u8) -> Result<Vec<u8>, String> {
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err("is geen gewoon WAV-bestand.".into());
    }
    let (mut kanalen, mut freq, mut bits, mut vorm) = (0u16, 0u32, 0u16, 0u16);
    let mut monsters: Option<&[u8]> = None;
    let mut p = 12;
    while p + 8 <= data.len() {
        let id = &data[p..p + 4];
        let len = u32::from_le_bytes(data[p + 4..p + 8].try_into().unwrap()) as usize;
        let body = &data[p + 8..(p + 8 + len).min(data.len())];
        if id == b"fmt " && body.len() >= 16 {
            vorm = u16::from_le_bytes([body[0], body[1]]);
            kanalen = u16::from_le_bytes([body[2], body[3]]);
            freq = u32::from_le_bytes(body[4..8].try_into().unwrap());
            bits = u16::from_le_bytes([body[14], body[15]]);
        } else if id == b"data" {
            monsters = Some(body);
        }
        p += 8 + len + (len & 1);
    }
    let Some(m) = monsters else { return Err("heeft geen geluid.".into()) };
    if !(vorm == 1 || vorm == 0xFFFE) || bits != 16 || kanalen == 0 || kanalen > 2 {
        return Err("moet een gewoon WAV-bestand zijn met 16 bits, mono of stereo, voor links, rechts of achter.".into());
    }
    // Eerst alles naar mono, als getallen tussen -1 en 1.
    let k = kanalen as usize;
    let mono: Vec<f32> = m
        .chunks_exact(2 * k)
        .map(|c| (0..k).map(|i| i16::from_le_bytes([c[2 * i], c[2 * i + 1]]) as f32).sum::<f32>() / (k as f32 * 32768.0))
        .collect();
    // Volume per oor, vertraging in monsters voor het verre oor (ongeveer 0,6 ms).
    let vertraging = (freq as f32 * 0.0006) as usize;
    let (h, d, v) = (richting % 3, (richting / 3) % 3, richting / 9);
    let (mut links, mut rechts, vert_l, vert_r) = match h {
        1 => (0.95f32, 0.30f32, 0, vertraging),
        2 => (0.30, 0.95, vertraging, 0),
        _ => (0.707, 0.707, 0, 0),
    };
    // Achter je: zachter en doffer. Onder je: nog doffer. Boven je: helderder.
    let dof = d == 1 || v == 2;
    let sterk_dof = v == 2;
    let helder = v == 1;
    if d == 1 {
        links *= 0.8;
        rechts *= 0.8;
    }
    let lengte = mono.len() + vertraging;
    let mut l = vec![0f32; lengte];
    let mut rr = vec![0f32; lengte];
    let mut vorige = 0f32;
    for (i, &x) in mono.iter().enumerate() {
        // Achter je: hoge tonen zachter (een eenvoudig laagdoorlaatfilter).
        let x = if dof {
            vorige += if sterk_dof { 0.12 } else { 0.25 } * (x - vorige);
            vorige
        } else if helder {
            // Hoge tonen harder: het verschil met het vorige monster erbij.
            let y = x + 0.8 * (x - vorige);
            vorige = x;
            y * 0.8
        } else {
            x
        };
        l[i + vert_l] += x * links;
        rr[i + vert_r] += x * rechts;
    }
    let mut uit = Vec::with_capacity(44 + lengte * 4);
    let grootte = (lengte * 4) as u32;
    uit.extend_from_slice(b"RIFF");
    uit.extend_from_slice(&(36 + grootte).to_le_bytes());
    uit.extend_from_slice(b"WAVEfmt ");
    uit.extend_from_slice(&16u32.to_le_bytes());
    uit.extend_from_slice(&1u16.to_le_bytes());
    uit.extend_from_slice(&2u16.to_le_bytes());
    uit.extend_from_slice(&freq.to_le_bytes());
    uit.extend_from_slice(&(freq * 4).to_le_bytes());
    uit.extend_from_slice(&4u16.to_le_bytes());
    uit.extend_from_slice(&16u16.to_le_bytes());
    uit.extend_from_slice(b"data");
    uit.extend_from_slice(&grootte.to_le_bytes());
    for i in 0..lengte {
        for v in [l[i], rr[i]] {
            uit.extend_from_slice(&((v.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
        }
    }
    Ok(uit)
}

/// Uitpakken van "deflate" (zoals in zip, docx en pdf). Zelf gemaakt, zonder hulpprogramma's.
fn inflate(data: &[u8]) -> Option<Vec<u8>> {
    struct Bits<'a> {
        d: &'a [u8],
        pos: usize,
        bit: u32,
        n: u32,
    }
    impl Bits<'_> {
        fn lees(&mut self, aantal: u32) -> Option<u32> {
            while self.n < aantal {
                let b = *self.d.get(self.pos)? as u32;
                self.pos += 1;
                self.bit |= b << self.n;
                self.n += 8;
            }
            let v = self.bit & ((1u32 << aantal) - 1);
            self.bit >>= aantal;
            self.n -= aantal;
            Some(v)
        }
    }
    // Een Huffman-tabel: aantal codes per lengte, en de symbolen op volgorde.
    fn maak(lengtes: &[u8]) -> (Vec<u16>, Vec<u16>) {
        let mut tel = vec![0u16; 16];
        for &l in lengtes {
            tel[l as usize] += 1;
        }
        tel[0] = 0;
        let mut begin = vec![0u16; 16];
        for i in 1..16 {
            begin[i] = begin[i - 1] + tel[i - 1];
        }
        let mut sym = vec![0u16; lengtes.len()];
        for (s, &l) in lengtes.iter().enumerate() {
            if l != 0 {
                sym[begin[l as usize] as usize] = s as u16;
                begin[l as usize] += 1;
            }
        }
        (tel, sym)
    }
    fn decodeer(b: &mut Bits, t: &(Vec<u16>, Vec<u16>)) -> Option<u16> {
        let (mut code, mut eerste, mut index) = (0i32, 0i32, 0i32);
        for len in 1..16 {
            code |= b.lees(1)? as i32;
            let aantal = t.0[len] as i32;
            if code - aantal < eerste {
                return t.1.get((index + code - eerste) as usize).copied();
            }
            index += aantal;
            eerste += aantal;
            eerste <<= 1;
            code <<= 1;
        }
        None
    }
    const LBASIS: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
    const LEXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
    const DBASIS: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
    const DEXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
    let mut b = Bits { d: data, pos: 0, bit: 0, n: 0 };
    let mut uit: Vec<u8> = Vec::new();
    loop {
        let laatste = b.lees(1)?;
        let soort = b.lees(2)?;
        match soort {
            0 => {
                b.bit = 0;
                b.n = 0;
                let len = u16::from_le_bytes([*data.get(b.pos)?, *data.get(b.pos + 1)?]) as usize;
                b.pos += 4;
                uit.extend_from_slice(data.get(b.pos..b.pos + len)?);
                b.pos += len;
            }
            1 | 2 => {
                let (lit, afst) = if soort == 1 {
                    let mut l = [0u8; 288];
                    for (i, x) in l.iter_mut().enumerate() {
                        *x = if i < 144 { 8 } else if i < 256 { 9 } else if i < 280 { 7 } else { 8 };
                    }
                    (maak(&l), maak(&[5u8; 30]))
                } else {
                    let hlit = b.lees(5)? as usize + 257;
                    let hdist = b.lees(5)? as usize + 1;
                    let hclen = b.lees(4)? as usize + 4;
                    const ORDE: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
                    let mut cl = [0u8; 19];
                    for &o in ORDE.iter().take(hclen) {
                        cl[o] = b.lees(3)? as u8;
                    }
                    let clt = maak(&cl);
                    let mut lengtes = Vec::with_capacity(hlit + hdist);
                    while lengtes.len() < hlit + hdist {
                        let s = decodeer(&mut b, &clt)?;
                        match s {
                            0..=15 => lengtes.push(s as u8),
                            16 => {
                                let vorige = *lengtes.last()?;
                                for _ in 0..3 + b.lees(2)? {
                                    lengtes.push(vorige);
                                }
                            }
                            17 => lengtes.extend(std::iter::repeat(0).take(3 + b.lees(3)? as usize)),
                            _ => lengtes.extend(std::iter::repeat(0).take(11 + b.lees(7)? as usize)),
                        }
                    }
                    (maak(&lengtes[..hlit]), maak(&lengtes[hlit..hlit + hdist]))
                };
                loop {
                    let s = decodeer(&mut b, &lit)? as usize;
                    if s < 256 {
                        uit.push(s as u8);
                    } else if s == 256 {
                        break;
                    } else {
                        let i = s - 257;
                        let len = *LBASIS.get(i)? as usize + b.lees(*LEXTRA.get(i)? as u32)? as usize;
                        let d = decodeer(&mut b, &afst)? as usize;
                        let afstand = *DBASIS.get(d)? as usize + b.lees(*DEXTRA.get(d)? as u32)? as usize;
                        if afstand > uit.len() {
                            return None;
                        }
                        let start = uit.len() - afstand;
                        for k in 0..len {
                            let c = uit[start + k];
                            uit.push(c);
                        }
                    }
                }
            }
            _ => return None,
        }
        if laatste == 1 {
            return Some(uit);
        }
    }
}

/// Eén bestand uit een zip halen (via de inhoudsopgave achteraan).
fn uit_zip(zip: &[u8], naam: &str) -> Option<Vec<u8>> {
    let u16_ = |p: usize| -> Option<usize> { Some(u16::from_le_bytes([*zip.get(p)?, *zip.get(p + 1)?]) as usize) };
    let u32_ = |p: usize| -> Option<usize> { Some(u32::from_le_bytes(zip.get(p..p + 4)?.try_into().ok()?) as usize) };
    let eind = (0..zip.len().saturating_sub(21)).rev().find(|&i| zip[i..].starts_with(b"PK\x05\x06"))?;
    let aantal = u16_(eind + 10)?;
    let mut p = u32_(eind + 16)?;
    for _ in 0..aantal {
        if !zip.get(p..)?.starts_with(b"PK\x01\x02") {
            return None;
        }
        let methode = u16_(p + 10)?;
        let grootte = u32_(p + 20)?;
        let (nl, el, cl) = (u16_(p + 28)?, u16_(p + 30)?, u16_(p + 32)?);
        let lokaal = u32_(p + 42)?;
        let bestandsnaam = zip.get(p + 46..p + 46 + nl)?;
        if bestandsnaam == naam.as_bytes() {
            let begin = lokaal + 30 + u16_(lokaal + 26)? + u16_(lokaal + 28)?;
            let data = zip.get(begin..begin + grootte)?;
            return match methode {
                0 => Some(data.to_vec()),
                8 => inflate(data),
                _ => None,
            };
        }
        p += 46 + nl + el + cl;
    }
    None
}

/// XML-tekens terug naar gewone tekens.
fn xml_tekst(t: &str) -> String {
    t.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

/// De tekst uit een Word-bestand (DOCX): elke alinea op een eigen regel.
fn docx_lezen(data: &[u8]) -> Option<String> {
    let xml = String::from_utf8(uit_zip(data, "word/document.xml")?).ok()?;
    let mut uit = String::new();
    let mut rest = xml.as_str();
    while let Some(p) = rest.find('<') {
        let Some(e) = rest[p..].find('>') else { break };
        let tag = &rest[p + 1..p + e];
        let naam = tag.trim_start_matches('/').split([' ', '/']).next().unwrap_or("");
        let na = &rest[p + e + 1..];
        match naam {
            "w:t" if !tag.starts_with('/') && !tag.ends_with('/') => {
                let eind = na.find("</w:t>").unwrap_or(na.len());
                uit.push_str(&xml_tekst(&na[..eind]));
                rest = &na[eind..];
                continue;
            }
            "w:tab" if !tag.starts_with('/') => uit.push('\t'),
            "w:br" | "w:cr" if !tag.starts_with('/') => uit.push('\n'),
            "w:p" if tag.starts_with('/') => uit.push('\n'),
            _ => {}
        }
        rest = na;
    }
    Some(uit.trim_end().to_string())
}

/// De tekst uit een PDF-bestand. Werkt met gewone PDF's met tekst (niet met scans).
fn pdf_lezen(data: &[u8]) -> Option<String> {
    if !data.starts_with(b"%PDF") {
        return None;
    }
    let mut uit = String::new();
    let mut p = 0;
    while let Some(s) = data[p..].windows(6).position(|w| w == b"stream") {
        let begin_dict = data[..p + s].iter().rposition(|&c| c == b'<').unwrap_or(p);
        let dict = String::from_utf8_lossy(&data[begin_dict.saturating_sub(200)..p + s]).to_string();
        let mut b = p + s + 6;
        if data.get(b) == Some(&b'\r') {
            b += 1;
        }
        if data.get(b) == Some(&b'\n') {
            b += 1;
        }
        let Some(e) = data[b..].windows(9).position(|w| w == b"endstream") else { break };
        let ruw = &data[b..b + e];
        p = b + e + 9;
        let inhoud = if dict.contains("/FlateDecode") {
            match ruw.get(2..).and_then(inflate) {
                Some(x) => x,
                None => continue,
            }
        } else if dict.contains("/Filter") {
            continue;
        } else {
            ruw.to_vec()
        };
        pdf_tekst_uit(&inhoud, &mut uit);
    }
    let uit = uit.trim().to_string();
    (!uit.is_empty()).then_some(uit)
}

/// Tekst uit een PDF-inhoud: (tekst) Tj, [(tekst)] TJ, en T* of Td voor een nieuwe regel.
fn pdf_tekst_uit(d: &[u8], uit: &mut String) {
    const WIN: [(u8, char); 10] = [(0x80, '€'), (0x82, '‚'), (0x84, '„'), (0x85, '…'), (0x91, '‘'), (0x92, '’'), (0x93, '“'), (0x94, '”'), (0x96, '–'), (0x97, '—')];
    let teken = |b: u8| WIN.iter().find(|x| x.0 == b).map(|x| x.1).unwrap_or(b as char);
    let mut i = 0;
    let mut in_tekst = false;
    let mut regel = String::new();
    while i < d.len() {
        match d[i] {
            b'(' => {
                let mut diepte = 1;
                i += 1;
                while i < d.len() && diepte > 0 {
                    match d[i] {
                        b'\\' if i + 1 < d.len() => {
                            i += 1;
                            match d[i] {
                                b'n' => regel.push('\n'),
                                b'(' | b')' | b'\\' => regel.push(d[i] as char),
                                b'0'..=b'7' => {
                                    let mut v = 0u32;
                                    let mut k = 0;
                                    while k < 3 && i < d.len() && (b'0'..=b'7').contains(&d[i]) {
                                        v = v * 8 + (d[i] - b'0') as u32;
                                        i += 1;
                                        k += 1;
                                    }
                                    regel.push(teken(v as u8));
                                    continue;
                                }
                                _ => {}
                            }
                        }
                        b'(' => {
                            diepte += 1;
                            regel.push('(');
                        }
                        b')' => {
                            diepte -= 1;
                            if diepte > 0 {
                                regel.push(')');
                            }
                        }
                        c => regel.push(teken(c)),
                    }
                    i += 1;
                }
                in_tekst = true;
                continue;
            }
            b'T' if i + 1 < d.len() && matches!(d[i + 1], b'*' | b'd' | b'D') => {
                if in_tekst {
                    uit.push_str(regel.trim_end());
                    uit.push('\n');
                    regel.clear();
                    in_tekst = false;
                }
            }
            b'E' if d.get(i + 1) == Some(&b'T') => {
                if !regel.is_empty() {
                    uit.push_str(regel.trim_end());
                    uit.push('\n');
                    regel.clear();
                }
                in_tekst = false;
            }
            _ => {}
        }
        i += 1;
    }
    if !regel.is_empty() {
        uit.push_str(&regel);
        uit.push('\n');
    }
}

/// HTML zonder opmaak: alleen de leesbare tekst, met een nieuwe regel na kopjes en alinea's.
fn leesbare_tekst(html: &str) -> String {
    let mut uit = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut overslaan = false;
    for c in html.chars() {
        if in_tag {
            if c == '>' {
                in_tag = false;
                let t = tag.to_lowercase();
                let naam = t.trim_start_matches('/').split(|x: char| x.is_whitespace() || x == '/').next().unwrap_or("").to_string();
                if naam == "script" || naam == "style" {
                    overslaan = !t.starts_with('/');
                }
                if matches!(naam.as_str(), "p" | "br" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "title") {
                    uit.push('\n');
                }
                tag.clear();
            } else {
                tag.push(c);
            }
        } else if c == '<' {
            in_tag = true;
        } else if !overslaan {
            uit.push(c);
        }
    }
    let uit = uit.replace("&nbsp;", " ").replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'");
    uit.lines().map(|l| l.split_whitespace().collect::<Vec<_>>().join(" ")).filter(|l| !l.is_empty()).collect::<Vec<_>>().join("\n")
}

/// Een Dropbox-link wordt een directe download (dl=1).
fn directe_link(adres: &str) -> String {
    let a = adres.trim().to_string();
    if a.contains("dropbox.com") {
        if a.contains("dl=0") {
            return a.replace("dl=0", "dl=1");
        }
        if !a.contains("dl=1") && !a.contains("raw=1") {
            return format!("{}{}dl=1", a, if a.contains('?') { "&" } else { "?" });
        }
    }
    a
}

/// Ophalen (GET) of versturen (POST) via internet. Met Windows zelf (WinHTTP).
fn internet(methode: &str, adres: &str, inhoud: Option<&[u8]>) -> Result<Vec<u8>, String> {
    let adres = directe_link(adres);
    #[cfg(windows)]
    {
        return win::internet(methode, &adres, inhoud);
    }
    Err("Internet werkt alleen op Windows.".to_string())
}

fn verstuur(a: W, w: W, r: usize) -> R<()> {
    internet("POST", &a.tekst(), Some(w.tekst().as_bytes())).map(|_| ()).or_else(|e| fout(r, e))
}

fn download(a: W, p: W, r: usize) -> R<()> {
    let data = internet("GET", &a.tekst(), None).or_else(|e| fout(r, e))?;
    let pad = volledig_pad(&p);
    if let Some(m) = pad.parent() {
        let _ = std::fs::create_dir_all(m);
    }
    std::fs::write(&pad, data).or_else(|e| fout(r, format!("Ik kan {} niet bewaren: {}", pad.display(), e)))
}

/// Versies vergelijken: 1.10 is nieuwer dan 1.9.
fn is_nieuwer(nieuw: &str, oud: &str) -> bool {
    let delen = |v: &str| -> Vec<u64> { v.split(|c: char| !c.is_ascii_digit()).filter(|x| !x.is_empty()).map(|x| x.parse().unwrap_or(0)).collect() };
    let (a, b) = (delen(nieuw), delen(oud));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    false
}

/// Een geluid zoeken: eerst naast het programma, dan in Documenten.
fn geluidspad(p: &W) -> std::path::PathBuf {
    let t = p.tekst();
    let pad = std::path::PathBuf::from(t.trim());
    if pad.is_absolute() {
        return pad;
    }
    if let Some(naast) = std::env::current_exe().ok().and_then(|e| e.parent().map(|m| m.join(&pad))) {
        if naast.is_file() {
            return naast;
        }
    }
    volledig_pad(p)
}

/// De tekst van de handleiding (F1): wat je in je programma schreef, of het bestand.
fn handleiding_tekst() -> String {
    if !HANDLEIDING.is_empty() {
        return HANDLEIDING.to_string();
    }
    if !HANDLEIDING_BESTAND.is_empty() {
        let pad = geluidspad(&W::T(HANDLEIDING_BESTAND.to_string()));
        return lees_tekstbestand(&pad).unwrap_or_else(|_| format!("Ik kan de handleiding {} niet vinden.", HANDLEIDING_BESTAND));
    }
    String::new()
}

/// De privacyverklaring (Shift+F1): wat je in je programma schreef, of Privacyverklaring.txt naast het programma.
fn privacy_tekst() -> String {
    if !PRIVACY.is_empty() {
        return PRIVACY.to_string();
    }
    let pad = geluidspad(&W::T("Privacyverklaring.txt".to_string()));
    lees_tekstbestand(&pad).unwrap_or_default()
}

/// Een map inpakken in een zipbestand, of uitpakken. Met Windows zelf (Compress-Archive en Expand-Archive).
fn zip(inpakken: bool, map: W, z: W, r: usize) -> R<()> {
    let (m, z) = (volledig_pad(&map), volledig_pad(&z));
    if !cfg!(windows) {
        return fout(r, "Zipbestanden maken en uitpakken kan alleen op Windows.".to_string());
    }
    let q = |p: &std::path::Path| format!("'{}'", p.display().to_string().replace('\'', "''"));
    let opdracht = if inpakken {
        if !m.is_dir() {
            return fout(r, format!("De map {} bestaat niet.", m.display()));
        }
        format!("Compress-Archive -Path (Join-Path {} '*') -DestinationPath {} -Force", q(&m), q(&z))
    } else {
        if !z.is_file() {
            return fout(r, format!("Het zipbestand {} bestaat niet.", z.display()));
        }
        format!("Expand-Archive -LiteralPath {} -DestinationPath {} -Force", q(&z), q(&m))
    };
    let mut cmd = std::process::Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &format!("$ErrorActionPreference='Stop'; {}", opdracht)]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    match cmd.output() {
        Ok(o) if o.status.success() => Ok(()),
        Ok(o) => fout(r, format!("Het zipbestand is niet gelukt: {}", String::from_utf8_lossy(&o.stderr).lines().next().unwrap_or("").trim())),
        Err(e) => fout(r, format!("Het zipbestand is niet gelukt: {}", e)),
    }
}

thread_local! {
    /// De radio die nu speelt (een proces op de achtergrond).
    static RADIO: std::cell::RefCell<Option<std::process::Child>> = const { std::cell::RefCell::new(None) };
    /// Het volume (0 tot 100) voor geluid en voorlezen.
    static VOLUME: std::cell::Cell<u32> = std::cell::Cell::new(100);
}

fn kopieer_alles(van: &std::path::Path, naar: &std::path::Path) -> std::io::Result<()> {
    if van.is_dir() {
        std::fs::create_dir_all(naar)?;
        for e in std::fs::read_dir(van)? {
            let e = e?;
            kopieer_alles(&e.path(), &naar.join(e.file_name()))?;
        }
        Ok(())
    } else {
        if let Some(m) = naar.parent() {
            std::fs::create_dir_all(m)?;
        }
        std::fs::copy(van, naar).map(|_| ())
    }
}

/// Kopiëren (0), verplaatsen (1) of naar de Prullenbak gooien (2).
fn bestand_actie(soort: u8, van: W, naar: Option<W>, r: usize) -> R<()> {
    let v = volledig_pad(&van);
    if !v.exists() {
        return fout(r, format!("{} bestaat niet.", v.display()));
    }
    if soort == 2 {
        #[cfg(windows)]
        {
            return if win::naar_prullenbak(&v) { Ok(()) } else { fout(r, format!("Ik kan {} niet weggooien.", v.display())) };
        }
        let bak = basismap().join("Prullenbak");
        let _ = std::fs::create_dir_all(&bak);
        let doel = bak.join(v.file_name().unwrap_or_default());
        return std::fs::rename(&v, &doel).or_else(|e| fout(r, format!("Ik kan {} niet weggooien: {}", v.display(), e)));
    }
    let mut n = volledig_pad(&naar.unwrap_or(W::T(String::new())));
    if n.is_dir() && !v.is_dir() {
        n = n.join(v.file_name().unwrap_or_default());
    }
    let uitkomst = if soort == 0 {
        kopieer_alles(&v, &n)
    } else {
        if let Some(m) = n.parent() {
            let _ = std::fs::create_dir_all(m);
        }
        std::fs::rename(&v, &n).or_else(|_| {
            kopieer_alles(&v, &n)?;
            if v.is_dir() {
                std::fs::remove_dir_all(&v)
            } else {
                std::fs::remove_file(&v)
            }
        })
    };
    uitkomst.or_else(|e| {
        fout(r, format!("{} van {} naar {} is niet gelukt: {}", if soort == 0 { "Kopiëren" } else { "Verplaatsen" }, v.display(), n.display(), e))
    })
}

// ---------------------------------------------------------------------
// Word (DOCX) en PDF maken, zonder extra programma's.
// ---------------------------------------------------------------------

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

/// Een zipbestand zonder compressie (zo bestaat een .docx uit losse onderdelen).
fn maak_zip(bestanden: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut uit = Vec::new();
    let mut lijst = Vec::new();
    for (naam, data) in bestanden {
        let plek = uit.len() as u32;
        let crc = crc32(data);
        let n = naam.as_bytes();
        uit.extend_from_slice(&[0x50, 0x4B, 3, 4, 20, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
        uit.extend_from_slice(&crc.to_le_bytes());
        uit.extend_from_slice(&(data.len() as u32).to_le_bytes());
        uit.extend_from_slice(&(data.len() as u32).to_le_bytes());
        uit.extend_from_slice(&(n.len() as u16).to_le_bytes());
        uit.extend_from_slice(&[0, 0]);
        uit.extend_from_slice(n);
        uit.extend_from_slice(data);
        lijst.push((n, crc, data.len() as u32, plek));
    }
    let begin = uit.len() as u32;
    for (n, crc, lengte, plek) in &lijst {
        uit.extend_from_slice(&[0x50, 0x4B, 1, 2, 20, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0x21, 0]);
        uit.extend_from_slice(&crc.to_le_bytes());
        uit.extend_from_slice(&lengte.to_le_bytes());
        uit.extend_from_slice(&lengte.to_le_bytes());
        uit.extend_from_slice(&(n.len() as u16).to_le_bytes());
        uit.extend_from_slice(&[0; 12]);
        uit.extend_from_slice(&plek.to_le_bytes());
        uit.extend_from_slice(n);
    }
    let grootte = uit.len() as u32 - begin;
    uit.extend_from_slice(&[0x50, 0x4B, 5, 6, 0, 0, 0, 0]);
    uit.extend_from_slice(&(lijst.len() as u16).to_le_bytes());
    uit.extend_from_slice(&(lijst.len() as u16).to_le_bytes());
    uit.extend_from_slice(&grootte.to_le_bytes());
    uit.extend_from_slice(&begin.to_le_bytes());
    uit.extend_from_slice(&[0, 0]);
    uit
}

fn xml(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn maak_docx(tekst: &str) -> Vec<u8> {
    let mut alineas = String::new();
    for regel in tekst.lines() {
        alineas.push_str(&format!("<w:p><w:r><w:t xml:space=\"preserve\">{}</w:t></w:r></w:p>", xml(regel)));
    }
    if alineas.is_empty() {
        alineas.push_str("<w:p/>");
    }
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body>{}<w:sectPr><w:pgSz w:w=\"11906\" w:h=\"16838\"/></w:sectPr></w:body></w:document>",
        alineas
    );
    let types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/></Types>";
    let rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?><Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/></Relationships>";
    maak_zip(&[
        ("[Content_Types].xml", types.as_bytes().to_vec()),
        ("_rels/.rels", rels.as_bytes().to_vec()),
        ("word/document.xml", document.into_bytes()),
    ])
}

/// Tekst naar Windows-1252 (de tekens die een standaard-PDF-letter kent), met \ ( ) veilig gemaakt.
fn pdf_tekst(t: &str) -> String {
    let mut uit = String::new();
    for c in t.chars() {
        let b: u32 = match c {
            '€' => 0x80, '‚' => 0x82, '„' => 0x84, '…' => 0x85, '‘' => 0x91, '’' => 0x92, '“' => 0x93, '”' => 0x94,
            '–' => 0x96, '—' => 0x97, c if (c as u32) < 256 => c as u32, _ => '?' as u32,
        };
        match b {
            0x28 | 0x29 | 0x5C => {
                uit.push('\\');
                uit.push(char::from_u32(b).unwrap());
            }
            0x20..=0x7E => uit.push(char::from_u32(b).unwrap()),
            _ => uit.push_str(&format!("\\{:03o}", b)),
        }
    }
    uit
}

fn maak_pdf(tekst: &str) -> Vec<u8> {
    // Regels afbreken op ongeveer 85 tekens, 46 regels per bladzijde (A4).
    let mut regels: Vec<String> = Vec::new();
    for alinea in tekst.lines() {
        let mut regel = String::new();
        for woord in alinea.split(' ') {
            if regel.chars().count() + woord.chars().count() + 1 > 85 && !regel.is_empty() {
                regels.push(std::mem::take(&mut regel));
            }
            if !regel.is_empty() {
                regel.push(' ');
            }
            regel.push_str(woord);
        }
        regels.push(regel);
    }
    let paginas: Vec<&[String]> = if regels.is_empty() { vec![&[][..]] } else { regels.chunks(46).collect() };
    let mut objecten: Vec<String> = Vec::new();
    let n = paginas.len();
    // 1 catalogus, 2 bladzijden, 3 letter, dan per bladzijde: bladzijde + inhoud.
    let kinderen: Vec<String> = (0..n).map(|i| format!("{} 0 R", 4 + 2 * i)).collect();
    objecten.push("<< /Type /Catalog /Pages 2 0 R /Lang (nl-NL) >>".into());
    objecten.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kinderen.join(" "), n));
    objecten.push("<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".into());
    for (i, pagina) in paginas.iter().enumerate() {
        let mut inhoud = String::from("BT /F1 12 Tf 16 TL 56 786 Td\n");
        for r in pagina.iter() {
            inhoud.push_str(&format!("({}) Tj T*\n", pdf_tekst(r)));
        }
        inhoud.push_str("ET");
        objecten.push(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 3 0 R >> >> /Contents {} 0 R >>",
            5 + 2 * i
        ));
        objecten.push(format!("<< /Length {} >>\nstream\n{}\nendstream", inhoud.len(), inhoud));
    }
    let mut uit = String::from("%PDF-1.4\n");
    let mut plekken = Vec::new();
    for (i, o) in objecten.iter().enumerate() {
        plekken.push(uit.len());
        uit.push_str(&format!("{} 0 obj\n{}\nendobj\n", i + 1, o));
    }
    let xref = uit.len();
    uit.push_str(&format!("xref\n0 {}\n0000000000 65535 f \n", objecten.len() + 1));
    for p in plekken {
        uit.push_str(&format!("{:010} 00000 n \n", p));
    }
    uit.push_str(&format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n", objecten.len() + 1, xref));
    uit.into_bytes()
}

/// Tekst in een Word-bestand (0) of PDF-bestand (1) schrijven.
fn schrijf_document(soort: u8, p: W, w: W, r: usize) -> R<()> {
    let pad = volledig_pad(&p);
    let tekst = w.tekst().replace("\r\n", "\n");
    let data = if soort == 0 { maak_docx(&tekst) } else { maak_pdf(&tekst) };
    if let Some(m) = pad.parent() {
        let _ = std::fs::create_dir_all(m);
    }
    std::fs::write(&pad, data).or_else(|e| fout(r, format!("Ik kan {} niet schrijven. Staat het misschien open in Word of een PDF-lezer? ({})", pad.display(), e)))
}

fn uitkomst_van(w: Option<W>, naam: &str, r: usize) -> R<W> {
    match w {
        Some(w) => Ok(w),
        None => fout(r, format!("Het stukje {} geeft geen uitkomst. Zet in het stukje een regel zoals: Geef als uitkomst ...", naam)),
    }
}

fn vergelijk(a: W, b: W, op: u8, r: usize) -> R<bool> {
    if op <= 1 {
        let gelijk = gelijk(&a, &b);
        return Ok(if op == 0 { gelijk } else { !gelijk });
    }
    let (x, y) = match (&a, &b) {
        (W::G(x), W::G(y)) => (*x, *y),
        (W::G(_), ander) | (ander, _) => {
            return fout(r, format!("Groter of kleiner kan ik alleen bij getallen bepalen, maar hier staat {}.", ander.soort()))
        }
    };
    Ok(match op {
        2 => x > y,
        3 => x < y,
        4 => x >= y,
        _ => x <= y,
    })
}

fn waar(a: W, r: usize) -> R<bool> {
    match a {
        W::B(b) => Ok(b),
        ander => fout(r, format!("Hier verwacht ik waar of niet waar, maar hier staat {}.", ander.soort())),
    }
}

fn aantal(a: W, r: usize) -> R<u64> {
    match a {
        W::G(g) if g >= 0.0 && g.fract() == 0.0 && g <= MAX_HERHALINGEN => Ok(g as u64),
        W::G(g) if g > MAX_HERHALINGEN => {
            fout(r, "Zo vaak herhalen mag niet. Het kan hoogstens een miljoen keer.".to_string())
        }
        W::G(g) => fout(r, format!("Het aantal keer moet een heel getal zijn, en niet {}.", schrijf_getal(g))),
        ander => fout(r, format!("Hoe vaak? Hier verwacht ik een getal, maar hier staat {}.", ander.soort())),
    }
}

struct Staat {
    /// De namen en waarden van de eigen stukjes die nu bezig zijn.
    lokaal: Vec<(&'static [&'static str], Vec<Option<W>>)>,
    waarden: Vec<Option<W>>,
    namen: &'static [&'static str],
    acties: &'static [&'static str],
    stappen: u64,
    scherm: Scherm,
}

impl Staat {
    fn naam_van(&self, p: P) -> &'static str {
        match p {
            P::G(i) => self.namen[i],
            P::L(i) => self.lokaal.last().map(|l| l.0[i]).unwrap_or("?"),
        }
    }
    fn plek(&mut self, p: P) -> &mut Option<W> {
        match p {
            P::G(i) => &mut self.waarden[i],
            P::L(i) => &mut self.lokaal.last_mut().expect("stukje").1[i],
        }
    }
    fn lees(&mut self, p: P, r: usize) -> R<W> {
        match self.plek(p).clone() {
            Some(w) => Ok(w),
            None => {
                let n = self.naam_van(p);
                fout(r, format!("{} heeft nog geen waarde. Zet eerst een regel zoals: Onthoud {} is 0.", n, n))
            }
        }
    }
    fn zet(&mut self, p: P, w: W) {
        *self.plek(p) = Some(w);
    }
    fn verander(&mut self, p: P, w: W, r: usize) -> R<()> {
        if self.plek(p).is_none() {
            let n = self.naam_van(p);
            return fout(r, format!("Je kunt {} pas veranderen als je het eerst hebt onthouden.", n));
        }
        *self.plek(p) = Some(w);
        Ok(())
    }
    fn tel_op(&mut self, p: P, w: W, erbij: bool, r: usize) -> R<()> {
        let oud = self.lees(p, r)?;
        let nieuw = reken(oud, if erbij { 0 } else { 1 }, w, r)?;
        *self.plek(p) = Some(nieuw);
        Ok(())
    }
    fn voeg_toe(&mut self, p: P, w: W, r: usize) -> R<()> {
        let n = self.naam_van(p);
        match self.plek(p) {
            Some(W::L(l)) => {
                if l.len() >= 10_000_000 {
                    return fout(r, format!("De lijst {} is te groot geworden (tien miljoen dingen).", n));
                }
                l.push(w);
                Ok(())
            }
            Some(ander) => {
                let soort = ander.soort();
                fout(r, format!("{} is {}, geen lijst.", n, soort))
            }
            None => fout(r, format!("De lijst {} bestaat nog niet. Maak hem eerst met: Maak een lijst met als naam {}.", n, n)),
        }
    }
    fn haal_uit(&mut self, p: P, w: W, r: usize) -> R<()> {
        let n = self.naam_van(p);
        match self.plek(p) {
            Some(W::L(l)) => {
                if let Some(i) = l.iter().position(|x| gelijk(x, &w)) {
                    l.remove(i);
                }
                Ok(())
            }
            _ => fout(r, format!("{} is geen lijst.", n)),
        }
    }
    fn begin_stukje(&mut self, namen: &'static [&'static str], invoer: Vec<W>, r: usize) -> R<()> {
        if self.lokaal.len() >= 1000 {
            return fout(r, "Een stukje roept zichzelf te vaak aan (meer dan duizend keer diep). Ik heb het gestopt.".to_string());
        }
        let mut waarden = vec![None; namen.len()];
        for (i, w) in invoer.into_iter().enumerate() {
            waarden[i] = Some(w);
        }
        self.lokaal.push((namen, waarden));
        Ok(())
    }
    fn einde_stukje(&mut self) {
        self.lokaal.pop();
    }
    fn stap(&mut self, r: usize) -> R<()> {
        self.stappen += 1;
        if self.stappen > MAX_STAPPEN {
            return fout(
                r,
                "Het programma doet te veel stappen, meer dan tien miljoen. Ik heb het gestopt, zodat de computer niet vastloopt."
                    .to_string(),
            );
        }
        Ok(())
    }
    fn toon(&mut self, w: W) {
        let t = w.tekst();
        self.scherm.toon(&t);
    }
    fn grens(&mut self, r: usize, max: u64) {
        let t = format!("Let op: de herhaling van regel {} is gestopt na {} keer. Dat was de grens die je had gezet.", r, max);
        self.scherm.toon(&t);
    }
    fn venster(&mut self, titel: &str) {
        self.scherm.venster(titel);
    }
    fn knop(&mut self, naam: &str) {
        let actie = self.acties.iter().position(|a| a.to_lowercase() == naam.to_lowercase());
        self.scherm.knop(naam, actie);
    }
    fn onderdeel(&mut self, soort: u8, naam: &str) {
        self.scherm.onderdeel(soort, naam);
    }
    fn zoek(&self, naam: &str, r: usize) -> R<usize> {
        match self.scherm.onderdelen.iter().position(|o| o.naam.to_lowercase() == naam.to_lowercase()) {
            Some(i) => Ok(i),
            None => fout(r, format!("Het onderdeel {} is er (nog) niet. Is het venster waar het in staat al gemaakt?", naam)),
        }
    }
    fn waarde_van(&mut self, naam: &str, als_getal: bool, r: usize) -> R<W> {
        let i = self.zoek(naam, r)?;
        let w = self.scherm.lees(i);
        if !als_getal {
            return Ok(w);
        }
        let t = w.tekst();
        let schoon: String = t.trim().replace(' ', "").replace('.', "").replace(',', ".");
        match schoon.parse::<f64>() {
            Ok(g) if g.is_finite() && !t.trim().is_empty() => Ok(W::G(g)),
            _ => {
                if t.trim().is_empty() {
                    fout(r, format!("In {} staat nog niets. Daar moet een getal in.", naam))
                } else {
                    fout(r, format!("In {} staat geen getal, maar: {}", naam, t.trim()))
                }
            }
        }
    }
    fn zet_in(&mut self, naam: &str, w: W, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        if !matches!(self.scherm.onderdelen[i].soort, 0 | 1 | 2) {
            return fout(r, format!("In {} kan geen tekst. Dat kan alleen bij een tekstvak of een label.", naam));
        }
        self.scherm.zet_tekst(i, &w.tekst());
        Ok(())
    }
    fn voeg_keuze(&mut self, naam: &str, w: W, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        if !matches!(self.scherm.onderdelen[i].soort, 4 | 5) {
            return fout(r, format!("{} is geen keuzelijst.", naam));
        }
        self.scherm.voeg_keuze(i, &w.tekst());
        Ok(())
    }
    fn vink(&mut self, naam: &str, aan: bool, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        if self.scherm.onderdelen[i].soort != 3 {
            return fout(r, format!("{} is geen selectievakje.", naam));
        }
        self.scherm.vink(i, aan);
        Ok(())
    }
    fn piep(&mut self, h: W, d: W, r: usize) -> R<()> {
        let (h, d) = match (h, d) {
            (W::G(h), W::G(d)) if (37.0..=32767.0).contains(&h) && (1.0..=10000.0).contains(&d) => (h as u32, d as u32),
            _ => return fout(r, "Een piep heeft een hoogte van 37 tot 32767 hertz en een duur van 1 tot 10000 milliseconden.".to_string()),
        };
        if self.scherm.tekststand {
            println!("[piep] {} hertz, {} milliseconden", h, d);
            return Ok(());
        }
        #[cfg(windows)]
        win::piep(h, d);
        Ok(())
    }
    fn speel(&mut self, p: W, wachten: bool, richting: u8, r: usize) -> R<()> {
        let pad = geluidspad(&p);
        if !pad.is_file() {
            return fout(r, format!("Ik kan het geluid {} niet vinden. Zet het naast je programma, of in Documenten.", p.tekst()));
        }
        let ruimtelijk = if richting == 0 {
            None
        } else {
            let data = std::fs::read(&pad).or_else(|e| fout(r, format!("Ik kan het geluid {} niet lezen: {}", p.tekst(), e)))?;
            Some(ruimtelijk_geluid(&data, richting).or_else(|e| fout(r, format!("{} {}", p.tekst(), e)))?)
        };
        if self.scherm.tekststand {
            let waar = if richting == 0 { String::new() } else { format!(" ({})", richting_naam(richting)) };
            println!("[geluid] {}{}{}", p.tekst(), waar, if wachten { " (wacht tot het klaar is)" } else { "" });
            if let (Some(wav), Ok(naar)) = (&ruimtelijk, std::env::var("JAAPTAAL_GELUID_BEWAAR")) {
                let _ = std::fs::write(naar, wav);
            }
            return Ok(());
        }
        #[cfg(windows)]
        if let Some(wav) = ruimtelijk {
            if !win::speel_geheugen(wav, wachten) {
                return fout(r, format!("Het geluid {} kan ik niet afspelen.", p.tekst()));
            }
            return Ok(());
        }
        #[cfg(windows)]
        {
            let laag = pad.to_string_lossy().to_lowercase();
            let gelukt = if laag.ends_with(".wav") { win::speel(&pad, wachten) } else { win::speel_mci(&pad, wachten) };
            if !gelukt {
                return fout(r, format!("Het geluid {} kan ik niet afspelen. Is het een gewoon WAV- of MP3-bestand?", p.tekst()));
            }
        }
        Ok(())
    }
    /// Internetradio: een zender spelen (via Windows Media Player op de achtergrond) of stoppen.
    fn radio(&mut self, adres: Option<W>, naam: &str, r: usize) -> R<()> {
        RADIO.with(|p| {
            if let Some(mut kind) = p.borrow_mut().take() {
                let _ = kind.kill();
            }
        });
        let Some(a) = adres else {
            if self.scherm.tekststand {
                println!("[radio uit]");
            }
            return Ok(());
        };
        let mut adres = a.tekst();
        if let Some(naam) = adres.strip_prefix("zoek:").map(|x| x.to_string()) {
            if self.scherm.tekststand && std::env::var("JAAPTAAL_NIET_ZOEKEN").is_ok() {
                println!("[radio zoeken] {}", naam);
                return Ok(());
            }
            adres = zoek_zender(&naam).ok_or(()).or_else(|_| fout(r, format!("De zender {} kan ik niet vinden op internet (radio-browser.info). Klopt de naam? Of gebruik het adres: Speel de radiozender met het adres https://...", naam)))?;
        }
        if !adres.starts_with("http://") && !adres.starts_with("https://") {
            return fout(r, format!("{} is geen internetadres van een radiozender.", adres));
        }
        if self.scherm.tekststand {
            println!("[radio] {} ({})", if naam.is_empty() { &adres } else { naam }, adres);
            return Ok(());
        }
        // Windows Media Player speelt de stream; hij stopt vanzelf als dit programma stopt.
        let script = format!(
            "$p=New-Object -ComObject WMPlayer.OCX; $p.settings.volume={}; $p.URL='{}'; $p.controls.play(); while(Get-Process -Id {} -ErrorAction SilentlyContinue){{Start-Sleep -Milliseconds 500}}",
            VOLUME.with(|v| v.get()),
            adres.replace('\'', "''"),
            std::process::id()
        );
        let mut cmd = std::process::Command::new("powershell.exe");
        cmd.args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &script]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x0800_0000);
        }
        match cmd.spawn() {
            Ok(kind) => {
                RADIO.with(|p| *p.borrow_mut() = Some(kind));
                Ok(())
            }
            Err(e) => fout(r, format!("De radio kan niet starten: {}", e)),
        }
    }
    /// Schermvullend, vraag bij afsluiten, maar één keer open, tip bij een knop.
    fn scherm_gedrag(&mut self, soort: u8, a: &str, b: &str) {
        if self.scherm.tekststand {
            match soort {
                0 => println!("[schermvullend]"),
                1 => println!("[vraag bij afsluiten] {}", a),
                2 => println!("[maar één keer open]"),
                _ => println!("[tip bij {}] {}", a, b),
            }
            return;
        }
        #[cfg(windows)]
        self.scherm.win.gedrag(soort, a, b);
    }
    /// Een werkbalk: knoppen naast elkaar.
    fn werkbalk(&mut self, namen: &[&str]) {
        if self.scherm.tekststand {
            println!("[werkbalk] {}", namen.join(", "));
        }
        let mut nummers = Vec::new();
        for naam in namen {
            let actie = self.acties.iter().position(|a| a.to_lowercase() == naam.to_lowercase());
            self.scherm.knoppen.push((naam.to_string(), actie));
            nummers.push(self.scherm.knoppen.len() - 1);
        }
        #[cfg(windows)]
        if !self.scherm.tekststand {
            self.scherm.win.werkbalk(namen, &nummers);
        }
    }
    /// Het opdrachtvak: een tekstvak Opdracht en een knop Doe het.
    fn opdrachtvak(&mut self) {
        self.scherm.onderdeel(0, "Opdracht");
        self.scherm.knop("Doe het", Some(OPDRACHT_ACTIE));
    }
    /// Wat je in het opdrachtvak zegt, koppelen aan een knop, menuregel of eigen zin. Geen AI:
    /// alle woorden van de zin moeten erin staan. Anders vragen, niet gokken.
    fn opdracht_geven(&mut self, bij_knop: BijKnop) -> R<()> {
        let zin = self.scherm.onderdelen.iter().position(|o| o.naam == "Opdracht").map(|i| self.scherm.lees(i).tekst()).unwrap_or_default();
        let woorden = |t: &str| -> Vec<String> {
            let t: String = t.to_lowercase().chars().map(|c| if c.is_alphanumeric() { c } else { ' ' }).collect();
            t.split_whitespace()
                .filter(|w| !matches!(*w, "ik" | "wil" | "graag" | "even" | "maar" | "de" | "het" | "een" | "op" | "druk" | "klik" | "kies" | "alsjeblieft" | "nu" | "je" | "mij" | "eens" | "doe"))
                .map(|w| w.to_string())
                .collect()
        };
        let invoer = woorden(&zin);
        if invoer.is_empty() {
            self.scherm.toon("Typ of dicteer eerst wat je wilt in het vak Opdracht.");
            return Ok(());
        }
        // Alles wat je kunt zeggen: je eigen zinnen, en de namen van knoppen en menuregels.
        let mut kan: Vec<(String, usize)> = Vec::new();
        for (i, a) in self.acties.iter().enumerate() {
            if let Some(z) = a.strip_prefix("zeg: ") {
                kan.push((z.to_string(), i));
            }
        }
        for (naam, actie) in &self.scherm.knoppen {
            if let Some(a) = actie {
                if *a != OPDRACHT_ACTIE && !kan.iter().any(|k| k.1 == *a) {
                    kan.push((naam.clone(), *a));
                }
            }
        }
        let past: Vec<&(String, usize)> = kan.iter().filter(|(z, _)| {
            let w = woorden(z);
            !w.is_empty() && w.iter().all(|x| invoer.contains(x))
        }).collect();
        let max = past.iter().map(|p| woorden(&p.0).len()).max().unwrap_or(0);
        let beste: Vec<&&(String, usize)> = past.iter().filter(|p| woorden(&p.0).len() == max).collect();
        if beste.len() == 1 {
            return bij_knop(self, beste[0].1);
        }
        let alles = kan.iter().map(|k| k.0.clone()).collect::<Vec<_>>().join(", ");
        if beste.len() > 1 {
            let namen = beste.iter().map(|b| b.0.clone()).collect::<Vec<_>>().join(" of ");
            self.scherm.toon(&format!("Je zin past bij meer dingen: {}. Zeg het preciezer.", namen));
            return Ok(());
        }
        // Lijkt het ergens op? Dan eerst vragen.
        let lijkt = kan.iter().map(|k| (k, woorden(&k.0).iter().filter(|w| invoer.contains(w)).count())).filter(|(_, n)| *n > 0).max_by_key(|(_, n)| *n);
        if let Some((k, _)) = lijkt {
            let vraag = format!("Bedoel je: {}?", k.0);
            if self.scherm.tekststand {
                println!("[vraag] {}", vraag);
                return Ok(());
            }
            #[cfg(windows)]
            {
                if win::vraag_ja_nee(&vraag) {
                    return bij_knop(self, k.1);
                }
            }
            return Ok(());
        }
        self.scherm.toon(&format!("Dat snap ik niet: {}. Je kunt zeggen: {}.", zin.trim(), alles));
        Ok(())
    }
    fn wacht(&mut self, w: W, r: usize) -> R<()> {
        let sec = match w {
            W::G(g) if (0.0..=3600.0).contains(&g) => g,
            ander => return fout(r, format!("Wachten kan van 0 tot 3600 seconden, en niet {}.", ander.tekst())),
        };
        if self.scherm.tekststand {
            println!("[wacht] {} {}", schrijf_getal(sec), if sec == 1.0 { "seconde" } else { "seconden" });
            if std::env::var("JAAPTAAL_NIET_WACHTEN").is_ok() {
                return Ok(());
            }
        }
        std::thread::sleep(std::time::Duration::from_millis((sec * 1000.0) as u64));
        Ok(())
    }
    fn open(&mut self, w: W, programma: bool, r: usize) -> R<()> {
        let naam = w.tekst();
        let laag = naam.to_lowercase();
        let internet = laag.starts_with("http://") || laag.starts_with("https://");
        let pad = if internet || programma && !naam.contains(['\\', '/']) { std::path::PathBuf::from(&naam) } else { volledig_pad(&w) };
        if !internet && !programma && !pad.exists() {
            return fout(r, format!("Ik kan {} niet vinden.", pad.display()));
        }
        if self.scherm.tekststand {
            println!("[{}] {}", if programma { "start" } else { "open" }, naam);
            return Ok(());
        }
        #[cfg(windows)]
        {
            if !win::open(&pad.to_string_lossy()) {
                return fout(r, format!("Ik kan {} niet {}.", naam, if programma { "starten" } else { "openen" }));
            }
        }
        Ok(())
    }
    fn zeg(&mut self, w: W) {
        let t = w.tekst();
        if self.scherm.tekststand {
            println!("[zeg] {}", t);
            return;
        }
        #[cfg(windows)]
        {
            if !win::zeg(self.scherm.win.actief(), &t) {
                // Geen venster of de schermlezer luistert niet naar meldingen: dan als melding tonen.
                self.scherm.toon(&t);
            }
        }
    }
    fn lees_voor(&mut self, w: W) {
        let t = w.tekst();
        if self.scherm.tekststand {
            println!("[voorlezen] {}", t);
            return;
        }
        #[cfg(windows)]
        {
            if !win::lees_voor(&t) {
                self.zeg(W::T(t));
            }
        }
    }
    fn stop_voorlezen(&mut self) {
        if self.scherm.tekststand {
            println!("[stop voorlezen]");
            return;
        }
        #[cfg(windows)]
        win::stop_voorlezen();
    }
    fn volume(&mut self, w: W, r: usize) -> R<()> {
        let v = match w {
            W::G(g) if (0.0..=100.0).contains(&g) => g as u32,
            _ => return fout(r, "Het volume is een getal van 0 tot 100.".to_string()),
        };
        VOLUME.with(|x| x.set(v));
        if self.scherm.tekststand {
            println!("[volume] {}", v);
            return Ok(());
        }
        #[cfg(windows)]
        win::zet_volume(v);
        Ok(())
    }
    fn tabblad(&mut self, naam: &str) {
        self.scherm.tabbladen.push(naam.to_string());
        if self.scherm.tekststand {
            println!("[tabblad] {}", naam);
            return;
        }
        #[cfg(windows)]
        self.scherm.win.tabblad(naam);
    }
    fn kies_tabblad(&mut self, naam: &str, r: usize) -> R<()> {
        if !self.scherm.tabbladen.iter().any(|t| t.to_lowercase() == naam.to_lowercase()) {
            return fout(r, format!("Het tabblad {} is er (nog) niet.", naam));
        }
        if self.scherm.tekststand {
            println!("[ga naar tabblad] {}", naam);
            return Ok(());
        }
        #[cfg(windows)]
        win::kies_tabblad(naam);
        Ok(())
    }
    fn menu(&mut self, naam: &str) {
        if self.scherm.tekststand {
            println!("[menu] {}", naam);
            return;
        }
        #[cfg(windows)]
        self.scherm.win.menu(naam);
    }
    fn menuregel(&mut self, naam: &str) {
        let actie = self.acties.iter().position(|a| a.to_lowercase() == naam.to_lowercase());
        self.scherm.knoppen.push((naam.to_string(), actie));
        if self.scherm.tekststand {
            println!("[menuregel] {}", naam);
            return;
        }
        #[cfg(windows)]
        self.scherm.win.menuregel(naam, self.scherm.knoppen.len() - 1);
    }
    /// Updates zoeken: het versiebestand ophalen, vergelijken, en na een vraag de nieuwe versie installeren.
    fn zoek_updates(&mut self, r: usize) -> R<()> {
        let data = internet("GET", UPDATE_ADRES, None).or_else(|e| fout(r, format!("Ik kan niet kijken of er een update is. {}", e)))?;
        let tekst = String::from_utf8_lossy(&data).to_string();
        let mut regels = tekst.lines().map(|l| l.trim()).filter(|l| !l.is_empty());
        let versie = regels.next().unwrap_or("").trim_start_matches(['v', 'V']).trim_start_matches("ersie").trim().to_string();
        let link = regels.next().unwrap_or("").to_string();
        if versie.is_empty() || !link.starts_with("http") {
            return fout(r, "Het versiebestand op het update-adres klopt niet. Op de eerste regel moet staan: versie 1.1, en op de tweede regel de link naar het installatiebestand.".to_string());
        }
        if !is_nieuwer(&versie, VERSIE) {
            self.zeg(W::T(format!("Je hebt de nieuwste versie ({}).", VERSIE)));
            return Ok(());
        }
        if self.scherm.tekststand {
            println!("[update] versie {} is er (nu {}): {}", versie, VERSIE, link);
            return Ok(());
        }
        #[cfg(windows)]
        {
            let vraag = format!("Er is een nieuwe versie van {}: versie {}. Je hebt nu versie {}.

Wil je de nieuwe versie downloaden en installeren?", PROGRAMMANAAM, versie, VERSIE);
            if win::vraag_ja(self.scherm.win.actief(), &vraag) {
                let data = internet("GET", &link, None).or_else(|e| fout(r, format!("Het downloaden van de update is niet gelukt. {}", e)))?;
                let pad = std::env::temp_dir().join(format!("{} update.exe", PROGRAMMANAAM));
                std::fs::write(&pad, data).or_else(|e| fout(r, format!("Ik kan de update niet bewaren: {}", e)))?;
                if std::process::Command::new(&pad).spawn().is_ok() {
                    // Dit programma stopt, zodat de update het kan vervangen.
                    std::process::exit(0);
                }
                return fout(r, "Ik kan de update niet starten.".to_string());
            }
        }
        Ok(())
    }
    fn stop_geluid(&mut self) {
        if self.scherm.tekststand {
            println!("[geluid stopt]");
            return;
        }
        #[cfg(windows)]
        win::stop_geluid();
    }
    fn sneltoets(&mut self, knop: &str, code: u16, ctrl: bool, alt: bool, shift: bool, naam: &str, r: usize) -> R<()> {
        let Some(i) = self.scherm.knoppen.iter().position(|k| k.0.to_lowercase() == knop.to_lowercase()) else {
            return fout(r, format!("De knop {} is er (nog) niet.", knop));
        };
        if self.scherm.tekststand {
            println!("[sneltoets] {}: {}", knop, naam);
            return Ok(());
        }
        #[cfg(windows)]
        win::sneltoets(i, code, ctrl, alt, shift, &format!("{} ({})", self.scherm.knoppen[i].0, naam));
        Ok(())
    }
    fn kleur(&mut self, achtergrond: bool, rgb: (u8, u8, u8), naam: &str) {
        if self.scherm.tekststand {
            println!("[kleur {}] {}", if achtergrond { "achtergrond" } else { "tekst" }, naam);
            return;
        }
        #[cfg(windows)]
        self.scherm.win.kleur(achtergrond, rgb);
    }
    fn tabel(&mut self, naam: &str, r: usize) -> R<usize> {
        let i = self.zoek(naam, r)?;
        if self.scherm.onderdelen[i].soort != 6 {
            return fout(r, format!("{} is geen tabel.", naam));
        }
        #[cfg(windows)]
        {
            if !self.scherm.tekststand {
                let h = self.scherm.onderdelen[i].h;
                self.scherm.onderdelen[i].gekozen = win::tabel_gekozen(h);
            }
        }
        Ok(i)
    }
    fn tabel_toon(&mut self, i: usize) {
        if self.scherm.tekststand {
            return;
        }
        #[cfg(windows)]
        {
            let o = &self.scherm.onderdelen[i];
            win::tabel_toon(o.h, &o.keuzes, &o.rijen, o.gekozen);
        }
    }
    fn tabel_kolom(&mut self, naam: &str, kolom: &str, r: usize) -> R<()> {
        let i = self.tabel(naam, r)?;
        let o = &mut self.scherm.onderdelen[i];
        o.keuzes.push(kolom.to_string());
        for rij in &mut o.rijen {
            rij.push(String::new());
        }
        if self.scherm.tekststand {
            println!("[tabel {}] kolom {}", naam, kolom);
        }
        self.tabel_toon(i);
        Ok(())
    }
    fn tabel_rij(&mut self, naam: &str, rij: Vec<W>, r: usize) -> R<()> {
        let i = self.tabel(naam, r)?;
        let o = &mut self.scherm.onderdelen[i];
        if o.keuzes.is_empty() {
            return fout(r, format!("De tabel {} heeft nog geen kolommen. Voeg eerst kolommen toe, bijvoorbeeld: Voeg de kolom Datum toe aan de tabel {}.", naam, naam));
        }
        if rij.len() != o.keuzes.len() {
            return fout(r, format!("De tabel {} heeft {} kolommen ({}), maar deze rij heeft {} waarden.", naam, o.keuzes.len(), o.keuzes.join(", "), rij.len()));
        }
        let tekst: Vec<String> = rij.iter().map(|w| w.tekst()).collect();
        if self.scherm.tekststand {
            println!("[tabel {}] rij {}: {}", naam, o.rijen.len() + 1, tekst.join(" | "));
        }
        o.rijen.push(tekst);
        o.gekozen = Some(o.rijen.len() - 1);
        self.tabel_toon(i);
        Ok(())
    }
    fn tabel_haal_weg(&mut self, naam: &str, r: usize) -> R<()> {
        let i = self.tabel(naam, r)?;
        let o = &mut self.scherm.onderdelen[i];
        let Some(k) = o.gekozen.filter(|k| *k < o.rijen.len()) else {
            return fout(r, format!("Er is geen rij gekozen in de tabel {}.", naam));
        };
        o.rijen.remove(k);
        o.gekozen = if o.rijen.is_empty() { None } else { Some(k.min(o.rijen.len() - 1)) };
        if self.scherm.tekststand {
            println!("[tabel {}] rij {} weg", naam, k + 1);
        }
        self.tabel_toon(i);
        Ok(())
    }
    fn tabel_leeg(&mut self, naam: &str, r: usize) -> R<()> {
        let i = self.tabel(naam, r)?;
        self.scherm.onderdelen[i].rijen.clear();
        self.scherm.onderdelen[i].gekozen = None;
        if self.scherm.tekststand {
            println!("[tabel {}] leeg", naam);
        }
        self.tabel_toon(i);
        Ok(())
    }
    fn tabel_sorteer(&mut self, naam: &str, kolom: &str, r: usize) -> R<()> {
        let i = self.tabel(naam, r)?;
        let o = &mut self.scherm.onderdelen[i];
        let Some(k) = o.keuzes.iter().position(|x| x.to_lowercase() == kolom.to_lowercase()) else {
            return fout(r, format!("De tabel {} heeft geen kolom {}. De kolommen zijn: {}.", naam, kolom, o.keuzes.join(", ")));
        };
        o.rijen.sort_by(|a, b| match (cel_waarde(&a[k]), cel_waarde(&b[k])) {
            (W::G(x), W::G(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            _ => a[k].to_lowercase().cmp(&b[k].to_lowercase()),
        });
        o.gekozen = if o.rijen.is_empty() { None } else { Some(0) };
        if self.scherm.tekststand {
            println!("[tabel {}] gesorteerd op {}", naam, kolom);
        }
        self.tabel_toon(i);
        Ok(())
    }
    fn tabel_csv(&mut self, naam: &str, p: W, opslaan: bool, r: usize) -> R<()> {
        let i = self.tabel(naam, r)?;
        let pad = volledig_pad(&p);
        if opslaan {
            let o = &self.scherm.onderdelen[i];
            let mut t = String::from("\u{FEFF}");
            for rij in std::iter::once(&o.keuzes).chain(o.rijen.iter()) {
                t.push_str(&rij.iter().map(|v| csv_veld(v)).collect::<Vec<_>>().join(";"));
                t.push_str("\r\n");
            }
            if let Some(m) = pad.parent() {
                let _ = std::fs::create_dir_all(m);
            }
            return std::fs::write(&pad, t).or_else(|e| fout(r, format!("Ik kan {} niet schrijven: {}", pad.display(), e)));
        }
        let tekst = lees_tekstbestand(&pad).or_else(|e| fout(r, format!("Ik kan {} niet lezen: {}", pad.display(), e)))?;
        let mut rijen = lees_csv(tekst.trim_start_matches('\u{FEFF}'));
        if rijen.is_empty() {
            return fout(r, format!("{} is leeg.", pad.display()));
        }
        let kolommen = rijen.remove(0);
        let n = kolommen.len();
        let o = &mut self.scherm.onderdelen[i];
        o.keuzes = kolommen;
        o.rijen = rijen.into_iter().map(|mut x| {
            x.resize(n, String::new());
            x
        }).collect();
        o.gekozen = if o.rijen.is_empty() { None } else { Some(0) };
        if self.scherm.tekststand {
            println!("[tabel {}] ingelezen: {} kolommen, {} rijen", naam, n, o.rijen.len());
        }
        self.tabel_toon(i);
        Ok(())
    }
    fn tabel_waarde(&mut self, wat: u8, invoer: Vec<W>, r: usize) -> R<W> {
        let naam = invoer[0].tekst();
        let i = self.tabel(&naam, r)?;
        let o = &self.scherm.onderdelen[i];
        let rij = |k: usize| W::L(o.rijen[k].iter().map(|c| cel_waarde(c)).collect());
        match wat {
            9 => match o.gekozen.filter(|k| *k < o.rijen.len()) {
                Some(k) => Ok(rij(k)),
                None => fout(r, format!("Er is geen rij gekozen in de tabel {}.", naam)),
            },
            10 => Ok(W::G(o.rijen.len() as f64)),
            _ => {
                let n = match invoer.get(1) {
                    Some(W::G(g)) if g.fract() == 0.0 && *g >= 1.0 => *g as usize,
                    _ => return fout(r, "Het rijnummer moet een heel getal zijn, 1 of hoger.".to_string()),
                };
                if n > o.rijen.len() {
                    return fout(r, format!("De tabel {} heeft {} rijen. Rij {} bestaat dus niet.", naam, o.rijen.len(), n));
                }
                Ok(rij(n - 1))
            }
        }
    }
    fn begin_opname(&mut self, r: usize) -> R<()> {
        if self.scherm.tekststand {
            println!("[opname begint]");
            return Ok(());
        }
        #[cfg(windows)]
        {
            return win::begin_opname().or_else(|e| fout(r, e));
        }
        fout(r, "Opnemen kan alleen op Windows.".to_string())
    }
    fn stop_opname(&mut self, p: W, r: usize) -> R<()> {
        let pad = volledig_pad(&p);
        let data: Vec<u8> = if self.scherm.tekststand {
            println!("[opname gestopt] {}", p.tekst());
            Vec::new()
        } else {
            #[cfg(windows)]
            {
                win::stop_opname().or_else(|e| fout(r, e))?
            }
            #[cfg(not(windows))]
            {
                Vec::new()
            }
        };
        // Een WAV-bestand: 44100 keer per seconde, 16 bit, mono.
        let mut wav = Vec::with_capacity(44 + data.len());
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&44100u32.to_le_bytes());
        wav.extend_from_slice(&88200u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&(data.len() as u32).to_le_bytes());
        wav.extend_from_slice(&data);
        if let Some(m) = pad.parent() {
            let _ = std::fs::create_dir_all(m);
        }
        std::fs::write(&pad, wav).or_else(|e| fout(r, format!("Ik kan de opname niet bewaren in {}: {}", pad.display(), e)))
    }
    fn afbeelding(&mut self, bestand: &str, beschrijving: &str, r: usize) -> R<()> {
        let pad = geluidspad(&W::T(bestand.to_string()));
        if !pad.is_file() {
            return fout(r, format!("Ik kan de afbeelding {} niet vinden. Zet hem naast je programma, of in Documenten.", bestand));
        }
        if self.scherm.tekststand {
            println!("[afbeelding] {}: {}", bestand, beschrijving);
            return Ok(());
        }
        #[cfg(windows)]
        {
            if !self.scherm.win.afbeelding(&pad, beschrijving) {
                return fout(r, format!("De afbeelding {} kan ik niet tonen. Is het een gewone PNG, JPG of BMP?", bestand));
            }
        }
        Ok(())
    }
    fn focus(&mut self, naam: &str, r: usize) -> R<()> {
        let i = self.zoek(naam, r)?;
        self.scherm.focus(i);
        Ok(())
    }
}

/// Een onderdeel op het scherm. Soort: 0 tekstvak, 1 groot tekstvak, 2 label, 3 selectievakje, 4 keuzelijst, 5 uitklaplijst.
struct Ond {
    soort: u8,
    naam: String,
    tekst: String,
    aan: bool,
    keuzes: Vec<String>,
    gekozen: Option<usize>,
    /// Bij een tabel: de rijen (de kolommen staan in `keuzes`).
    rijen: Vec<Vec<String>>,
    h: usize,
}

const SOORT_WOORD: [&str; 7] = ["tekstvak", "groot tekstvak", "label", "selectievakje", "keuzelijst", "uitklaplijst", "tabel"];

/// Een getal in een tabelcel (3,5 of 3.5)? Dan rekent het programma ermee als getal.
fn cel_waarde(t: &str) -> W {
    let schoon = t.trim().replace(' ', "");
    let punt = if schoon.contains(',') { schoon.replace('.', "").replace(',', ".") } else { schoon.clone() };
    match punt.parse::<f64>() {
        Ok(g) if !schoon.is_empty() && g.is_finite() => W::G(g),
        _ => W::T(t.to_string()),
    }
}

fn csv_veld(t: &str) -> String {
    if t.contains([';', '"', '\n', ',']) {
        format!("\"{}\"", t.replace('"', "\"\""))
    } else {
        t.to_string()
    }
}

fn lees_csv(tekst: &str) -> Vec<Vec<String>> {
    let scheiding = if tekst.lines().next().map_or(false, |l| l.contains(';')) { ';' } else { ',' };
    let mut rijen = Vec::new();
    let mut rij = Vec::new();
    let mut veld = String::new();
    let mut in_aanhaling = false;
    let mut tekens = tekst.chars().peekable();
    while let Some(c) = tekens.next() {
        match c {
            '"' if in_aanhaling && tekens.peek() == Some(&'"') => {
                veld.push('"');
                tekens.next();
            }
            '"' => in_aanhaling = !in_aanhaling,
            c if c == scheiding && !in_aanhaling => rij.push(std::mem::take(&mut veld)),
            '\r' if !in_aanhaling => {}
            '\n' if !in_aanhaling => {
                rij.push(std::mem::take(&mut veld));
                rijen.push(std::mem::take(&mut rij));
            }
            c => veld.push(c),
        }
    }
    if !veld.is_empty() || !rij.is_empty() {
        rij.push(veld);
        rijen.push(rij);
    }
    rijen
}

type Hoofd = fn(&mut Staat) -> R<()>;
type BijKnop = fn(&mut Staat, usize) -> R<()>;

fn foutzin(f: &Fout) -> String {
    let t = format!("Er ging iets mis op regel {}: {}", f.regel, f.tekst);
    logboek(&format!("FOUT: {}", t));
    t
}

// ---------------------------------------------------------------------
// Tekst-stand: voor proeven en voor computers zonder Windows.
// ---------------------------------------------------------------------
fn start_tekst(namen: &'static [&'static str], acties: &'static [&'static str], hoofd: Hoofd, bij_knop: BijKnop, stappen: Vec<(String, String)>) {
    let mut s = Staat { lokaal: Vec::new(), waarden: vec![None; namen.len()], namen, acties, stappen: 0, scherm: Scherm::tekst() };
    if let Err(f) = hoofd(&mut s) {
        println!("{}", foutzin(&f));
        std::process::exit(1);
    }
    for (soort, waarde) in stappen {
        let (naam, inhoud) = waarde.split_once('=').map(|(a, b)| (a.to_string(), b.to_string())).unwrap_or((waarde.clone(), String::new()));
        if soort != "--druk" {
            let Some(i) = s.scherm.onderdelen.iter().position(|o| o.naam.to_lowercase() == naam.to_lowercase()) else {
                println!("Er is geen onderdeel met de naam {}.", naam);
                continue;
            };
            match soort.as_str() {
                "--typ" => {
                    println!("[typ in {}] {}", naam, inhoud);
                    s.scherm.onderdelen[i].tekst = inhoud;
                }
                "--vink" => {
                    println!("[vink aan] {}", naam);
                    s.scherm.onderdelen[i].aan = true;
                }
                _ if s.scherm.onderdelen[i].soort == 6 => {
                    let o = &mut s.scherm.onderdelen[i];
                    o.gekozen = inhoud.trim().parse::<usize>().ok().filter(|n| *n >= 1 && *n <= o.rijen.len()).map(|n| n - 1);
                    println!("[kies in {}] rij {}", naam, inhoud);
                }
                _ => {
                    let o = &mut s.scherm.onderdelen[i];
                    o.gekozen = o.keuzes.iter().position(|k| k.to_lowercase() == inhoud.to_lowercase());
                    println!("[kies in {}] {}", naam, inhoud);
                }
            }
            continue;
        }
        let naam = waarde;
        println!("[druk op] {}", naam);
        let knop = s.scherm.knoppen.iter().find(|k| k.0.to_lowercase() == naam.to_lowercase()).map(|k| k.1);
        match knop {
            None => println!("Er is geen knop met de naam {}.", naam),
            Some(None) => println!("Deze knop doet nog niets."),
            Some(Some(a)) => {
                s.stappen = 0;
                let uitkomst = if a == OPDRACHT_ACTIE { s.opdracht_geven(bij_knop) } else { bij_knop(&mut s, a) };
                if let Err(f) = uitkomst {
                    println!("{}", foutzin(&f));
                    std::process::exit(1);
                }
            }
        }
    }
}

fn start(namen: &'static [&'static str], acties: &'static [&'static str], hoofd: Hoofd, bij_knop: BijKnop) {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tekststand = args.iter().any(|a| a == "--tekst") || !cfg!(windows);
    if tekststand && args.iter().any(|a| a == "--privacy") {
        let t = privacy_tekst();
        println!("{}", if t.is_empty() { "Dit programma heeft geen privacyverklaring.".to_string() } else { t });
        return;
    }
    if tekststand && args.iter().any(|a| a == "--handleiding") {
        let t = handleiding_tekst();
        println!("{}", if t.is_empty() { "Dit programma heeft geen handleiding.".to_string() } else { t });
        return;
    }
    if tekststand {
        let mut stappen = Vec::new();
        let mut i = 0;
        while i < args.len() {
            if matches!(args[i].as_str(), "--druk" | "--typ" | "--vink" | "--kies") && i + 1 < args.len() {
                stappen.push((args[i].clone(), args[i + 1].clone()));
                i += 1;
            }
            i += 1;
        }
        start_tekst(namen, acties, hoofd, bij_knop, stappen);
        return;
    }
    #[cfg(windows)]
    win::start(namen, acties, hoofd, bij_knop);
}

// ---------------------------------------------------------------------
// Het scherm.
// ---------------------------------------------------------------------
struct Scherm {
    tekststand: bool,
    /// (naam, actie) van elke knop, in volgorde.
    knoppen: Vec<(String, Option<usize>)>,
    onderdelen: Vec<Ond>,
    tabbladen: Vec<String>,
    uitvoer: Vec<String>,
    #[cfg(windows)]
    win: win::WinScherm,
}

impl Scherm {
    fn tekst() -> Self {
        Scherm {
            tekststand: true,
            knoppen: Vec::new(),
            onderdelen: Vec::new(),
            tabbladen: Vec::new(),
            uitvoer: Vec::new(),
            #[cfg(windows)]
            win: win::WinScherm::default(),
        }
    }
    fn onderdeel(&mut self, soort: u8, naam: &str) {
        let mut o = Ond { soort, naam: naam.to_string(), tekst: String::new(), aan: false, keuzes: Vec::new(), gekozen: None, rijen: Vec::new(), h: 0 };
        if soort == 2 {
            o.tekst = naam.to_string();
        }
        if self.tekststand {
            println!("[{}] {}", SOORT_WOORD[soort as usize], naam);
        } else {
            #[cfg(windows)]
            {
                o.h = self.win.onderdeel(soort, naam);
            }
        }
        self.onderdelen.push(o);
    }
    fn lees(&self, i: usize) -> W {
        let o = &self.onderdelen[i];
        if !self.tekststand {
            #[cfg(windows)]
            return win::lees(o);
        }
        match o.soort {
            3 => W::B(o.aan),
            4 | 5 => W::T(o.gekozen.and_then(|k| o.keuzes.get(k).cloned()).unwrap_or_default()),
            _ => W::T(o.tekst.clone()),
        }
    }
    fn zet_tekst(&mut self, i: usize, t: &str) {
        let o = &mut self.onderdelen[i];
        o.tekst = t.to_string();
        if self.tekststand {
            println!("[{} {}] {}", SOORT_WOORD[o.soort as usize], o.naam, t);
            return;
        }
        #[cfg(windows)]
        win::zet_tekst(o.h, t);
    }
    fn voeg_keuze(&mut self, i: usize, t: &str) {
        let o = &mut self.onderdelen[i];
        o.keuzes.push(t.to_string());
        if o.gekozen.is_none() {
            o.gekozen = Some(0);
        }
        if !self.tekststand {
            #[cfg(windows)]
            win::voeg_keuze(o.h, o.soort, t, o.keuzes.len() == 1);
        }
    }
    fn vink(&mut self, i: usize, aan: bool) {
        let o = &mut self.onderdelen[i];
        o.aan = aan;
        if self.tekststand {
            println!("[selectievakje {}] {}", o.naam, if aan { "aan" } else { "uit" });
            return;
        }
        #[cfg(windows)]
        win::vink(o.h, aan);
    }
    fn focus(&mut self, i: usize) {
        if self.tekststand {
            println!("[focus] {}", self.onderdelen[i].naam);
            return;
        }
        #[cfg(windows)]
        win::focus(self.onderdelen[i].h);
    }
    fn toon(&mut self, t: &str) {
        if self.tekststand {
            println!("{}", t);
            return;
        }
        #[cfg(windows)]
        {
            if self.win.vensters.is_empty() {
                self.uitvoer.push(t.to_string());
            } else {
                win::melding(self.win.actief(), t, "Melding");
            }
        }
    }
    fn venster(&mut self, titel: &str) {
        if self.tekststand {
            println!("[venster] {}", titel);
            return;
        }
        #[cfg(windows)]
        self.win.venster(titel);
    }
    fn knop(&mut self, naam: &str, actie: Option<usize>) {
        self.knoppen.push((naam.to_string(), actie));
        if self.tekststand {
            println!("[knop] {}", naam);
            return;
        }
        #[cfg(windows)]
        self.win.knop(naam, self.knoppen.len() - 1);
    }
}

// ---------------------------------------------------------------------
// Windows: echte vensters en knoppen, voorleesbaar en met het toetsenbord.
// ---------------------------------------------------------------------
#[cfg(windows)]
mod win {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    type HWND = *mut c_void;
    type HANDLE = *mut c_void;
    type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

    #[repr(C)]
    struct WNDCLASSEXW {
        cbSize: u32,
        style: u32,
        lpfnWndProc: Option<WndProc>,
        cbClsExtra: i32,
        cbWndExtra: i32,
        hInstance: HANDLE,
        hIcon: HANDLE,
        hCursor: HANDLE,
        hbrBackground: HANDLE,
        lpszMenuName: *const u16,
        lpszClassName: *const u16,
        hIconSm: HANDLE,
    }

    #[repr(C)]
    struct POINT {
        x: i32,
        y: i32,
    }

    #[repr(C)]
    struct MSG {
        hwnd: HWND,
        message: u32,
        wParam: usize,
        lParam: isize,
        time: u32,
        pt: POINT,
    }

    #[repr(C)]
    struct RECT {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn RegisterClassExW(wc: *const WNDCLASSEXW) -> u16;
        fn CreateWindowExW(ex: u32, class: *const u16, name: *const u16, style: u32, x: i32, y: i32, w: i32, h: i32, parent: HWND, menu: HANDLE, inst: HANDLE, param: *mut c_void) -> HWND;
        fn DefWindowProcW(h: HWND, m: u32, w: usize, l: isize) -> isize;
        fn ShowWindow(h: HWND, cmd: i32) -> i32;
        fn GetMessageW(m: *mut MSG, h: HWND, a: u32, b: u32) -> i32;
        fn TranslateMessage(m: *const MSG) -> i32;
        fn DispatchMessageW(m: *const MSG) -> isize;
        fn IsDialogMessageW(h: HWND, m: *mut MSG) -> i32;
        fn PostQuitMessage(code: i32);
        fn MessageBoxW(h: HWND, text: *const u16, title: *const u16, kind: u32) -> i32;
        fn GetForegroundWindow() -> HWND;
        fn IsZoomed(h: HWND) -> i32;
        fn SendMessageW(h: HWND, m: u32, w: usize, l: isize) -> isize;
        fn PostMessageW(h: HWND, m: u32, w: usize, l: isize) -> i32;
        fn SetFocus(h: HWND) -> HWND;
        fn GetFocus() -> HWND;
        fn SetForegroundWindow(h: HWND) -> i32;
        fn GetAncestor(h: HWND, flags: u32) -> HWND;
        fn DestroyWindow(h: HWND) -> i32;
        fn SetWindowPos(h: HWND, after: HWND, x: i32, y: i32, w: i32, hh: i32, flags: u32) -> i32;
        fn LoadCursorW(inst: HANDLE, name: *const u16) -> HANDLE;
        fn SetProcessDPIAware() -> i32;
        fn GetDC(h: HWND) -> HANDLE;
        fn ReleaseDC(h: HWND, dc: HANDLE) -> i32;
        fn GetClientRect(h: HWND, r: *mut RECT) -> i32;
        fn MoveWindow(h: HWND, x: i32, y: i32, w: i32, hh: i32, repaint: i32) -> i32;
        fn GetWindow(h: HWND, cmd: u32) -> HWND;
        fn IsWindow(h: HWND) -> i32;
        fn SetWindowTextW(h: HWND, text: *const u16) -> i32;
        fn GetWindowTextW(h: HWND, buf: *mut u16, max: i32) -> i32;
        fn GetWindowTextLengthW(h: HWND) -> i32;
        fn GetKeyState(toets: i32) -> i16;
        fn LoadImageW(inst: HANDLE, naam: *const u16, soort: u32, b: i32, h: i32, vlaggen: u32) -> HANDLE;
        fn FillRect(dc: HANDLE, r: *const RECT, kwast: HANDLE) -> i32;
        fn RedrawWindow(h: HWND, r: *const RECT, regio: HANDLE, vlaggen: u32) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> HANDLE;
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateFontW(h: i32, w: i32, esc: i32, ori: i32, weight: i32, italic: u32, underline: u32, strike: u32, charset: u32, outp: u32, clip: u32, quality: u32, pitch: u32, face: *const u16) -> HANDLE;
        fn GetDeviceCaps(dc: HANDLE, index: i32) -> i32;
        fn CreateSolidBrush(kleur: u32) -> HANDLE;
        fn SetTextColor(dc: HANDLE, kleur: u32) -> u32;
        fn SetBkColor(dc: HANDLE, kleur: u32) -> u32;
    }

    const WM_DESTROY: u32 = 0x0002;
    const WM_SIZE: u32 = 0x0005;
    const WM_CLOSE: u32 = 0x0010;
    const WM_SETFONT: u32 = 0x0030;
    const WM_KEYDOWN: u32 = 0x0100;
    const WM_COMMAND: u32 = 0x0111;
    const BM_CLICK: u32 = 0x00F5;
    const EM_SETSEL: u32 = 0x00B1;
    const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
    const WS_CHILD: u32 = 0x4000_0000;
    const WS_CLIPCHILDREN: u32 = 0x0200_0000;
    const WS_VISIBLE: u32 = 0x1000_0000;
    const WS_TABSTOP: u32 = 0x0001_0000;
    const WS_VSCROLL: u32 = 0x0020_0000;
    const WS_EX_CONTROLPARENT: u32 = 0x0001_0000;
    const WS_EX_CLIENTEDGE: u32 = 0x0000_0200;
    const ES_MULTILINE: u32 = 0x0004;
    const ES_AUTOVSCROLL: u32 = 0x0040;
    const ES_READONLY: u32 = 0x0800;
    const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;
    const SW_SHOW: i32 = 5;
    const MB_ICONINFORMATION: u32 = 0x40;
    const MB_ICONERROR: u32 = 0x10;
    const MB_SETFOREGROUND: u32 = 0x0001_0000;
    const VK_ESCAPE: usize = 0x1B;
    const VK_RETURN: usize = 0x0D;
    const GA_ROOT: u32 = 2;
    const GW_CHILD: u32 = 5;
    const SWP_NOMOVE: u32 = 0x0002;
    const SWP_NOZORDER: u32 = 0x0004;
    const LOGPIXELSY: i32 = 90;
    const KNOP_ID: usize = 1000;
    const WS_BORDER: u32 = 0x0080_0000;
    const ES_AUTOHSCROLL: u32 = 0x0080;
    const ES_WANTRETURN: u32 = 0x1000;
    const BS_AUTOCHECKBOX: u32 = 0x0003;
    const LBS_NOTIFY: u32 = 0x0001;
    const CBS_DROPDOWNLIST: u32 = 0x0003;
    const BM_GETCHECK: u32 = 0x00F0;
    const BM_SETCHECK: u32 = 0x00F1;
    const LB_ADDSTRING: u32 = 0x0180;
    const LB_SETCURSEL: u32 = 0x0186;
    const LB_GETCURSEL: u32 = 0x0188;
    const LB_GETTEXT: u32 = 0x0189;
    const LB_GETTEXTLEN: u32 = 0x018A;
    const CB_ADDSTRING: u32 = 0x0143;
    const CB_GETCURSEL: u32 = 0x0147;
    const CB_GETLBTEXT: u32 = 0x0148;
    const CB_GETLBTEXTLEN: u32 = 0x0149;
    const CB_SETCURSEL: u32 = 0x014E;

    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    thread_local! {
        static STAAT: RefCell<Option<Staat>> = RefCell::new(None);
        static BIJ_KNOP: Cell<Option<BijKnop>> = Cell::new(None);
        static OPEN: Cell<usize> = Cell::new(0);
        static KNOP_HWNDS: RefCell<Vec<HWND>> = RefCell::new(Vec::new());
        /// Tekstvensters (uitvoer, handleiding) met hun tekstvak, zodat het vak meegroeit met het venster.
        static TEKSTVAKKEN: RefCell<Vec<(HWND, HWND)>> = RefCell::new(Vec::new());
        /// (toets, ctrl, alt, shift, knop of venster, menunummer of 0)
        static SNELTOETSEN: RefCell<Vec<(u16, bool, bool, bool, HWND, usize)>> = RefCell::new(Vec::new());
        /// Per venster: achtergrondkleur, tekstkleur en de kwast voor de achtergrond.
        static KLEUREN: RefCell<Vec<(HWND, Option<u32>, Option<u32>, HANDLE)>> = RefCell::new(Vec::new());
        static HANDLEIDING_VENSTER: Cell<HWND> = Cell::new(std::ptr::null_mut());
    }

    #[derive(Default)]
    pub(super) struct WinScherm {
        pub(super) vensters: Vec<HWND>,
        /// Per venster: waar het volgende onderdeel komt, en of er al iets de focus heeft.
        volgende_y: Vec<i32>,
        heeft_focus: Vec<bool>,
        /// Per venster: hoe breed het minstens moet zijn (voor een werkbalk).
        breedte: Vec<(HWND, i32)>,
        pub(super) klik_venster: Option<HWND>,
        lettertype: Option<HANDLE>,
        schaal: f32,
    }

    #[repr(C)]
    struct SYSTEMTIME {
        jaar: u16,
        maand: u16,
        weekdag: u16,
        dag: u16,
        uur: u16,
        minuut: u16,
        seconde: u16,
        ms: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLocalTime(t: *mut SYSTEMTIME);
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetFolderPathW(h: HWND, csidl: i32, token: HANDLE, flags: u32, path: *mut u16) -> i32;
    }

    pub(super) fn nu() -> (u32, u32, u32, u32, u32, u32) {
        let mut t = SYSTEMTIME { jaar: 0, maand: 0, weekdag: 0, dag: 0, uur: 0, minuut: 0, seconde: 0, ms: 0 };
        unsafe {
            GetLocalTime(&mut t);
        }
        (t.jaar as u32, t.maand as u32, t.dag as u32, t.uur as u32, t.minuut as u32, t.seconde as u32)
    }

    pub(super) fn documenten() -> Option<std::path::PathBuf> {
        let mut buf = [0u16; 520];
        let r = unsafe { SHGetFolderPathW(null_mut(), 0x05, null_mut(), 0, buf.as_mut_ptr()) };
        if r != 0 {
            return None;
        }
        let n = buf.iter().position(|&c| c == 0).unwrap_or(0);
        Some(std::path::PathBuf::from(String::from_utf16_lossy(&buf[..n])))
    }

    #[link(name = "winmm")]
    extern "system" {
        fn PlaySoundW(pad: *const u16, module: HANDLE, vlaggen: u32) -> i32;
        fn mciSendStringW(opdracht: *const u16, antwoord: *mut u16, lengte: u32, venster: HWND) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn Beep(hoogte: u32, duur: u32) -> i32;
    }

    pub(super) fn piep(h: u32, d: u32) {
        unsafe {
            Beep(h, d);
        }
    }

    thread_local! {
        /// Het geluid in het geheugen moet blijven bestaan zolang het speelt.
        static GELUID: std::cell::RefCell<Vec<u8>> = const { std::cell::RefCell::new(Vec::new()) };
    }

    pub(super) fn speel_geheugen(wav: Vec<u8>, wachten: bool) -> bool {
        unsafe {
            // Eerst het vorige geluid stoppen, dan pas het geheugen vervangen.
            PlaySoundW(null(), null_mut(), 0);
        }
        GELUID.with(|g| *g.borrow_mut() = wav);
        // SND_MEMORY | SND_NODEFAULT, en SND_ASYNC als we niet wachten.
        let vlaggen = 0x0004 | 0x0002 | if wachten { 0 } else { 0x0001 };
        GELUID.with(|g| unsafe { PlaySoundW(g.borrow().as_ptr() as *const u16, null_mut(), vlaggen) != 0 })
    }

    pub(super) fn speel(pad: &std::path::Path, wachten: bool) -> bool {
        // SND_FILENAME | SND_NODEFAULT, en SND_ASYNC als we niet wachten.
        let vlaggen = 0x0002_0000 | 0x0002 | if wachten { 0 } else { 0x0001 };
        unsafe { PlaySoundW(w(&pad.to_string_lossy()).as_ptr(), null_mut(), vlaggen) != 0 }
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(naam: *const u16) -> HANDLE;
        fn GetProcAddress(module: HANDLE, naam: *const u8) -> *const c_void;
    }

    /// Laat de schermlezer iets zeggen via een UI Automation-melding (zonder meldingsvenster).
    pub(super) fn zeg(venster: HWND, tekst: &str) -> bool {
        if venster.is_null() {
            return false;
        }
        unsafe {
            let uia = LoadLibraryW(w("uiautomationcore.dll").as_ptr());
            let ole = LoadLibraryW(w("oleaut32.dll").as_ptr());
            if uia.is_null() || ole.is_null() {
                return false;
            }
            let host = GetProcAddress(uia, b"UiaHostProviderFromHwnd\0".as_ptr());
            let melding = GetProcAddress(uia, b"UiaRaiseNotificationEvent\0".as_ptr());
            let maak = GetProcAddress(ole, b"SysAllocString\0".as_ptr());
            let vrij = GetProcAddress(ole, b"SysFreeString\0".as_ptr());
            if host.is_null() || melding.is_null() || maak.is_null() || vrij.is_null() {
                return false;
            }
            type Host = unsafe extern "system" fn(HWND, *mut *mut c_void) -> i32;
            type Melding = unsafe extern "system" fn(*mut c_void, i32, i32, *mut u16, *mut u16) -> i32;
            type Maak = unsafe extern "system" fn(*const u16) -> *mut u16;
            type Vrij = unsafe extern "system" fn(*mut u16);
            type Los = unsafe extern "system" fn(*mut c_void) -> u32;
            let host: Host = std::mem::transmute(host);
            let melding: Melding = std::mem::transmute(melding);
            let maak: Maak = std::mem::transmute(maak);
            let vrij: Vrij = std::mem::transmute(vrij);
            let mut p: *mut c_void = null_mut();
            if host(venster, &mut p) != 0 || p.is_null() {
                return false;
            }
            let t = maak(w(tekst).as_ptr());
            let id = maak(w("JaapsProgrammeertaal").as_ptr());
            // Soort: Other (4). Verwerking: All (2), zodat geen melding wegvalt.
            let gelukt = melding(p, 4, 2, t, id) == 0;
            vrij(t);
            vrij(id);
            let vtabel = *(p as *const *const usize);
            let los: Los = std::mem::transmute(*vtabel.add(2));
            los(p);
            gelukt
        }
    }

    #[link(name = "user32")]
    extern "system" {
        fn CreateMenu() -> HANDLE;
        fn CreatePopupMenu() -> HANDLE;
        fn AppendMenuW(menu: HANDLE, vlaggen: u32, id: usize, tekst: *const u16) -> i32;
        fn ModifyMenuW(menu: HANDLE, plek: u32, vlaggen: u32, id: usize, tekst: *const u16) -> i32;
        fn SetMenu(h: HWND, menu: HANDLE) -> i32;
        fn DrawMenuBar(h: HWND) -> i32;
    }

    #[link(name = "winmm")]
    extern "system" {
        fn waveOutSetVolume(apparaat: HANDLE, volume: u32) -> u32;
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(r: *mut c_void, vlaggen: u32) -> i32;
        fn CoCreateInstance(clsid: *const [u8; 16], buiten: *mut c_void, ctx: u32, iid: *const [u8; 16], uit: *mut *mut c_void) -> i32;
    }

    #[repr(C)]
    struct SHFILEOPSTRUCTW {
        hwnd: HWND,
        functie: u32,
        van: *const u16,
        naar: *const u16,
        vlaggen: u16,
        afgebroken: i32,
        namen: *mut c_void,
        titel: *const u16,
    }

    #[link(name = "shell32")]
    extern "system" {
        fn SHFileOperationW(op: *mut SHFILEOPSTRUCTW) -> i32;
    }

    #[link(name = "shell32")]
    extern "system" {
        fn ShellExecuteW(h: HWND, wat: *const u16, bestand: *const u16, opties: *const u16, map: *const u16, toon: i32) -> isize;
    }

    /// Een programma, bestand, map of webpagina openen zoals Windows dat zelf doet.
    pub(super) fn open(naam: &str) -> bool {
        let wat = w("open");
        let b = w(naam);
        unsafe { ShellExecuteW(std::ptr::null_mut(), wat.as_ptr(), b.as_ptr(), std::ptr::null(), std::ptr::null(), 1) > 32 }
    }

    #[repr(C)]
    struct TCITEMW {
        mask: u32,
        state: u32,
        state_mask: u32,
        tekst: *const u16,
        max: i32,
        beeld: i32,
        param: isize,
    }

    #[repr(C)]
    struct INITCOMMONCONTROLSEX {
        grootte: u32,
        soorten: u32,
    }

    #[link(name = "comctl32")]
    extern "system" {
        fn InitCommonControlsEx(i: *const INITCOMMONCONTROLSEX) -> i32;
    }

    /// De tabbladen van één venster: het Windows-onderdeel, en per tabblad de onderdelen erop.
    pub(super) struct Tabs {
        venster: HWND,
        ctrl: HWND,
        namen: Vec<String>,
        onderdelen: Vec<Vec<HWND>>,
        y: Vec<i32>,
        begin_y: i32,
    }

    thread_local! {
        pub(super) static TABS: RefCell<Vec<Tabs>> = RefCell::new(Vec::new());
    }

    /// Laat alleen de onderdelen van tabblad `nr` zien.
    pub(super) fn toon_tab(ctrl: HWND, nr: usize) {
        TABS.with(|t| {
            if let Some(tabs) = t.borrow().iter().find(|x| x.ctrl == ctrl) {
                for (i, lijst) in tabs.onderdelen.iter().enumerate() {
                    for h in lijst {
                        unsafe {
                            ShowWindow(*h, if i == nr { SW_SHOW } else { 0 });
                        }
                    }
                }
            }
        });
    }

    /// Ctrl+Tab of Ctrl+Shift+Tab: naar het volgende of vorige tabblad. Geeft true als er tabbladen zijn.
    pub(super) fn wissel_tab(venster: HWND, terug: bool) -> bool {
        let Some((ctrl, aantal)) = TABS.with(|t| t.borrow().iter().find(|x| x.venster == venster).map(|x| (x.ctrl, x.namen.len()))) else {
            return false;
        };
        unsafe {
            let nu = SendMessageW(ctrl, 0x130B, 0, 0).max(0) as usize;
            let nieuw = if terug { (nu + aantal - 1) % aantal } else { (nu + 1) % aantal };
            SendMessageW(ctrl, 0x130C, nieuw, 0);
            toon_tab(ctrl, nieuw);
            SetFocus(ctrl);
        }
        true
    }

    pub(super) fn kies_tabblad(naam: &str) {
        let gevonden = TABS.with(|t| {
            t.borrow().iter().find_map(|x| x.namen.iter().position(|n| n.to_lowercase() == naam.to_lowercase()).map(|i| (x.ctrl, i)))
        });
        if let Some((ctrl, i)) = gevonden {
            unsafe {
                SendMessageW(ctrl, 0x130C, i, 0);
                toon_tab(ctrl, i);
                SetFocus(ctrl);
            }
        }
    }

    thread_local! {
        /// Per venster: de menubalk en het laatste menu.
        static MENUS: RefCell<Vec<(HWND, HANDLE, HANDLE)>> = RefCell::new(Vec::new());
        /// Per menuregel (nummer van de knop): het menu waar hij in staat.
        static MENUREGELS: RefCell<Vec<(usize, HWND, HANDLE)>> = RefCell::new(Vec::new());
        static STEM: Cell<*mut c_void> = Cell::new(std::ptr::null_mut());
    }

    fn guid(d1: u32, d2: u16, d3: u16, d4: [u8; 8]) -> [u8; 16] {
        let mut g = [0u8; 16];
        g[..4].copy_from_slice(&d1.to_le_bytes());
        g[4..6].copy_from_slice(&d2.to_le_bytes());
        g[6..8].copy_from_slice(&d3.to_le_bytes());
        g[8..].copy_from_slice(&d4);
        g
    }

    impl WinScherm {
        pub(super) fn tabblad(&mut self, naam: &str) {
            let Some(&venster) = self.vensters.last() else { return };
            let heeft = TABS.with(|t| t.borrow().iter().any(|x| x.venster == venster));
            if !heeft {
                unsafe {
                    InitCommonControlsEx(&INITCOMMONCONTROLSEX { grootte: 8, soorten: 0x8 });
                }
                let y = *self.volgende_y.last().unwrap();
                let ctrl = self.plaats("SysTabControl32", "", WS_TABSTOP | 0x0400_0000, 0, 44, 0, true);
                TABS.with(|t| {
                    t.borrow_mut().push(Tabs { venster, ctrl, namen: Vec::new(), onderdelen: Vec::new(), y: Vec::new(), begin_y: y + self.s(56) })
                });
            }
            TABS.with(|t| {
                let mut t = t.borrow_mut();
                let tabs = t.iter_mut().find(|x| x.venster == venster).unwrap();
                let tekst = w(naam);
                let item = TCITEMW { mask: 1, state: 0, state_mask: 0, tekst: tekst.as_ptr(), max: 0, beeld: 0, param: 0 };
                unsafe {
                    SendMessageW(tabs.ctrl, 0x133E, tabs.namen.len(), &item as *const TCITEMW as isize);
                }
                tabs.namen.push(naam.to_string());
                tabs.onderdelen.push(Vec::new());
                let begin = tabs.begin_y;
                tabs.y.push(begin);
            });
        }

        pub(super) fn menu(&mut self, naam: &str) {
            let Some(&venster) = self.vensters.last() else { return };
            unsafe {
                let popup = CreatePopupMenu();
                MENUS.with(|m| {
                    let mut m = m.borrow_mut();
                    let balk = match m.iter().find(|x| x.0 == venster) {
                        Some(x) => x.1,
                        None => {
                            let b = CreateMenu();
                            SetMenu(venster, b);
                            b
                        }
                    };
                    AppendMenuW(balk, 0x10, popup as usize, w(naam).as_ptr());
                    m.retain(|x| x.0 != venster);
                    m.push((venster, balk, popup));
                });
                DrawMenuBar(venster);
            }
        }

        pub(super) fn menuregel(&mut self, naam: &str, nummer: usize) {
            let Some(&venster) = self.vensters.last() else { return };
            let Some(popup) = MENUS.with(|m| m.borrow().iter().find(|x| x.0 == venster).map(|x| x.2)) else { return };
            unsafe {
                AppendMenuW(popup, 0, KNOP_ID + nummer, w(naam).as_ptr());
            }
            // Houd de lijst met knoppen gelijk: deze plek heeft geen knop.
            KNOP_HWNDS.with(|k| k.borrow_mut().push(null_mut()));
            MENUREGELS.with(|m| m.borrow_mut().push((nummer, venster, popup)));
        }
    }

    /// De Windows-stem (SAPI) laat tekst voorlezen. Gaat meteen verder (niet wachten).
    pub(super) fn lees_voor(tekst: &str) -> bool {
        unsafe {
            let mut stem = STEM.with(|s| s.get());
            if stem.is_null() {
                CoInitializeEx(null_mut(), 2);
                let clsid = guid(0x9674_9377, 0x3391, 0x11D2, [0x9E, 0xE3, 0x00, 0xC0, 0x4F, 0x79, 0x73, 0x96]);
                let iid = guid(0x6C44_DF74, 0x72B9, 0x4992, [0xA1, 0xEC, 0xEF, 0x99, 0x6E, 0x04, 0x22, 0xD4]);
                if CoCreateInstance(&clsid, null_mut(), 1 | 4, &iid, &mut stem) != 0 || stem.is_null() {
                    return false;
                }
                STEM.with(|s| s.set(stem));
            }
            let vtabel = *(stem as *const *const usize);
            type Volume = unsafe extern "system" fn(*mut c_void, u16) -> i32;
            type Spreek = unsafe extern "system" fn(*mut c_void, *const u16, u32, *mut u32) -> i32;
            let volume: Volume = std::mem::transmute(*vtabel.add(30));
            volume(stem, VOLUME.with(|v| v.get()) as u16);
            let spreek: Spreek = std::mem::transmute(*vtabel.add(20));
            // SPF_ASYNC (1): meteen verder. SPF_IS_NOT_XML (16): tekens zoals < gewoon voorlezen.
            spreek(stem, w(tekst).as_ptr(), 1 | 16, null_mut()) >= 0
        }
    }

    pub(super) fn stop_voorlezen() {
        let stem = STEM.with(|s| s.get());
        if stem.is_null() {
            return;
        }
        unsafe {
            let vtabel = *(stem as *const *const usize);
            type Spreek = unsafe extern "system" fn(*mut c_void, *const u16, u32, *mut u32) -> i32;
            let spreek: Spreek = std::mem::transmute(*vtabel.add(20));
            // SPF_PURGEBEFORESPEAK (2) met een lege tekst: stoppen.
            spreek(stem, w("").as_ptr(), 1 | 2, null_mut());
        }
    }

    pub(super) fn zet_volume(v: u32) {
        let kanaal = v * 0xFFFF / 100;
        unsafe {
            waveOutSetVolume(null_mut(), kanaal | (kanaal << 16));
            mciSendStringW(w(&format!("setaudio jaapgeluid volume to {}", v * 10)).as_ptr(), null_mut(), 0, null_mut());
        }
    }

    /// Naar de Prullenbak, zodat je het terug kunt halen.
    pub(super) fn naar_prullenbak(pad: &std::path::Path) -> bool {
        let van: Vec<u16> = pad.to_string_lossy().encode_utf16().chain([0, 0]).collect();
        let mut op = SHFILEOPSTRUCTW {
            hwnd: null_mut(),
            functie: 3,
            van: van.as_ptr(),
            naar: null(),
            vlaggen: 0x40 | 0x10 | 0x04 | 0x400,
            afgebroken: 0,
            namen: null_mut(),
            titel: null(),
        };
        unsafe { SHFileOperationW(&mut op) == 0 && !pad.exists() }
    }

    /// Het pictogram van het programma op een venster zetten (titelbalk en taakbalk).
    pub(super) fn zet_pictogram(venster: HWND) {
        if PICTOGRAM.is_empty() {
            return;
        }
        let naam = std::env::current_exe().ok().and_then(|p| p.file_stem().map(|n| n.to_string_lossy().to_string())).unwrap_or_default();
        let map = std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir).join(naam);
        let _ = std::fs::create_dir_all(&map);
        let pad = map.join("pictogram.ico");
        if std::fs::read(&pad).map_or(true, |b| b != PICTOGRAM) {
            let _ = std::fs::write(&pad, PICTOGRAM);
        }
        unsafe {
            for (soort, maat) in [(1usize, 0i32), (0usize, 16i32)] {
                let ico = LoadImageW(null_mut(), w(&pad.to_string_lossy()).as_ptr(), 1, maat, maat, 0x10 | if maat == 0 { 0x40 } else { 0 });
                if !ico.is_null() {
                    SendMessageW(venster, 0x0080, soort, ico as isize);
                }
            }
        }
    }

    pub(super) fn stop_geluid() {
        unsafe {
            PlaySoundW(null(), null_mut(), 0);
            mciSendStringW(w("close jaapgeluid").as_ptr(), null_mut(), 0, null_mut());
        }
    }

    /// MP3 en andere soorten, via de geluidsonderdelen van Windows zelf (MCI).
    pub(super) fn speel_mci(pad: &std::path::Path, wachten: bool) -> bool {
        unsafe {
            mciSendStringW(w("close jaapgeluid").as_ptr(), null_mut(), 0, null_mut());
            let open = format!("open \"{}\" type mpegvideo alias jaapgeluid", pad.display());
            if mciSendStringW(w(&open).as_ptr(), null_mut(), 0, null_mut()) != 0 {
                return false;
            }
            let volume = format!("setaudio jaapgeluid volume to {}", VOLUME.with(|v| v.get()) * 10);
            mciSendStringW(w(&volume).as_ptr(), null_mut(), 0, null_mut());
            let speel = if wachten { "play jaapgeluid wait" } else { "play jaapgeluid" };
            mciSendStringW(w(speel).as_ptr(), null_mut(), 0, null_mut()) == 0
        }
    }

    pub(super) fn sneltoets(knop: usize, code: u16, ctrl: bool, alt: bool, shift: bool, tekst: &str) {
        let Some(h) = KNOP_HWNDS.with(|k| k.borrow().get(knop).copied()) else { return };
        if h.is_null() {
            // Een menuregel: de sneltoets komt achter de naam in het menu, zoals in elk Windows-programma.
            if let Some((_, venster, popup)) = MENUREGELS.with(|m| m.borrow().iter().find(|x| x.0 == knop).copied()) {
                let naam = tekst.rsplit_once(" (").map(|(n, t)| format!("{}\t{}", n, t.trim_end_matches(')'))).unwrap_or(tekst.to_string());
                unsafe {
                    ModifyMenuW(popup, (KNOP_ID + knop) as u32, 0, KNOP_ID + knop, w(&naam).as_ptr());
                }
                SNELTOETSEN.with(|s| s.borrow_mut().push((code, ctrl, alt, shift, venster, KNOP_ID + knop)));
            }
            return;
        }
        unsafe {
            // De sneltoets staat ook in de naam van de knop: zo hoor je hem.
            SetWindowTextW(h, w(tekst).as_ptr());
        }
        SNELTOETSEN.with(|s| s.borrow_mut().push((code, ctrl, alt, shift, h, 0)));
    }

    fn rgb(k: (u8, u8, u8)) -> u32 {
        k.0 as u32 | (k.1 as u32) << 8 | (k.2 as u32) << 16
    }

    thread_local! {
        pub(super) static AFSLUITVRAAG: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(attr: *const c_void, eigenaar: i32, naam: *const u16) -> HANDLE;
        fn GetLastError() -> u32;
    }

    impl WinScherm {
        /// 0 schermvullend, 1 vraag bij afsluiten, 2 maar één keer open, 3 tip bij een knop.
        pub(super) fn gedrag(&mut self, soort: u8, a: &str, b: &str) {
            unsafe {
                match soort {
                    0 => {
                        if let Some(&v) = self.vensters.last() {
                            ShowWindow(v, 3); // SW_MAXIMIZE
                        }
                    }
                    1 => AFSLUITVRAAG.with(|v| *v.borrow_mut() = Some(a.to_string())),
                    2 => {
                        let naam = format!("JaapsProgrammeertaal-{}", if super::PROGRAMMANAAM.is_empty() { "programma" } else { super::PROGRAMMANAAM });
                        CreateMutexW(null(), 0, w(&naam).as_ptr());
                        if GetLastError() == 183 {
                            MessageBoxW(null_mut(), w("Dit programma is al open. Kijk in de taakbalk (Windows-toets plus T).").as_ptr(), w("Al open").as_ptr(), 0x40);
                            std::process::exit(0);
                        }
                    }
                    _ => {
                        // Een tip (tooltip) bij de knop met deze naam.
                        let knop = KNOP_HWNDS.with(|k| {
                            k.borrow().iter().copied().find(|&h| {
                                let mut buf = [0u16; 256];
                                let n = GetWindowTextW(h, buf.as_mut_ptr(), 256);
                                String::from_utf16_lossy(&buf[..n.max(0) as usize]).to_lowercase().starts_with(&a.to_lowercase())
                            })
                        });
                        let (Some(knop), Some(&venster)) = (knop, self.vensters.last()) else { return };
                        #[repr(C)]
                        struct TOOLINFOW {
                            grootte: u32,
                            vlaggen: u32,
                            hwnd: HWND,
                            id: usize,
                            rect: [i32; 4],
                            inst: HANDLE,
                            tekst: *const u16,
                            lparam: isize,
                            reserve: *mut c_void,
                        }
                        let tip = CreateWindowExW(0x8, w("tooltips_class32").as_ptr(), null(), 0x80000000 | 0x01 | 0x02, 0, 0, 0, 0, venster, null_mut(), GetModuleHandleW(null()), null_mut());
                        let tekst = w(b);
                        let info = TOOLINFOW {
                            grootte: std::mem::size_of::<TOOLINFOW>() as u32,
                            vlaggen: 0x01 | 0x10,
                            hwnd: venster,
                            id: knop as usize,
                            rect: [0; 4],
                            inst: null_mut(),
                            tekst: tekst.as_ptr(),
                            lparam: 0,
                            reserve: null_mut(),
                        };
                        SendMessageW(tip, 0x0432, 0, &info as *const TOOLINFOW as isize);
                    }
                }
            }
        }
    }

    pub(super) fn vraag_ja_nee(tekst: &str) -> bool {
        unsafe { MessageBoxW(GetForegroundWindow(), w(tekst).as_ptr(), w("Vraag").as_ptr(), 0x4 | 0x20 | MB_SETFOREGROUND) == 6 }
    }

    pub(super) fn vraag_ja(eigenaar: HWND, tekst: &str) -> bool {
        unsafe { MessageBoxW(eigenaar, w(tekst).as_ptr(), w("Update").as_ptr(), 0x4 | 0x20 | MB_SETFOREGROUND) == 6 }
    }

    #[link(name = "winhttp")]
    extern "system" {
        fn WinHttpOpen(agent: *const u16, toegang: u32, proxy: *const u16, behalve: *const u16, vlaggen: u32) -> HANDLE;
        fn WinHttpConnect(sessie: HANDLE, host: *const u16, poort: u16, r: u32) -> HANDLE;
        fn WinHttpOpenRequest(c: HANDLE, werk: *const u16, pad: *const u16, versie: *const u16, verwijzer: *const u16, soorten: *const *const u16, vlaggen: u32) -> HANDLE;
        fn WinHttpSendRequest(v: HANDLE, kop: *const u16, koplengte: u32, inhoud: *const c_void, lengte: u32, totaal: u32, ctx: usize) -> i32;
        fn WinHttpReceiveResponse(v: HANDLE, r: *mut c_void) -> i32;
        fn WinHttpQueryHeaders(v: HANDLE, info: u32, naam: *const u16, buf: *mut c_void, lengte: *mut u32, index: *mut u32) -> i32;
        fn WinHttpReadData(v: HANDLE, buf: *mut c_void, lengte: u32, gelezen: *mut u32) -> i32;
        fn WinHttpCloseHandle(h: HANDLE) -> i32;
    }

    pub(super) fn internet(methode: &str, adres: &str, inhoud: Option<&[u8]>) -> Result<Vec<u8>, String> {
        let (veilig, rest) = if let Some(r) = adres.strip_prefix("https://") {
            (true, r)
        } else if let Some(r) = adres.strip_prefix("http://") {
            (false, r)
        } else {
            return Err(format!("Een internetadres begint met https:// of http://, en niet zoals {}.", adres));
        };
        let (hostdeel, pad) = match rest.find('/') {
            Some(p) => (&rest[..p], &rest[p..]),
            None => (rest, "/"),
        };
        let (host, poort) = match hostdeel.rsplit_once(':') {
            Some((h, p)) if p.parse::<u16>().is_ok() => (h, p.parse::<u16>().unwrap()),
            _ => (hostdeel, if veilig { 443 } else { 80 }),
        };
        unsafe {
            let sessie = WinHttpOpen(w("JaapsProgrammeertaal").as_ptr(), 4, null(), null(), 0);
            let sessie = if sessie.is_null() { WinHttpOpen(w("JaapsProgrammeertaal").as_ptr(), 0, null(), null(), 0) } else { sessie };
            if sessie.is_null() {
                return Err("Internet kan ik niet openen op deze computer.".into());
            }
            let sluit = |h: HANDLE| {
                if !h.is_null() {
                    WinHttpCloseHandle(h);
                }
            };
            let verbinding = WinHttpConnect(sessie, w(host).as_ptr(), poort, 0);
            let verzoek = if verbinding.is_null() {
                null_mut()
            } else {
                WinHttpOpenRequest(verbinding, w(methode).as_ptr(), w(pad).as_ptr(), null(), null(), null(), if veilig { 0x0080_0000 } else { 0 })
            };
            let mut uitkomst = Err(format!("Ik kan {} niet bereiken. Is er internet?", host));
            if !verzoek.is_null() {
                let kop = w("Content-Type: text/plain; charset=utf-8");
                let (kp, kl) = if inhoud.is_some() { (kop.as_ptr(), u32::MAX) } else { (null(), 0) };
                let data = inhoud.unwrap_or(&[]);
                if WinHttpSendRequest(verzoek, kp, kl, data.as_ptr() as *const c_void, data.len() as u32, data.len() as u32, 0) != 0
                    && WinHttpReceiveResponse(verzoek, null_mut()) != 0
                {
                    let mut status: u32 = 0;
                    let mut lengte: u32 = 4;
                    WinHttpQueryHeaders(verzoek, 19 | 0x2000_0000, null(), &mut status as *mut u32 as *mut c_void, &mut lengte, null_mut());
                    let mut alles = Vec::new();
                    let mut buf = vec![0u8; 65536];
                    loop {
                        let mut gelezen: u32 = 0;
                        if WinHttpReadData(verzoek, buf.as_mut_ptr() as *mut c_void, buf.len() as u32, &mut gelezen) == 0 || gelezen == 0 {
                            break;
                        }
                        alles.extend_from_slice(&buf[..gelezen as usize]);
                    }
                    uitkomst = if (200..300).contains(&status) {
                        Ok(alles)
                    } else {
                        Err(format!("De website {} gaf een fout terug (code {}).", host, status))
                    };
                }
            }
            sluit(verzoek);
            sluit(verbinding);
            sluit(sessie);
            uitkomst
        }
    }

    pub(super) fn melding(eigenaar: HWND, tekst: &str, titel: &str) {
        unsafe {
            MessageBoxW(eigenaar, w(tekst).as_ptr(), w(titel).as_ptr(), MB_ICONINFORMATION | MB_SETFOREGROUND);
        }
    }

    fn foutmelding(eigenaar: HWND, tekst: &str) {
        unsafe {
            MessageBoxW(eigenaar, w(tekst).as_ptr(), w("Fout in het programma").as_ptr(), MB_ICONERROR | MB_SETFOREGROUND);
        }
    }

    impl WinScherm {
        pub(super) fn actief(&self) -> HWND {
            self.klik_venster.or(self.vensters.last().copied()).unwrap_or(null_mut())
        }

        fn voorbereiden(&mut self) {
            if self.lettertype.is_some() {
                return;
            }
            unsafe {
                let dc = GetDC(null_mut());
                let dpi = GetDeviceCaps(dc, LOGPIXELSY);
                ReleaseDC(null_mut(), dc);
                self.schaal = if dpi > 0 { dpi as f32 / 96.0 } else { 1.0 };
                let hoogte = -((22.0 * self.schaal) as i32);
                self.lettertype = Some(CreateFontW(hoogte, 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w("Segoe UI").as_ptr()));
            }
        }

        fn s(&self, px: i32) -> i32 {
            (px as f32 * self.schaal) as i32
        }

        pub(super) fn venster(&mut self, titel: &str) {
            self.voorbereiden();
            unsafe {
                let h = CreateWindowExW(
                    WS_EX_CONTROLPARENT,
                    klasse().as_ptr(),
                    w(titel).as_ptr(),
                    WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
                    CW_USEDEFAULT,
                    CW_USEDEFAULT,
                    self.s(460),
                    self.s(220),
                    null_mut(),
                    null_mut(),
                    GetModuleHandleW(null()),
                    null_mut(),
                );
                OPEN.with(|o| o.set(o.get() + 1));
                zet_pictogram(h);
                self.vensters.push(h);
                self.volgende_y.push(20);
                self.heeft_focus.push(false);
                SetForegroundWindow(h);
                SetFocus(h);
            }
        }

        /// Zet een Windows-onderdeel onder het vorige in het laatste venster.
        fn plaats(&mut self, klasse: &str, tekst: &str, stijl: u32, ex: u32, hoogte: i32, id: usize, focus: bool) -> HWND {
            let Some(&venster) = self.vensters.last() else { return null_mut() };
            // Op een tabblad: onder de vorige onderdelen van dat tabblad.
            let tab = if klasse == "SysTabControl32" {
                None
            } else {
                TABS.with(|t| t.borrow().iter().find(|x| x.venster == venster).map(|x| (x.namen.len() - 1, *x.y.last().unwrap())))
            };
            let y = tab.map(|t| t.1).unwrap_or(*self.volgende_y.last().unwrap());
            unsafe {
                let h = CreateWindowExW(
                    ex,
                    w(klasse).as_ptr(),
                    w(tekst).as_ptr(),
                    WS_CHILD | WS_VISIBLE | stijl,
                    self.s(20),
                    y,
                    self.s(400),
                    self.s(hoogte),
                    venster,
                    id as HANDLE,
                    GetModuleHandleW(null()),
                    null_mut(),
                );
                if let Some(f) = self.lettertype {
                    SendMessageW(h, WM_SETFONT, f as usize, 1);
                }
                let nieuw_y = y + self.s(hoogte) + self.s(12);
                let mut hoogste = nieuw_y;
                match tab {
                    Some((nr, _)) => TABS.with(|t| {
                        let mut t = t.borrow_mut();
                        let tabs = t.iter_mut().find(|x| x.venster == venster).unwrap();
                        *tabs.y.last_mut().unwrap() = nieuw_y;
                        tabs.onderdelen[nr].push(h);
                        hoogste = *tabs.y.iter().max().unwrap();
                        if nr != 0 {
                            ShowWindow(h, 0);
                        }
                    }),
                    None => *self.volgende_y.last_mut().unwrap() = nieuw_y,
                }
                let breed = self.breedte.iter().find(|b| b.0 == venster).map_or(self.s(460), |b| b.1.max(self.s(460)));
                if IsZoomed(venster) == 0 {
                    SetWindowPos(venster, null_mut(), 0, 0, breed, hoogste + self.s(60), SWP_NOMOVE | SWP_NOZORDER);
                }
                let op_ander_tabblad = tab.map_or(false, |t| t.0 != 0);
                if focus && !op_ander_tabblad && !*self.heeft_focus.last().unwrap() {
                    *self.heeft_focus.last_mut().unwrap() = true;
                    SetFocus(h);
                }
                h
            }
        }

        pub(super) fn kleur(&mut self, achtergrond: bool, k: (u8, u8, u8)) {
            let Some(&venster) = self.vensters.last() else { return };
            KLEUREN.with(|lijst| {
                let mut lijst = lijst.borrow_mut();
                if !lijst.iter().any(|x| x.0 == venster) {
                    lijst.push((venster, None, None, null_mut()));
                }
                let e = lijst.iter_mut().find(|x| x.0 == venster).unwrap();
                if achtergrond {
                    e.1 = Some(rgb(k));
                    e.3 = unsafe { CreateSolidBrush(rgb(k)) };
                } else {
                    e.2 = Some(rgb(k));
                }
            });
            unsafe {
                RedrawWindow(venster, null(), null_mut(), 0x0001 | 0x0004 | 0x0080);
            }
        }

        /// Knoppen naast elkaar, op de plek waar het volgende onderdeel zou komen.
        pub(super) fn werkbalk(&mut self, namen: &[&str], nummers: &[usize]) {
            let Some(&venster) = self.vensters.last() else { return };
            let y = *self.volgende_y.last().unwrap();
            let breedte = self.s(130);
            unsafe {
                for (i, (naam, nummer)) in namen.iter().zip(nummers).enumerate() {
                    let h = CreateWindowExW(
                        0,
                        w("BUTTON").as_ptr(),
                        w(naam).as_ptr(),
                        WS_CHILD | WS_VISIBLE | WS_TABSTOP,
                        self.s(20) + i as i32 * (breedte + self.s(8)),
                        y,
                        breedte,
                        self.s(44),
                        venster,
                        (KNOP_ID + nummer) as HANDLE,
                        GetModuleHandleW(null()),
                        null_mut(),
                    );
                    if let Some(f) = self.lettertype {
                        SendMessageW(h, WM_SETFONT, f as usize, 1);
                    }
                    KNOP_HWNDS.with(|k| k.borrow_mut().push(h));
                    if i == 0 && !*self.heeft_focus.last().unwrap() {
                        *self.heeft_focus.last_mut().unwrap() = true;
                        SetFocus(h);
                    }
                }
                let nieuw_y = y + self.s(44) + self.s(12);
                *self.volgende_y.last_mut().unwrap() = nieuw_y;
                let breed = (self.s(40) + namen.len() as i32 * (breedte + self.s(8))).max(self.s(460));
                self.breedte.retain(|b| b.0 != venster);
                self.breedte.push((venster, breed));
                if IsZoomed(venster) == 0 {
                    SetWindowPos(venster, null_mut(), 0, 0, breed, nieuw_y + self.s(60), SWP_NOMOVE | SWP_NOZORDER);
                }
            }
        }

        pub(super) fn knop(&mut self, naam: &str, nummer: usize) {
            if self.vensters.is_empty() {
                return;
            }
            let h = self.plaats("BUTTON", naam, WS_TABSTOP, 0, 52, KNOP_ID + nummer, true);
            KNOP_HWNDS.with(|k| k.borrow_mut().push(h));
        }

        /// Maakt een tekstvak, label, selectievakje of keuzelijst. Geeft het Windows-onderdeel terug.
        pub(super) fn onderdeel(&mut self, soort: u8, naam: &str) -> usize {
            if self.vensters.is_empty() {
                return 0;
            }
            // Een tekstvak of lijst krijgt een label erboven: zo leest de schermlezer de naam voor.
            let label = |s: &mut Self| {
                s.plaats("STATIC", &format!("{}:", naam), 0, 0, 30, 0, false);
            };
            let h = match soort {
                0 => {
                    label(self);
                    self.plaats("EDIT", "", WS_TABSTOP | WS_BORDER | ES_AUTOHSCROLL, WS_EX_CLIENTEDGE, 38, 0, true)
                }
                1 => {
                    label(self);
                    self.plaats(
                        "EDIT",
                        "",
                        WS_TABSTOP | WS_BORDER | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_WANTRETURN,
                        WS_EX_CLIENTEDGE,
                        200,
                        0,
                        true,
                    )
                }
                2 => self.plaats("STATIC", naam, 0, 0, 34, 0, false),
                3 => self.plaats("BUTTON", naam, WS_TABSTOP | BS_AUTOCHECKBOX, 0, 36, 0, true),
                4 => {
                    label(self);
                    self.plaats("LISTBOX", "", WS_TABSTOP | WS_BORDER | WS_VSCROLL | LBS_NOTIFY, 0, 160, 0, true)
                }
                6 => {
                    label(self);
                    unsafe {
                        InitCommonControlsEx(&INITCOMMONCONTROLSEX { grootte: 8, soorten: 0x1 });
                    }
                    // Een tabel: LVS_REPORT | LVS_SINGLESEL | LVS_SHOWSELALWAYS, met rasterlijnen en hele rijen kiezen.
                    let h = self.plaats("SysListView32", naam, WS_TABSTOP | WS_BORDER | 0x0001 | 0x0004 | 0x0008, 0, 220, 0, true);
                    unsafe {
                        SendMessageW(h, 0x1036, 0, 0x20 | 0x01);
                    }
                    h
                }
                _ => {
                    label(self);
                    let h = self.plaats("COMBOBOX", "", WS_TABSTOP | WS_VSCROLL | CBS_DROPDOWNLIST, 0, 240, 0, true);
                    // De open lijst is 240 hoog, het dichte vak maar 40: schuif de volgende plek terug.
                    *self.volgende_y.last_mut().unwrap() -= self.s(200);
                    h
                }
            };
            h as usize
        }
    }

    fn tekst_van(h: HWND) -> String {
        unsafe {
            let n = GetWindowTextLengthW(h);
            let mut buf = vec![0u16; n as usize + 1];
            let m = GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32);
            String::from_utf16_lossy(&buf[..m.max(0) as usize])
        }
    }

    pub(super) fn lees(o: &Ond) -> W {
        let h = o.h as HWND;
        unsafe {
            match o.soort {
                3 => W::B(SendMessageW(h, BM_GETCHECK, 0, 0) == 1),
                4 | 5 => {
                    let (cur, len, get) = if o.soort == 4 { (LB_GETCURSEL, LB_GETTEXTLEN, LB_GETTEXT) } else { (CB_GETCURSEL, CB_GETLBTEXTLEN, CB_GETLBTEXT) };
                    let i = SendMessageW(h, cur, 0, 0);
                    if i < 0 {
                        return W::T(String::new());
                    }
                    let n = SendMessageW(h, len, i as usize, 0).max(0) as usize;
                    let mut buf = vec![0u16; n + 1];
                    SendMessageW(h, get, i as usize, buf.as_mut_ptr() as isize);
                    W::T(String::from_utf16_lossy(&buf[..n]))
                }
                _ => W::T(tekst_van(h).replace("\r\n", "\n")),
            }
        }
    }

    // ----------------------------------------------------------------- opnemen

    #[repr(C)]
    struct WAVEFORMATEX {
        soort: u16,
        kanalen: u16,
        per_seconde: u32,
        bytes_per_seconde: u32,
        blok: u16,
        bits: u16,
        extra: u16,
    }

    #[repr(C)]
    struct WAVEHDR {
        data: *mut u8,
        lengte: u32,
        opgenomen: u32,
        gebruiker: usize,
        vlaggen: u32,
        lussen: u32,
        volgende: *mut WAVEHDR,
        gereserveerd: usize,
    }

    #[link(name = "winmm")]
    extern "system" {
        fn waveInOpen(h: *mut HANDLE, apparaat: u32, fmt: *const WAVEFORMATEX, terug: usize, inst: usize, vlaggen: u32) -> u32;
        fn waveInPrepareHeader(h: HANDLE, hdr: *mut WAVEHDR, grootte: u32) -> u32;
        fn waveInUnprepareHeader(h: HANDLE, hdr: *mut WAVEHDR, grootte: u32) -> u32;
        fn waveInAddBuffer(h: HANDLE, hdr: *mut WAVEHDR, grootte: u32) -> u32;
        fn waveInStart(h: HANDLE) -> u32;
        fn waveInReset(h: HANDLE) -> u32;
        fn waveInClose(h: HANDLE) -> u32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(attr: *mut c_void, handmatig: i32, begin: i32, naam: *const u16) -> HANDLE;
        fn WaitForSingleObject(h: HANDLE, ms: u32) -> u32;
        fn CloseHandle(h: HANDLE) -> i32;
    }

    struct Opname {
        stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
        draad: std::thread::JoinHandle<Result<Vec<u8>, String>>,
    }

    static OPNAME: std::sync::Mutex<Option<Opname>> = std::sync::Mutex::new(None);

    pub(super) fn begin_opname() -> Result<(), String> {
        let mut o = OPNAME.lock().unwrap();
        if o.is_some() {
            return Err("Er loopt al een opname. Stop die eerst.".into());
        }
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (klaar_tx, klaar_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let stop2 = stop.clone();
        let draad = std::thread::spawn(move || -> Result<Vec<u8>, String> {
            unsafe {
                let fmt = WAVEFORMATEX { soort: 1, kanalen: 1, per_seconde: 44100, bytes_per_seconde: 88200, blok: 2, bits: 16, extra: 0 };
                let gebeurtenis = CreateEventW(null_mut(), 0, 0, null());
                let mut h: HANDLE = null_mut();
                // WAVE_MAPPER: de standaardmicrofoon van Windows. CALLBACK_EVENT.
                if waveInOpen(&mut h, u32::MAX, &fmt, gebeurtenis as usize, 0, 0x0005_0000) != 0 {
                    CloseHandle(gebeurtenis);
                    let _ = klaar_tx.send(Err("Ik vind geen microfoon, of Windows geeft geen toestemming. Kijk bij Instellingen, Privacy, Microfoon.".into()));
                    return Ok(Vec::new());
                }
                let grootte = std::mem::size_of::<WAVEHDR>() as u32;
                let mut buffers: Vec<Box<[u8]>> = (0..8).map(|_| vec![0u8; 8820].into_boxed_slice()).collect();
                let mut koppen: Vec<Box<WAVEHDR>> = buffers
                    .iter_mut()
                    .map(|b| {
                        Box::new(WAVEHDR { data: b.as_mut_ptr(), lengte: b.len() as u32, opgenomen: 0, gebruiker: 0, vlaggen: 0, lussen: 0, volgende: null_mut(), gereserveerd: 0 })
                    })
                    .collect();
                for k in koppen.iter_mut() {
                    waveInPrepareHeader(h, &mut **k, grootte);
                    waveInAddBuffer(h, &mut **k, grootte);
                }
                waveInStart(h);
                let _ = klaar_tx.send(Ok(()));
                let mut data = Vec::new();
                let mut oogst = |koppen: &mut Vec<Box<WAVEHDR>>, opnieuw: bool, data: &mut Vec<u8>| {
                    for k in koppen.iter_mut() {
                        if k.vlaggen & 1 != 0 {
                            data.extend_from_slice(std::slice::from_raw_parts(k.data, k.opgenomen as usize));
                            k.vlaggen &= !1;
                            if opnieuw {
                                waveInAddBuffer(h, &mut **k, grootte);
                            }
                        }
                    }
                };
                while !stop2.load(std::sync::atomic::Ordering::SeqCst) {
                    WaitForSingleObject(gebeurtenis, 50);
                    oogst(&mut koppen, true, &mut data);
                    if data.len() > 44100 * 2 * 60 * 120 {
                        break; // Hoogstens twee uur.
                    }
                }
                waveInReset(h);
                oogst(&mut koppen, false, &mut data);
                for k in koppen.iter_mut() {
                    waveInUnprepareHeader(h, &mut **k, grootte);
                }
                waveInClose(h);
                CloseHandle(gebeurtenis);
                drop(buffers);
                Ok(data)
            }
        });
        match klaar_rx.recv() {
            Ok(Ok(())) => {
                *o = Some(Opname { stop, draad });
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err("Het opnemen kon niet beginnen.".into()),
        }
    }

    pub(super) fn stop_opname() -> Result<Vec<u8>, String> {
        let Some(o) = OPNAME.lock().unwrap().take() else {
            return Err("Er loopt geen opname. Begin eerst met: Begin met opnemen.".into());
        };
        o.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        o.draad.join().map_err(|_| "Het opnemen ging mis.".to_string())?
    }

    // ----------------------------------------------------------------- afbeeldingen (WIC)

    #[repr(C)]
    struct BITMAPINFOHEADER {
        grootte: u32,
        breedte: i32,
        hoogte: i32,
        vlakken: u16,
        bits: u16,
        compressie: u32,
        beeld: u32,
        xp: i32,
        yp: i32,
        kleuren: u32,
        belangrijk: u32,
    }

    #[link(name = "gdi32")]
    extern "system" {
        fn CreateDIBSection(dc: HANDLE, info: *const BITMAPINFOHEADER, gebruik: u32, bits: *mut *mut c_void, sectie: HANDLE, plek: u32) -> HANDLE;
    }

    unsafe fn methode_van(obj: *mut c_void, nr: usize) -> usize {
        *(*(obj as *const *const usize)).add(nr)
    }

    unsafe fn los(obj: *mut c_void) {
        if !obj.is_null() {
            let f: unsafe extern "system" fn(*mut c_void) -> u32 = std::mem::transmute(methode_van(obj, 2));
            f(obj);
        }
    }

    /// Leest een afbeelding met WIC en maakt er een Windows-bitmap van, hoogstens `max` breed.
    unsafe fn laad_afbeelding(pad: &std::path::Path, max: u32) -> Option<(HANDLE, u32, u32)> {
        CoInitializeEx(null_mut(), 2);
        let clsid = guid(0xCACA_F262, 0x9370, 0x4615, [0xA1, 0x3B, 0x9F, 0x55, 0x39, 0xDA, 0x4C, 0x0A]);
        let iid = guid(0xEC5E_C8A9, 0xC395, 0x4314, [0x9C, 0x77, 0x54, 0xD7, 0xA9, 0x35, 0xFF, 0x70]);
        let mut fabriek: *mut c_void = null_mut();
        if CoCreateInstance(&clsid, null_mut(), 1, &iid, &mut fabriek) != 0 || fabriek.is_null() {
            return None;
        }
        type Lees = unsafe extern "system" fn(*mut c_void, *const u16, *const c_void, u32, u32, *mut *mut c_void) -> i32;
        type Vraag = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32;
        type Plaatje = unsafe extern "system" fn(*mut c_void, u32, *mut *mut c_void) -> i32;
        type Maat = unsafe extern "system" fn(*mut c_void, *mut u32, *mut u32) -> i32;
        type Schaal = unsafe extern "system" fn(*mut c_void, *mut c_void, u32, u32, u32) -> i32;
        type Zet = unsafe extern "system" fn(*mut c_void, *mut c_void, *const [u8; 16], u32, *mut c_void, f64, u32) -> i32;
        type Kopieer = unsafe extern "system" fn(*mut c_void, *const c_void, u32, u32, *mut u8) -> i32;
        let mut lezer: *mut c_void = null_mut();
        let lees: Lees = std::mem::transmute(methode_van(fabriek, 3));
        if lees(fabriek, w(&pad.to_string_lossy()).as_ptr(), null(), 0x8000_0000, 0, &mut lezer) != 0 {
            los(fabriek);
            return None;
        }
        let mut beeld: *mut c_void = null_mut();
        let plaatje: Plaatje = std::mem::transmute(methode_van(lezer, 13));
        plaatje(lezer, 0, &mut beeld);
        let (mut b, mut h) = (0u32, 0u32);
        let maat: Maat = std::mem::transmute(methode_van(beeld, 3));
        maat(beeld, &mut b, &mut h);
        let (nb, nh) = if b > max { (max, (h as u64 * max as u64 / b.max(1) as u64) as u32) } else { (b, h) };
        let mut schaler: *mut c_void = null_mut();
        let maak_schaler: Vraag = std::mem::transmute(methode_van(fabriek, 11));
        maak_schaler(fabriek, &mut schaler);
        let schaal: Schaal = std::mem::transmute(methode_van(schaler, 8));
        schaal(schaler, beeld, nb.max(1), nh.max(1), 3);
        let mut omzetter: *mut c_void = null_mut();
        let maak_omzetter: Vraag = std::mem::transmute(methode_van(fabriek, 10));
        maak_omzetter(fabriek, &mut omzetter);
        let bgra = guid(0x6FDD_C324, 0x4E03, 0x4BFE, [0xB1, 0x85, 0x3D, 0x77, 0x76, 0x8D, 0xC9, 0x0F]);
        let zet: Zet = std::mem::transmute(methode_van(omzetter, 8));
        let gelukt = zet(omzetter, schaler, &bgra, 0, null_mut(), 0.0, 0) == 0;
        let mut uit = None;
        if gelukt {
            let info = BITMAPINFOHEADER { grootte: 40, breedte: nb as i32, hoogte: -(nh as i32), vlakken: 1, bits: 32, compressie: 0, beeld: 0, xp: 0, yp: 0, kleuren: 0, belangrijk: 0 };
            let mut bits: *mut c_void = null_mut();
            let bmp = CreateDIBSection(null_mut(), &info, 0, &mut bits, null_mut(), 0);
            if !bmp.is_null() && !bits.is_null() {
                let kopieer: Kopieer = std::mem::transmute(methode_van(omzetter, 7));
                if kopieer(omzetter, null(), nb * 4, nb * nh * 4, bits as *mut u8) == 0 {
                    uit = Some((bmp, nb, nh));
                }
            }
        }
        for o in [omzetter, schaler, beeld, lezer, fabriek] {
            los(o);
        }
        uit
    }

    impl WinScherm {
        /// Een afbeelding in het laatste venster. De beschrijving is de naam die de schermlezer voorleest.
        pub(super) fn afbeelding(&mut self, pad: &std::path::Path, beschrijving: &str) -> bool {
            self.voorbereiden();
            let max = (400.0 * self.schaal) as u32;
            let Some((bmp, _, h)) = (unsafe { laad_afbeelding(pad, max) }) else { return false };
            let hoogte = (h as f32 / self.schaal.max(0.1)) as i32;
            // SS_BITMAP (0x0E) | SS_NOTIFY (0x100)
            let vak = self.plaats("STATIC", beschrijving, 0x0E | 0x100, 0, hoogte, 0, false);
            unsafe {
                SendMessageW(vak, 0x0172, 0, bmp as isize);
            }
            true
        }
    }

    #[repr(C)]
    struct LVCOLUMNW {
        mask: u32,
        fmt: i32,
        cx: i32,
        tekst: *const u16,
        max: i32,
        sub: i32,
        beeld: i32,
        volgorde: i32,
    }

    #[repr(C)]
    struct LVITEMW {
        mask: u32,
        item: i32,
        sub: i32,
        state: u32,
        state_mask: u32,
        tekst: *const u16,
        max: i32,
        beeld: i32,
        param: isize,
        inspring: i32,
        groep: i32,
        kolommen: u32,
        pu: *mut u32,
        pi: *mut i32,
        groep2: i32,
    }

    /// De gekozen rij van een tabel (of geen).
    pub(super) fn tabel_gekozen(h: usize) -> Option<usize> {
        let i = unsafe { SendMessageW(h as HWND, 0x100C, usize::MAX, 2) };
        (i >= 0).then_some(i as usize)
    }

    /// De hele tabel opnieuw tonen: kolommen, rijen, en de gekozen rij.
    pub(super) fn tabel_toon(h: usize, kolommen: &[String], rijen: &[Vec<String>], gekozen: Option<usize>) {
        let h = h as HWND;
        unsafe {
            SendMessageW(h, 0x1009, 0, 0);
            while SendMessageW(h, 0x101C, 0, 0) != 0 {}
            let breedte = if kolommen.is_empty() { 100 } else { (380 / kolommen.len() as i32).max(80) };
            for (k, naam) in kolommen.iter().enumerate() {
                let t = w(naam);
                let c = LVCOLUMNW { mask: 0x1 | 0x2 | 0x4 | 0x8, fmt: 0, cx: breedte, tekst: t.as_ptr(), max: 0, sub: k as i32, beeld: 0, volgorde: 0 };
                SendMessageW(h, 0x1061, k, &c as *const LVCOLUMNW as isize);
            }
            for (r, rij) in rijen.iter().enumerate() {
                for (k, cel) in rij.iter().enumerate() {
                    let t = w(cel);
                    let mut it = LVITEMW {
                        mask: 1,
                        item: r as i32,
                        sub: k as i32,
                        state: 0,
                        state_mask: 0,
                        tekst: t.as_ptr(),
                        max: 0,
                        beeld: 0,
                        param: 0,
                        inspring: 0,
                        groep: 0,
                        kolommen: 0,
                        pu: null_mut(),
                        pi: null_mut(),
                        groep2: 0,
                    };
                    if k == 0 {
                        SendMessageW(h, 0x104D, 0, &mut it as *mut LVITEMW as isize);
                    } else {
                        SendMessageW(h, 0x1074, r, &mut it as *mut LVITEMW as isize);
                    }
                }
            }
            // Elke kolom zo breed als zijn naam of inhoud (LVSCW_AUTOSIZE_USEHEADER).
            for k in 0..kolommen.len() {
                SendMessageW(h, 0x101E, k, -2);
            }
            if let Some(g) = gekozen {
                let mut it: LVITEMW = std::mem::zeroed();
                it.state = 0x1 | 0x2;
                it.state_mask = 0x1 | 0x2;
                SendMessageW(h, 0x102B, g, &mut it as *mut LVITEMW as isize);
                SendMessageW(h, 0x1013, g, 0);
            }
        }
    }

    pub(super) fn zet_tekst(h: usize, t: &str) {
        unsafe {
            SetWindowTextW(h as HWND, w(&t.replace("\r\n", "\n").replace('\n', "\r\n")).as_ptr());
        }
    }

    pub(super) fn voeg_keuze(h: usize, soort: u8, t: &str, eerste: bool) {
        let (voeg, kies) = if soort == 4 { (LB_ADDSTRING, LB_SETCURSEL) } else { (CB_ADDSTRING, CB_SETCURSEL) };
        unsafe {
            SendMessageW(h as HWND, voeg, 0, w(t).as_ptr() as isize);
            if eerste {
                SendMessageW(h as HWND, kies, 0, 0);
            }
        }
    }

    pub(super) fn vink(h: usize, aan: bool) {
        unsafe {
            SendMessageW(h as HWND, BM_SETCHECK, aan as usize, 0);
        }
    }

    pub(super) fn focus(h: usize) {
        unsafe {
            SetFocus(h as HWND);
        }
    }

    fn klasse() -> Vec<u16> {
        w("JaapsProgrammeertaalVenster")
    }

    fn druk(nummer: usize, venster: HWND) {
        let Some(mut staat) = STAAT.with(|s| s.borrow_mut().take()) else { return };
        let actie = staat.scherm.knoppen.get(nummer).and_then(|k| k.1);
        staat.scherm.win.klik_venster = Some(venster);
        match actie {
            None => melding(venster, "Deze knop doet nog niets.", "Melding"),
            Some(a) => {
                staat.stappen = 0;
                if let Some(f) = BIJ_KNOP.with(|b| b.get()) {
                    let uitkomst = if a == super::OPDRACHT_ACTIE { staat.opdracht_geven(f) } else { f(&mut staat, a) };
                    if let Err(fout) = uitkomst {
                        foutmelding(venster, &foutzin(&fout));
                    }
                }
            }
        }
        staat.scherm.win.klik_venster = None;
        STAAT.with(|s| *s.borrow_mut() = Some(staat));
    }

    unsafe extern "system" fn venster_proc(h: HWND, m: u32, wp: usize, lp: isize) -> isize {
        match m {
            WM_COMMAND => {
                let id = wp & 0xFFFF;
                let code = (wp >> 16) & 0xFFFF;
                if code == 0 && id >= KNOP_ID {
                    druk(id - KNOP_ID, h);
                    return 0;
                }
            }
            WM_SIZE => {
                let vak = TEKSTVAKKEN.with(|t| t.borrow().iter().find(|x| x.0 == h).map(|x| x.1)).unwrap_or(null_mut());
                if !vak.is_null() {
                    let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    GetClientRect(h, &mut r);
                    MoveWindow(vak, 0, 0, r.right, r.bottom, 1);
                }
            }
            0x004E => {
                // WM_NOTIFY: een ander tabblad gekozen (TCN_SELCHANGE).
                #[repr(C)]
                struct NMHDR {
                    van: HWND,
                    id: usize,
                    code: i32,
                }
                let n = &*(lp as *const NMHDR);
                if n.code == -551 {
                    let nr = SendMessageW(n.van, 0x130B, 0, 0).max(0) as usize;
                    toon_tab(n.van, nr);
                }
            }
            0x0014 => {
                // WM_ERASEBKGND: de achtergrondkleur.
                let kwast = KLEUREN.with(|k| k.borrow().iter().find(|x| x.0 == h).map(|x| x.3)).filter(|k| !k.is_null());
                if let Some(kwast) = kwast {
                    let mut rc = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                    GetClientRect(h, &mut rc);
                    FillRect(wp as HANDLE, &rc, kwast);
                    return 1;
                }
            }
            0x0138 | 0x0135 => {
                // WM_CTLCOLORSTATIC en WM_CTLCOLORBTN: kleur van labels, selectievakjes en knoppen.
                if let Some((_, bg, fg, kwast)) = KLEUREN.with(|k| k.borrow().iter().find(|x| x.0 == h).copied()) {
                    let dc = wp as HANDLE;
                    if let Some(f) = fg {
                        SetTextColor(dc, f);
                    }
                    if let Some(b) = bg {
                        SetBkColor(dc, b);
                        return kwast as isize;
                    }
                }
            }
            WM_CLOSE => {
                // Vraag bij afsluiten: alleen bij het laatste venster.
                let vraag = AFSLUITVRAAG.with(|v| v.borrow().clone());
                if let Some(v) = vraag {
                    if OPEN.with(|o| o.get()) <= 1 && MessageBoxW(h, w(&v).as_ptr(), w("Afsluiten").as_ptr(), 0x4 | 0x20) != 6 {
                        return 0;
                    }
                }
                DestroyWindow(h);
                return 0;
            }
            WM_DESTROY => {
                let over = OPEN.with(|o| {
                    o.set(o.get().saturating_sub(1));
                    o.get()
                });
                if over == 0 {
                    PostQuitMessage(0);
                }
                return 0;
            }
            _ => {}
        }
        DefWindowProcW(h, m, wp, lp)
    }

    fn uitvoervenster(regels: &[String]) {
        tekstvenster("Uitvoer van het programma", regels);
    }

    fn tekstvenster(titel: &str, regels: &[String]) -> HWND {
        unsafe {
            let h = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                klasse().as_ptr(),
                w(titel).as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                700,
                500,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null()),
                null_mut(),
            );
            OPEN.with(|o| o.set(o.get() + 1));
            let tekst = regels.join("\r\n");
            let vak = CreateWindowExW(
                WS_EX_CLIENTEDGE,
                w("EDIT").as_ptr(),
                w(&tekst).as_ptr(),
                WS_CHILD | WS_VISIBLE | WS_TABSTOP | WS_VSCROLL | ES_MULTILINE | ES_AUTOVSCROLL | ES_READONLY,
                0,
                0,
                100,
                100,
                h,
                null_mut(),
                GetModuleHandleW(null()),
                null_mut(),
            );
            TEKSTVAKKEN.with(|t| t.borrow_mut().push((h, vak)));
            let dc = GetDC(null_mut());
            let dpi = GetDeviceCaps(dc, LOGPIXELSY);
            ReleaseDC(null_mut(), dc);
            let f = CreateFontW(-(22 * dpi / 96), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 5, 0, w("Segoe UI").as_ptr());
            SendMessageW(vak, WM_SETFONT, f as usize, 1);
            let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
            GetClientRect(h, &mut r);
            MoveWindow(vak, 0, 0, r.right, r.bottom, 1);
            SetForegroundWindow(h);
            SetFocus(vak);
            SendMessageW(vak, EM_SETSEL, 0, 0);
            h
        }
    }

    pub(super) fn start(namen: &'static [&'static str], acties: &'static [&'static str], hoofd: Hoofd, bij_knop: BijKnop) {
        unsafe {
            SetProcessDPIAware();
            let naam = klasse();
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(venster_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(null()),
                hIcon: null_mut(),
                hCursor: LoadCursorW(null_mut(), 32512 as *const u16),
                hbrBackground: 16 as HANDLE, // kleur van knoppen + 1
                lpszMenuName: null(),
                lpszClassName: naam.as_ptr(),
                hIconSm: null_mut(),
            };
            RegisterClassExW(&wc);
        }
        BIJ_KNOP.with(|b| b.set(Some(bij_knop)));
        let mut staat = Staat {
            lokaal: Vec::new(),
            waarden: vec![None; namen.len()],
            namen,
            acties,
            stappen: 0,
            scherm: Scherm { tekststand: false, knoppen: Vec::new(), onderdelen: Vec::new(), tabbladen: Vec::new(), uitvoer: Vec::new(), win: WinScherm::default() },
        };
        if let Err(f) = hoofd(&mut staat) {
            foutmelding(staat.scherm.win.actief(), &foutzin(&f));
            return;
        }
        if staat.scherm.win.vensters.is_empty() {
            let mut regels = std::mem::take(&mut staat.scherm.uitvoer);
            if regels.is_empty() {
                regels.push("Het programma is klaar. Er was niets om te tonen.".to_string());
            }
            regels.push(String::new());
            regels.push("Druk op Escape om te sluiten.".to_string());
            uitvoervenster(&regels);
        } else if let Some(&laatste) = staat.scherm.win.vensters.last() {
            unsafe {
                SetForegroundWindow(laatste);
            }
            // Tekst die getoond werd voordat er een venster was, gaat niet verloren.
            let eerder = std::mem::take(&mut staat.scherm.uitvoer);
            if !eerder.is_empty() {
                melding(laatste, &eerder.join("\n"), "Melding");
            }
        }
        STAAT.with(|s| *s.borrow_mut() = Some(staat));

        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                let wortel = GetAncestor(msg.hwnd, GA_ROOT);
                if msg.message == WM_KEYDOWN && msg.wParam == VK_ESCAPE && !wortel.is_null() {
                    PostMessageW(wortel, WM_CLOSE, 0, 0);
                    continue;
                }
                if (msg.message == WM_KEYDOWN || msg.message == 0x0104) && !SNELTOETSEN.with(|s| s.borrow().is_empty()) {
                    let ctrl = GetKeyState(0x11) < 0;
                    let alt = GetKeyState(0x12) < 0;
                    let shift = GetKeyState(0x10) < 0;
                    let knop = SNELTOETSEN.with(|s| {
                        s.borrow().iter().find(|t| t.0 as usize == msg.wParam && t.1 == ctrl && t.2 == alt && t.3 == shift).map(|t| (t.4, t.5))
                    });
                    if let Some((doel, menu)) = knop {
                        if menu == 0 {
                            SendMessageW(doel, BM_CLICK, 0, 0);
                        } else {
                            PostMessageW(doel, WM_COMMAND, menu, 0);
                        }
                        continue;
                    }
                }
                if msg.message == WM_KEYDOWN && msg.wParam == 0x70 && GetKeyState(0x10) < 0 {
                    // Shift+F1: de privacyverklaring.
                    let tekst = privacy_tekst();
                    if tekst.is_empty() {
                        melding(wortel, "Dit programma heeft geen privacyverklaring.", "Privacyverklaring");
                    } else {
                        let mut regels: Vec<String> = tekst.lines().map(|l| l.to_string()).collect();
                        regels.push(String::new());
                        regels.push("Druk op Escape om de privacyverklaring te sluiten.".to_string());
                        tekstvenster("Privacyverklaring", &regels);
                    }
                    continue;
                }
                if msg.message == WM_KEYDOWN && msg.wParam == 0x70 {
                    // F1: de handleiding van dit programma.
                    let oud = HANDLEIDING_VENSTER.with(|v| v.get());
                    if !oud.is_null() && IsWindow(oud) != 0 {
                        SetForegroundWindow(oud);
                        continue;
                    }
                    let tekst = handleiding_tekst();
                    if tekst.is_empty() {
                        melding(wortel, "Dit programma heeft geen handleiding.", "Handleiding");
                    } else {
                        let mut regels: Vec<String> = tekst.lines().map(|l| l.to_string()).collect();
                        regels.push(String::new());
                        regels.push("Druk op Escape om de handleiding te sluiten.".to_string());
                        let h = tekstvenster("Handleiding", &regels);
                        HANDLEIDING_VENSTER.with(|v| v.set(h));
                    }
                    continue;
                }
                if msg.message == WM_KEYDOWN && msg.wParam == 0x09 && GetKeyState(0x11) < 0 && !wortel.is_null() {
                    if wissel_tab(wortel, GetKeyState(0x10) < 0) {
                        continue;
                    }
                }
                if msg.message == WM_KEYDOWN && msg.wParam == VK_RETURN {
                    let focus = GetFocus();
                    if KNOP_HWNDS.with(|k| k.borrow().contains(&focus)) {
                        SendMessageW(focus, BM_CLICK, 0, 0);
                        continue;
                    }
                }
                if !wortel.is_null() && IsDialogMessageW(wortel, &mut msg) != 0 {
                    continue;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}
// ===================== Einde van de looptijd ==========================
