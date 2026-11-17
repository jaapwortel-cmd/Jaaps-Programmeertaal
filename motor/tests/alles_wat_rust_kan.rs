//! Tests voor: eigen zinnen, Rust-stukjes, rekenen en tekst, wachten, programma's starten,
//! en je zinnen weer uit een gebouwd programma halen.

use motor::lees;
use std::process::Command;

fn bouw(bron: &str, map: &std::path::Path, stand: motor::bouwen::Stand) -> std::path::PathBuf {
    let code = motor::rust_code(bron).unwrap_or_else(|m| panic!("Meldingen: {:#?}", m));
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let werk = map.join("werk");
    let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
    motor::bouwen::bouw(&code, &exe, &rustc, stand, &werk).expect("bouwen");
    exe
}

fn voer_uit(bron: &str, map: &std::path::Path) -> String {
    let exe = bouw(bron, map, motor::bouwen::Stand::Snel);
    let uit = Command::new(&exe).arg("--tekst").env("JAAPTAAL_MAP", map).env("JAAPTAAL_NIET_WACHTEN", "1").output().unwrap();
    String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n")
}

fn map(naam: &str) -> std::path::PathBuf {
    let m = std::env::temp_dir().join(format!("jaaptaal-rust-{}-{}", naam, std::process::id()));
    std::fs::create_dir_all(&m).unwrap();
    m
}

#[test]
fn eigen_zinnen() {
    let m = map("zinnen");
    let bron = "Nieuwe zin: Begroet [wie]\nToon \"Hallo\" en dan [wie]\nEinde van de zin\nNieuwe zin: Zeg dag tegen [wie] betekent Toon \"Dag\" en dan [wie]\nBegroet \"Jaap\"\nZeg dag tegen \"Piet\"\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron, &m), "Hallo Jaap\nDag Piet\n");
    // Uit een bibliotheek (Mijn eigen zinnen).
    let u = motor::lees_eigen("Begroet \"Marcel\"\n", &[], "Nieuwe zin: Begroet [wie] betekent Toon [wie]\n");
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    // Niet gesloten: een duidelijke fout.
    assert!(!lees("Nieuwe zin: Begroet [wie]\nToon [wie]\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn rust_stukje() {
    let m = map("ruststukje");
    let bron = "Maak een Rust-stukje met als naam Omdraaien, met als invoer woord\nlet t = woord.tekst();\nW::T(t.chars().rev().collect())\nEinde van het Rust-stukje\nToon de uitkomst van Omdraaien met \"Jaap\"\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron, &m), "paaJ\n");
    assert!(!lees("Maak een Rust-stukje met als naam X\nW::G(1.0)\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn rekenen_en_tekst() {
    let m = map("functies");
    let bron = "Onthoud naam is \"  Jaap Wortel  \"\nToon de wortel van 16\nToon 2 tot de macht 10\nToon 10 gedeeld door 3 afgerond op 2 decimalen\nToon 7,6 afgerond\nToon de rest bij het delen van 17 door 5\nToon de absolute waarde van -4\nToon \"12,5\" als getal plus 1\nToon de lengte van naam\nOnthoud kort is naam zonder spaties\nToon kort in hoofdletters\nToon de eerste 4 tekens van kort\nToon de laatste 6 tekens van kort\nOnthoud w is de woorden van \"Dit is een zin, echt.\"\nToon het aantal in de lijst w\nOnthoud stukken is \"appel,peer\" gesplitst bij \",\"\nToon de lijst stukken samengevoegd met \" en \"\nToon \"ik heb een fiets\" waarin \"fiets\" vervangen is door \"auto\"\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(
        voer_uit(bron, &m),
        "4\n1024\n3,33\n8\n2\n4\n13,5\n15\nJAAP WORTEL\nJaap\nWortel\n5\nappel en peer\nik heb een auto\n"
    );
    assert!(!lees("Toon de wortel van \"tekst\"\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn wachten_starten_openen() {
    let m = map("wacht");
    std::fs::write(m.join("brief.txt"), "hallo").unwrap();
    let bron = "Wacht 2 seconden\nStart het programma notepad.exe\nOpen het bestand brief.txt\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(voer_uit(bron, &m), "[wacht] 2 seconden\n[start] notepad.exe\n[open] brief.txt\n");
    assert!(!lees("Wacht 5000 seconden\n").is_goed());
    assert!(!lees("Wacht \"even\" seconden\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn zinnen_weer_uit_het_programma() {
    let m = map("terug");
    let bron = "Maak een venster met als titel Hallo\nToon \"Dag Jaap\"\n";
    let exe = bouw(bron, &m, motor::bouwen::Stand::Klein);
    let inhoud = std::fs::read(&exe).unwrap();
    assert_eq!(motor::zinnen_uit_programma(&inhoud).as_deref(), Some(bron.trim_end()));
    assert_eq!(motor::zinnen_uit_programma(b"geen programma"), None);
    let _ = std::fs::remove_dir_all(&m);
}
