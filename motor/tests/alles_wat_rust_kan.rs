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

#[test]
fn opdrachtvak_in_eigen_programma() {
    let m = map("opdrachtvak");
    let bron = "Maak een venster met als titel Hulp\nGeef mijn programma een opdrachtvak\nMaak een knop met als naam Opslaan\nAls ik op de knop Opslaan druk\nToon \"opgeslagen\"\nEinde van de knop\nAls ik zeg \"maak het donker\"\nToon \"donker\"\nEinde van de opdracht\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let exe = bouw(bron, &m, motor::bouwen::Stand::Snel);
    let zeg = |zin: &str| {
        let uit = Command::new(&exe).args(["--tekst", "--typ", &format!("Opdracht={}", zin), "--druk", "Doe het"]).env("JAAPTAAL_MAP", &m).output().unwrap();
        String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n").lines().last().unwrap_or("").to_string()
    };
    assert_eq!(zeg("maak het nu maar donker"), "donker");
    assert_eq!(zeg("druk op opslaan"), "opgeslagen");
    assert_eq!(zeg("donker"), "[vraag] Bedoel je: maak het donker?");
    assert!(zeg("bananen").starts_with("Dat snap ik niet"));
    // Zonder opdrachtvak: eerst vragen.
    assert!(!lees("Maak een venster met als titel A\nAls ik zeg \"hallo\"\nToon 1\nEinde van de opdracht\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn verzoeken_in_gewone_taal() {
    use motor::opdrachten::{begrijp_alles, Antwoord, Begrip, Functie};
    use motor::wensen::Voorstel;
    // Zinnen zoals Jaap ze tegen Claude Code zei, uit zijn programma's.
    let editor = [
        ("Ga maar ook verder met stap twee", Functie::Verder),
        ("Maak alvast maar het programma installatieprogramma", Functie::Installatie),
        ("Geef het nieuwe installatie Stant zodra het klaar is", Functie::Installatie),
        ("Heb je nou begrepen wat ik wil?", Functie::Beschrijf),
        ("wat moet er nog gebeuren voor het lijstje", Functie::Wensenlijst),
        ("maak er een exe van", Functie::BouwExe),
        ("zet de versie omhoog", Functie::VersieOmhoog),
    ];
    for (zin, f) in editor {
        match begrijp_alles(zin, &[]) {
            Antwoord::Editor(Begrip::Doe { functie, .. }) => assert_eq!(functie, f, "{}", zin),
            ander => panic!("{} gaf {:?}", zin, ander),
        }
    }
    let taal = [
        ("Ook tabbladen inbouwen: een tabblad Geluid", "Maak een tabblad met als naam Geluid"),
        ("Een eigen pictogram voor mijn programma, logo.ico", "Het pictogram staat in het bestand logo.ico"),
        ("Eigen meldingen die je schermlezer uitspreekt: \"Klaar\"", "Zeg \"Klaar\""),
        ("geef de knop Opslaan de sneltoets Ctrl+S", "Geef de knop Opslaan de sneltoets Ctrl+S"),
    ];
    for (zin, regel) in taal {
        match begrijp_alles(zin, &[]) {
            Antwoord::Taal(Voorstel::Regels { regels, .. }) => {
                assert_eq!(regels.join(" | "), regel, "{}", zin);
                assert!(lees(&format!("Maak een venster met als titel A\nMaak een knop met als naam Opslaan\n{}\n", regel)).programma.opdrachten.len() >= 2);
            }
            ander => panic!("{} gaf {:?}", zin, ander),
        }
    }
    assert!(matches!(begrijp_alles("Jaap wil geen Microsoft Edge", &[]), Antwoord::Taal(Voorstel::NogNiet(_))));
}

#[test]
fn word_en_pdf_lezen() {
    let m = map("lezen");
    let data = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("data");
    std::fs::copy(data.join("brief.docx"), m.join("brief.docx")).unwrap();
    std::fs::copy(data.join("brief.pdf"), m.join("brief.pdf")).unwrap();
    // Ook een PDF die het programma zelf maakt, weer lezen.
    let bron = "Schrijf \"Dag Jaap\" in het PDF-bestand eigen.pdf\nOnthoud w is de tekst van het Word-bestand brief.docx\nToon de eerste 30 tekens van w\nToon de tekst van het PDF-bestand brief.pdf\nToon de tekst van het PDF-bestand eigen.pdf\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit(bron, &m);
    assert_eq!(uit, "Beste Piet,\nDank je wel voor j\nHallo uit een PDF\nTweede regel met (haakjes) en é\nDag Jaap\n", "{}", uit);
    let docx = voer_uit("Toon de tekst van het Word-bestand brief.docx\n", &m);
    assert!(docx.contains("brief & de foto's. Groetjes, Jaap\nherhaling herhaling"), "{}", docx);
    let _ = std::fs::remove_dir_all(&m);
}

/// Een korte toon als WAV (mono, 16 bit).
fn toon_wav() -> Vec<u8> {
    let monsters: Vec<i16> = (0..4410).map(|i| ((i as f32 * 0.06).sin() * 12000.0) as i16).collect();
    let mut w = Vec::new();
    w.extend_from_slice(b"RIFF");
    w.extend_from_slice(&(36 + monsters.len() as u32 * 2).to_le_bytes());
    w.extend_from_slice(b"WAVEfmt ");
    for x in [16u32.to_le_bytes().to_vec(), 1u16.to_le_bytes().to_vec(), 1u16.to_le_bytes().to_vec(), 44100u32.to_le_bytes().to_vec(), 88200u32.to_le_bytes().to_vec(), 2u16.to_le_bytes().to_vec(), 16u16.to_le_bytes().to_vec()] {
        w.extend_from_slice(&x);
    }
    w.extend_from_slice(b"data");
    w.extend_from_slice(&(monsters.len() as u32 * 2).to_le_bytes());
    for m in monsters {
        w.extend_from_slice(&m.to_le_bytes());
    }
    w
}

#[test]
fn geluid_links_rechts_achter() {
    let m = map("ruimte");
    std::fs::write(m.join("toon.wav"), toon_wav()).unwrap();
    let bron = "Speel het geluid toon.wav links\nSpeel het geluid toon.wav aan de rechterkant\nSpeel het geluid toon.wav achter me af en wacht\nSpeel het geluid toon.wav boven me\nSpeel het geluid toon.wav links onder me\nSpeel het geluid toon.wav rechts achter en boven mij\nSpeel het geluid toon.wav\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    assert_eq!(
        voer_uit(bron, &m),
        "[geluid] toon.wav (links)\n[geluid] toon.wav (rechts)\n[geluid] toon.wav (achter je) (wacht tot het klaar is)\n[geluid] toon.wav (boven je)\n[geluid] toon.wav (links, onder je)\n[geluid] toon.wav (rechts, achter je, boven je)\n[geluid] toon.wav\n"
    );
    // Links: het linkeroor hoort het harder dan het rechteroor.
    let exe = bouw("Speel het geluid toon.wav links\n", &m, motor::bouwen::Stand::Snel);
    let uitvoer = m.join("uit.wav");
    Command::new(&exe).arg("--tekst").env("JAAPTAAL_MAP", &m).env("JAAPTAAL_GELUID_BEWAAR", &uitvoer).output().unwrap();
    let wav = std::fs::read(&uitvoer).expect("ruimtelijk geluid gemaakt");
    assert_eq!(u16::from_le_bytes([wav[22], wav[23]]), 2, "stereo");
    let (mut l, mut r) = (0f64, 0f64);
    for c in wav[44..].chunks_exact(4) {
        l += (i16::from_le_bytes([c[0], c[1]]) as f64).abs();
        r += (i16::from_le_bytes([c[2], c[3]]) as f64).abs();
    }
    assert!(l > r * 2.5, "links {} rechts {}", l, r);
    assert!(!lees("Speel het geluid muziek.mp3 links\n").is_goed(), "mp3 kan nog niet links");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn werkbalk() {
    let m = map("werkbalk");
    let bron = "Maak een venster met als titel Tekst\nMaak een werkbalk met de knoppen Nieuw, Openen en Opslaan\nMaak een groot tekstvak met als naam Brief\nAls ik op de knop Opslaan druk\nToon \"opgeslagen\"\nEinde van de knop\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let exe = bouw(bron, &m, motor::bouwen::Stand::Snel);
    let uit = Command::new(&exe).args(["--tekst", "--druk", "Opslaan"]).env("JAAPTAAL_MAP", &m).output().unwrap();
    let uit = String::from_utf8_lossy(&uit.stdout).replace("\r\n", "\n");
    assert!(uit.contains("[werkbalk] Nieuw, Openen, Opslaan\n"), "{}", uit);
    assert!(uit.ends_with("opgeslagen\n"), "{}", uit);
    // Een knop op de werkbalk zonder venster, of een onbekende knop: eerst vragen.
    assert!(!lees("Maak een werkbalk met de knoppen A en B\n").is_goed());
    assert!(!lees("Maak een venster met als titel A\nMaak een werkbalk met de knoppen A en B\nAls ik op de knop C druk\nToon 1\nEinde van de knop\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn qr_code_maken() {
    let m = map("qr");
    let bron = "Maak een QR-code van \"https://www.voorbeeld.nl\" in het bestand link.png\nOnthoud lang is \"abcdefghij\"\nMaak een QR-code van lang en dan lang in het bestand twee.png\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    voer_uit(bron, &m);
    for naam in ["link.png", "twee.png"] {
        let png = std::fs::read(m.join(naam)).expect("PNG gemaakt");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"), "{} is geen PNG", naam);
        // Versie 2 (25 vakjes) met een rand van 4 en 8 beeldpunten per vakje: 264 breed.
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), (25 + 8) * 8, "{}", naam);
    }
    assert!(!lees("Maak een QR-code van \"x\" in het bestand qr.jpg\n").is_goed());
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn jaaps_eigen_zinnen_als_voorbeeld() {
    use motor::opdrachten::{begrijp_alles, Antwoord, Begrip, Functie};
    use motor::wensen::Voorstel;
    // Precies zoals Jaap het zei.
    assert!(matches!(begrijp_alles("Ga maar verder met de punten van de lijst.", &[]), Antwoord::Editor(Begrip::Doe { functie: Functie::Wensenlijst, .. })));
    assert!(matches!(begrijp_alles("Geef mij hier het installatie best direct", &[]), Antwoord::Editor(Begrip::Doe { functie: Functie::Installatie, .. })));
    // Een vraag of iets kan: uitleg met de zin die je kunt gebruiken.
    match begrijp_alles("Kan ik ook QR-codes maken?", &[]) {
        Antwoord::Taal(Voorstel::Uitleg(u)) => assert!(u.contains("Maak een QR-code"), "{}", u),
        Antwoord::Lijkt { dan, .. } => assert!(matches!(*dan, Antwoord::Taal(Voorstel::Uitleg(_)))),
        ander => panic!("{:?}", ander),
    }
    match begrijp_alles("ruimtelijk geluid boven en onder mij", &[]) {
        Antwoord::Taal(Voorstel::Uitleg(u)) => assert!(u.contains("boven me"), "{}", u),
        Antwoord::Lijkt { dan, .. } => assert!(matches!(*dan, Antwoord::Taal(Voorstel::Uitleg(ref u)) if u.contains("boven"))),
        ander => panic!("{:?}", ander),
    }
}

#[test]
fn uit_bouwmaatje_schermlezer_en_braille() {
    // Een sneltoets van NVDA: eerst vragen.
    let m = lees("Maak een venster met als titel A\nMaak een knop met als naam Nieuw\nGeef de knop Nieuw de sneltoets Ctrl+Alt+N\n");
    assert!(m.meldingen.iter().any(|x| x.tekst.contains("start NVDA")), "{:#?}", m.meldingen);
    // Pijlen en emoji: een tip voor braille bij Ctrl+F6.
    let u = lees("Maak een venster met als titel A\nMaak een knop met als naam Verder →\n");
    assert!(motor::scherm::beschrijf_scherm(&u.programma).contains("Tip voor braille"));
}

#[test]
fn zelfde_programma_niet_opnieuw_bouwen() {
    let m = map("snel");
    let code = motor::rust_code("Toon 1\n").unwrap();
    let rustc = motor::bouwen::zoek_rustc(None).unwrap();
    let werk = m.join("werk");
    let exe = m.join(if cfg!(windows) { "p.exe" } else { "p" });
    let eerste = motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).unwrap();
    let tweede = motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).unwrap();
    assert!(tweede.duur < eerste.duur / 5, "eerste {:?}, tweede {:?}", eerste.duur, tweede.duur);
    // Andere code: wel opnieuw bouwen.
    let ander = motor::rust_code("Toon 2\n").unwrap();
    motor::bouwen::bouw(&ander, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).unwrap();
    let uit = Command::new(&exe).arg("--tekst").output().unwrap();
    assert_eq!(String::from_utf8_lossy(&uit.stdout).trim(), "2");
    let _ = std::fs::remove_dir_all(&m);
}

#[test]
fn venstergedrag_en_reservekopie() {
    let m = map("gedrag");
    std::fs::write(m.join("notities.txt"), "hallo").unwrap();
    let bron = "Maak een venster met als titel Notities\nHet venster start schermvullend\nVraag bij afsluiten \"Wil je echt stoppen?\"\nMijn programma mag maar één keer open zijn\nMaak een knop met als naam Opslaan\nGeef de knop Opslaan de tip \"Bewaart je werk\"\n";
    assert!(lees(bron).is_goed(), "{:#?}", lees(bron).meldingen);
    let uit = voer_uit(bron, &m);
    for d in ["[schermvullend]", "[vraag bij afsluiten] Wil je echt stoppen?", "[maar één keer open]", "[tip bij Opslaan] Bewaart je werk"] {
        assert!(uit.contains(d), "{} ontbreekt in {}", d, uit);
    }
    assert!(!lees("Maak een venster met als titel A\nGeef de knop Weg de tip \"x\"\n").is_goed());
    // Reservekopieën: na 12 keer blijven er 10.
    let mut b = String::new();
    for _ in 0..12 {
        b.push_str("Maak een reservekopie van het bestand notities.txt\n");
    }
    voer_uit(&b, &m);
    let n = std::fs::read_dir(m.join("Reservekopieën")).unwrap().count();
    assert_eq!(n, 10);
    let _ = std::fs::remove_dir_all(&m);
}
