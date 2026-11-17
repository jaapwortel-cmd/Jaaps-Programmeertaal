//! Tests voor bestanden, mappen, het logboek, tijd, datum en willekeurige getallen.

use motor::lees;
use std::process::Command;

fn voer_uit_in(bron: &str, map: &std::path::Path) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:#?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = map.join("werk");
    let exe = werk.join(if cfg!(windows) { "proef.exe" } else { "proef" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let uit = Command::new(&exe).arg("--tekst").env("JAAPTAAL_MAP", map).output().unwrap();
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

fn tijdelijke_map(naam: &str) -> std::path::PathBuf {
    let m = std::env::temp_dir().join(format!("jaaptaal-{}-{}", naam, std::process::id()));
    let _ = std::fs::remove_dir_all(&m);
    std::fs::create_dir_all(&m).unwrap();
    m
}

#[test]
fn bestanden_schrijven_lezen_en_mappen() {
    let map = tijdelijke_map("bestand");
    let bron = "Maak een map met als naam Notities\nSchrijf \"Eerste regel\" in het bestand Notities/a.txt\nVoeg \"Tweede regel\" toe aan het bestand Notities/a.txt\nToon de inhoud van het bestand Notities/a.txt\nOnthoud r is de regels van het bestand Notities/a.txt\nToon het aantal in de lijst r\nAls het bestand Notities/a.txt bestaat\nToon de tekst ja\nEinde als\nAls het bestand b.txt bestaat niet\nToon de tekst nee\nEinde als\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit_in(bron, &map);
    assert_eq!(uit, "Eerste regel\nTweede regel\n2\nja\nnee\n");
    let bytes = std::fs::read(map.join("Notities").join("a.txt")).unwrap();
    assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
    assert!(String::from_utf8_lossy(&bytes).contains("Eerste regel\r\nTweede regel\r\n"));
    let _ = std::fs::remove_dir_all(&map);
}

#[test]
fn logboek_ook_bij_fouten() {
    let map = tijdelijke_map("logboek");
    let bron = "Schrijf in het logboek \"Gestart\"\nOnthoud x is 0\nToon 1 gedeeld door x\n";
    let uit = voer_uit_in(bron, &map);
    assert!(uit.contains("regel 3"), "{}", uit);
    let log = std::fs::read_to_string(map.join(if cfg!(windows) { "proef logboek.txt" } else { "proef logboek.txt" })).unwrap();
    assert!(log.contains("  Gestart\r\n"), "{}", log);
    assert!(log.contains("FOUT: Er ging iets mis op regel 3"), "{}", log);
    let _ = std::fs::remove_dir_all(&map);
}

#[test]
fn bestand_dat_niet_bestaat_geeft_fout() {
    let map = tijdelijke_map("geenbestand");
    let uit = voer_uit_in("Toon de inhoud van het bestand bestaatniet.txt\n", &map);
    assert!(uit.contains("regel 1") && uit.contains("niet lezen"), "{}", uit);
    let _ = std::fs::remove_dir_all(&map);
}

#[test]
fn dobbelsteen_tijd_en_datum() {
    let map = tijdelijke_map("toeval");
    let bron = "Maak een lijst met als naam worpen\nHerhaal voor elke beurt van 1 tot 600\nVoeg een willekeurig getal van 1 tot 6 toe aan de lijst worpen\nEinde herhaal\nHerhaal voor elk getal van 1 tot 6\nAls de lijst worpen bevat getal\nToon getal\nEinde als\nEinde herhaal\nAls de lijst worpen bevat 7\nToon de tekst fout\nEinde als\nToon de datum\nToon de tijd\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit_in(bron, &map);
    let regels: Vec<&str> = uit.lines().collect();
    assert_eq!(&regels[..6], &["1", "2", "3", "4", "5", "6"], "{}", uit);
    assert!(!uit.contains("fout"));
    assert_eq!(regels[6].len(), 10, "datum dd-mm-jjjj: {}", regels[6]);
    assert_eq!(regels[7].len(), 5, "tijd uu:mm: {}", regels[7]);
    let _ = std::fs::remove_dir_all(&map);
}

#[test]
fn willekeurig_verkeerd_om() {
    let map = tijdelijke_map("verkeerd");
    let uit = voer_uit_in("Toon een willekeurig getal van 6 tot 1\n", &map);
    assert!(uit.contains("kleinste"), "{}", uit);
    let _ = std::fs::remove_dir_all(&map);
}
