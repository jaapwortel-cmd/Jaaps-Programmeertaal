//! Stap 4: van de opdrachten Rust-code maken.
//!
//! De Rust-code bestaat uit twee delen: de looptijd (altijd hetzelfde, zie
//! `looptijd/looptijd.rs`) en jouw programma. Bij elke regel staat als
//! opmerking jouw eigen zin, zodat je kunt zien wat er van elke zin is gemaakt.

use std::collections::HashMap;

use crate::taal::*;

/// De looptijd die in elk programma meegaat.
pub const LOOPTIJD: &str = include_str!("../looptijd/looptijd.rs");

struct Maker<'a> {
    namen: HashMap<String, usize>,
    namenlijst: Vec<String>,
    acties: Vec<String>,
    regels: &'a [&'a str],
    uit: String,
}

fn rust_tekst(s: &str) -> String {
    format!("{:?}", s)
}

impl<'a> Maker<'a> {
    fn naam_nr(&mut self, naam: &str) -> usize {
        if let Some(&n) = self.namen.get(naam) {
            return n;
        }
        let n = self.namenlijst.len();
        self.namen.insert(naam.to_string(), n);
        self.namenlijst.push(naam.to_string());
        n
    }

    fn uitdrukking(&mut self, u: &Uitdrukking, r: usize) -> String {
        match u {
            Uitdrukking::Vast(Waarde::Getal(g)) => format!("W::G({:?}f64)", g),
            Uitdrukking::Vast(Waarde::Tekst(t)) => format!("W::T({}.to_string())", rust_tekst(t)),
            Uitdrukking::Vast(Waarde::Waarheid(b)) => format!("W::B({})", b),
            Uitdrukking::Naam(n) => {
                let i = self.naam_nr(n);
                format!("s.lees({}, {})?", i, r)
            }
            Uitdrukking::Onderdeel { naam, als_getal } => format!("s.waarde_van({}, {}, {})?", rust_tekst(naam), als_getal, r),
            Uitdrukking::Negatief(a) => {
                let a = self.uitdrukking(a, r);
                format!("negatief({}, {})?", a, r)
            }
            Uitdrukking::Bewerking(a, t, b) => {
                let a = self.uitdrukking(a, r);
                let b = self.uitdrukking(b, r);
                let nr = match t {
                    Teken::Plus => 0,
                    Teken::Min => 1,
                    Teken::Keer => 2,
                    Teken::GedeeldDoor => 3,
                    Teken::EnDan => 4,
                };
                format!("reken({}, {}, {}, {})?", a, nr, b, r)
            }
        }
    }

    fn voorwaarde(&mut self, v: &Voorwaarde, r: usize) -> String {
        match v {
            Voorwaarde::IsWaar(u) => {
                let u = self.uitdrukking(u, r);
                format!("waar({}, {})?", u, r)
            }
            Voorwaarde::Vergelijk(a, verg, b) => {
                let a = self.uitdrukking(a, r);
                let b = self.uitdrukking(b, r);
                let op = match verg {
                    Vergelijking::Gelijk => 0,
                    Vergelijking::NietGelijk => 1,
                    Vergelijking::Groter => 2,
                    Vergelijking::Kleiner => 3,
                    Vergelijking::Minstens => 4,
                    Vergelijking::Hoogstens => 5,
                };
                format!("vergelijk({}, {}, {}, {})?", a, b, op, r)
            }
        }
    }

    fn regel(&mut self, diepte: usize, tekst: &str) {
        for _ in 0..diepte {
            self.uit.push_str("    ");
        }
        self.uit.push_str(tekst);
        self.uit.push('\n');
    }

    fn opmerking(&mut self, diepte: usize, nr: usize) {
        let zin = self.regels.get(nr.wrapping_sub(1)).map(|z| z.trim()).unwrap_or("");
        let zin: String = zin.chars().filter(|c| !c.is_control()).collect();
        self.regel(diepte, &format!("// Regel {}: {}", nr, zin));
    }

