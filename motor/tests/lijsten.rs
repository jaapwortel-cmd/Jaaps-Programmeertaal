//! Tests voor lijsten, tellen en eigen stukjes.

use motor::{lees, pas_toe};
use std::process::Command;

fn voer_uit(bron: &str) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:#?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = std::env::temp_dir().join(format!("jaaptaal-lijst-{}-{}", std::process::id(), bron.len()));
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let uit = Command::new(&exe).arg("--tekst").output().unwrap();
    let _ = std::fs::remove_dir_all(&werk);
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

#[test]
fn voorbeeld_lijsten_en_stukjes() {
    let bron = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../voorbeelden/Lijsten en stukjes.txt")).unwrap();
    assert!(lees(&bron).is_goed(), "{:#?}", lees(&bron).meldingen);
    assert_eq!(voer_uit(&bron), "melk, kaas\n2\nkaas\nmelk\nkaas staat erop\nKoop melk\nKoop kaas\n55\n42\n8\nGoedemorgen\n110\n");
}

#[test]
fn stukje_met_eigen_namen_en_herhaling() {
    let bron = "Maak een stukje met als naam Faculteit, met als invoer n\nOnthoud uitkomst is 1\nHerhaal voor elk getal van 1 tot n\nOnthoud uitkomst is uitkomst keer getal\nEinde herhaal\nGeef als uitkomst uitkomst\nEinde van het stukje\nToon de uitkomst van Faculteit met 5\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron), "120\n");
}

#[test]
fn stukje_dat_zichzelf_aanroept_stopt_netjes() {
    let bron = "Maak een stukje met als naam Eindeloos\nDoe Eindeloos\nEinde van het stukje\nDoe Eindeloos\n";
    assert!(lees(bron).is_goed());
    let uit = voer_uit(bron);
    assert!(uit.contains("te vaak"), "{}", uit);
}

#[test]
fn lijst_te_ver_geeft_fout_met_regel() {
    let bron = "Maak een lijst met als naam l\nVoeg 1 toe aan de lijst l\nToon nummer 5 uit de lijst l\n";
    let uit = voer_uit(bron);
    assert!(uit.contains("regel 3") && uit.contains("Nummer 5"), "{}", uit);
}

#[test]
fn onbekend_stukje_krijgt_voorstel() {
    let bron = "Maak een stukje met als naam Verdubbel, met als invoer x\nGeef als uitkomst x keer 2\nEinde van het stukje\nToon de uitkomst van Verdubel met 2\n";
    let u = lees(bron);
    let m = &u.meldingen[0];
    assert_eq!(m.regel, 4);
    assert!(lees(&pas_toe(bron, &m.voorstel.as_ref().expect("voorstel").wijziging)).is_goed());
}

#[test]
fn verkeerd_aantal_invoer() {
    let u = lees("Maak een stukje met als naam Optellen, met als invoer a en b\nGeef als uitkomst a plus b\nEinde van het stukje\nDoe Optellen met 1\n");
    assert_eq!(u.meldingen[0].regel, 4);
    assert!(u.meldingen[0].tekst.contains("2 invoer"));
}

#[test]
fn geef_uitkomst_buiten_stukje() {
    let u = lees("Geef als uitkomst 3\n");
    assert!(u.meldingen[0].tekst.contains("alleen binnen een eigen stukje"));
}

#[test]
fn woord_zonder_aanhalingstekens_in_lijst() {
    let bron = "Maak een lijst met als naam boodschappen\nVoeg melk toe aan de lijst boodschappen\n";
    let u = lees(bron);
    let m = &u.meldingen[0];
    let nieuw = pas_toe(bron, &m.voorstel.as_ref().expect("voorstel").wijziging);
    assert!(nieuw.contains("Voeg \"melk\" toe"), "{}", nieuw);
    assert!(lees(&nieuw).is_goed());
}

#[test]
fn rekenen_met_een_lijst_kan_niet() {
    let u = lees("Maak een lijst met als naam l\nToon l plus 1\n");
    assert_eq!(u.meldingen[0].regel, 2);
}

#[test]
fn stukje_binnen_blok_kan_niet() {
    let u = lees("Herhaal 2 keer\nMaak een stukje met als naam X\nEinde van het stukje\nEinde herhaal\n");
    assert!(u.meldingen.iter().any(|m| m.tekst.contains("los staan")));
}
