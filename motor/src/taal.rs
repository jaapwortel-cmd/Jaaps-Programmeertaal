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
    /// Een nieuwe, lege lijst.
    LegeLijst,
    /// Hoeveel dingen er in een lijst staan.
    Aantal(Box<Uitdrukking>),
    /// Eén ding uit een lijst: nummer 1 is het eerste, -1 is het laatste.
    Element(Box<Uitdrukking>, Box<Uitdrukking>),
    /// De uitkomst van een eigen stukje.
    Aanroep { naam: String, invoer: Vec<Uitdrukking> },
    /// Iets dat het programma zelf weet of kan: een bestand lezen, de tijd, een willekeurig getal.
    Ingebouwd { wat: Ingebouwd, invoer: Vec<Uitdrukking> },
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
            Uitdrukking::Negatief(a) | Uitdrukking::Aantal(a) => a.namen(uit),
            Uitdrukking::Element(a, b) => {
                a.namen(uit);
                b.namen(uit);
            }
            Uitdrukking::LegeLijst => {}
            Uitdrukking::Aanroep { invoer, .. } | Uitdrukking::Ingebouwd { invoer, .. } => {
                for i in invoer {
                    i.namen(uit);
                }
            }
        }
    }

    /// Alle aanroepen van eigen stukjes in deze uitdrukking: (naam, aantal invoer).
    pub fn aanroepen(&self, uit: &mut Vec<(String, usize)>) {
        match self {
            Uitdrukking::Aanroep { naam, invoer } => {
                uit.push((naam.clone(), invoer.len()));
                for i in invoer {
                    i.aanroepen(uit);
                }
            }
            Uitdrukking::Bewerking(a, _, b) | Uitdrukking::Element(a, b) => {
                a.aanroepen(uit);
                b.aanroepen(uit);
            }
            Uitdrukking::Negatief(a) | Uitdrukking::Aantal(a) => a.aanroepen(uit),
            Uitdrukking::Ingebouwd { invoer, .. } => {
                for i in invoer {
                    i.aanroepen(uit);
                }
            }
            _ => {}
        }
    }

    /// Verandert een naam van een eigen stukje in een aanroep zonder invoer.
    pub fn verbind_stukjes(&mut self, stukjes: &[String]) {
        match self {
            Uitdrukking::Naam(n) if stukjes.contains(n) => {
                *self = Uitdrukking::Aanroep { naam: n.clone(), invoer: Vec::new() };
            }
            Uitdrukking::Bewerking(a, _, b) | Uitdrukking::Element(a, b) => {
                a.verbind_stukjes(stukjes);
                b.verbind_stukjes(stukjes);
            }
            Uitdrukking::Negatief(a) | Uitdrukking::Aantal(a) => a.verbind_stukjes(stukjes),
            Uitdrukking::Aanroep { invoer, .. } | Uitdrukking::Ingebouwd { invoer, .. } => {
                for i in invoer {
                    i.verbind_stukjes(stukjes);
                }
            }
            _ => {}
        }
    }
}

