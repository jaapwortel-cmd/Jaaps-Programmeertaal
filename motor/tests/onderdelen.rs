//! Tests voor tekstvakken, labels, selectievakjes en keuzelijsten.

use motor::{lees, pas_toe};
use std::process::Command;

fn voer_uit(bron: &str, stappen: &[(&str, &str)]) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:#?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = std::env::temp_dir().join(format!("jaaptaal-ond-{}-{}", std::process::id(), bron.len() + stappen.len()));
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let mut cmd = Command::new(&exe);
    cmd.arg("--tekst");
    for (a, b) in stappen {
        cmd.arg(a).arg(b);
    }
    let uit = cmd.output().unwrap();
    let _ = std::fs::remove_dir_all(&werk);
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

const VENSTER: &str = "Maak een venster met als titel Proef\n";

#[test]
fn alle_onderdeelzinnen_worden_herkend() {
    let bron = format!(
        "{}Maak een tekstvak met als naam Naam\nMaak een groot tekstvak met als naam Verhaal\nMaak een label met als naam Uitleg\nMaak een selectievakje met als naam Geluid aan\nMaak een keuzelijst met als naam Kleur\nMaak een uitklaplijst met als naam Maat\nVoeg de keuze Rood toe aan de keuzelijst Kleur\nVoeg Groot toe aan de uitklaplijst Maat\nZet de tekst Hallo in het label Uitleg\nZet de tekst Jaap in het tekstvak Naam\nMaak het tekstvak Naam leeg\nVink het selectievakje Geluid aan aan\nVink het selectievakje Geluid aan uit\nZet de focus op het tekstvak Naam\nToon de tekst in het tekstvak Naam\nToon wat er in het tekstvak Naam staat\nToon de keuze in de keuzelijst Kleur\nOnthoud n is het getal in het tekstvak Naam\nAls het selectievakje Geluid aan is aangevinkt\nEinde als\nAls het selectievakje Geluid aan is niet aangevinkt\nEinde als\nAls het tekstvak Naam is leeg\nEinde als\nAls de tekst in het tekstvak Naam is \"Jaap\"\nEinde als\n",
        VENSTER
    );
    let u = lees(&bron);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
}

#[test]
fn rekenhulp_rekent() {
    let bron = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../voorbeelden/Rekenhulp.txt")).unwrap();
    assert!(lees(&bron).is_goed());
    let uit = voer_uit(
        &bron,
        &[("--typ", "Eerste getal=12"), ("--typ", "Tweede getal=3,5"), ("--druk", "Uitrekenen"), ("--kies", "Som=keer"), ("--druk", "Uitrekenen")],
    );
    assert!(uit.contains("[label Antwoord] De uitkomst is 15,5"), "{}", uit);
    assert!(uit.contains("[label Antwoord] De uitkomst is 42"), "{}", uit);
}

#[test]
fn geen_getal_in_tekstvak_geeft_duidelijke_fout() {
    let bron = format!(
        "{}Maak een tekstvak met als naam Leeftijd\nMaak een knop met als naam Ok\nAls ik op de knop Ok druk\nToon het getal in het tekstvak Leeftijd plus 1\nEinde van de knop\n",
        VENSTER
    );
    let uit = voer_uit(&bron, &[("--typ", "Leeftijd=veertig"), ("--druk", "Ok")]);
    assert!(uit.contains("regel 5") && uit.contains("geen getal"), "{}", uit);
}

#[test]
fn selectievakje_en_leeg_tekstvak() {
    let bron = format!(
        "{}Maak een tekstvak met als naam Naam\nMaak een selectievakje met als naam Hardop\nMaak een knop met als naam Ok\nAls ik op de knop Ok druk\nAls het tekstvak Naam is leeg\nToon de tekst Vul je naam in\nAnders\nToon \"Hallo\" en dan de tekst in het tekstvak Naam\nEinde als\nAls het selectievakje Hardop is aangevinkt\nToon de tekst HARDOP\nEinde als\nEinde van de knop\n",
        VENSTER
    );
    assert!(lees(&bron).is_goed(), "{:#?}", lees(&bron).meldingen);
    let uit = voer_uit(&bron, &[("--druk", "Ok"), ("--typ", "Naam=Jaap"), ("--vink", "Hardop"), ("--druk", "Ok")]);
    assert!(uit.contains("Vul je naam in\n"), "{}", uit);
    assert!(uit.contains("Hallo Jaap\nHARDOP\n"), "{}", uit);
}

#[test]
fn onbekend_onderdeel_krijgt_voorstel() {
    let bron = format!("{}Maak een label met als naam Antwoord\nZet de tekst Hoi in het label Antwoort\n", VENSTER);
    let u = lees(&bron);
    let m = &u.meldingen[0];
    assert_eq!(m.regel, 3);
    let nieuw = pas_toe(&bron, &m.voorstel.as_ref().expect("voorstel").wijziging);
    assert!(lees(&nieuw).is_goed(), "{}", nieuw);
}

#[test]
fn verkeerd_soort_onderdeel() {
    let u = lees(&format!("{}Maak een label met als naam Uitleg\nVink het selectievakje Uitleg aan\n", VENSTER));
    assert_eq!(u.meldingen[0].regel, 3);
    assert!(u.meldingen[0].tekst.contains("selectievakje"));
}

#[test]
fn onderdeel_zonder_venster() {
    let u = lees("Maak een tekstvak met als naam Naam\n");
    assert!(u.meldingen[0].tekst.contains("geen venster"));
}

#[test]
fn twee_onderdelen_met_dezelfde_naam() {
    let u = lees(&format!("{}Maak een tekstvak met als naam Naam\nMaak een knop met als naam Naam\n", VENSTER));
    assert_eq!(u.meldingen[0].regel, 3);
}

#[test]
fn selectievakje_als_tekst_gebruiken_kan_niet_als_getal() {
    let u = lees(&format!("{}Maak een selectievakje met als naam Geluid\nToon het getal in het selectievakje Geluid\n", VENSTER));
    assert!(!u.is_goed());
}
