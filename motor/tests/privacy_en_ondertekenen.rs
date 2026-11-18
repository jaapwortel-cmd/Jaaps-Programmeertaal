//! Tests voor de privacyverklaring, ondertekenen en bestanden van elders.

use motor::installatie;
use motor::lees;

const PROGRAMMA: &str = "Maak een venster met als titel A\nMaak een knop met als naam B\n";

#[test]
fn privacyverklaring_beschrijven() {
    let bron = format!(
        "{}Maak een installatiebestand met als naam A\nDe maker is Jaap Wortel\nIn de privacyverklaring staat: Dit programma verstuurt geen gegevens.\nIn de privacyverklaring staat: Er wordt niets bewaard\n",
        PROGRAMMA
    );
    let u = lees(&bron);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    let i = installatie::uit_programma(&u.programma).unwrap();
    assert_eq!(
        installatie::privacytekst(&i, None).unwrap(),
        "Privacyverklaring van A. Gemaakt door Jaap Wortel.\n\nDit programma verstuurt geen gegevens.\nEr wordt niets bewaard."
    );
    let code = motor::rust_code(&bron).unwrap();
    assert!(code.contains("const PRIVACY: &str = \"Dit programma verstuurt geen gegevens.\\nEr wordt niets bewaard.\";"));
}

#[test]
fn privacy_twee_keer_beschreven() {
    let u = lees(&format!(
        "{}Maak een installatiebestand met als naam A\nIn de privacyverklaring staat: x\nDe privacyverklaring staat in het bestand P.txt\n",
        PROGRAMMA
    ));
    assert!(u.meldingen.iter().any(|m| m.tekst.contains("Welke van de twee")));
}

#[test]
fn ondertekenen_zin() {
    let bron = format!("{}Maak een installatiebestand met als naam A\nOnderteken het programma met mijn eigen certificaat op naam van Jaap Wortel\n", PROGRAMMA);
    let u = lees(&bron);
    assert!(u.is_goed(), "{:#?}", u.meldingen);
    assert_eq!(installatie::uit_programma(&u.programma).unwrap().onderteken.as_deref(), Some("Jaap Wortel"));
}

#[test]
fn bestand_van_elders_herkennen() {
    assert_eq!(motor::bestand::hoort_bij("Dit is code voor het programma Rekenhulp\nToon 1\n"), Some((true, "Rekenhulp".into())));
    assert_eq!(motor::bestand::hoort_bij("\n  Dit is een omschrijving voor het programma Mijn radio.\nIk wil..."), Some((false, "Mijn radio".into())));
    assert_eq!(motor::bestand::hoort_bij("Toon 1\n"), None);
    // De zin zelf mag ook in een programma staan; hij doet dan niets.
    assert!(lees("Dit is code voor het programma Rekenhulp\nToon 1\n").is_goed());
}

/// Echt ondertekenen kan alleen op Windows (in de bouwstraat).
#[cfg(windows)]
#[test]
fn echt_ondertekenen_met_eigen_certificaat() {
    let rustc = motor::bouwen::zoek_rustc(None).expect("rustc nodig");
    let map = std::env::temp_dir().join(format!("jaaptaal-teken-{}", std::process::id()));
    let exe = map.join("getekend.exe");
    let code = motor::rust_code("Toon 1\n").unwrap();
    motor::bouwen::bouw(&code, &exe, &rustc, motor::bouwen::Stand::Snel, &map.join("werk")).unwrap();
    let naam = format!("Jaaptaal Proef {}", std::process::id());
    motor::ondertekenen::onderteken(&exe, &naam).expect("ondertekenen");
    assert!(motor::ondertekenen::heeft_certificaat(&naam).unwrap());
    // Opruimen: het proefcertificaat weer weghalen.
    let _ = std::process::Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &format!("Get-ChildItem Cert:\\CurrentUser\\My | Where-Object {{ $_.Subject -eq 'CN={}' }} | Remove-Item", naam)])
        .status();
    let _ = std::fs::remove_dir_all(&map);
}

#[test]
fn certificaat_als_tekst_lezen() {
    assert_eq!(motor::ondertekenen::base64("aGFsbG8=").unwrap(), b"hallo");
    assert_eq!(motor::ondertekenen::base64("-----BEGIN-----\r\naGFs\r\nbG8gamFhcA==\r\n-----END-----\r\n").unwrap(), b"hallo jaap");
    assert!(motor::ondertekenen::base64("geen base64!").is_none());
}