/// Wat het programma zelf weet of kan.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ingebouwd {
    /// De tekst in een bestand.
    Inhoud,
    /// De regels van een bestand, als lijst.
    Regels,
    /// Bestaat het bestand (of de map)?
    Bestaat,
    /// Waar het logboek staat.
    LogboekPlek,
    /// Een willekeurig heel getal van ... tot en met ...
    Willekeurig,
    /// De tijd nu, zoals 14:05.
    Tijd,
    /// De datum van vandaag, zoals 17-11-2026.
    Datum,
    /// Wat er op een webpagina staat (zoals het is, met HTML).
    Webpagina,
    /// Alleen de leesbare tekst van een webpagina (zonder HTML).
    WebTekst,
    /// De gekozen rij van een tabel (een lijst).
    GekozenRij,
    /// Hoeveel rijen er in een tabel staan.
    AantalRijen,
    /// Een rij van een tabel, met een nummer (een lijst).
    RijNummer,
    /// De wortel van een getal.
    Wortel,
    /// Een getal tot de macht een ander getal.
    Macht,
    /// Een getal afgerond (op een aantal cijfers achter de komma).
    Afgerond,
    /// Wat overblijft bij het delen.
    Rest,
    /// Een getal zonder min ervoor.
    Absoluut,
    /// Tekst omzetten naar een getal.
    AlsGetal,
    /// Iets omzetten naar tekst.
    AlsTekst,
    /// Hoeveel tekens een tekst heeft.
    Lengte,
    /// Tekst in hoofdletters.
    Hoofdletters,
    /// Tekst in kleine letters.
    KleineLetters,
    /// De eerste tekens van een tekst.
    EersteTekens,
    /// De laatste tekens van een tekst.
    LaatsteTekens,
    /// De woorden van een tekst, als lijst.
    Woorden,
    /// Een tekst in stukken gehakt bij een teken, als lijst.
    Gesplitst,
    /// Een lijst aan elkaar geplakt tot een tekst.
    Samengevoegd,
    /// Een tekst waarin iets vervangen is.
    Vervangen,
    /// Tekst zonder spaties aan het begin en eind.
    ZonderSpaties,
    /// De tekst uit een Word- of PDF-bestand.
    DocumentTekst,
    /// De namen van alle radiozenders die het programma kent (een lijst).
    Radiozenders,
    /// De radiozenders van een land, van internet (radio-browser.info).
    RadiozendersLand,
    /// Het streamadres van een radiozender.
    ZenderAdres,
    /// Een bestand dat je kiest in het gewone Openen-venster van Windows.
    KiesBestand,
    /// Een plek om op te slaan, gekozen in het gewone Opslaan als-venster.
    KiesOpslaan,
}

impl Ingebouwd {
    pub fn nummer(self) -> u8 {
        self as u8
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
    /// Staat iets in een lijst?
    Bevat(Uitdrukking, Uitdrukking),
    /// Allebei waar ("en").
    En(Box<Voorwaarde>, Box<Voorwaarde>),
    /// Minstens één van de twee waar ("of").
    Of(Box<Voorwaarde>, Box<Voorwaarde>),
}

impl Voorwaarde {
    pub fn namen(&self, uit: &mut Vec<String>) {
        match self {
            Voorwaarde::Vergelijk(a, _, b) => {
                a.namen(uit);
                b.namen(uit);
            }
            Voorwaarde::IsWaar(a) => a.namen(uit),
            Voorwaarde::Bevat(a, b) => {
                a.namen(uit);
                b.namen(uit);
            }
            Voorwaarde::En(a, b) | Voorwaarde::Of(a, b) => {
                a.namen(uit);
                b.namen(uit);
            }
        }
    }

    pub fn uitdrukkingen_mut(&mut self) -> Vec<&mut Uitdrukking> {
        match self {
            Voorwaarde::Vergelijk(a, _, b) | Voorwaarde::Bevat(a, b) => vec![a, b],
            Voorwaarde::IsWaar(a) => vec![a],
            Voorwaarde::En(a, b) | Voorwaarde::Of(a, b) => {
                let mut v = a.uitdrukkingen_mut();
                v.extend(b.uitdrukkingen_mut());
                v
            }
        }
    }