    fn lijst(&mut self, lijst: &[Opdracht], diepte: usize) {
        for o in lijst {
            let r = o.regel;
            match &o.soort {
                Soort::BijKnop { .. } => {}
                Soort::Opmerking(_) | Soort::Installatie(_) => self.opmerking(diepte, r),
                Soort::MaakVenster { titel } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.venster({});", rust_tekst(titel)));
                }
                Soort::MaakKnop { naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.knop({});", rust_tekst(naam)));
                }
                Soort::MaakOnderdeel { soort, naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.onderdeel({}, {});", soort.nummer(), rust_tekst(naam)));
                }
                Soort::VoegKeuzeToe { keuze, lijst } => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(keuze, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.voeg_keuze({}, w, {})?; }}", u, rust_tekst(lijst), r));
                }
                Soort::ZetIn { waarde, naam } => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(waarde, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.zet_in({}, w, {})?; }}", u, rust_tekst(naam), r));
                }
                Soort::Vink { naam, aan } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.vink({}, {}, {})?;", rust_tekst(naam), aan, r));
                }
                Soort::Focus { naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.focus({}, {})?;", rust_tekst(naam), r));
                }
                Soort::Toon(u) => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(u, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.toon(w); }}", u));
                }
                Soort::Onthoud { naam, waarde, .. } => {
                    self.opmerking(diepte, r);
                    let i = self.naam_nr(naam);
                    let u = self.uitdrukking(waarde, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.zet({}, w); }}", u, i));
                }
                Soort::Verander { naam, waarde } => {
                    self.opmerking(diepte, r);
                    let i = self.naam_nr(naam);
                    let u = self.uitdrukking(waarde, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.verander({}, w, {})?; }}", u, i, r));
                }
                Soort::TelOp { naam, waarde } | Soort::TrekAf { naam, waarde } => {
                    self.opmerking(diepte, r);
                    let i = self.naam_nr(naam);
                    let u = self.uitdrukking(waarde, r);
                    let erbij = matches!(o.soort, Soort::TelOp { .. });
                    self.regel(diepte, &format!("{{ let w = {}; s.tel_op({}, w, {}, {})?; }}", u, i, erbij, r));
                }
                Soort::Als { voorwaarde, dan, anders, anders_regel, einde } => {
                    self.opmerking(diepte, r);
                    let v = self.voorwaarde(voorwaarde, r);
                    self.regel(diepte, &format!("if {} {{", v));
                    self.lijst(dan, diepte + 1);
                    self.regel(diepte, "} else {");
                    if let Some(a) = anders_regel {
                        self.opmerking(diepte + 1, *a);
                    }
                    self.lijst(anders, diepte + 1);
                    self.regel(diepte, "}");
                    self.opmerking(diepte, *einde);
                }
                Soort::Herhaal { aantal, blok, einde } => {
                    self.opmerking(diepte, r);
                    let a = self.uitdrukking(aantal, r);
                    self.regel(diepte, &format!("for _ in 0..aantal({}, {})? {{", a, r));
                    self.regel(diepte + 1, &format!("s.stap({})?;", r));
                    self.lijst(blok, diepte + 1);
                    self.regel(diepte, "}");
                    self.opmerking(diepte, *einde);
                }
                Soort::HerhaalZolang { voorwaarde, hoogstens, blok, einde } => {
                    self.opmerking(diepte, r);
                    let a = self.uitdrukking(hoogstens, r);
                    self.regel(diepte, "{");
                    self.regel(diepte + 1, &format!("let hoogstens = aantal({}, {})?;", a, r));
                    self.regel(diepte + 1, "let mut keer = 0u64;");
                    self.regel(diepte + 1, "loop {");
                    self.regel(diepte + 2, &format!("s.stap({})?;", r));
                    let v = self.voorwaarde(voorwaarde, r);
                    self.regel(diepte + 2, &format!("if !({}) {{ break; }}", v));
                    self.regel(diepte + 2, &format!("if keer == hoogstens {{ s.grens({}, hoogstens); break; }}", r));
                    self.regel(diepte + 2, "keer += 1;");
                    self.lijst(blok, diepte + 2);
                    self.regel(diepte + 1, "}");
                    self.regel(diepte, "}");
                    self.opmerking(diepte, *einde);
                }
            }
        }
    }
}

/// Maakt de volledige Rust-code (looptijd + jouw programma) van een gecontroleerd programma.
pub fn maak(p: &Programma, regels: &[&str]) -> String {
    let mut m = Maker { namen: HashMap::new(), namenlijst: Vec::new(), acties: Vec::new(), regels, uit: String::new() };

    // Wat de knoppen doen.
    let knop_blokken: Vec<(&String, &Vec<Opdracht>, usize, usize)> = p
        .opdrachten
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::BijKnop { naam, blok, einde } => Some((naam, blok, o.regel, *einde)),
            _ => None,
        })
        .collect();

    m.uit.push_str("\n// ==================== Jouw programma ====================\n\n");
    m.uit.push_str("fn hoofd(s: &mut Staat) -> R<()> {\n");
    m.lijst(&p.opdrachten, 1);
    m.uit.push_str("    Ok(())\n}\n\n");

    m.uit.push_str("fn bij_knop(s: &mut Staat, actie: usize) -> R<()> {\n    match actie {\n");
    for (i, (naam, blok, regel, einde)) in knop_blokken.iter().enumerate() {
        m.acties.push((*naam).clone());
        m.regel(2, &format!("{} => {{", i));
        m.opmerking(3, *regel);
        m.lijst(blok, 3);
        m.opmerking(3, *einde);
        m.regel(2, "}");
    }
    m.uit.push_str("        _ => {}\n    }\n    Ok(())\n}\n\n");

    let namen: Vec<String> = m.namenlijst.iter().map(|n| rust_tekst(n)).collect();
    let acties: Vec<String> = m.acties.iter().map(|n| rust_tekst(n)).collect();
    m.uit.push_str(&format!("const NAMEN: &[&str] = &[{}];\n", namen.join(", ")));
    m.uit.push_str(&format!("const ACTIES: &[&str] = &[{}];\n\n", acties.join(", ")));
    m.uit.push_str("fn main() {\n    start(NAMEN, ACTIES, hoofd, bij_knop);\n}\n");

    format!("{}{}", LOOPTIJD, m.uit)
}

/// Alleen het deel dat van jouw zinnen is gemaakt (zonder de looptijd). Om te laten zien.
pub fn alleen_programma(volledig: &str) -> &str {
    match volledig.find("// ==================== Jouw programma") {
        Some(p) => &volledig[p..],
        None => volledig,
    }
}
