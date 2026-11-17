//! Automatische tests voor Jaap's Programmeertaal.
//! Elke test zegt in de naam wat hij controleert.

use motor::{lees, pas_toe, MeldingSoort, Wijziging};
use std::path::PathBuf;
use std::process::Command;

/// Bouwt een programma echt met de Rust-compiler en voert het uit in tekst-stand.
fn voer_uit(bron: &str, drukken: &[&str]) -> String {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("Voor deze test is rustc nodig");
    let id = format!("{:x}", {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        bron.hash(&mut h);
        drukken.hash(&mut h);
        h.finish()
    });
    let werk = std::env::temp_dir().join(format!("jaaptaal-test-{}-{}", std::process::id(), id));
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).expect("bouwen");
    let mut cmd = Command::new(&exe);
    cmd.arg("--tekst");
    for d in drukken {
        cmd.arg("--druk").arg(d);
    }
    let uit = cmd.output().expect("starten");
    let _ = std::fs::remove_dir_all(&werk);
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

fn goed(bron: &str) {
    let u = lees(bron);
    assert!(u.is_goed(), "Verwacht geen meldingen bij:\n{}\nmaar kreeg: {:#?}", bron, u.meldingen);
}

fn eerste_melding(bron: &str) -> motor::Melding {
    let u = lees(bron);
    assert!(!u.is_goed(), "Verwacht een melding bij:\n{}", bron);
    u.meldingen[0].clone()
}

// ---------------------------------------------------------------- het voorbeeld

const BEGROETING: &str = "Maak een venster met als titel Begroeting\r\nMaak een knop met als naam Welkom\r\nAls ik op de knop Welkom druk\r\nToon de tekst Hallo\r\nEinde van de knop\r\n";

#[test]
fn voorbeeld_begroeting_werkt() {
    goed(BEGROETING);
    let uit = voer_uit(BEGROETING, &["Welkom"]);
    assert_eq!(uit, "[venster] Begroeting\n[knop] Welkom\n[druk op] Welkom\nHallo\n");
}

#[test]
fn voorbeeld_zonder_drukken_toont_geen_hallo() {
    let uit = voer_uit(BEGROETING, &[]);
    assert!(!uit.contains("Hallo"));
}

#[test]
fn voorbeeld_met_punten_en_kleine_letters() {
    goed("maak een venster met als titel Begroeting.\nmaak een knop met als naam Welkom.\nals ik op de knop welkom druk.\ntoon de tekst Hallo.\neinde van de knop.\n");
}

// ---------------------------------------------------------------- elke zin

#[test]
fn elke_ondersteunde_zin_wordt_herkend() {
    let zinnen = [
        "Opmerking: dit doet niets",
        "Commentaar dit ook niet",
        "// ook dit niet",
        "Maak een venster met als titel Mijn venster",
        "Maak een venster met de titel Twee",
        "Maak een knop met als naam Derde",
        "Maak een knop met de naam Nog eentje",
        "Onthoud teller is 0",
        "Onthoud dat stand 5 is",
        "Onthoud mijn naam is \"Jaap\"",
        "Onthoud klaar is waar",
        "Onthoud fout is niet waar",
        "Verander teller in 3",
        "Verander teller naar 4",
        "Zet teller op 5",
        "Tel 1 op bij teller",
        "Trek 2 af van teller",
        "Verhoog teller met 3",
        "Verlaag teller met 1",
        "Toon de tekst Hallo daar",
        "Toon teller",
        "Toon de waarde van teller",
        "Toon de uitkomst van 3 plus 4",
        "Toon \"Je naam is\" en dan mijn naam",
        "Toon de tekst aanhalingsteken openen Hallo aanhalingsteken sluiten",
        "Als teller is groter dan 5\nEinde als",
        "Als teller groter is dan 5 dan\nAnders\nEinde als",
        "Als teller is kleiner dan 5\nEinde als",
        "Als teller is gelijk aan 5\nEinde als",
        "Als teller is niet gelijk aan 5\nEinde als",
        "Als teller is 5\nEinde",
        "Als teller is niet 5\nEinde als",
        "Als teller is minstens 5\nEinde als",
        "Als teller is hoogstens 5\nEinde als",
        "Als klaar\nEinde als",
        "Als mijn naam is \"Jaap\"\nEinde als",
        "Herhaal 10 keer\nEinde herhaal",
        "Herhaal tien keer\nEinde van de herhaling",
        "Herhaal zolang teller is kleiner dan 10, hoogstens 100 keer\nEinde herhaal",
        "Als ik op de knop Ok druk\nEinde van de knop",
    ];
    for zin in zinnen {
        let bron = format!(
            "Maak een venster met als titel Proef\nMaak een knop met als naam Ok\nMaak een knop met als naam Nog een\nOnthoud teller is 1\nOnthoud mijn naam is \"Jaap\"\nOnthoud klaar is waar\n{}\n",
            zin
        );
        let u = lees(&bron);
        let echte: Vec<_> = u.meldingen.iter().filter(|m| !m.tekst.contains("al op regel")).collect();
        assert!(echte.is_empty(), "Zin niet goed herkend: {}\nMeldingen: {:#?}", zin, echte);
    }
}

