//! Programma's ondertekenen met een eigen certificaat op jouw naam.
//!
//! Dit gebruikt alleen Windows zelf (Windows PowerShell met New-SelfSignedCertificate en
//! Set-AuthenticodeSignature). Er is geen internet nodig en geen extra programma.
//!
//! Heb je al een eigen certificaat (een .pfx-bestand met de geheime sleutel), dan lees je dat één keer in
//! ([`importeer`]). Het komt dan in het certificaatarchief van Windows, alleen voor jou op deze computer.
//! Daarna wordt elk programma en elk installatiebestand daarmee ondertekend, ook zonder de zin Onderteken.
//! De geheime sleutel staat nooit in Jaap's Programmeertaal zelf.
//!
//! Eerlijk: een eigen certificaat bewijst dat het programma van jou komt en niet is veranderd.
//! Op andere computers blijft Windows SmartScreen toch waarschuwen, want die kennen jouw
//! certificaat niet. Alleen een gekocht certificaat haalt die waarschuwing weg.

use std::path::Path;
use std::process::Command;

fn ps_tekst(s: &str) -> String {
    format!("'{}'", s.replace('\'', "''"))
}

fn powershell(script: &str) -> Result<String, String> {
    if !cfg!(windows) {
        return Err("Ondertekenen kan alleen op Windows.".into());
    }
    let mut cmd = Command::new("powershell.exe");
    cmd.args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-Command", script]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
    }
    let uit = cmd.output().map_err(|e| format!("Ik kan Windows PowerShell niet starten: {}", e))?;
    let tekst = String::from_utf8_lossy(&uit.stdout).trim().to_string();
    if !uit.status.success() {
        let fout = String::from_utf8_lossy(&uit.stderr);
        return Err(format!("Ondertekenen is niet gelukt. {}", fout.lines().next().unwrap_or("").trim()));
    }
    Ok(tekst)
}

fn zoek_certificaat(naam: &str) -> String {
    let gewoon = format!(
        "$c = Get-ChildItem Cert:\\CurrentUser\\My -CodeSigningCert | Where-Object {{ $_.Subject -eq ('CN=' + {}) -and $_.NotAfter -gt (Get-Date) -and $_.HasPrivateKey }} | Select-Object -First 1",
        ps_tekst(naam)
    );
    // Het ingelezen eigen certificaat gaat voor (er kan ook nog een zelfgemaakt certificaat met dezelfde naam zijn).
    match eigen() {
        Some((vingerafdruk, eigen_naam)) if eigen_naam.to_lowercase() == naam.to_lowercase() => format!(
            "$c = Get-ChildItem Cert:\\CurrentUser\\My | Where-Object {{ $_.Thumbprint -eq {} -and $_.HasPrivateKey }} | Select-Object -First 1; if (-not $c) {{ {} }}",
            ps_tekst(&vingerafdruk),
            gewoon
        ),
        _ => gewoon,
    }
}

/// Waar onthouden wordt welk eigen certificaat je hebt ingelezen.
fn eigen_bestand() -> std::path::PathBuf {
    std::env::var_os("APPDATA").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir).join("Jaaps Programmeertaal").join("mijn certificaat.txt")
}

/// Het eigen certificaat dat je hebt ingelezen: (vingerafdruk, naam). None als je er geen hebt ingelezen.
pub fn eigen() -> Option<(String, String)> {
    let t = std::fs::read_to_string(eigen_bestand()).ok()?;
    let (v, n) = t.trim().trim_start_matches('\u{FEFF}').split_once('|')?;
    (!v.is_empty() && !n.is_empty()).then(|| (v.trim().to_string(), n.trim().to_string()))
}

/// Base64 lezen (voor een certificaat dat als tekst is bewaard). Regels met ----- worden overgeslagen.
pub fn base64(tekst: &str) -> Option<Vec<u8>> {
    let mut uit = Vec::new();
    let mut buf = 0u32;
    let mut bits = 0;
    for regel in tekst.lines().filter(|r| !r.trim_start().starts_with("-----")) {
        for c in regel.chars() {
            let v = match c {
                'A'..='Z' => c as u32 - 'A' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 26,
                '0'..='9' => c as u32 - '0' as u32 + 52,
                '+' | '-' => 62,
                '/' | '_' => 63,
                '=' | ' ' | '\t' | '\r' | '\u{FEFF}' => continue,
                _ => return None,
            };
            buf = (buf << 6) | v;
            bits += 6;
            if bits >= 8 {
                bits -= 8;
                uit.push((buf >> bits) as u8);
                buf &= (1 << bits) - 1;
            }
        }
    }
    (!uit.is_empty()).then_some(uit)
}

