//! Tests voor: Word en PDF, internet en updates, tabellen, opnemen en afbeeldingen.

use motor::{lees, pas_toe};
use std::process::Command;

fn map(naam: &str) -> std::path::PathBuf {
    let m = std::env::temp_dir().join(format!("jaaptaal-groot-{}-{}", naam, std::process::id()));
    std::fs::create_dir_all(&m).unwrap();
    m
}

fn voer_uit(bron: &str, m: &std::path::Path, extra: &[&str]) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|x| panic!("Meldingen: {:#?}", x));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = m.join("werk");
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let uit = Command::new(&exe).arg("--tekst").args(extra).env("JAAPTAAL_MAP", m).output().unwrap();
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

#[test]
fn word_en_pdf() {
    let m = map("doc");
    let bron = "Schrijf \"Beste Piet, dank voor het café\" in het Word-bestand brief.docx\nSchrijf \"Factuur (één)\" in het PDF-bestand factuur.pdf\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    voer_uit(bron, &m, &[]);
    let docx = std::fs::read(m.join("brief.docx")).unwrap();
    assert!(docx.starts_with(b"PK\x03\x04"));
    let tekst = String::from_utf8_lossy(&docx);
    assert!(tekst.contains("word/document.xml") && tekst.contains("Beste Piet, dank voor het café"));
    let pdf = std::fs::read(m.join("factuur.pdf")).unwrap();
    assert!(pdf.starts_with(b"%PDF-1.4") && pdf.ends_with(b"%%EOF\n"));
    assert!(String::from_utf8_lossy(&pdf).contains("(Factuur \\(\\351\\351n\\)) Tj"));
    assert!(!lees("Schrijf \"x\" in het Word-bestand brief.txt\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn internetzinnen() {
    let bron = "Toon de tekst van de webpagina https://example.com\nOnthoud html is de inhoud van de webpagina https://example.com\nVerstuur \"hallo\" naar het adres https://example.com/post\nDownload het bestand van het adres https://example.com naar voorbeeld.html\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert!(!lees("Verstuur \"x\" naar het adres example.com\n").is_goed());
    let zonder = lees("Zoek naar updates\n");
    assert!(zonder.meldingen[0].tekst.contains("De versie is"));
    let met = "Maak een installatiebestand met als naam A\nDe versie is 1.0\nDe updates staan op het adres https://www.dropbox.com/s/x/versie.txt?dl=0\nZoek naar updates\n";
    assert!(lees(met).is_goed(), "{:#?}", lees(met).meldingen);
    let code = motor::rust_code(met).unwrap();
    assert!(code.contains("const VERSIE: &str = \"1.0\";") && code.contains("const UPDATE_ADRES: &str = \"https://www.dropbox.com/s/x/versie.txt?dl=0\";"));
}

#[cfg(windows)]
#[test]
fn internet_echt_ophalen() {
    let m = map("net");
    let uit = voer_uit("Toon de tekst van de webpagina https://example.com\n", &m, &[]);
    assert!(uit.contains("Example Domain"), "{}", uit);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn tabellen() {
    let m = map("tabel");
    let bron = "Maak een venster met als titel U\nMaak een tabel met als naam Uitgaven\nVoeg de kolom Wat toe aan de tabel Uitgaven\nVoeg de kolom Bedrag toe aan de tabel Uitgaven\nVoeg de rij \"Brood\" en 2,50 toe aan de tabel Uitgaven\nVoeg de rij \"Melk\" en 1,10 toe aan de tabel Uitgaven\nOnthoud totaal is 0\nHerhaal voor elk nummer van 1 tot het aantal rijen in de tabel Uitgaven\nOnthoud rij is rij nummer nummer van de tabel Uitgaven\nTel nummer 2 uit de lijst rij op bij totaal\nEinde herhaal\nToon totaal\nSorteer de tabel Uitgaven op de kolom Bedrag\nToon de gekozen rij van de tabel Uitgaven\nSla de tabel Uitgaven op in het bestand u.csv\nHaal de gekozen rij weg uit de tabel Uitgaven\nToon het aantal rijen in de tabel Uitgaven\nLees de tabel Uitgaven uit het bestand u.csv\nToon het aantal rijen in de tabel Uitgaven\nMaak de tabel Uitgaven leeg\nToon het aantal rijen in de tabel Uitgaven\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit(bron, &m, &[]);
    assert!(uit.contains("\n3,6\n"), "{}", uit);
    assert!(uit.contains("\nMelk, 1,1\n"), "{}", uit);
    assert!(uit.contains("\n1\n") && uit.contains("\n2\n") && uit.ends_with("\n0\n"), "{}", uit);
    let csv = std::fs::read_to_string(m.join("u.csv")).unwrap();
    assert!(csv.contains("Wat;Bedrag\r\nMelk;\"1,1\"\r\nBrood;\"2,5\"\r\n"), "{:?}", csv);
    let fout = voer_uit("Maak een venster met als titel U\nMaak een tabel met als naam T\nVoeg de kolom A toe aan de tabel T\nVoeg de rij 1 en 2 toe aan de tabel T\n", &m, &[]);
    assert!(fout.contains("regel 4") && fout.contains("1 kolommen"), "{}", fout);
    let typ = "Maak een venster met als titel U\nMaak een tabel met als naam Uitgaven\nSorteer de tabel Uitgavn op de kolom A\n";
    let u = lees(typ);
    assert!(lees(&pas_toe(typ, &u.meldingen[0].voorstel.as_ref().expect("voorstel").wijziging)).is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn opnemen_en_afbeeldingen() {
    let m = map("beeld");
    std::fs::write(m.join("hond.png"), [0x89, b'P', b'N', b'G']).unwrap();
    let bron = "Maak een venster met als titel F\nMaak een afbeelding van het bestand hond.png met de beschrijving Een rode hond\nBegin met opnemen\nStop met opnemen en sla het op in het bestand opname.wav\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit(bron, &m, &[]);
    assert!(uit.contains("[afbeelding] hond.png: Een rode hond\n[opname begint]\n[opname gestopt] opname.wav\n"), "{}", uit);
    let wav = std::fs::read(m.join("opname.wav")).unwrap();
    assert!(wav.starts_with(b"RIFF") && &wav[8..16] == b"WAVEfmt " && wav.len() == 44);
    assert!(lees("Maak een venster met als titel F\nMaak een afbeelding van het bestand hond.png\n").meldingen[0].tekst.contains("beschrijving"));
    assert!(!lees("Stop met opnemen en sla het op in het bestand opname.mp3\n").is_goed());
    let weg = voer_uit("Maak een venster met als titel F\nMaak een afbeelding van het bestand weg.png met de beschrijving x\n", &m, &[]);
    assert!(weg.contains("regel 2") && weg.contains("niet vinden"), "{}", weg);
    let _ = std::fs::remove_dir_all(&m);
}