// ---------------------------------------------------------------- rekenen

#[test]
fn rekenen_klopt() {
    let bron = "Onthoud a is 7\nOnthoud b is 3\nToon a plus b\nToon a min b\nToon a keer b\nToon a gedeeld door 2\nToon 2 plus 3 keer 4\nToon min 5 plus 2\nToon 3,5 plus 1\nToon tien plus eenentwintig\nOnthoud c is a gedeeld door b\nToon c\n";
    goed(bron);
    let uit = voer_uit(bron, &[]);
    assert_eq!(uit, "10\n4\n21\n3,5\n14\n-3\n4,5\n31\n2,3333333333\n");
}

#[test]
fn variabelen_onthouden_en_veranderen() {
    let bron = "Onthoud teller is 1\nTel 4 op bij teller\nToon teller\nTrek 2 af van teller\nToon teller\nVerander teller in 10\nToon teller\nVerhoog teller met 5\nToon teller\nOnthoud naam is \"Jaap\"\nToon \"Hallo\" en dan naam\n";
    goed(bron);
    assert_eq!(voer_uit(bron, &[]), "5\n3\n10\n15\nHallo Jaap\n");
}

// ---------------------------------------------------------------- keuzes

#[test]
fn keuzes_kloppen() {
    let bron = "Onthoud x is 7\nAls x is groter dan 5\nToon de tekst groot\nAnders\nToon de tekst klein\nEinde als\nAls x is kleiner dan 5\nToon de tekst klein\nAnders\nToon de tekst niet klein\nEinde als\nAls x is 7\nToon de tekst zeven\nEinde als\nAls x is niet 7\nToon de tekst fout\nEinde als\nOnthoud klaar is niet waar\nAls klaar\nToon de tekst fout\nAnders\nToon de tekst nog niet klaar\nEinde als\n";
    goed(bron);
    assert_eq!(voer_uit(bron, &[]), "groot\nniet klein\nzeven\nnog niet klaar\n");
}

// ---------------------------------------------------------------- herhaling

#[test]
fn herhaling_met_grens() {
    let bron = "Onthoud teller is 0\nHerhaal 3 keer\nTel 1 op bij teller\nToon teller\nEinde herhaal\nHerhaal zolang teller is kleiner dan 100, hoogstens 2 keer\nTel 10 op bij teller\nEinde herhaal\nToon teller\n";
    goed(bron);
    let uit = voer_uit(bron, &[]);
    assert!(uit.starts_with("1\n2\n3\n"), "{}", uit);
    assert!(uit.contains("is gestopt na 2 keer"), "{}", uit);
    assert!(uit.ends_with("23\n"), "{}", uit);
}

#[test]
fn herhaling_zonder_grens_kan_niet() {
    let m = eerste_melding("Onthoud x is 0\nHerhaal zolang x is kleiner dan 5\nEinde herhaal\n");
    assert_eq!(m.regel, 2);
    assert!(m.tekst.contains("geen grens"));
    let v = m.voorstel.expect("voorstel");
    let nieuw = pas_toe("Onthoud x is 0\nHerhaal zolang x is kleiner dan 5\nEinde herhaal\n", &v.wijziging);
    goed(&nieuw);
}

#[test]
fn herhaling_te_vaak_mag_niet() {
    let m = eerste_melding("Herhaal 5000000 keer\nEinde herhaal\n");
    assert!(m.tekst.contains("miljoen"));
}

// ---------------------------------------------------------------- foutmeldingen