/// Leest je eigen certificaat in: een .pfx- of .p12-bestand, of datzelfde bestand als base64-tekst (.txt).
/// Het komt in het certificaatarchief van Windows (alleen voor jou). Geeft de naam op het certificaat.
pub fn importeer(pad: &Path, wachtwoord: &str) -> Result<String, String> {
    let is_tekst = pad.extension().map_or(false, |e| e.eq_ignore_ascii_case("txt"));
    let tijdelijk = std::env::temp_dir().join(format!("jaaptaal-certificaat-{}.pfx", std::process::id()));
    let bestand = if is_tekst {
        let t = std::fs::read_to_string(pad).map_err(|e| format!("Ik kan het bestand niet lezen: {}", e))?;
        let data = base64(&t).ok_or("In dit tekstbestand staat geen certificaat (base64).")?;
        std::fs::write(&tijdelijk, data).map_err(|e| format!("Ik kan het certificaat niet klaarzetten: {}", e))?;
        tijdelijk.clone()
    } else {
        pad.to_path_buf()
    };
    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         $w = New-Object System.Security.SecureString; foreach ($t in {ww}.ToCharArray()) {{ $w.AppendChar($t) }}; \
         $l = Import-PfxCertificate -FilePath {pad} -CertStoreLocation Cert:\\CurrentUser\\My -Password $w; \
         $c = $l | Where-Object {{ $_.HasPrivateKey }} | Select-Object -First 1; \
         if (-not $c) {{ throw 'In dit bestand zit geen geheime sleutel. Dan kun je er niet mee ondertekenen.' }}; \
         $c.Thumbprint + '|' + $c.GetNameInfo('SimpleName', $false)",
        ww = ps_tekst(wachtwoord),
        pad = ps_tekst(&bestand.to_string_lossy())
    );
    let uit = powershell(&script);
    if is_tekst {
        let _ = std::fs::remove_file(&tijdelijk);
    }
    let uit = uit.map_err(|e| e.replace("Ondertekenen is niet gelukt.", "Inlezen is niet gelukt."))?;
    let regel = uit.lines().last().unwrap_or("").trim().to_string();
    let Some((_, naam)) = regel.split_once('|') else {
        return Err(format!("Inlezen is niet gelukt: {}", regel));
    };
    let map = eigen_bestand();
    if let Some(m) = map.parent() {
        let _ = std::fs::create_dir_all(m);
    }
    std::fs::write(&map, &regel).map_err(|e| format!("Ik kan niet onthouden welk certificaat je gebruikt: {}", e))?;
    Ok(naam.to_string())
}

/// Is er al een eigen certificaat op deze naam (op deze computer, voor deze gebruiker)?
pub fn heeft_certificaat(naam: &str) -> Result<bool, String> {
    let script = format!("{}; if ($c) {{ 'ja' }} else {{ 'nee' }}", zoek_certificaat(naam));
    Ok(powershell(&script)? == "ja")
}

/// Ondertekent een .exe met het certificaat op deze naam. Is dat er nog niet, dan wordt het gemaakt
/// (5 jaar geldig, alleen voor deze gebruiker).
pub fn onderteken(exe: &Path, naam: &str) -> Result<(), String> {
    let script = format!(
        "$ErrorActionPreference = 'Stop'; {zoek}; \
         if (-not $c) {{ $c = New-SelfSignedCertificate -Type CodeSigningCert -Subject ('CN=' + {naam}) -CertStoreLocation Cert:\\CurrentUser\\My -NotAfter (Get-Date).AddYears(5) -FriendlyName ('Jaap''s Programmeertaal: ' + {naam}) }}; \
         $null = Set-AuthenticodeSignature -FilePath {pad} -Certificate $c -HashAlgorithm SHA256; \
         $s = Get-AuthenticodeSignature -FilePath {pad}; \
         if ($s.SignerCertificate -and $s.SignerCertificate.Thumbprint -eq $c.Thumbprint) {{ 'ondertekend' }} else {{ 'niet: ' + $s.StatusMessage }}",
        zoek = zoek_certificaat(naam),
        naam = ps_tekst(naam),
        pad = ps_tekst(&exe.to_string_lossy())
    );
    let uit = powershell(&script)?;
    if uit.ends_with("ondertekend") {
        Ok(())
    } else {
        Err(format!("Ondertekenen is niet gelukt: {}", uit))
    }
}

/// Laat deze computer jouw eigen certificaat vertrouwen. Windows vraagt daarbij zelf om toestemming.
pub fn vertrouw(naam: &str) -> Result<(), String> {
    let script = format!(
        "$ErrorActionPreference = 'Stop'; {zoek}; if (-not $c) {{ throw 'Er is nog geen certificaat op deze naam.' }}; \
         $f = Join-Path $env:TEMP 'jaaptaal-certificaat.cer'; $null = Export-Certificate -Cert $c -FilePath $f; \
         $null = Import-Certificate -FilePath $f -CertStoreLocation Cert:\\CurrentUser\\Root; \
         $null = Import-Certificate -FilePath $f -CertStoreLocation Cert:\\CurrentUser\\TrustedPublisher; \
         Remove-Item $f; 'vertrouwd'",
        zoek = zoek_certificaat(naam)
    );
    powershell(&script).map(|_| ())
}
