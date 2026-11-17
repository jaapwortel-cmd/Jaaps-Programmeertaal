//! Programma's ondertekenen met een eigen certificaat op jouw naam.
//!
//! Dit gebruikt alleen Windows zelf (Windows PowerShell met New-SelfSignedCertificate en
//! Set-AuthenticodeSignature). Er is geen internet nodig en geen extra programma.
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
    format!(
        "$c = Get-ChildItem Cert:\\CurrentUser\\My -CodeSigningCert | Where-Object {{ $_.Subject -eq ('CN=' + {}) -and $_.NotAfter -gt (Get-Date) }} | Select-Object -First 1",
        ps_tekst(naam)
    )
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