#[test]
fn foutmeldingen_hebben_regelnummer() {
    let bron = "Onthoud x is 1\n\nOnthoud y is \"tekst\"\nToon x plus y\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 4);
    assert!(m.volledig().starts_with("Regel 4: "));
    assert!(m.tekst.contains("en dan"), "{}", m.tekst);
}

#[test]
fn blok_niet_gesloten() {
    let bron = "Herhaal 2 keer\nToon 1\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 1);
    assert!(m.tekst.contains("nergens gesloten"));
    let nieuw = pas_toe(bron, &m.voorstel.unwrap().wijziging);
    goed(&nieuw);
}

#[test]
fn einde_zonder_blok() {
    let m = eerste_melding("Toon 1\nEinde als\n");
    assert_eq!(m.regel, 2);
}

#[test]
fn delen_door_nul() {
    let m = eerste_melding("Toon 5 gedeeld door 0\n");
    assert!(m.tekst.contains("nul"));
}

#[test]
fn fout_tijdens_uitvoeren_heeft_regelnummer() {
    let bron = "Onthoud x is 0\nToon 5 gedeeld door x\n";
    goed(bron);
    let uit = voer_uit(bron, &[]);
    assert!(uit.contains("regel 2"), "{}", uit);
    assert!(uit.contains("nul"), "{}", uit);
}

// ---------------------------------------------------------------- tikfouten

#[test]
fn tikfout_voorstel_verandert_pas_na_ja() {
    let bron = "Mak een venster met als titel Begroeting\n";
    let u = lees(bron);
    let m = &u.meldingen[0];
    assert_eq!(m.soort, MeldingSoort::Vraag);
    assert_eq!(m.regel, 1);
    let v = m.voorstel.as_ref().expect("een voorstel");
    assert!(v.vraag.contains("Maak een venster met als titel Begroeting"), "{}", v.vraag);
    // Zonder ja blijft de tekst zoals hij was: lees() verandert niets.
    assert_eq!(bron, "Mak een venster met als titel Begroeting\n");
    // Na ja:
    let nieuw = pas_toe(bron, &v.wijziging);
    assert_eq!(nieuw, "Maak een venster met als titel Begroeting\n");
    goed(&nieuw);
}

#[test]
fn tikfouten_in_andere_woorden() {
    for (fout, goed_) in [
        ("Toom de tekst Hallo", "Toon de tekst Hallo"),
        ("Herhal 3 keer\nEinde herhaal", "Herhaal 3 keer"),
        ("Maak een venster met als titel A\nMaak een kop met als naam B", "Maak een knop met als naam B"),
    ] {
        let m = eerste_melding(fout);
        let v = m.voorstel.unwrap_or_else(|| panic!("geen voorstel bij {}", fout));
        assert!(v.vraag.contains(goed_), "{} -> {}", fout, v.vraag);
    }
}

#[test]
fn verkeerde_naam_krijgt_voorstel() {
    let bron = "Onthoud teller is 1\nToon telle\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 2);
    let nieuw = pas_toe(bron, &m.voorstel.unwrap().wijziging);
    assert_eq!(nieuw, "Onthoud teller is 1\nToon teller\n");
}

#[test]
fn verkeerde_knopnaam_krijgt_voorstel() {
    let bron = "Maak een venster met als titel A\nMaak een knop met als naam Welkom\nAls ik op de knop Welkm druk\nToon 1\nEinde van de knop\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 3);
    let nieuw = pas_toe(bron, &m.voorstel.unwrap().wijziging);
    goed(&nieuw);
}

// ---------------------------------------------------------------- onbekende zinnen

#[test]
fn onbekende_zin_geeft_vraag_en_geen_gok() {
    let m = eerste_melding("Toon 1\nSpring drie keer in de lucht\n");
    assert_eq!(m.regel, 2);
    assert_eq!(m.soort, MeldingSoort::Vraag);
    assert!(m.voorstel.is_none());
    assert!(m.tekst.contains("ken ik niet"));
    assert!(m.tekst.contains("gok niet"));
}

#[test]
fn nog_niet_gebouwd_wordt_eerlijk_gezegd() {
    let m = eerste_melding("Maak een tabel met als naam Uren\n");
    assert!(m.tekst.contains("nog niet"));
}

#[test]
fn programma_met_meldingen_wordt_niet_gebouwd() {
    assert!(motor::rust_code("Spring\n").is_err());
}

