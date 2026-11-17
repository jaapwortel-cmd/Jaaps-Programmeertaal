//! Tekstbestanden (TXT) lezen en veilig opslaan.

use std::fs;
use std::io;
use std::path::Path;

/// Windows-1252: de oude standaard van Kladblok. Alleen de tekens 0x80-0x9F wijken af van Latin-1.
const CP1252: [char; 32] = [
    '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8D}', 'Ž', '\u{8F}', '\u{90}', '‘', '’',
    '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9D}', 'ž', 'Ÿ',
];

/// Leest een tekstbestand. Werkt met UTF-8 (met of zonder BOM) en met oude Kladblok-bestanden.
pub fn lees_txt(pad: &Path) -> io::Result<String> {
    let bytes = fs::read(pad)?;
    Ok(tekst_uit_bytes(&bytes))
}

pub fn tekst_uit_bytes(bytes: &[u8]) -> String {
    if bytes.starts_with(&[0xFF, 0xFE]) {
        let woorden: Vec<u16> = bytes[2..].chunks(2).map(|c| u16::from_le_bytes([c[0], *c.get(1).unwrap_or(&0)])).collect();
        return String::from_utf16_lossy(&woorden);
    }
    let zonder_bom = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(zonder_bom) {
        Ok(s) => s.to_string(),
        Err(_) => zonder_bom
            .iter()
            .map(|&b| if (0x80..0xA0).contains(&b) { CP1252[(b - 0x80) as usize] } else { b as char })
            .collect(),
    }
}

/// Slaat tekst op als UTF-8 met Windows-regeleinden.
/// Eerst naar een tijdelijk bestand, daarna pas het echte bestand vervangen.
/// Zo raak je nooit je programma kwijt als er halverwege iets misgaat.
pub fn schrijf_txt(pad: &Path, tekst: &str) -> io::Result<()> {
    let genormaliseerd = tekst.replace("\r\n", "\n").replace('\n', "\r\n");
    let mut inhoud = vec![0xEF, 0xBB, 0xBF];
    inhoud.extend_from_slice(genormaliseerd.as_bytes());
    let mut tijdelijk = pad.as_os_str().to_owned();
    tijdelijk.push(".nieuw");
    let tijdelijk = std::path::PathBuf::from(tijdelijk);
    fs::write(&tijdelijk, &inhoud)?;
    fs::rename(&tijdelijk, pad)
}
