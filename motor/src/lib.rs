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
pub mod eigen_zinnen;
pub mod engels;
pub mod getallen;
pub mod installatie;
pub mod jaaps_zinnen;
pub mod melding;
pub mod motoren;
pub mod ondertekenen;
pub mod opdrachten;
mod ontleden;
pub mod rustcode;
pub mod samengesteld;
pub mod scherm;
pub mod taal;
pub mod tekst;
pub mod website;
pub mod wensen;

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
    lees_eigen(bron, eigen_woorden, "")
}

/// Leest een programma met jouw eigen woorden én je eigen zinnen uit een bibliotheek
/// (de tekst van "Mijn eigen zinnen", met regels zoals: Nieuwe zin: Begroet [wie] ...).
pub fn lees_eigen(bron: &str, eigen_woorden: &[(String, String)], bibliotheek: &str) -> Uitslag {
    if website::is_website(bron) {
        let (_, meldingen) = website::lees(bron);
        return Uitslag { programma: taal::Programma { opdrachten: Vec::new() }, meldingen };
    }
    let regels: Vec<&str> = bron.lines().collect();
    let (programma, mut meldingen) = ontleden::lees_programma_eigen(bron, eigen_woorden, bibliotheek);
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
    rust_code_eigen(bron, eigen_woorden, "")
}

/// Rust-code maken, met jouw eigen woorden en je eigen zinnen uit een bibliotheek.
pub fn rust_code_eigen(bron: &str, eigen_woorden: &[(String, String)], bibliotheek: &str) -> Result<String, Vec<Melding>> {
    let uitslag = lees_eigen(bron, eigen_woorden, bibliotheek);
    if !uitslag.is_goed() {
        return Err(uitslag.meldingen);
    }
    let regels: Vec<&str> = bron.lines().collect();
    Ok(rustcode::maak(&uitslag.programma, &regels))
}

const ZINNEN: &str = "JAAPTAAL-ZINNEN";

/// Het begin van je zinnen in een gebouwd programma. (In stukjes gemaakt, zodat dit
/// programma zelf het teken niet bevat.)
pub fn zinnen_begin() -> String {
    format!("<<{}-BEGIN>>\n", ZINNEN)
}

pub fn zinnen_einde() -> String {
    format!("\n<<{}-EINDE>>", ZINNEN)
}

/// Haalt je zinnen uit een programma (.exe) dat met Jaap's Programmeertaal is gebouwd.
pub fn zinnen_uit_programma(inhoud: &[u8]) -> Option<String> {
    let begin = zinnen_begin().into_bytes();
    let einde = zinnen_einde().into_bytes();
    let b = inhoud.windows(begin.len()).position(|x| x == begin.as_slice())? + begin.len();
    let e = inhoud[b..].windows(einde.len()).position(|x| x == einde.as_slice())? + b;
    String::from_utf8(inhoud[b..e].to_vec()).ok()
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
