//! Wat de motor van je zinnen maakt: een boom van opdrachten.
//! Dit is de tussenstap tussen jouw zinnen en de Rust-code.

/// Een vaste waarde: een getal, een tekst, of waar/niet waar.
#[derive(Clone, Debug, PartialEq)]
pub enum Waarde {
    Getal(f64),
    Tekst(String),
    Waarheid(bool),
}

/// Rekenen en tekst aan elkaar plakken.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Teken {
    Plus,
    Min,
    Keer,
    GedeeldDoor,
    /// "en dan": zet twee stukken tekst achter elkaar, met een spatie ertussen.
    EnDan,
}

impl Teken {
    pub fn woord(self) -> &'static str {
        match self {
            Teken::Plus => "plus",
            Teken::Min => "min",
            Teken::Keer => "keer",
            Teken::GedeeldDoor => "gedeeld door",
            Teken::EnDan => "en dan",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Uitdrukking {
    Vast(Waarde),
    /// Een naam van iets dat je hebt onthouden (altijd in kleine letters).
    Naam(String),
    /// Wat er in een onderdeel van het scherm staat (tekstvak, label, selectievakje, keuzelijst).
    /// `als_getal`: de tekst als getal lezen ("het getal in het tekstvak Leeftijd").
    Onderdeel { naam: String, als_getal: bool },
    Bewerking(Box<Uitdrukking>, Teken, Box<Uitdrukking>),
    Negatief(Box<Uitdrukking>),
}

impl Uitdrukking {
    /// Alle namen die in deze uitdrukking worden gebruikt.
    pub fn namen(&self, uit: &mut Vec<String>) {
        match self {
            Uitdrukking::Vast(_) | Uitdrukking::Onderdeel { .. } => {}
            Uitdrukking::Naam(n) => uit.push(n.clone()),
            Uitdrukking::Bewerking(a, _, b) => {
                a.namen(uit);
                b.namen(uit);
            }
            Uitdrukking::Negatief(a) => a.namen(uit),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Vergelijking {
    Gelijk,
    NietGelijk,
    Groter,
    Kleiner,
    Minstens,
    Hoogstens,
}

impl Vergelijking {
    pub fn woorden(self) -> &'static str {
        match self {
            Vergelijking::Gelijk => "is gelijk aan",
            Vergelijking::NietGelijk => "is niet gelijk aan",
            Vergelijking::Groter => "is groter dan",
            Vergelijking::Kleiner => "is kleiner dan",
            Vergelijking::Minstens => "is minstens",
            Vergelijking::Hoogstens => "is hoogstens",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Voorwaarde {
    Vergelijk(Uitdrukking, Vergelijking, Uitdrukking),
    /// Een waarde die zelf waar of niet waar is.
    IsWaar(Uitdrukking),
}

impl Voorwaarde {
    pub fn namen(&self, uit: &mut Vec<String>) {
        match self {
            Voorwaarde::Vergelijk(a, _, b) => {
                a.namen(uit);
                b.namen(uit);
            }
            Voorwaarde::IsWaar(a) => a.namen(uit),
        }
    }
}

/// Eén opdracht, met het regelnummer waar hij staat.
#[derive(Clone, Debug, PartialEq)]
pub struct Opdracht {
    pub regel: usize,
    pub soort: Soort,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Soort {
    Opmerking(String),
    MaakVenster { titel: String },
    MaakKnop { naam: String },
    /// "Als ik op de knop ... druk" tot "Einde van de knop".
    BijKnop { naam: String, blok: Vec<Opdracht>, einde: usize },
    Toon(Uitdrukking),
    /// `bron` is de waarde zoals je hem schreef (voor voorstellen).
    Onthoud { naam: String, waarde: Uitdrukking, bron: String },
    Verander { naam: String, waarde: Uitdrukking },
    TelOp { naam: String, waarde: Uitdrukking },
    TrekAf { naam: String, waarde: Uitdrukking },
    Als { voorwaarde: Voorwaarde, dan: Vec<Opdracht>, anders: Vec<Opdracht>, anders_regel: Option<usize>, einde: usize },
    Herhaal { aantal: Uitdrukking, blok: Vec<Opdracht>, einde: usize },
    HerhaalZolang { voorwaarde: Voorwaarde, hoogstens: Uitdrukking, blok: Vec<Opdracht>, einde: usize },
    /// Een tekstvak, label, selectievakje of keuzelijst in het laatste venster.
    MaakOnderdeel { soort: OnderdeelSoort, naam: String },
    /// Een keuze toevoegen aan een keuzelijst of uitklaplijst.
    VoegKeuzeToe { keuze: Uitdrukking, lijst: String },
    /// Tekst in een tekstvak of label zetten.
    ZetIn { waarde: Uitdrukking, naam: String },
    /// Een selectievakje aan- of uitvinken.
    Vink { naam: String, aan: bool },
    /// De focus op een onderdeel zetten.
    Focus { naam: String },
    /// Een zin die het installatiebestand beschrijft. Doet niets als het programma draait.
    Installatie(InstallatieZin),
}

/// De soorten onderdelen die je in een venster kunt zetten.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OnderdeelSoort {
    Tekstvak,
    GrootTekstvak,
    Label,
    Selectievakje,
    Keuzelijst,
    Uitklaplijst,
}

impl OnderdeelSoort {
    pub fn woord(self) -> &'static str {
        match self {
            OnderdeelSoort::Tekstvak => "tekstvak",
            OnderdeelSoort::GrootTekstvak => "groot tekstvak",
            OnderdeelSoort::Label => "label",
            OnderdeelSoort::Selectievakje => "selectievakje",
            OnderdeelSoort::Keuzelijst => "keuzelijst",
            OnderdeelSoort::Uitklaplijst => "uitklaplijst",
        }
    }
    pub fn nummer(self) -> u8 {
        self as u8
    }
}

/// Wie het programma mag installeren.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rechten {
    /// Alleen voor jezelf (geen beheerder nodig) of voor iedereen (wel beheerder nodig). Je kiest bij het installeren.
    MetEnZonder,
    /// Alleen voor jezelf, zonder beheerdersrechten.
    AlleenZonder,
    /// Voor alle gebruikers, met beheerdersrechten.
    AlleenMet,
}

/// De zinnen waarmee je het installatiebestand beschrijft.
#[derive(Clone, Debug, PartialEq)]
pub enum InstallatieZin {
    Naam(String),
    Versie(String),
    Maker(String),
    Bureaublad,
    Startmenu,
    Rechten(Rechten),
    Bestand(String),
    /// Een regel tekst van de licentie.
    LicentieRegel(String),
    /// De licentie staat in een eigen tekstbestand.
    LicentieBestand(String),
}

/// Een heel programma.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Programma {
    pub opdrachten: Vec<Opdracht>,
}

impl Programma {
    /// Loopt langs alle opdrachten, ook die binnen blokken.
    pub fn alle(&self) -> Vec<&Opdracht> {
        fn loop_langs<'a>(lijst: &'a [Opdracht], uit: &mut Vec<&'a Opdracht>) {
            for o in lijst {
                uit.push(o);
                match &o.soort {
                    Soort::BijKnop { blok, .. } | Soort::Herhaal { blok, .. } | Soort::HerhaalZolang { blok, .. } => {
                        loop_langs(blok, uit)
                    }
                    Soort::Als { dan, anders, .. } => {
                        loop_langs(dan, uit);
                        loop_langs(anders, uit);
                    }
                    _ => {}
                }
            }
        }
        let mut uit = Vec::new();
        loop_langs(&self.opdrachten, &mut uit);
        uit
    }

    /// Maakt dit programma vensters? Dan is het een programma met een scherm.
    pub fn heeft_vensters(&self) -> bool {
        self.alle().iter().any(|o| matches!(o.soort, Soort::MaakVenster { .. }))
    }
}