    pub fn uitdrukkingen(&self) -> Vec<&Uitdrukking> {
        match self {
            Voorwaarde::Vergelijk(a, _, b) | Voorwaarde::Bevat(a, b) => vec![a, b],
            Voorwaarde::IsWaar(a) => vec![a],
            Voorwaarde::En(a, b) | Voorwaarde::Of(a, b) => {
                let mut v = a.uitdrukkingen();
                v.extend(b.uitdrukkingen());
                v
            }
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
    /// Iets aan een lijst toevoegen.
    VoegToeAanLijst { waarde: Uitdrukking, lijst: String },
    /// Iets uit een lijst weghalen (de eerste keer dat het erin staat).
    HaalUitLijst { waarde: Uitdrukking, lijst: String },
    /// "Herhaal voor elk ding in de lijst boodschappen".
    VoorElk { naam: String, lijst: Uitdrukking, blok: Vec<Opdracht>, einde: usize },
    /// "Herhaal voor elk getal van 1 tot 10".
    Tel { naam: String, van: Uitdrukking, tot: Uitdrukking, blok: Vec<Opdracht>, einde: usize },
    /// Een eigen stukje met een naam, invoer en misschien een uitkomst.
    /// `rust`: bij een Rust-stukje de eigen Rust-code (dan is `blok` leeg).
    Stukje { naam: String, invoer: Vec<String>, blok: Vec<Opdracht>, einde: usize, rust: Option<String> },
    /// "Doe Verdubbel met 5": een eigen stukje uitvoeren.
    Doe(Uitdrukking),
    /// "Geef als uitkomst ...": het stukje stopt en geeft dit terug.
    GeefUitkomst(Uitdrukking),
    /// Tekst in een bestand schrijven (of eraan toevoegen).
    SchrijfBestand { waarde: Uitdrukking, pad: Uitdrukking, toevoegen: bool },
    /// Een map maken.
    MaakMap(Uitdrukking),
    /// Tekst naar een internetadres versturen.
    Verstuur { waarde: Uitdrukking, adres: Uitdrukking },
    /// Een bestand van internet downloaden.
    Download { adres: Uitdrukking, pad: Uitdrukking },
    /// Kijken of er een nieuwe versie is, en die (na een vraag) installeren.
    ZoekUpdates,
    /// Beginnen met opnemen van de microfoon.
    BeginOpname,
    /// Een aantal seconden wachten.
    Wacht(Uitdrukking),
    /// Een vak waarin je in gewone zinnen zegt wat het programma moet doen.
    Opdrachtvak,
    /// Hoe het venster zich gedraagt: schermvullend, vraag bij afsluiten, maar één keer open, tip bij een knop.
    Venstergedrag(Gedrag),
    /// Een keuzelijst vullen met alles uit een lijst.
    VulKeuzelijst { lijst: String, waarde: Uitdrukking },
    /// Een internetradiozender spelen (naam, adres), of stoppen (None).
    Radio(Option<(String, Uitdrukking)>),
    /// Een reservekopie van een bestand (de laatste 10 blijven bewaard).
    Reservekopie(Uitdrukking),
    /// Een QR-code als PNG-afbeelding.
    QrCode { waarde: Uitdrukking, pad: Uitdrukking },
    /// Een werkbalk: knoppen naast elkaar bovenaan het venster.
    Werkbalk(Vec<String>),
    /// Een ander programma starten.
    StartProgramma(Uitdrukking),
    /// Een bestand of map openen met het programma dat daarbij hoort.
    OpenBestand(Uitdrukking),
    /// Stoppen met opnemen en de opname bewaren als WAV-bestand.
    StopOpname(Uitdrukking),
    /// Een afbeelding in het laatste venster, met een beschrijving voor de schermlezer.
    MaakAfbeelding { pad: String, beschrijving: String },
    /// Iets met een tabel: kolom toevoegen, rij toevoegen, gekozen rij weghalen, leegmaken, sorteren,
    /// opslaan als CSV of inlezen uit CSV.
    Tabel { tabel: String, actie: TabelActie },
    /// Tekst in een Word-bestand (0) of PDF-bestand (1) schrijven.
    Document { soort: u8, waarde: Uitdrukking, pad: Uitdrukking },
    /// Een regel in het logboek schrijven.
    Logboek(Uitdrukking),
    /// Een piep: hoogte in hertz, duur in milliseconden.
    Piep { hoogte: Uitdrukking, duur: Uitdrukking },
    /// Een WAV-bestand afspelen.
    /// `richting`: 0 gewoon; anders links/rechts + 3 × (achter/voor) + 9 × (boven/onder), zie `richting_naam`.
    SpeelGeluid { pad: Uitdrukking, wachten: bool, richting: u8 },
    /// Het geluid stoppen.
    StopGeluid,
    /// De schermlezer iets laten zeggen, zonder meldingsvenster.
    Zeg(Uitdrukking),
    /// Voorlezen met de stem van Windows (los van de schermlezer).
    LeesVoor(Uitdrukking),
    /// Stoppen met voorlezen.
    StopVoorlezen,
    /// Het volume van geluid en voorlezen, van 0 tot 100.
    Volume(Uitdrukking),
    /// Een tabblad in het laatste venster. Wat je daarna maakt, komt op dit tabblad.
    MaakTabblad { naam: String },
    /// Naar een tabblad gaan.
    KiesTabblad { naam: String },
    /// Een menu in de menubalk van het laatste venster.
    MaakMenu { naam: String },
    /// Een regel in het laatste menu. Wat hij doet, schrijf je zoals bij een knop.
    MaakMenuregel { naam: String },
    /// Een bestand of map kopiëren (0), verplaatsen (1) of naar de Prullenbak gooien (2).
    BestandActie { soort: u8, van: Uitdrukking, naar: Option<Uitdrukking> },
    /// Het pictogram van het programma (een .ico-bestand).
    Pictogram(String),
    /// Een map inpakken in een zipbestand, of een zipbestand uitpakken in een map.
    Zip { inpakken: bool, map: Uitdrukking, zip: Uitdrukking },
    /// Een sneltoets voor een knop, zoals Ctrl+W.
    Sneltoets { knop: String, toets: Toets },
    /// Een kleur voor het laatste venster: de achtergrond of de tekst.
    Kleur { achtergrond: bool, kleur: String, rgb: (u8, u8, u8) },
    /// Een regel van de handleiding van je programma (F1), of het bestand waar hij in staat.
    Handleiding { regel: Option<String>, bestand: Option<String> },
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

/// Een toets met Ctrl, Alt of Shift erbij.
#[derive(Clone, Debug, PartialEq)]
pub struct Toets {
    /// De Windows-code van de toets (zoals 0x57 voor W, 0x74 voor F5).
    pub code: u16,
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    /// Hoe de toets heet, zoals Ctrl+W.
    pub naam: String,
}

/// De kleuren die je bij naam kunt noemen.
pub const KLEUREN: &[(&str, (u8, u8, u8))] = &[
    ("zwart", (0, 0, 0)),
    ("wit", (255, 255, 255)),
    ("rood", (200, 0, 0)),
    ("donkerrood", (128, 0, 0)),
    ("lichtrood", (255, 120, 120)),
    ("groen", (0, 128, 0)),
    ("donkergroen", (0, 80, 0)),
    ("lichtgroen", (144, 238, 144)),
    ("blauw", (0, 70, 200)),
    ("donkerblauw", (0, 0, 110)),
    ("lichtblauw", (173, 216, 230)),
    ("geel", (255, 230, 0)),
    ("lichtgeel", (255, 255, 200)),
    ("oranje", (255, 140, 0)),
    ("paars", (110, 30, 160)),
    ("roze", (255, 182, 193)),
    ("bruin", (120, 70, 20)),
    ("grijs", (128, 128, 128)),
    ("lichtgrijs", (220, 220, 220)),
    ("donkergrijs", (64, 64, 64)),
];

/// Hoe goed tekst te lezen is op een achtergrond (de WCAG-regel: 4,5 of meer is goed).
pub fn contrast(a: (u8, u8, u8), b: (u8, u8, u8)) -> f64 {
    fn kanaal(c: u8) -> f64 {
        let c = c as f64 / 255.0;
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    fn licht(k: (u8, u8, u8)) -> f64 {
        0.2126 * kanaal(k.0) + 0.7152 * kanaal(k.1) + 0.0722 * kanaal(k.2)
    }
    let (x, y) = (licht(a), licht(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[derive(Clone, Debug, PartialEq)]
pub enum TabelActie {
    Kolom(String),
    Rij(Vec<Uitdrukking>),
    HaalGekozenWeg,
    Leeg,
    Sorteer(String),
    Opslaan(Uitdrukking),
    Inlezen(Uitdrukking),
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
    Tabel,
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
            OnderdeelSoort::Tabel => "tabel",
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
    /// Een regel van de privacyverklaring.
    PrivacyRegel(String),
    /// De privacyverklaring staat in een eigen tekstbestand.
    PrivacyBestand(String),
    /// Onderteken het programma met een eigen certificaat op deze naam.
    Onderteken(String),
    /// Waar het versiebestand voor updates staat (een internetadres).
    UpdateAdres(String),
}

impl Soort {
    /// Alle uitdrukkingen in deze opdracht (niet in de blokken eronder).
    pub fn uitdrukkingen_mut(&mut self) -> Vec<&mut Uitdrukking> {
        match self {
            Soort::Toon(u) | Soort::Zeg(u) | Soort::LeesVoor(u) | Soort::Volume(u) | Soort::Doe(u) | Soort::GeefUitkomst(u) | Soort::MaakMap(u) | Soort::Logboek(u) => vec![u],
            Soort::Wacht(u) | Soort::StartProgramma(u) | Soort::OpenBestand(u) | Soort::Reservekopie(u) | Soort::Radio(Some((_, u))) => vec![u],
            Soort::VulKeuzelijst { waarde, .. } => vec![waarde],
            Soort::SchrijfBestand { waarde, pad, .. } | Soort::Document { waarde, pad, .. } | Soort::QrCode { waarde, pad } => vec![waarde, pad],
            Soort::Verstuur { waarde, adres } => vec![waarde, adres],
            Soort::StopOpname(p) => vec![p],
            Soort::Tabel { actie: TabelActie::Rij(w), .. } => w.iter_mut().collect(),
            Soort::Tabel { actie: TabelActie::Opslaan(p) | TabelActie::Inlezen(p), .. } => vec![p],
            Soort::Download { adres, pad } => vec![adres, pad],
            Soort::Piep { hoogte, duur } => vec![hoogte, duur],
            Soort::SpeelGeluid { pad, .. } => vec![pad],
            Soort::Zip { map, zip, .. } => vec![map, zip],
            Soort::BestandActie { van, naar, .. } => {
                let mut v = vec![van];
                if let Some(n) = naar {
                    v.push(n);
                }
                v
            }
            Soort::Onthoud { waarde, .. }
            | Soort::Verander { waarde, .. }
            | Soort::TelOp { waarde, .. }
            | Soort::TrekAf { waarde, .. }
            | Soort::VoegToeAanLijst { waarde, .. }
            | Soort::HaalUitLijst { waarde, .. }
            | Soort::ZetIn { waarde, .. } => vec![waarde],
            Soort::VoegKeuzeToe { keuze, .. } => vec![keuze],
            Soort::Als { voorwaarde, .. } => voorwaarde.uitdrukkingen_mut(),
            Soort::Herhaal { aantal, .. } => vec![aantal],
            Soort::HerhaalZolang { voorwaarde, hoogstens, .. } => {
                let mut v = voorwaarde.uitdrukkingen_mut();
                v.push(hoogstens);
                v
            }
            Soort::VoorElk { lijst, .. } => vec![lijst],
            Soort::Tel { van, tot, .. } => vec![van, tot],
            _ => Vec::new(),
        }
    }

    /// Alle uitdrukkingen in deze opdracht (alleen lezen).
    pub fn uitdrukkingen(&self) -> Vec<&Uitdrukking> {
        match self {
            Soort::Toon(u) | Soort::Zeg(u) | Soort::LeesVoor(u) | Soort::Volume(u) | Soort::Doe(u) | Soort::GeefUitkomst(u) | Soort::MaakMap(u) | Soort::Logboek(u) => vec![u],
            Soort::Wacht(u) | Soort::StartProgramma(u) | Soort::OpenBestand(u) | Soort::Reservekopie(u) | Soort::Radio(Some((_, u))) => vec![u],
            Soort::VulKeuzelijst { waarde, .. } => vec![waarde],
            Soort::SchrijfBestand { waarde, pad, .. } | Soort::Document { waarde, pad, .. } | Soort::QrCode { waarde, pad } => vec![waarde, pad],
            Soort::Verstuur { waarde, adres } => vec![waarde, adres],
            Soort::StopOpname(p) => vec![p],
            Soort::Tabel { actie: TabelActie::Rij(w), .. } => w.iter().collect(),
            Soort::Tabel { actie: TabelActie::Opslaan(p) | TabelActie::Inlezen(p), .. } => vec![p],
            Soort::Download { adres, pad } => vec![adres, pad],
            Soort::Piep { hoogte, duur } => vec![hoogte, duur],
            Soort::SpeelGeluid { pad, .. } => vec![pad],
            Soort::Zip { map, zip, .. } => vec![map, zip],
            Soort::BestandActie { van, naar, .. } => {
                let mut v = vec![van];
                if let Some(n) = naar {
                    v.push(n);
                }
                v
            }
            Soort::Onthoud { waarde, .. }
            | Soort::Verander { waarde, .. }
            | Soort::TelOp { waarde, .. }
            | Soort::TrekAf { waarde, .. }
            | Soort::VoegToeAanLijst { waarde, .. }
            | Soort::HaalUitLijst { waarde, .. }
            | Soort::ZetIn { waarde, .. } => vec![waarde],
            Soort::VoegKeuzeToe { keuze, .. } => vec![keuze],
            Soort::Als { voorwaarde, .. } => voorwaarde.uitdrukkingen(),
            Soort::Herhaal { aantal, .. } => vec![aantal],
            Soort::HerhaalZolang { voorwaarde, hoogstens, .. } => {
                let mut v = voorwaarde.uitdrukkingen();
                v.push(hoogstens);
                v
            }
            Soort::VoorElk { lijst, .. } => vec![lijst],
            Soort::Tel { van, tot, .. } => vec![van, tot],
            _ => Vec::new(),
        }
    }

    /// De blokken onder deze opdracht.
    pub fn blokken_mut(&mut self) -> Vec<&mut Vec<Opdracht>> {
        match self {
            Soort::BijKnop { blok, .. }
            | Soort::Herhaal { blok, .. }
            | Soort::HerhaalZolang { blok, .. }
            | Soort::VoorElk { blok, .. }
            | Soort::Tel { blok, .. }
            | Soort::Stukje { blok, .. } => vec![blok],
            Soort::Als { dan, anders, .. } => vec![dan, anders],
            _ => Vec::new(),
        }
    }
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
                    Soort::BijKnop { blok, .. }
                    | Soort::Herhaal { blok, .. }
                    | Soort::HerhaalZolang { blok, .. }
                    | Soort::VoorElk { blok, .. }
                    | Soort::Tel { blok, .. }
                    | Soort::Stukje { blok, .. } => loop_langs(blok, uit),
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

/// Radiozenders die je bij naam kunt noemen: (naam, adres, andere namen). Overgenomen uit Computerhulp Maatje
/// (Radio.cs); de NPO-zenders zijn op 17 november 2026 getest. Een zender die hier niet staat, zoekt je
/// programma op bij radio-browser.info (zoals Jaaps Audio Editor).
pub const ZENDERS: &[(&str, &str, &str)] = &[
    ("NPO Radio 1", "https://icecast.omroep.nl/radio1-bb-mp3", "npo radio 1, radio 1, radio een, radio één, npo 1"),
    ("NPO Radio 2", "https://icecast.omroep.nl/radio2-bb-mp3", "npo radio 2, radio 2, radio twee, npo 2"),
    ("NPO 3FM", "https://icecast.omroep.nl/3fm-bb-mp3", "npo 3fm, 3fm, 3 fm, drie fm, radio 3, radio drie"),
    ("NPO Klassiek", "https://icecast.omroep.nl/radio4-bb-mp3", "npo klassiek, radio 4, radio vier, npo radio 4, klassiek"),
    ("NPO Radio 5", "https://icecast.omroep.nl/radio5-bb-mp3", "npo radio 5, radio 5, radio vijf"),
    ("FunX", "https://icecast.omroep.nl/funx-bb-mp3", "funx, fun x"),
    ("Radio 538", "https://22533.live.streamtheworld.com/RADIO538.mp3", "radio 538, 538, vijf drie acht, vijfhonderdachtendertig"),
    ("Qmusic", "https://icecast-qmusicnl-cdp.triple-it.nl/Qmusic_nl_live.mp3", "qmusic, q music, kju music, q-music"),
    ("Sky Radio", "https://19993.live.streamtheworld.com/SKYRADIO.mp3", "sky radio, skyradio, sky"),
    ("Radio 10", "https://19993.live.streamtheworld.com/RADIO10.mp3", "radio 10, radio tien"),
    ("Radio Veronica", "https://19993.live.streamtheworld.com/VERONICA.mp3", "radio veronica, veronica"),
    ("100% NL", "https://stream.100p.nl/100pctnl.mp3", "100 nl, 100 procent nl, honderd procent nl, 100 procent nederlands"),
    ("SLAM!", "https://stream.slam.nl/slam_mp3", "slam, slam fm"),
    ("BNR Nieuwsradio", "https://stream.bnr.nl/bnr_mp3_128_20", "bnr, bnr nieuwsradio, nieuwsradio"),
    ("KINK", "https://playerservices.streamtheworld.com/api/livestream-redirect/KINK.mp3", "kink, kink fm"),
    ("Sublime", "https://playerservices.streamtheworld.com/api/livestream-redirect/SUBLIME.mp3", "sublime, sublime fm"),
    ("Classic NL", "https://stream.classic.nl/classicnl.mp3", "classic nl, classic n l"),
    ("Radio 1 (Vlaanderen)", "https://icecast.vrtcdn.be/radio1-high.mp3", "radio 1 vlaanderen, vrt radio 1, belgische radio 1"),
    ("Radio 2 (Vlaanderen)", "https://icecast.vrtcdn.be/ra2ant-high.mp3", "radio 2 vlaanderen, vrt radio 2, belgische radio 2"),
    ("Studio Brussel", "https://icecast.vrtcdn.be/stubru-high.mp3", "studio brussel, stubru"),
    ("MNM", "https://icecast.vrtcdn.be/mnm-high.mp3", "mnm, m n m"),
    ("Klara", "https://icecast.vrtcdn.be/klara-high.mp3", "klara"),
    ("Radio Duivenstraat", "https://server-28.stream-server.nl:18816/stream", "radio duivenstraat, duivenstraat, duivenstraat radio, radio duivestraat, duivestraat"),
    ("NPO 3FM Alternative", "https://icecast.omroep.nl/3fm-alternative-mp3", "npo 3fm alternative, 3fm alternative, 3 fm alternative"),
    ("NPO Radio 2 Sterren NL", "https://icecast.omroep.nl/radio2-sterrennl-mp3", "npo radio 2 sterren nl, radio 2 sterren nl, sterren nl, sterren"),
    ("538 Hitzone", "https://playerservices.streamtheworld.com/api/livestream-redirect/TLPSTR11.mp3", "538 hitzone, hitzone"),
    ("538 Non-Stop", "https://playerservices.streamtheworld.com/api/livestream-redirect/TLPSTR09.mp3", "538 non stop, 538 nonstop"),
    ("538 Top 50", "https://playerservices.streamtheworld.com/api/livestream-redirect/TLPSTR13.mp3", "538 top 50, top 50"),
    ("Qmusic Non-Stop", "https://icecast-qmusicnl-cdp.triple-it.nl/Qmusic_nl_nonstop_96.mp3", "qmusic non stop, qmusic nonstop, q music non stop"),
    ("Qmusic Foute Uur", "https://icecast-qmusicnl-cdp.triple-it.nl/Qmusic_nl_fouteuur.mp3", "qmusic foute uur, foute uur, q music foute uur"),
    ("Joe", "https://stream.joe.nl/joe/mp3", "joe, joe fm, radio joe"),
    ("OOG Radio", "http://opml.radiotime.com/Tune.ashx?id=s8895&formats=mp3,aac", "oog radio, oog, radio oog"),
    ("Zwaluw Radio", "tunein:zwaluw radio", "zwaluw radio, zwaluw, radio zwaluw, de zwaluw, zwaluwradio, swallow radio, swallow"),
    ("Arrow Bluesbox Radio", "https://playerservices.streamtheworld.com/api/livestream-redirect/ARROWBLUESBOX.mp3", "arrow bluesbox radio, arrow bluesbox, bluesbox, arrow blues"),
    ("Capital London", "https://media-ice.musicradio.com/CapitalMP3", "capital london, capital fm, capital"),
    ("Capital XTRA", "https://media-ice.musicradio.com/CapitalXTRALondonMP3", "capital xtra, capital extra"),
    ("Heart London", "https://media-ice.musicradio.com/HeartLondonMP3", "heart london"),
    ("Heart UK", "https://media-ice.musicradio.com/HeartUKMP3", "heart uk, heart"),
    ("Smooth UK", "https://media-ice.musicradio.com/SmoothUKMP3", "smooth uk, smooth radio, smooth"),
    ("Classic FM", "https://media-ice.musicradio.com/ClassicFMMP3", "classic fm, classicfm"),
    ("Classic FM Calm", "https://media-ice.musicradio.com/ClassicFMCalmMP3", "classic fm calm"),
    ("Classic FM Movies", "https://media-ice.musicradio.com/ClassicFMMoviesMP3", "classic fm movies"),
    ("LBC London", "https://media-ice.musicradio.com/LBCLondonMP3", "lbc london, lbc"),
    ("LBC News", "https://media-ice.musicradio.com/LBCNewsLondonMP3", "lbc news"),
    ("Radio X UK", "https://media-ice.musicradio.com/RadioXUKMP3", "radio x uk, radio x"),
    ("Big Top 40", "https://media-ice.musicradio.com/BigTop40V1MP3", "big top 40, top 40"),
    ("Gold", "https://media-ice.musicradio.com/GoldMP3", "gold radio, gold"),
];

fn schoon(t: &str) -> String {
    let t: String = t.to_lowercase().chars().map(|c| if c.is_alphanumeric() || c == '%' { c } else { ' ' }).collect();
    t.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Zoekt een zender bij naam ("radio 2", "NPO Radio 2", "3FM", "qmusic").
pub fn zender(naam: &str) -> Option<(&'static str, &'static str)> {
    let n = schoon(naam);
    ZENDERS
        .iter()
        .find(|(z, _, a)| schoon(z) == n || a.split(',').any(|x| schoon(x) == n))
        .map(|(z, u, _)| (*z, *u))
}

#[derive(Clone, Debug, PartialEq)]
pub enum Gedrag {
    Schermvullend,
    VraagBijAfsluiten(String),
    MaarEenKeerOpen,
    Tip { knop: String, tekst: String },
}

/// Waar een geluid klinkt, in woorden. 0 = gewoon. Links/rechts (1, 2), achter/voor (×3), boven/onder (×9).
pub fn richting_naam(r: u8) -> String {
    if r == 0 {
        return String::new();
    }
    let (h, d, v) = (r % 3, (r / 3) % 3, r / 9);
    let mut delen = Vec::new();
    match h {
        1 => delen.push("links"),
        2 => delen.push("rechts"),
        _ => {}
    }
    match d {
        1 => delen.push("achter je"),
        2 => delen.push("voor je"),
        _ => {}
    }
    match v {
        1 => delen.push("boven je"),
        2 => delen.push("onder je"),
        _ => {}
    }
    if delen.is_empty() {
        delen.push("in het midden");
    }
    delen.join(", ")
}
