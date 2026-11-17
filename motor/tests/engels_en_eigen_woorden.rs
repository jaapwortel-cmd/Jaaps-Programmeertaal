//! Tests voor Engelse opdrachten, omzetten naar het Engels, en eigen woorden (zelflerend).

use motor::{lees, lees_met};

#[test]
fn engelse_zinnen_doen_hetzelfde_als_nederlandse() {
    let en = "Make a window with title Greeting\nMake a button named Welcome\nWhen I press the button Welcome\nShow the text Hello\nEnd of the button\nRemember counter is 1\nAdd 2 to counter\nSubtract 1 from counter\nChange counter to 5\nSet counter to 6\nIf counter is greater than 2 then\nShow \"Big\" and then counter\nElse\nShow \"small\"\nEnd if\nRepeat 2 times\nShow counter times 10\nEnd repeat\nRepeat while counter is less than 10, at most 100 times\nAdd 1 to counter\nEnd repeat\nComment: this does nothing\nBeep\n";
    let u = lees(en);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    let b = motor::beschrijf(&u);
    assert!(b.contains("maak een venster. De titel is: Greeting."));
    assert!(b.contains("tel het getal 2 op bij counter."));
    assert!(b.contains("kijk of wat er in counter staat is groter dan het getal 2"));
}

#[test]
fn engelse_tekst_tussen_aanhalingstekens_blijft_staan() {
    assert_eq!(motor::engels::naar_nederlands("Show \"one times two\" and then x"), Some("Toon \"one times two\" en dan x".into()));
    assert_eq!(motor::engels::naar_nederlands("Show the text Hello times two"), Some("Toon de tekst Hello times two".into()));
    assert_eq!(motor::engels::naar_nederlands("Jump high"), None);
}

#[test]
fn onbekende_engelse_zin_blijft_een_vraag() {
    let u = lees("Jump three times\n");
    assert!(u.meldingen[0].tekst.contains("ken ik niet"));
}

#[test]
fn naar_het_engels_omzetten() {
    let nl = "Maak een venster met als titel Begroeting\nMaak een knop met als naam Welkom\nAls ik op de knop Welkom druk\nToon de tekst Hallo\nEinde van de knop\nOnthoud x is 2\nAls x is groter dan 1\nToon x keer 3\nEinde als\n";
    let u = lees(nl);
    let (en, niet, overgeslagen) = motor::engels::naar_engels(&u.programma);
    assert!(en.contains("Make a window with title Begroeting"));
    assert!(en.contains("When I press the button Welkom\n    Show \"Hallo\"\nEnd of the button"));
    assert!(en.contains("If x is greater than 1\n    Show x times 3\nEnd if"));
    assert!(niet.contains(&"Hallo".to_string()));
    assert!(overgeslagen.is_empty());
    // En terug: de Engelse versie werkt ook.
    assert!(lees(&en).is_goed(), "{}\n{:#?}", en, lees(&en).meldingen);
}

#[test]
fn eigen_woorden() {
    let woorden = motor::bestand::eigen_woorden("# uitleg\nlaat zien = toon\ntoom = toon\n\nfout regel\n");
    assert_eq!(woorden, vec![("laat zien".to_string(), "toon".to_string()), ("toom".to_string(), "toon".to_string())]);
    let bron = "Laat zien 3 plus 4\nToom de tekst Hallo\n";
    assert!(!lees(bron).is_goed());
    let u = lees_met(bron, &woorden);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    let nieuw = motor::bestand::voeg_eigen_woord_toe("laat zien = toon\n", "Kop", "knop");
    assert_eq!(nieuw, "laat zien = toon\nkop = knop\n");
    assert_eq!(motor::bestand::voeg_eigen_woord_toe(&nieuw, "kop", "iets"), nieuw);
}
