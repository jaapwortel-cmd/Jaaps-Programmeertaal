//! Proeven voor de zinnen die de assistent nodig had: pauzeren, kopiëren en plakken, afdrukken,
//! zoeken in een tekstvak, en onderdelen groter of kleiner maken.

use std::process::Command;

const BRON: &str = "Maak een venster met als titel Proef
Maak een groot tekstvak met als naam Verhaal
Maak een tekstvak met als naam Zoekwoord
Maak een knop met als naam Opslaan
Maak de knop Opslaan groter
Geef het tekstvak Zoekwoord kleine letters
Zet \"Er was eens een kat. De kat sliep.\" in het tekstvak Verhaal
Zet \"kat\" in het tekstvak Zoekwoord
Zoek de tekst in het tekstvak Zoekwoord in het tekstvak Verhaal
Zoek de tekst in het tekstvak Zoekwoord in het tekstvak Verhaal
Zoek \"hond\" in het tekstvak Verhaal
Kopieer de tekst in het tekstvak Verhaal
Zet de tekst van het klembord in het tekstvak Zoekwoord
Druk de tekst in het tekstvak Verhaal af
Pauzeer het geluid
Ga verder met het geluid
";

#[test]
fn nieuwe_zinnen_lezen_en_werken() {
    let u = motor::lees(BRON);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    let code = motor::rust_code(BRON).unwrap();
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let map = std::env::temp_dir().join(format!("jaaptaal-nieuw-{}", std::process::id()));
    let werk = map.join("werk");
    std::fs::create_dir_all(&werk).unwrap();
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let uit = Command::new(&exe).arg("--tekst").env("JAAPTAAL_MAP", &map).output().unwrap();
    let t = String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n");
    for verwacht in [
        "[groter] Opslaan",
        "[kleiner] Zoekwoord",
        "[gevonden] kat op regel 1",
        "[niet gevonden] hond",
        "[klembord] Er was eens een kat. De kat sliep.",
        "[tekstvak Zoekwoord] Er was eens een kat.",
        "[afdrukken] Er was eens een kat.",
        "[geluid gepauzeerd]",
        "[geluid speelt verder]",
    ] {
        assert!(t.contains(verwacht), "{} ontbreekt in:\n{}", verwacht, t);
    }
    let _ = std::fs::remove_dir_all(&map);
}

#[test]
fn verkeerde_namen_geven_een_vraag() {
    let u = motor::lees("Maak een venster met als titel X\nMaak de knop Bestaatniet groter\n");
    assert!(!u.is_goed());
    let u = motor::lees("Maak een venster met als titel X\nZoek \"a\" in het tekstvak Nergens\n");
    assert!(!u.is_goed());
}
