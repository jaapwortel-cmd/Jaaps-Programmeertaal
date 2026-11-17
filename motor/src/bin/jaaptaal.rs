//! jaaptaal: de motor gebruiken vanaf de opdrachtregel (voor proeven en de bouwstraat).
//!
//!   jaaptaal controleer programma.txt
//!   jaaptaal beschrijf  programma.txt
//!   jaaptaal rust       programma.txt
//!   jaaptaal bouw       programma.txt uit.exe [--snel]
//!   jaaptaal proef      programma.txt [--typ "Naam=Jaap"] [--vink Naam] [--kies "Lijst=Keuze"] [--druk Knopnaam]...
//!   jaaptaal installatie programma.txt "Naam installatie.exe"

use std::path::{Path, PathBuf};
use std::process::{exit, Command};

use motor::bouwen::{self, Stand};

fn uitleg() -> ! {
    eprintln!("Gebruik:");
    eprintln!("  jaaptaal controleer <programma.txt>");
    eprintln!("  jaaptaal beschrijf <programma.txt>");
    eprintln!("  jaaptaal rust <programma.txt>");
    eprintln!("  jaaptaal bouw <programma.txt> <uit.exe> [--snel]");
    eprintln!("  jaaptaal proef <programma.txt> [--druk Knopnaam]...");
    eprintln!("  jaaptaal installatie <programma.txt> <installatie.exe>");
    exit(2)
}

fn lees(pad: &str) -> String {
    motor::bestand::lees_txt(Path::new(pad)).unwrap_or_else(|e| {
        eprintln!("Ik kan het bestand {} niet openen: {}", pad, e);
        exit(2)
    })
}

fn rustc() -> PathBuf {
    let map = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));
    motor::bouwen::zoek_rustc(map.as_deref()).unwrap_or_else(|| {
        eprintln!("Ik kan de Rust-compiler niet vinden. Hij hoort in de map rust naast dit programma.");
        exit(2)
    })
}

fn code_of_stop(bron: &str) -> String {
    match motor::rust_code(bron) {
        Ok(c) => c,
        Err(meldingen) => {
            for m in meldingen {
                println!("{}", m);
            }
            exit(1)
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        uitleg();
    }
    let bron = lees(&args[1]);
    match args[0].as_str() {
        "controleer" => {
            let u = motor::lees(&bron);
            println!("{}", motor::samenvatting(&u));
            for m in &u.meldingen {
                println!("{}", m);
            }
            exit(if u.is_goed() { 0 } else { 1 });
        }
        "beschrijf" => {
            let u = motor::lees(&bron);
            println!("{}", motor::beschrijf(&u));
            for m in &u.meldingen {
                println!("{}", m);
            }
        }
        "rust" => {
            let code = code_of_stop(&bron);
            if args.iter().any(|a| a == "--alles") {
                print!("{}", code);
            } else {
                print!("{}", motor::rustcode::alleen_programma(&code));
            }
        }
        "bouw" => {
            let Some(uit) = args.get(2) else { uitleg() };
            let code = code_of_stop(&bron);
            let stand = if args.iter().any(|a| a == "--snel") { Stand::Snel } else { Stand::Klein };
            let werk = std::env::temp_dir().join("jaaptaal-bouw");
            match bouwen::bouw(&code, Path::new(uit), &rustc(), stand, &werk) {
                Ok(v) => println!(
                    "Klaar: {} ({} bytes, {}), gebouwd in {:.1} seconden.",
                    uit,
                    v.grootte,
                    bouwen::grootte_in_woorden(v.grootte),
                    v.duur.as_secs_f64()
                ),
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1)
                }
            }
        }
        "installatie" => {
            let Some(uit) = args.get(2) else { uitleg() };
            let bronmap = Path::new(&args[1]).parent().map(|p| p.to_path_buf());
            let werk = std::env::temp_dir().join(format!("jaaptaal-installatie-{}", std::process::id()));
            match motor::installatie::bouw(&bron, bronmap.as_deref(), Path::new(uit), &rustc(), &werk) {
                Ok(v) => println!(
                    "Klaar: {} ({} bytes, {}), gebouwd in {:.1} seconden.",
                    uit,
                    v.grootte,
                    bouwen::grootte_in_woorden(v.grootte),
                    v.duur.as_secs_f64()
                ),
                Err(e) => {
                    eprintln!("{}", e);
                    exit(1)
                }
            }
            let _ = std::fs::remove_dir_all(&werk);
        }
        "proef" => {
            let code = code_of_stop(&bron);
            let werk = std::env::temp_dir().join(format!("jaaptaal-proef-{}", std::process::id()));
            let exe = werk.join(if cfg!(windows) { "proef.exe" } else { "proef" });
            if let Err(e) = bouwen::bouw(&code, &exe, &rustc(), Stand::Snel, &werk) {
                eprintln!("{}", e);
                exit(1)
            }
            let mut extra = vec!["--tekst".to_string()];
            let mut i = 2;
            while i < args.len() {
                if matches!(args[i].as_str(), "--druk" | "--typ" | "--vink" | "--kies") && i + 1 < args.len() {
                    extra.push(args[i].clone());
                    extra.push(args[i + 1].clone());
                    i += 1;
                }
                i += 1;
            }
            let uit = Command::new(&exe).args(&extra).output().unwrap_or_else(|e| {
                eprintln!("Ik kan het programma niet starten: {}", e);
                exit(1)
            });
            print!("{}", String::from_utf8_lossy(&uit.stdout));
            let _ = std::fs::remove_dir_all(&werk);
            exit(uit.status.code().unwrap_or(1));
        }
        _ => uitleg(),
    }
}
