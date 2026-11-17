//! Getallen lezen (ook in woorden, zoals "tien" of "eenentwintig") en netjes schrijven.

const EENHEDEN: &[(&str, u64)] = &[
    ("nul", 0),
    ("een", 1),
    ("één", 1),
    ("eén", 1),
    ("twee", 2),
    ("drie", 3),
    ("vier", 4),
    ("vijf", 5),
    ("zes", 6),
    ("zeven", 7),
    ("acht", 8),
    ("negen", 9),
    ("tien", 10),
    ("elf", 11),
    ("twaalf", 12),
    ("dertien", 13),
    ("veertien", 14),
    ("vijftien", 15),
    ("zestien", 16),
    ("zeventien", 17),
    ("achttien", 18),
    ("negentien", 19),
];

const TIENTALLEN: &[(&str, u64)] = &[
    ("twintig", 20),
    ("dertig", 30),
    ("veertig", 40),
    ("vijftig", 50),
    ("zestig", 60),
    ("zeventig", 70),
    ("tachtig", 80),
    ("negentig", 90),
];

fn onder_honderd(w: &str) -> Option<u64> {
    if w.is_empty() {
        return None;
    }
    if let Some((_, n)) = EENHEDEN.iter().find(|(s, _)| *s == w) {
        return Some(*n);
    }
    if let Some((_, n)) = TIENTALLEN.iter().find(|(s, _)| *s == w) {
        return Some(*n);
    }
    // eenentwintig, tweeëntwintig, drieëndertig
    for (tiental, t) in TIENTALLEN {
        if let Some(voor) = w.strip_suffix(tiental) {
            let voor = voor
                .strip_suffix("ën")
                .or_else(|| voor.strip_suffix("en"))?;
            let e = EENHEDEN.iter().find(|(s, n)| *s == voor && *n >= 1 && *n <= 9)?;
            return Some(t + e.1);
        }
    }
    None
}

fn onder_duizend(w: &str) -> Option<u64> {
    if let Some(pos) = w.find("honderd") {
        let voor = &w[..pos];
        let na = &w[pos + "honderd".len()..];
        let honderdtallen = if voor.is_empty() { 1 } else { onder_honderd(voor).filter(|n| *n >= 1 && *n <= 9)? };
        let rest = if na.is_empty() { 0 } else { onder_honderd(na.strip_prefix("en").unwrap_or(na))? };
        return Some(honderdtallen * 100 + rest);
    }
    onder_honderd(w)
}

/// Leest een getal dat in woorden is geschreven, bijvoorbeeld "tien" of "tweehonderdvijf".
pub fn uit_woord(woord: &str) -> Option<f64> {
    let w = woord.to_lowercase();
    if let Some(pos) = w.find("duizend") {
        let voor = &w[..pos];
        let na = &w[pos + "duizend".len()..];
        let duizendtallen = if voor.is_empty() { 1 } else { onder_duizend(voor)? };
        let rest = if na.is_empty() { 0 } else { onder_duizend(na.strip_prefix("en").unwrap_or(na))? };
        return Some((duizendtallen * 1000 + rest) as f64);
    }
    if w == "miljoen" {
        return Some(1_000_000.0);
    }
    onder_duizend(&w).map(|n| n as f64)
}

/// Leest een getal in cijfers: 42, -3, 3,5 of 3.5.
pub fn uit_cijfers(woord: &str) -> Option<f64> {
    let w = woord.replace('−', "-");
    let mut tekens = w.chars();
    let eerste = tekens.next()?;
    if !(eerste.is_ascii_digit() || eerste == '-') {
        return None;
    }
    if !w.chars().skip(1).all(|c| c.is_ascii_digit() || c == ',' || c == '.') {
        return None;
    }
    if w.matches([',', '.']).count() > 1 {
        return None;
    }
    w.replace(',', ".").parse::<f64>().ok().filter(|g| g.is_finite())
}

/// Leest een getal in cijfers of in woorden.
pub fn lees(woord: &str) -> Option<f64> {
    uit_cijfers(woord).or_else(|| uit_woord(woord))
}

/// Schrijft een getal zoals een Nederlander het leest: 3,5 en 42 (zonder ,0).
pub fn schrijf(g: f64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn woorden() {
        assert_eq!(lees("tien"), Some(10.0));
        assert_eq!(lees("Eenentwintig"), Some(21.0));
        assert_eq!(lees("tweeëntwintig"), Some(22.0));
        assert_eq!(lees("honderd"), Some(100.0));
        assert_eq!(lees("tweehonderdvijf"), Some(205.0));
        assert_eq!(lees("duizend"), Some(1000.0));
        assert_eq!(lees("fiets"), None);
    }

    #[test]
    fn cijfers() {
        assert_eq!(lees("42"), Some(42.0));
        assert_eq!(lees("3,5"), Some(3.5));
        assert_eq!(lees("-3"), Some(-3.0));
        assert_eq!(lees("1.000.000"), None);
        assert_eq!(lees("x"), None);
    }

    #[test]
    fn schrijven() {
        assert_eq!(schrijf(42.0), "42");
        assert_eq!(schrijf(3.5), "3,5");
        assert_eq!(schrijf(1.0 / 3.0), "0,3333333333");
        assert_eq!(schrijf(-2.0), "-2");
    }
}
