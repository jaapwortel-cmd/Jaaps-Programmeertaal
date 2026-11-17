//! Tests voor: en/of, menu's, voorlezen, volume, en bestanden kopiëren, verplaatsen en weggooien.

use motor::{lees, pas_toe};
use std::process::Command;

fn voer_uit(bron: &str, map: &std::path::Path, extra: &[&str]) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:#?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = map.join("werk");
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let uit = Command::new(&exe).arg("--tekst").args(extra).env("JAAPTAAL_MAP", map).output().unwrap();
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

fn map(naam: &str) -> std::path::PathBuf {
    let m = std::env::temp_dir().join(format!("jaaptaal-vijf-{}-{}", naam, std::process::id()));
    std::fs::create_dir_all(&m).unwrap();
    m
}

#[test]
fn en_en_of() {
    let m = map("enof");
    let bron = "Onthoud a is 5\nOnthoud naam is \"Jaap\"\nAls a is groter dan 1 en naam is \"Jaap\"\nToon 1\nEinde als\nAls a is 9 of naam is \"Jaap\"\nToon 2\nEinde als\nAls a is 9 of a is 5 en naam is \"Piet\"\nToon 3\nEinde als\nAls a is groter dan of gelijk aan 5 en a is kleiner dan 6\nToon 4\nEinde als\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron, &m, &[]), "1\n2\n4\n");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn menu_met_sneltoets() {
    let m = map("menu");
    let bron = "Maak een venster met als titel A\nMaak een menu met als naam Bestand\nMaak een menuregel met als naam Opslaan\nGeef de menuregel Opslaan de sneltoets Ctrl+S\nAls ik de menuregel Opslaan kies\nToon de tekst opgeslagen\nEinde van de knop\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit(bron, &m, &["--druk", "Opslaan"]);
    assert!(uit.contains("[menu] Bestand\n[menuregel] Opslaan\n[sneltoets] Opslaan: Ctrl+S\n"), "{}", uit);
    assert!(uit.ends_with("opgeslagen\n"), "{}", uit);
    let zonder = "Maak een venster met als titel A\nMaak een menuregel met als naam Opslaan\n";
    let u = lees(zonder);
    assert!(u.meldingen[0].tekst.contains("nog geen menu"));
    assert!(lees(&pas_toe(zonder, &u.meldingen[0].voorstel.as_ref().unwrap().wijziging)).is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn voorlezen_en_volume() {
    let m = map("stem");
    let bron = "Zet het volume op 40\nLees voor \"Goedemorgen\"\nLees de tekst Hallo daar voor\nStop met voorlezen\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron, &m, &[]), "[volume] 40\n[voorlezen] Goedemorgen\n[voorlezen] Hallo daar\n[stop voorlezen]\n");
    assert!(!lees("Zet het volume op 150\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn kopieren_verplaatsen_weggooien() {
    let m = map("bestand");
    std::fs::write(m.join("brief.txt"), "hallo").unwrap();
    let bron = "Kopieer het bestand brief.txt naar Reserve/brief.txt\nVerplaats het bestand Reserve/brief.txt naar Oud/brief.txt\nKopieer de map Oud naar Kopie\nGooi de map Oud weg\nToon de inhoud van het bestand Kopie/brief.txt\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron, &m, &[]), "hallo\n");
    assert!(m.join("brief.txt").exists() && !m.join("Reserve/brief.txt").exists() && !m.join("Oud").exists());
    let uit = voer_uit("Gooi het bestand bestaatniet.txt weg\n", &m, &[]);
    assert!(uit.contains("regel 1") && uit.contains("bestaat niet"), "{}", uit);
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn tabbladen() {
    let m = map("tab");
    let bron = "Maak een venster met als titel Instellingen\nMaak een tabblad met als naam Algemeen\nMaak een tekstvak met als naam Naam\nMaak een knop met als naam Verder\nMaak een tabblad met als naam Geluid\nMaak een selectievakje met als naam Geluid aan\nAls ik op de knop Verder druk\nGa naar het tabblad Geluid\nEinde van de knop\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit(bron, &m, &["--druk", "Verder"]);
    assert!(uit.contains("[tabblad] Algemeen\n[tekstvak] Naam\n[knop] Verder\n[tabblad] Geluid\n"), "{}", uit);
    assert!(uit.ends_with("[ga naar tabblad] Geluid\n"), "{}", uit);
    let b = motor::scherm::beschrijf_scherm(&lees(bron).programma);
    assert!(b.contains("Er staan 2 tabbladen: Algemeen, Geluid."), "{}", b);
    assert!(b.contains("Op het tabblad Geluid staat, van boven naar beneden:\n  1. het selectievakje Geluid aan"), "{}", b);
    let fout = "Maak een venster met als titel A\nMaak een tabblad met als naam Algemeen\nGa naar het tabblad Algemen\n";
    let u = lees(fout);
    assert!(lees(&pas_toe(fout, &u.meldingen[0].voorstel.as_ref().expect("voorstel").wijziging)).is_goed());
    assert!(lees("Maak een tabblad met als naam A\n").meldingen[0].tekst.contains("geen venster"));
    let _ = std::fs::remove_dir_all(&m);
}
