//! Tests voor het installatiebestand en de licentie.

use motor::installatie;
use motor::taal::Rechten;
use motor::{lees, pas_toe, MeldingSoort};

const PROGRAMMA: &str = "Maak een venster met als titel Begroeting\nMaak een knop met als naam Welkom\nAls ik op de knop Welkom druk\nToon de tekst Hallo\nEinde van de knop\n";

fn met(extra: &str) -> String {
    format!("{}{}", PROGRAMMA, extra)
}

#[test]
fn installatiezinnen_worden_herkend() {
    let bron = met("Maak een installatiebestand met als naam Begroeting\nDe versie is 1.0\nDe maker is Jaap Wortel\nZet een snelkoppeling in het startmenu\nZet een snelkoppeling op het bureaublad\nDe installatie mag alleen zonder beheerdersrechten\nVoeg het bestand Uitleg.txt toe aan de installatie\nIn de licentie staat: Je mag dit programma gratis gebruiken.\nIn de licentie staat: Niet verkopen\n");
    let u = lees(&bron);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    let i = installatie::uit_programma(&u.programma).expect("installatie");
    assert_eq!(i.naam, "Begroeting");
    assert_eq!(i.versie, "1.0");
    assert_eq!(i.maker, "Jaap Wortel");
    assert!(i.startmenu && i.bureaublad);
    assert_eq!(i.rechten, Rechten::AlleenZonder);
    assert_eq!(i.bestanden, vec!["Uitleg.txt".to_string()]);
    assert_eq!(i.licentie_regels, vec!["Je mag dit programma gratis gebruiken.".to_string(), "Niet verkopen.".to_string()]);
    let lic = installatie::licentietekst(&i, None).unwrap();
    assert!(lic.starts_with("Licentie van Begroeting, versie 1.0. Gemaakt door Jaap Wortel."));
}

#[test]
fn rechten_zinnen() {
    for (zin, r) in [
        ("De installatie mag met en zonder beheerdersrechten", Rechten::MetEnZonder),
        ("De installatie mag alleen zonder beheerdersrechten", Rechten::AlleenZonder),
        ("De installatie mag alleen met beheerdersrechten", Rechten::AlleenMet),
        ("De installatie heeft beheerdersrechten nodig", Rechten::AlleenMet),
    ] {
        let u = lees(&met(&format!("Maak een installatiebestand met als naam B\n{}\n", zin)));
        assert!(u.is_goed(), "{}: {:#?}", zin, u.meldingen);
        assert_eq!(installatie::uit_programma(&u.programma).unwrap().rechten, r, "{}", zin);
    }
}

#[test]
fn installatiezinnen_doen_niets_bij_uitvoeren() {
    let bron = met("Maak een installatiebestand met als naam Begroeting\nZet een snelkoppeling in het startmenu\n");
    let code = motor::rust_code(&bron).unwrap();
    assert!(code.contains("// Regel 6: Maak een installatiebestand met als naam Begroeting"));
    assert!(!code.contains("s.installatie"));
}

#[test]
fn installatie_zonder_naam_krijgt_voorstel() {
    let bron = met("Zet een snelkoppeling in het startmenu\n");
    let u = lees(&bron);
    let m = &u.meldingen[0];
    assert_eq!(m.regel, 6);
    assert_eq!(m.soort, MeldingSoort::Vraag);
    let nieuw = pas_toe(&bron, &m.voorstel.as_ref().unwrap().wijziging);
    assert!(lees(&nieuw).is_goed());
}

#[test]
fn tegenstrijdige_rechten() {
    let u = lees(&met("Maak een installatiebestand met als naam B\nDe installatie mag alleen zonder beheerdersrechten\nDe installatie mag alleen met beheerdersrechten\n"));
    assert_eq!(u.meldingen[0].regel, 8);
    assert!(u.meldingen[0].tekst.contains("tegen"));
}

#[test]
fn licentie_twee_keer_beschreven() {
    let u = lees(&met("Maak een installatiebestand met als naam B\nIn de licentie staat: Gratis.\nDe licentie staat in het bestand Licentie.txt\n"));
    assert!(!u.is_goed());
    assert!(u.meldingen[0].tekst.contains("Welke van de twee"));
}

