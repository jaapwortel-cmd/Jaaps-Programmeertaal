//! Meldingen: fouten en vragen, altijd met een regelnummer.
//! Een melding kan één voorstel hebben. Een voorstel verandert je tekst pas
//! als jij er "ja" op zegt (zie [`pas_toe`]).

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Soort {
    /// Iets klopt niet. Je moet het zelf aanpassen.
    Fout,
    /// Ik weet niet wat je bedoelt. Ik vraag het je, en ik gok niet.
    Vraag,
}

/// Wat er met je tekst gebeurt als je "ja" zegt op een voorstel.
#[derive(Clone, Debug, PartialEq)]
pub enum Wijziging {
    /// Regel vervangen door een nieuwe regel.
    VervangRegel { regel: usize, nieuw: String },
    /// Een nieuwe regel invoegen vóór deze regel. Een regel na de laatste regel = aan het eind.
    VoegInVoor { regel: usize, nieuw: String },
    /// Een regel weghalen.
    HaalWeg { regel: usize },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Voorstel {
    /// De vraag die je met ja of nee beantwoordt.
    pub vraag: String,
    pub wijziging: Wijziging,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Melding {
    pub regel: usize,
    pub soort: Soort,
    pub tekst: String,
    pub voorstel: Option<Voorstel>,
    /// Een opdracht voor de editor die na "ja" wordt uitgevoerd (zoals "BouwExe", of "Openen|Begroeting").
    pub opdracht: Option<String>,
}

impl Melding {
    pub fn fout(regel: usize, tekst: impl Into<String>) -> Self {
        Melding { regel, soort: Soort::Fout, tekst: tekst.into(), voorstel: None, opdracht: None }
    }

    pub fn vraag(regel: usize, tekst: impl Into<String>) -> Self {
        Melding { regel, soort: Soort::Vraag, tekst: tekst.into(), voorstel: None, opdracht: None }
    }

    pub fn met_opdracht(mut self, opdracht: impl Into<String>) -> Self {
        self.opdracht = Some(opdracht.into());
        self
    }

    pub fn met_voorstel(mut self, vraag: impl Into<String>, wijziging: Wijziging) -> Self {
        self.voorstel = Some(Voorstel { vraag: vraag.into(), wijziging });
        self
    }

    /// De hele melding als één zin om voor te lezen.
    pub fn volledig(&self) -> String {
        let mut s = format!("Regel {}: {}", self.regel, self.tekst);
        if let Some(v) = &self.voorstel {
            s.push(' ');
            s.push_str(&v.vraag);
            s.push_str(" Antwoord met ja of nee.");
        }
        s
    }
}

impl std::fmt::Display for Melding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.volledig())
    }
}

/// Voert een voorstel uit op je tekst. Dit gebeurt alleen als jij "ja" zei.
pub fn pas_toe(bron: &str, wijziging: &Wijziging) -> String {
    let regeleinde = if bron.contains("\r\n") { "\r\n" } else { "\n" };
    let eindigt_met_regeleinde = bron.ends_with('\n');
    let mut regels: Vec<String> = bron.lines().map(|r| r.to_string()).collect();

    let inspringing = |regels: &Vec<String>, nr: usize| -> String {
        regels
            .get(nr.wrapping_sub(1))
            .map(|r| r.chars().take_while(|c| *c == ' ' || *c == '\t').collect())
            .unwrap_or_default()
    };

    match wijziging {
        Wijziging::VervangRegel { regel, nieuw } => {
            if *regel >= 1 && *regel <= regels.len() {
                // Eén regel kan ook door meer regels worden vervangen.
                let insp = inspringing(&regels, *regel);
                let nieuwe: Vec<String> = nieuw.lines().map(|r| format!("{}{}", insp, r.trim_start())).collect();
                regels.splice(regel - 1..*regel, nieuwe);
            }
        }
        Wijziging::VoegInVoor { regel, nieuw } => {
            if *regel >= 1 && *regel <= regels.len() {
                let insp = inspringing(&regels, *regel);
                regels.insert(regel - 1, format!("{}{}", insp, nieuw.trim_start()));
            } else {
                regels.push(nieuw.clone());
            }
        }
        Wijziging::HaalWeg { regel } => {
            if *regel >= 1 && *regel <= regels.len() {
                regels.remove(regel - 1);
            }
        }
    }
    let mut uit = regels.join(regeleinde);
    if eindigt_met_regeleinde || matches!(wijziging, Wijziging::VoegInVoor { .. }) && bron.is_empty() {
        uit.push_str(regeleinde);
    }
    uit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vervangen_houdt_inspringing() {
        let bron = "Als x is 1\r\n    Toom 3\r\nEinde als\r\n";
        let nieuw = pas_toe(bron, &Wijziging::VervangRegel { regel: 2, nieuw: "Toon 3".into() });
        assert_eq!(nieuw, "Als x is 1\r\n    Toon 3\r\nEinde als\r\n");
    }

    #[test]
    fn invoegen_en_weghalen() {
        let bron = "a\nb";
        assert_eq!(pas_toe(bron, &Wijziging::VoegInVoor { regel: 2, nieuw: "x".into() }), "a\nx\nb");
        assert_eq!(pas_toe(bron, &Wijziging::VoegInVoor { regel: 3, nieuw: "x".into() }), "a\nb\nx");
        assert_eq!(pas_toe(bron, &Wijziging::HaalWeg { regel: 1 }), "b");
    }
}
