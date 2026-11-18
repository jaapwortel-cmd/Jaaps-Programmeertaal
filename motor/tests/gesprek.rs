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
    assert!(r.tekst.contains("Zeg ja, dan maak ik het"), "{}", r.tekst);
    assert!(r.maak.is_none(), "eerst vragen of het klopt");
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
    let bron = r.maak.expect("hij weet precies wat je wilt, dus hij maakt het programma");
    assert!(motor::lees(&bron).is_goed(), "{}", bron);
    assert!(bron.contains("Maak een groot tekstvak met als naam Tekst"), "{}", bron);
    assert!(bron.contains("Maak een knop met als naam Opslaan") && bron.contains("Als ik op de knop Opslaan druk"), "{}", bron);
    assert!(r.tekst.contains("Ik maak het programma nu en start het."), "{}", r.tekst);
    assert_eq!(g.onderdelen.len(), 2);
    // Nog een keer "ga maar verder": opnieuw maken en starten, niet opnieuw beginnen.
    assert!(g.zeg("ga maar verder").maak.is_some());
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
    assert!(r.tekst.starts_with("Nee, nog niet."), "eerlijk: {}", r.tekst);
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
fn editoropdracht_wordt_doorgegeven() {
    let mut g = Gesprek::nieuw();
    let r = g.zeg("maak er een exe van");
    assert_eq!(r.doe, Some(motor::opdrachten::Functie::BouwExe), "{}", r.tekst);
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

#[test]
fn na_ja_maakt_hij_het_programma() {
    let mut g = Gesprek::nieuw();
    g.zeg("Maak een programma met een knop Opslaan en een tekstvak voor mijn verhaal");
    let r = g.zeg("ja");
    let bron = r.maak.expect("programma");
    assert!(motor::lees(&bron).is_goed(), "{}", bron);
    assert!(bron.contains("Maak een groot tekstvak met als naam Verhaal"), "{}", bron);
    assert!(bron.contains("Schrijf de tekst in het tekstvak Verhaal in het bestand met de naam plek"), "{}", bron);
    // Iets erbij: meteen opnieuw maken.
    let r = g.zeg("zet er ook een knop Voorlezen bij");
    let bron = r.maak.expect("opnieuw");
    assert!(bron.contains("Lees de tekst in het tekstvak Verhaal voor"), "{}", bron);
    // Een knop waarvan niet bekend is wat hij doet: wel maken, eerlijk zeggen.
    let r = g.zeg("en een knop Hallo");
    assert!(r.tekst.contains("wat de knop Hallo moet doen"), "{}", r.tekst);
    assert!(motor::lees(r.maak.as_ref().unwrap()).is_goed());
    // Een vraag maakt niets.
    assert!(g.zeg("wat is een label?").maak.is_none());
}

#[test]
fn programmas_die_hij_kan_maken() {
    for gesprek in [
        vec!["Ik wil dat het programma mijn geluidsbestanden kan openen en afspelen.", "Ja, en ik wil ook kunnen stoppen."],
        vec!["Maak een rekenmachine waarmee ik kan optellen", "ja"],
        vec!["Kun je een klein programma maken waar ik tekst in kan typen?", "Zet er ook een knop Opslaan bij.", "en een knop Openen", "ga maar verder"],
        vec!["Maak een programma met een keuzelijst en een selectievakje", "klopt"],
    ] {
        let mut g = Gesprek::nieuw();
        let mut laatste = None;
        for z in &gesprek {
            let r = g.zeg(z);
            if r.maak.is_some() {
                laatste = r.maak;
            }
        }
        let bron = laatste.unwrap_or_else(|| panic!("niets gemaakt: {:?}\n{}", gesprek, g.als_tekst()));
        let u = motor::lees(&bron);
        assert!(u.is_goed(), "{}\n{:#?}", bron, u.meldingen);
    }
    // Pauzeren kan de taal nog niet: eerlijk zeggen.
    let mut g = Gesprek::nieuw();
    g.zeg("Ik wil dat het programma mijn geluidsbestanden kan afspelen en pauzeren.");
    let r = g.zeg("ja");
    assert!(r.tekst.contains("pauzeren"), "{}", r.tekst);
    assert!(r.maak.is_some());
}

#[test]
fn gemaakte_programmas_bouwen_echt() {
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    for (i, gesprek) in [
        vec!["Kun je een klein programma maken waar ik tekst in kan typen?", "Zet er ook een knop Opslaan bij.", "en een knop Openen en een knop Voorlezen", "ga maar verder"],
        vec!["Ik wil dat het programma mijn geluidsbestanden kan openen en afspelen.", "Ja, en ik wil ook kunnen stoppen."],
        vec!["Maak een rekenmachine waarmee ik kan optellen", "ja"],
    ]
    .into_iter()
    .enumerate()
    {
        let mut g = Gesprek::nieuw();
        let mut bron = None;
        for z in &gesprek {
            if let Some(b) = g.zeg(z).maak {
                bron = Some(b);
            }
        }
        let bron = bron.expect("programma");
        let code = motor::rust_code(&bron).unwrap_or_else(|m| panic!("{}\n{:#?}", bron, m));
        let werk = std::env::temp_dir().join(format!("jaaptaal-gesprek-{}-{}", i, std::process::id()));
        std::fs::create_dir_all(&werk).unwrap();
        let exe = werk.join(if cfg!(windows) { "p.exe" } else { "p" });
        motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &werk).unwrap_or_else(|e| panic!("{}\n{:?}", bron, e));
        assert!(exe.exists());
        let _ = std::fs::remove_dir_all(&werk);
    }
}