#[test]
fn twee_namen_spreken_tegen() {
    let u = lees(&met("Maak een installatiebestand met als naam A\nMaak een installatiebestand met als naam B\n"));
    assert_eq!(u.meldingen[0].regel, 7);
}

#[test]
fn installatie_niet_binnen_een_blok() {
    let u = lees("Herhaal 2 keer\nZet een snelkoppeling in het startmenu\nEinde herhaal\n");
    assert!(u.meldingen.iter().any(|m| m.regel == 2 && m.tekst.contains("los staan")));
}

#[test]
fn zet_blijft_ook_veranderen() {
    assert!(lees("Onthoud x is 1\nZet x op 2\n").is_goed());
}

#[test]
fn licentie_uit_een_bestand() {
    let map = std::env::temp_dir().join(format!("jaaptaal-lic-{}", std::process::id()));
    std::fs::create_dir_all(&map).unwrap();
    std::fs::write(map.join("Licentie.txt"), "Alleen voor thuis.\r\n").unwrap();
    let u = lees(&met("Maak een installatiebestand met als naam B\nDe licentie staat in het bestand Licentie.txt\n"));
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    let i = installatie::uit_programma(&u.programma).unwrap();
    assert_eq!(installatie::licentietekst(&i, Some(&map)).unwrap(), "Licentie van B.\n\nAlleen voor thuis.");
    assert!(installatie::licentietekst(&i, None).is_err());
    std::fs::remove_dir_all(&map).unwrap();
}

#[test]
fn bestandsnaam_zonder_verboden_tekens() {
    assert_eq!(installatie::bestandsnaam("Mijn: programma?"), "Mijn programma");
    assert_eq!(installatie::bestandsnaam("  "), "Mijn programma");
}

/// Bouwt het installatiebestand echt. Op Windows: ook stil installeren, kijken en verwijderen.
#[test]
fn installatiebestand_bouwen_en_installeren() {
    let Some(rustc) = motor::bouwen::zoek_rustc(None) else {
        eprintln!("Geen rustc: deze test is overgeslagen.");
        return;
    };
    let map = std::env::temp_dir().join(format!("jaaptaal-inst-{}", std::process::id()));
    std::fs::create_dir_all(&map).unwrap();
    std::fs::write(map.join("Uitleg.txt"), "Druk op Welkom.\r\n").unwrap();
    let bron = met("Maak een installatiebestand met als naam Proef Begroeting\nDe versie is 0.9\nZet een snelkoppeling in het startmenu\nVoeg het bestand Uitleg.txt toe aan de installatie\nIn de licentie staat: Alleen om te testen.\n");
    let uit = map.join(if cfg!(windows) { "Proef installatie.exe" } else { "proef-installatie" });
    let v = installatie::bouw(&bron, Some(&map), &uit, &rustc, &map.join("werk")).expect("bouwen");
    assert!(v.grootte > 0);

    #[cfg(windows)]
    {
        let doel = map.join("geinstalleerd");
        let s = std::process::Command::new(&uit)
            .args(["--stil", "--voor-mij", "--map"])
            .arg(&doel)
            .status()
            .unwrap();
        assert!(s.success());
        for f in ["Proef Begroeting.exe", "Uitleg.txt", "Licentie.txt", "Verwijderen.exe"] {
            assert!(doel.join(f).is_file(), "{} ontbreekt", f);
        }
        let uitvoer = std::process::Command::new(doel.join("Proef Begroeting.exe")).args(["--tekst", "--druk", "Welkom"]).output().unwrap();
        assert!(String::from_utf8_lossy(&uitvoer.stdout).contains("Hallo"));
        let s = std::process::Command::new(doel.join("Verwijderen.exe")).args(["--verwijder", "--stil"]).status().unwrap();
        assert!(s.success());
        for _ in 0..40 {
            if !doel.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
        assert!(!doel.exists(), "De programmamap is na verwijderen niet weg.");
    }
    let _ = std::fs::remove_dir_all(&map);
}
