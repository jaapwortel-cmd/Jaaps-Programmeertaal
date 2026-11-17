//! Controle: er wordt nergens AI, een taalmodel of een internetverbinding gebruikt.
//! Deze test leest alle broncode en alle Cargo-bestanden van het project.

use std::path::{Path, PathBuf};

fn project() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
}

fn bestanden(map: &Path, uit: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(map).unwrap().flatten() {
        let p = e.path();
        let naam = p.file_name().unwrap().to_string_lossy().to_string();
        if naam == "target" || naam.starts_with('.') {
            continue;
        }
        if p.is_dir() {
            bestanden(&p, uit);
        } else if naam.ends_with(".rs") || naam == "Cargo.toml" || naam == "Cargo.lock" {
            uit.push(p);
        }
    }
}

/// Pakketten die AI, taalmodellen of internet binnenhalen. Geen van deze mag erin zitten.
const VERBODEN_PAKKETTEN: &[&str] = &[
    "openai", "async-openai", "anthropic", "ollama", "ollama-rs", "llama", "llama-cpp", "llm", "candle", "candle-core",
    "tch", "ort", "onnxruntime", "tokenizers", "rust-bert", "whisper-rs", "tract", "burn", "reqwest", "ureq", "hyper",
    "curl", "isahc", "surf", "tokio",
];

/// Woorden in code die op een modelverbinding of internet wijzen.
const VERBODEN_IN_CODE: &[&str] = &[
    "TcpStream", "UdpSocket", "std::net", "api.openai", "api.anthropic", "localhost:11434", "/v1/chat", "/api/generate",
    "huggingface", "gguf",
];

#[test]
fn geen_ai_en_geen_internet() {
    let mut lijst = Vec::new();
    bestanden(&project(), &mut lijst);
    assert!(!lijst.is_empty());
    let deze = Path::new(file!()).file_name().unwrap();
    for pad in lijst {
        if pad.file_name() == Some(deze) {
            continue;
        }
        let inhoud = std::fs::read_to_string(&pad).unwrap();
        let naam = pad.file_name().unwrap().to_string_lossy();
        if naam == "Cargo.toml" || naam == "Cargo.lock" {
            for regel in inhoud.lines() {
                let regel = regel.trim();
                let pakket = regel.split(['=', ' ']).next().unwrap_or("").trim_matches('"');
                let pakket = regel.strip_prefix("name = ").map(|n| n.trim_matches('"')).unwrap_or(pakket);
                assert!(
                    !VERBODEN_PAKKETTEN.contains(&pakket),
                    "{} gebruikt het pakket {}, en dat mag niet (AI of internet).",
                    pad.display(),
                    pakket
                );
            }
        } else {
            for woord in VERBODEN_IN_CODE {
                assert!(!inhoud.contains(woord), "{} bevat {}, en dat mag niet.", pad.display(), woord);
            }
        }
    }
}

#[test]
fn motor_heeft_geen_andere_pakketten() {
    let toml = std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let na = toml.split("[dependencies]").nth(1).unwrap_or("");
    let eerste_blok = na.split("\n[").next().unwrap_or("");
    let pakketten: Vec<&str> =
        eerste_blok.lines().map(|r| r.trim()).filter(|r| !r.is_empty() && !r.starts_with('#')).collect();
    assert!(pakketten.is_empty(), "De motor hoort geen pakketten te gebruiken, maar heeft: {:?}", pakketten);
}

#[test]
fn looptijd_gebruikt_alleen_windows_zelf() {
    let looptijd = motor::rustcode::LOOPTIJD;
    for regel in looptijd.lines() {
        if let Some(rest) = regel.trim().strip_prefix("#[link(name = \"") {
            let lib = rest.split('"').next().unwrap();
            assert!(
                ["user32", "kernel32", "gdi32", "shell32", "winmm", "ole32", "advapi32", "comdlg32"].contains(&lib),
                "Onverwachte bibliotheek: {} (alleen onderdelen van Windows zelf mogen)",
                lib
            );
        }
    }
    assert!(!looptijd.contains("extern crate"));
}