// ---------------------------------------------------------------- tegenstrijdig

#[test]
fn twee_keer_dezelfde_knop_beschreven() {
    let bron = "Maak een venster met als titel A\nMaak een knop met als naam B\nAls ik op de knop B druk\nToon 1\nEinde van de knop\nAls ik op de knop B druk\nToon 2\nEinde van de knop\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 6);
    assert_eq!(m.soort, MeldingSoort::Vraag);
    assert!(m.tekst.contains("tegen"));
}

#[test]
fn twee_knoppen_met_dezelfde_naam() {
    let m = eerste_melding("Maak een venster met als titel A\nMaak een knop met als naam B\nMaak een knop met als naam B\n");
    assert_eq!(m.regel, 3);
}

#[test]
fn getal_wordt_tekst_spreekt_tegen() {
    let m = eerste_melding("Onthoud x is 1\nVerander x in \"hallo\"\n");
    assert_eq!(m.regel, 2);
    assert!(m.tekst.contains("tegen"));
}

#[test]
fn twee_keer_onthouden_geeft_voorstel() {
    let bron = "Onthoud x is 1\nOnthoud x is 2\nToon x\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 2);
    let nieuw = pas_toe(bron, &m.voorstel.unwrap().wijziging);
    assert_eq!(nieuw, "Onthoud x is 1\nVerander x in 2\nToon x\n");
    goed(&nieuw);
}

#[test]
fn knop_zonder_venster_sluit_niet_aan() {
    let bron = "Maak een knop met als naam Ok\n";
    let m = eerste_melding(bron);
    assert!(m.tekst.contains("geen venster"));
    let nieuw = pas_toe(bron, &m.voorstel.unwrap().wijziging);
    goed(&nieuw);
}

#[test]
fn verkeerd_einde_sluit_niet_aan() {
    let bron = "Herhaal 2 keer\nToon 1\nEinde als\n";
    let m = eerste_melding(bron);
    assert_eq!(m.regel, 3);
    let nieuw = pas_toe(bron, &m.voorstel.unwrap().wijziging);
    assert_eq!(nieuw, "Herhaal 2 keer\nToon 1\nEinde herhaal\n");
}

#[test]
fn tekst_vergelijken_met_getal() {
    let m = eerste_melding("Onthoud naam is \"Jaap\"\nAls naam is groter dan 3\nEinde als\n");
    assert_eq!(m.regel, 2);
}

#[test]
fn naam_zonder_onthoud_krijgt_voorstel() {
    let bron = "Tel 1 op bij score\n";
    let m = eerste_melding(bron);
    let v = m.voorstel.unwrap();
    assert!(matches!(v.wijziging, Wijziging::VoegInVoor { .. }));
    goed(&pas_toe(bron, &v.wijziging));
}

// ---------------------------------------------------------------- bestanden

#[test]
fn txt_openen_en_opslaan() {
    let map = std::env::temp_dir().join(format!("jaaptaal-txt-{}", std::process::id()));
    std::fs::create_dir_all(&map).unwrap();
    let pad: PathBuf = map.join("programma.txt");
    motor::bestand::schrijf_txt(&pad, BEGROETING).unwrap();
    let bytes = std::fs::read(&pad).unwrap();
    assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]), "UTF-8 met BOM");
    let terug = motor::bestand::lees_txt(&pad).unwrap();
    assert_eq!(terug, BEGROETING);
    goed(&terug);
    // Een oud Kladblok-bestand (Windows-1252) met é.
    std::fs::write(&pad, b"Toon de tekst caf\xe9\r\n").unwrap();
    assert_eq!(motor::bestand::lees_txt(&pad).unwrap(), "Toon de tekst café\r\n");
    std::fs::remove_dir_all(&map).unwrap();
}

// ---------------------------------------------------------------- de tussenstap

#[test]
fn beschrijving_en_rustcode_laten_zien_wat_er_gemaakt_is() {
    let u = lees(BEGROETING);
    let b = motor::beschrijf(&u);
    assert!(b.contains("Regel 1: maak een venster"));
    assert!(b.contains("Regel 4: toon de tekst \"Hallo\""));
    let code = motor::rust_code(BEGROETING).unwrap();
    assert!(code.contains("// Regel 4: Toon de tekst Hallo"));
}
