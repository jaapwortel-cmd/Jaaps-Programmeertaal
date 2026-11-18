//! Proeven voor het gesprek (eerste ontwikkelstap 0.2): de vijf testvoorbeelden uit het ontwerp,
//! en de regels uit het ontwerp (vraag/idee/uitleg/opdracht, één vraag tegelijk, niets uitvoeren).

use motor::gesprek::{Gesprek, Soort};

fn een_vraag(tekst: &str) {
    assert!(tekst.matches('?').count() <= 1, "meer dan één vraag tegelijk: {}", tekst);
}

#[test]
fn test_1_gewone_spreektaal() {
    let mut g = Gesprek::nieuw();
    let r = g.zeg("Kun je een klein programma maken waar ik tekst in kan typen?");
    assert_eq!(r.soort, Soort::Opdracht, "{}", r.tekst);
    assert!(r.tekst.contains("een klein programma met een tekstvak"), "{}", r.tekst);
    assert!(r.tekst.contains("Ik maak nog niets"), "{}", r.tekst);
    een_vraag(&r.tekst);
}

#[test]
fn test_2_aanvulling() {
    let mut g = Gesprek::nieuw();
    g.zeg("Kun je een klein programma maken waar ik tekst in kan typen?");
    let r = g.zeg("Zet er ook een knop Opslaan bij.");
    assert!(r.tekst.contains("een knop Opslaan"), "{}", r.tekst);
    assert!(r.tekst.contains("een klein programma met een tekstvak om tekst in te typen en een knop Opslaan"), "{}", r.tekst);
    assert!(!r.tekst.contains('?'), "vraagt opnieuw: {}", r.tekst);
}

#[test]
fn test_3_dicteerfout() {
    // Met context: knop in dezelfde zin. Dan "tekstvak", en het zegt dat erbij.
    let mut g = Gesprek::nieuw();
    let r = g.zeg("Maak een knop Opslaan en een tekstpak voor mijn verhaal.");
    assert!(r.tekst.contains("Ik lees \"tekstpak\" als \"tekstvak\""), "{}", r.tekst);
    assert!(r.tekst.contains("een tekstvak voor je verhaal"), "{}", r.tekst);
    assert!(r.tekst.contains("een knop Opslaan"), "{}", r.tekst);
    // Zonder context: vriendelijk navragen.
    let mut g = Gesprek::nieuw();
    let r = g.zeg("Maak een tekstpak");
    assert!(r.tekst.contains("Bedoel je tekstvak?"), "{}", r.tekst);
    let r = g.zeg("ja");
    assert!(r.tekst.contains("tekstvak"), "{}", r.tekst);
    // In een lopend gesprek: het tekstvak krijgt een doel, geen tweede knop.
    let mut g = Gesprek::nieuw();
    g.zeg("Kun je een klein programma maken waar ik tekst in kan typen?");
    g.zeg("Zet er ook een knop Opslaan bij.");
    let r = g.zeg("Maak een knop Opslaan en een tekstpak voor mijn verhaal.");
    assert!(r.tekst.contains("tekstvak voor je verhaal"), "{}", r.tekst);
    assert_eq!(g.onderdelen.len(), 2, "{:?}", g.onderdelen);
}

#[test]
fn test_4_onduidelijke_opdracht() {
    let mut g = Gesprek::nieuw();
    g.zeg("Maak een programma met een knop Opslaan en een tekstvak voor mijn verhaal");
    g.zeg("ja");
    let r = g.zeg("Maak dat anders.");
    assert!(r.tekst.starts_with("Welk onderdeel wil je anders hebben?"), "{}", r.tekst);
    assert!(g.wijzigingen.is_empty());
    een_vraag(&r.tekst);
    let r = g.zeg("de knop");
    assert!(r.tekst.contains("Wat wil je anders aan de knop Opslaan?"), "{}", r.tekst);
    let r = g.zeg("groter");
    assert!(r.tekst.contains("de knop Opslaan groter"), "{}", r.tekst);
    // Ook zonder gesprek: niets doen, maar vragen.
    let mut g = Gesprek::nieuw();
    let r = g.zeg("Maak dat anders.");
    assert!(r.tekst.starts_with("Wat wil je anders hebben?"), "{}", r.tekst);
}

#[test]
fn test_5_verdergaan() {
    let mut g = Gesprek::nieuw();
    g.zeg("Kun je een klein programma maken waar ik tekst in kan typen?");
    g.zeg("Zet er ook een knop Opslaan bij.");
    let r = g.zeg("Ga maar verder.");
    assert!(r.tekst.contains("we gaan verder vanaf wat we hebben afgesproken"), "{}", r.tekst);
    assert!(r.tekst.contains("tekstvak") && r.tekst.contains("knop Opslaan"), "begint opnieuw: {}", r.tekst);
    assert!(r.tekst.contains("nog niet"), "{}", r.tekst);
    assert_eq!(g.onderdelen.len(), 2);
    // Zonder afspraak: eerlijk zeggen.
    let mut g = Gesprek::nieuw();
    let r = g.zeg("ga maar verder");
    assert!(r.tekst.contains("Ik weet niet waar we gebleven waren"), "{}", r.tekst);
}

