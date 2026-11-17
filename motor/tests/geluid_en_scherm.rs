//! Tests voor geluid, sneltoetsen, kleuren, de handleiding (F1) en de beschrijving van het scherm.

use motor::{lees, pas_toe};
use std::process::Command;

fn voer_uit(bron: &str, extra: &[&str]) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:#?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = std::env::temp_dir().join(format!("jaaptaal-gs-{}-{}", std::process::id(), bron.len() + extra.len()));
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let uit = Command::new(&exe).arg("--tekst").args(extra).env("JAAPTAAL_MAP", &werk).output().unwrap();
    let _ = std::fs::remove_dir_all(&werk);
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

const BASIS: &str = "Maak een venster met als titel Proef\nMaak een knop met als naam Piep\n";

#[test]
fn geluidzinnen_worden_herkend() {
    let bron = format!("{}Piep\nSpeel een piep\nSpeel een piep van 440 hertz en 300 milliseconden\nSpeel het geluid bel.wav\nSpeel het geluid bel.wav af en wacht\nStop het geluid\n", BASIS);
    assert!(lees(&bron).is_goed(), "{:#?}", lees(&bron).meldingen);
}

#[test]
fn piep_in_tekststand() {
    let uit = voer_uit("Speel een piep van 440 hertz en 300 milliseconden\nPiep\n", &[]);
    assert_eq!(uit, "[piep] 440 hertz, 300 milliseconden\n[piep] 880 hertz, 200 milliseconden\n");
}

#[test]
fn alleen_wav() {
    let u = lees("Speel het geluid liedje.ogg\n");
    assert!(u.meldingen[0].tekst.contains("MP3"));
}

#[test]
fn geluid_dat_er_niet_is() {
    let uit = voer_uit("Speel het geluid bestaatniet.wav\n", &[]);
    assert!(uit.contains("regel 1") && uit.contains("niet vinden"), "{}", uit);
}

#[test]
fn sneltoetsen() {
    for (zin, naam) in [("Ctrl+W", "Ctrl+W"), ("control shift s", "Ctrl+Shift+S"), ("Alt 1", "Alt+1"), ("F5", "F5"), ("Ctrl+Enter", "Ctrl+Enter")] {
        let bron = format!("{}Geef de knop Piep de sneltoets {}\n", BASIS, zin);
        let u = lees(&bron);
        assert!(u.is_goed(), "{}: {:#?}", zin, u.meldingen);
        assert!(motor::beschrijf(&u).contains(naam), "{}", zin);
    }
    for slecht in ["W", "F1", "Escape", "Ctrl+Q+W"] {
        let u = lees(&format!("{}Geef de knop Piep de sneltoets {}\n", BASIS, slecht));
        assert!(!u.is_goed(), "{} had niet mogen werken", slecht);
    }
}

#[test]
fn twee_knoppen_zelfde_sneltoets() {
    let u = lees(&format!(
        "{}Maak een knop met als naam Stop\nGeef de knop Piep de sneltoets Ctrl+P\nGeef de knop Stop de sneltoets Ctrl+P\n",
        BASIS
    ));
    assert_eq!(u.meldingen[0].regel, 5);
    assert!(u.meldingen[0].tekst.contains("tegen"));
}

#[test]
fn kleuren_en_contrast() {
    assert!(lees(&format!("{}Geef het venster de kleur donkerblauw\nGeef de tekst de kleur wit\n", BASIS)).is_goed());
    let bron = format!("{}Geef het venster de kleur geel\nGeef de tekst de kleur wit\n", BASIS);
    let u = lees(&bron);
    let m = &u.meldingen[0];
    assert_eq!(m.regel, 4);
    assert!(m.tekst.contains("slecht te lezen"));
    let nieuw = pas_toe(&bron, &m.voorstel.as_ref().unwrap().wijziging);
    assert!(nieuw.contains("Geef de tekst de kleur zwart"), "{}", nieuw);
    assert!(lees(&nieuw).is_goed());
    assert!(lees("Geef het venster de kleur paarsig\n").meldingen[0].tekst.contains("ken ik niet"));
    assert!(lees("Geef het venster de kleur rood\n").meldingen[0].tekst.contains("geen venster"));
}

#[test]
fn handleiding_met_f1() {
    let bron = format!("{}In de handleiding staat: Druk op Piep.\nIn de handleiding staat: Escape sluit\n", BASIS);
    assert!(lees(&bron).is_goed());
    assert_eq!(voer_uit(&bron, &["--handleiding"]), "Druk op Piep.\nEscape sluit.\n");
    assert_eq!(voer_uit(BASIS, &["--handleiding"]), "Dit programma heeft geen handleiding.\n");
    let tegen = lees(&format!("{}In de handleiding staat: a\nDe handleiding staat in het bestand H.txt\n", BASIS));
    assert!(!tegen.is_goed());
}

#[test]
fn scherm_beschrijving_en_tips() {
    let bron = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../voorbeelden/Rekenhulp.txt")).unwrap();
    let b = motor::scherm::beschrijf_scherm(&lees(&bron).programma);
    assert!(b.contains("Venster 1: \"Rekenhulp\""));
    assert!(b.contains("1. het tekstvak Eerste getal"));
    assert!(b.contains("Met Tab ga je in deze volgorde langs: Eerste getal, Tweede getal, Som, Uitrekenen."));
    let slecht = "Maak een venster met als titel X\nMaak een knop met als naam Ok\nMaak een tekstvak met als naam Naam\n";
    let b = motor::scherm::beschrijf_scherm(&lees(slecht).programma);
    assert!(b.contains("Zet de knop dan onder de vakken"), "{}", b);
}

#[test]
fn mp3_en_zip_zinnen() {
    let bron = "Speel het geluid liedje.mp3\nPak de map Foto's in tot het zipbestand Foto's.zip\nPak het zipbestand Foto's.zip uit in de map Terug\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert!(!lees("Speel het geluid liedje.ogg\n").is_goed());
    assert!(!lees("Pak de map A in tot het zipbestand A.rar\n").is_goed());
}

#[cfg(windows)]
#[test]
fn zip_inpakken_en_uitpakken() {
    let werk = std::env::temp_dir().join(format!("jaaptaal-zip-{}", std::process::id()));
    std::fs::create_dir_all(werk.join("Bron")).unwrap();
    std::fs::write(werk.join("Bron").join("a.txt"), "hallo").unwrap();
    let code = motor::rust_code("Pak de map Bron in tot het zipbestand pak.zip\nPak het zipbestand pak.zip uit in de map Terug\nToon de inhoud van het bestand Terug/a.txt\n").unwrap();
    let rustc = motor::bouwen::zoek_rustc(None).unwrap();
    let exe = werk.join("z.exe");
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk.join("w")).unwrap();
    let uit = Command::new(&exe).arg("--tekst").env("JAAPTAAL_MAP", &werk).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&uit.stdout).trim(), "hallo");
    let _ = std::fs::remove_dir_all(&werk);
}
