//! Stap 5: met de ingebouwde Rust-compiler een echt .exe-bestand maken.
//!
//! De Rust-compiler zit in de map `rust` naast het programma. Je hoeft hem
//! niet apart te installeren.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Hoe het programma gebouwd wordt.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Stand {
    /// Snel bouwen om meteen uit te proberen.
    Snel,
    /// Klein en snel programma, voor een .exe die je bewaart of weggeeft.
    Klein,
}

#[derive(Debug)]
pub struct Verslag {
    pub duur: Duration,
    pub grootte: u64,
}

fn exe_naam() -> &'static str {
    if cfg!(windows) {
        "rustc.exe"
    } else {
        "rustc"
    }
}

/// Zoekt de Rust-compiler. Eerst in de map `rust` naast het programma,
/// dan via JAAPTAAL_RUSTC, en als laatste een rustc die al op de computer staat.
pub fn zoek_rustc(programmamap: Option<&Path>) -> Option<PathBuf> {
    if let Some(map) = programmamap {
        let pad = map.join("rust").join("bin").join(exe_naam());
        if pad.is_file() {
            return Some(pad);
        }
    }
    if let Ok(p) = std::env::var("JAAPTAAL_RUSTC") {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Some(p);
        }
    }
    let lukt = Command::new("rustc").arg("--version").output().map(|o| o.status.success()).unwrap_or(false);
    lukt.then(|| PathBuf::from("rustc"))
}

/// Is dit de ingebouwde compiler (in een map `rust`)? Dan linkt hij met zijn eigen hulpmiddelen.
fn is_ingebouwd(rustc: &Path) -> bool {
    rustc.parent().and_then(|p| p.parent()).and_then(|p| p.file_name()).map_or(false, |n| n == "rust")
}

/// Bouwt Rust-code tot een programma. `werkmap` is een map voor tijdelijke bestanden.
pub fn bouw(code: &str, uit: &Path, rustc: &Path, stand: Stand, werkmap: &Path) -> Result<Verslag, String> {
    let begin = Instant::now();
    // Een pad zonder map hoort bij de map waar je nu bent, niet bij de werkmap.
    let uit_volledig;
    let uit = if uit.is_absolute() {
        uit
    } else {
        uit_volledig = std::env::current_dir().map(|d| d.join(uit)).unwrap_or_else(|_| uit.to_path_buf());
        uit_volledig.as_path()
    };
    std::fs::create_dir_all(werkmap).map_err(|e| format!("Ik kan de werkmap {} niet maken: {}", werkmap.display(), e))?;
    // Sneller: is er niets veranderd sinds de vorige keer? Dan het vorige programma gebruiken.
    let vingerafdruk = format!("{:016x}-{:?}-{}", vingerafdruk(code), stand, rustc.display());
    let bewaard = werkmap.join("vorige-uitkomst.bin");
    let afdruk_bestand = werkmap.join("vorige-uitkomst.txt");
    if std::fs::read_to_string(&afdruk_bestand).ok().as_deref() == Some(vingerafdruk.as_str()) && bewaard.is_file() {
        if uit.is_file() && std::fs::metadata(uit).map(|m| m.len()).ok() == std::fs::metadata(&bewaard).map(|m| m.len()).ok() {
            let grootte = std::fs::metadata(uit).map(|m| m.len()).unwrap_or(0);
            return Ok(Verslag { duur: begin.elapsed(), grootte });
        }
        if std::fs::copy(&bewaard, uit).is_ok() {
            let grootte = std::fs::metadata(uit).map(|m| m.len()).unwrap_or(0);
            return Ok(Verslag { duur: begin.elapsed(), grootte });
        }
    }
    let _ = std::fs::remove_file(&afdruk_bestand);
    let bron = werkmap.join("programma.rs");
    std::fs::write(&bron, code).map_err(|e| format!("Ik kan {} niet schrijven: {}", bron.display(), e))?;
    let _ = std::fs::remove_file(uit);

    let mut cmd = Command::new(rustc);
    cmd.arg("--edition=2021").arg("--crate-name=programma").arg("-o").arg(uit).arg(&bron);
    match stand {
        Stand::Snel => {
            cmd.args(["-C", "opt-level=0", "-C", "debuginfo=0", "-C", "strip=symbols"]);
        }
        Stand::Klein => {
            cmd.args(["-C", "opt-level=s", "-C", "panic=abort", "-C", "strip=symbols", "-C", "codegen-units=1"]);
        }
    }
    if cfg!(windows) && is_ingebouwd(rustc) {
        cmd.args(["-C", "link-self-contained=yes"]);
    }
    cmd.current_dir(werkmap);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // geen zwart venster laten zien
    }
    let uitkomst = cmd.output().map_err(|e| format!("Ik kan de Rust-compiler niet starten ({}): {}", rustc.display(), e))?;
    if !uitkomst.status.success() {
        let fouten = String::from_utf8_lossy(&uitkomst.stderr);
        if code.contains("// Rust-stukje: begin van de eigen Rust-code") {
            return Err(format!(
                "Het bouwen is mislukt. Je programma heeft een Rust-stukje: kijk eerst daar, want de fout zit waarschijnlijk in die eigen Rust-code. Wat de Rust-compiler zei:\n{}",
                fouten
            ));
        }
        return Err(format!(
            "Het bouwen is mislukt. Dit is een fout in Jaap's Programmeertaal zelf, niet in jouw zinnen. Wat de Rust-compiler zei:\n{}",
            fouten
        ));
    }
    let grootte = std::fs::metadata(uit).map(|m| m.len()).unwrap_or(0);
    if std::fs::copy(uit, &bewaard).is_ok() {
        let _ = std::fs::write(&afdruk_bestand, &vingerafdruk);
    }
    Ok(Verslag { duur: begin.elapsed(), grootte })
}

/// Een vingerafdruk van de code (FNV-1a, 64 bits): hetzelfde bij dezelfde code.
fn vingerafdruk(code: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in code.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// Een grootte in gewone woorden, bijvoorbeeld "312 kilobyte".
pub fn grootte_in_woorden(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} megabyte", bytes as f64 / (1024.0 * 1024.0)).replace('.', ",")
    } else {
        format!("{} kilobyte", (bytes + 1023) / 1024)
    }
}
