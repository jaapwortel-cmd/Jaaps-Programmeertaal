//! # De motor van Jaap's Programmeertaal
//!
//! De motor leest Nederlandse zinnen, controleert ze en maakt er Rust-code van.
//! Hij gebruikt geen AI en geen taalmodel: alleen vaste regels die hier zijn ingebouwd.
//! De motor heeft geen scherm, zodat hij ook in andere programma's (zoals Bouwmaatje)
//! kan worden gebruikt.
//!
//! De stappen:
//! 1. [`lees`]: de zinnen ontleden en controleren. Dit geeft een [`Uitslag`] met meldingen.
//! 2. [`beschrijf`]: in gewone taal vertellen wat er van de zinnen is gemaakt.
//! 3. [`rust_code`]: de Rust-code maken.
//! 4. [`bouwen::bouw`]: met de Rust-compiler een .exe maken.

pub mod beschrijf;
pub mod bestand;
pub mod bouwen;
mod controle;
pub mod engels;
pub mod getallen;
pub mod installatie;
pub mod melding;
pub mod ondertekenen;
mod ontleden;
pub mod rustcode;
pub mod scherm;
pub mod taal;
pub mod tekst;

pub use melding::{pas_toe, Melding, Soort as MeldingSoort, Voorstel, Wijziging};
pub use taal::Programma;

/// De versie van de taal.
pub const VERSIE: &str = env!("CARGO_PKG_VERSION");

/// Wat er uit het lezen komt: het programma en de meldingen.
#[derive(Debug, Clone)]
pub struct Uitslag {
    pub programma: Programma,
    /// Fouten en vragen, op volgorde van regelnummer. Leeg = alles is goed.
    pub meldingen: Vec<Melding>,
}

impl Uitslag {
    pub fn is_goed(&self) -> bool {
        self.meldingen.is_empty()
    }
}

/// Leest en controleert een programma.
pub fn lees(bron: &str) -> Uitslag {
    lees_met(bron, &[])
}

/// Leest en controleert een programma, met jouw eigen woorden (zoals "laat zien" = "toon").
pub fn lees_met(bron: &str, eigen_woorden: &[(String, String)]) -> Uitslag {
    let regels: Vec<&str> = bron.lines().collect();
    let (programma, mut meldingen) = ontleden::lees_programma_met(bron, eigen_woorden);
    if meldingen.is_empty() {
        meldingen = controle::controleer(&programma, &regels);
    }
    meldingen.sort_by_key(|m| m.regel);
    Uitslag { programma, meldingen }
}

/// Vertelt in gewone taal wat de motor van het programma heeft gemaakt.
pub fn beschrijf(uitslag: &Uitslag) -> String {
    beschrijf::beschrijf(&uitslag.programma)
}

/// Maakt de Rust-code van een programma. Lukt alleen als er geen meldingen meer zijn.
pub fn rust_code(bron: &str) -> Result<String, Vec<Melding>> {
    rust_code_met(bron, &[])
}

/// Rust-code maken, met jouw eigen woorden.
pub fn rust_code_met(bron: &str, eigen_woorden: &[(String, String)]) -> Result<String, Vec<Melding>> {
    let uitslag = lees_met(bron, eigen_woorden);
    if !uitslag.is_goed() {
        return Err(uitslag.meldingen);
    }
    let regels: Vec<&str> = bron.lines().collect();
    Ok(rustcode::maak(&uitslag.programma, &regels))
}

/// Een korte samenvatting om voor te lezen.
pub fn samenvatting(uitslag: &Uitslag) -> String {
    let aantal = uitslag.meldingen.len();
    match aantal {
        0 => "Alles is goed. Ik heb geen fouten gevonden.".to_string(),
        1 => "Ik heb 1 ding gevonden dat ik je moet vragen of dat niet klopt.".to_string(),
        n => format!("Ik heb {} dingen gevonden die ik je moet vragen of die niet kloppen.", n),
    }
}