#[test]
fn voorbeeld_uit_het_ontwerp_geluid() {
    let mut g = Gesprek::nieuw();
    let r = g.zeg("Ik wil dat het programma mijn geluidsbestanden kan openen en afspelen.");
    assert!(r.tekst.contains("een programma voor: geluidsbestanden openen en afspelen"), "{}", r.tekst);
    assert!(r.tekst.contains("Klopt dat?"), "{}", r.tekst);
    let r = g.zeg("Ja, en ik wil ook kunnen pauzeren.");
    assert!(r.tekst.contains("Dan hoort pauzeren er ook bij."), "{}", r.tekst);
    assert!(r.tekst.contains("Ik begrijp je opdracht nu als: een programma voor: geluidsbestanden openen, afspelen en pauzeren."), "{}", r.tekst);
    assert!(g.bevestigd);
}

#[test]
fn vraag_idee_uitleg_en_opdracht() {
    let mut g = Gesprek::nieuw();
    assert_eq!(g.zeg("Wat is een tekstvak?").soort, Soort::Vraag);
    assert_eq!(g.zeg("Misschien is een donkere stand later wel handig.").soort, Soort::Idee);
    assert_eq!(g.zeg("nee").soort, Soort::Antwoord);
    assert!(g.onderdelen.is_empty() && g.ideeen.len() == 1, "een idee wordt niet uitgevoerd");
    let r = g.zeg("Ik ben blind en ik gebruik JAWS");
    assert_eq!(r.soort, Soort::Uitleg, "{}", r.tekst);
    assert!(r.tekst.contains("je bent blind"), "{}", r.tekst);
    assert_eq!(g.zeg("Maak een programma met een knop Stoppen").soort, Soort::Opdracht);
    let r = g.zeg("Heb je het al gemaakt?");
    assert!(r.tekst.starts_with("Nee."), "eerlijk: {}", r.tekst);
}

#[test]
fn verwijzingen_en_veranderen() {
    let mut g = Gesprek::nieuw();
    g.zeg("Maak een programma met een knop Opslaan en een tekstvak");
    g.zeg("ja");
    let r = g.zeg("maak die knop groter");
    assert!(r.tekst.contains("Dan wordt de knop Opslaan groter."), "{}", r.tekst);
    let r = g.zeg("en het tekstvak blauw");
    assert!(r.tekst.contains("het tekstvak blauw"), "{}", r.tekst);
    let r = g.zeg("maak het toch kleiner");
    assert!(r.tekst.contains("kleiner"), "{}", r.tekst);
    let r = g.zeg("haal de knop weg");
    assert!(r.tekst.contains("Dan hoort de knop Opslaan er niet meer bij."), "{}", r.tekst);
    assert_eq!(g.onderdelen.len(), 1);
}

#[test]
fn niet_raden_en_nieuw_programma() {
    let mut g = Gesprek::nieuw();
    let r = g.zeg("blabla fiets wolk");
    assert_eq!(r.soort, Soort::Onduidelijk);
    assert!(r.tekst.contains("Ik wil niet raden"), "{}", r.tekst);
    g.zeg("Maak een programma met een knop Opslaan");
    let r = g.zeg("Maak een rekenmachine");
    assert!(r.tekst.starts_with("Bedoel je een nieuw programma"), "{}", r.tekst);
    let r = g.zeg("nieuw");
    assert!(g.onderdelen.is_empty(), "{:?}", g.onderdelen);
    assert!(r.tekst.contains("rekenmachine"), "{}", r.tekst);
}

#[test]
fn editoropdracht_wordt_niet_uitgevoerd() {
    let mut g = Gesprek::nieuw();
    let r = g.zeg("maak er een exe van");
    assert!(r.tekst.contains("Dat voer ik hier niet uit"), "{}", r.tekst);
    assert!(r.tekst.contains("F8"), "{}", r.tekst);
}

#[test]
fn het_gesprek_staat_in_de_geschiedenis() {
    let mut g = Gesprek::nieuw();
    g.zeg("Maak een programma met een knop Opslaan");
    let t = g.als_tekst();
    assert!(t.starts_with("Jij: Maak een programma"), "{}", t);
    assert!(t.contains("Assistent: Je wilt dus"), "{}", t);
    let r = g.zeg("waar waren we?");
    assert!(r.tekst.contains("Ik begrijp je opdracht nu als"), "{}", r.tekst);
}
