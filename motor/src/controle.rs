//! Stap 3: controleren of de zinnen samen kloppen.
//!
//! Hier zoekt de motor naar zinnen die elkaar tegenspreken of niet op elkaar
//! aansluiten. Bijvoorbeeld een knop zonder venster, een naam die nog niet
//! bestaat, of rekenen met tekst. Dan komt er een fout of een vraag, nooit een gok.

use std::collections::{HashMap, HashSet};

use crate::melding::{Melding, Wijziging};
use crate::taal::*;
use crate::tekst;

/// Hoe vaak een herhaling hoogstens mag. Zo kan een programma nooit vastlopen.
pub const MAX_HERHALINGEN: f64 = 1_000_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Type {
    Getal,
    Tekst,
    Waarheid,
    Onbekend,
}

fn type_naam(t: Type) -> &'static str {
    match t {
        Type::Getal => "een getal",
        Type::Tekst => "tekst",
        Type::Waarheid => "waar of niet waar",
        Type::Onbekend => "onbekend",
    }
}

fn kort(u: &Uitdrukking) -> String {
    match u {
        Uitdrukking::Naam(n) => n.clone(),
        Uitdrukking::Vast(Waarde::Tekst(t)) => format!("\"{}\"", t),
        Uitdrukking::Vast(Waarde::Waarheid(b)) => if *b { "waar".into() } else { "niet waar".into() },
        _ => "dit".into(),
    }
}

fn vast_getal(u: &Uitdrukking) -> Option<f64> {
    match u {
        Uitdrukking::Vast(Waarde::Getal(g)) => Some(*g),
        Uitdrukking::Negatief(a) => vast_getal(a).map(|g| -g),
        _ => None,
    }
}

struct Controle<'a> {
    regels: &'a [&'a str],
    meldingen: Vec<Melding>,
    types: HashMap<String, (Type, usize)>,
    /// (naam, regel, staat los in het programma)
    onthoud: Vec<(String, usize, bool)>,
    al_gemeld: HashSet<(usize, String)>,
}

impl<'a> Controle<'a> {
    fn type_van(&self, u: &Uitdrukking) -> Result<Type, String> {
        match u {
            Uitdrukking::Vast(Waarde::Getal(_)) => Ok(Type::Getal),
            Uitdrukking::Vast(Waarde::Tekst(_)) => Ok(Type::Tekst),
            Uitdrukking::Vast(Waarde::Waarheid(_)) => Ok(Type::Waarheid),
            Uitdrukking::Naam(n) => Ok(self.types.get(n).map(|t| t.0).unwrap_or(Type::Onbekend)),
            Uitdrukking::Negatief(a) => {
                let t = self.type_van(a)?;
                if matches!(t, Type::Tekst | Type::Waarheid) {
                    return Err(format!("Min ervoor kan alleen bij een getal. {} is {}.", kort(a), type_naam(t)));
                }
                Ok(Type::Getal)
            }
            Uitdrukking::Bewerking(a, teken, b) => {
                let ta = self.type_van(a)?;
                let tb = self.type_van(b)?;
                if *teken == Teken::EnDan {
                    return Ok(Type::Tekst);
                }
                for (deel, t) in [(a, ta), (b, tb)] {
                    if matches!(t, Type::Tekst | Type::Waarheid) {
                        return Err(format!(
                            "Rekenen ({}) kan alleen met getallen, maar {} is {}. Wil je stukken tekst aan elkaar zetten? Gebruik dan \"en dan\".",
                            teken.woord(),
                            kort(deel),
                            type_naam(t)
                        ));
                    }
                }
                if *teken == Teken::GedeeldDoor && vast_getal(b) == Some(0.0) {
                    return Err("Delen door nul kan niet. Dat kan geen enkele rekenmachine.".into());
                }
                Ok(Type::Getal)
            }
        }
    }

    fn meld(&mut self, m: Melding) {
        self.meldingen.push(m);
    }

    fn is_bekend(&self, naam: &str, regel: usize, in_knop: bool) -> bool {
        self.onthoud.iter().any(|(n, r, _)| n == naam && (*r < regel || in_knop))
    }

    fn controleer_namen(&mut self, regel: usize, namen: Vec<String>, in_knop: bool, toon: Option<&Uitdrukking>) {
        for naam in namen {
            if self.is_bekend(&naam, regel, in_knop) || !self.al_gemeld.insert((regel, naam.clone())) {
                continue;
            }
            let tekst = format!("Je gebruikt \"{}\", maar je hebt nog niet gezegd wat {} is.", naam, naam);
            let lijkt_op = self
                .onthoud
                .iter()
                .map(|o| o.0.clone())
                .filter(|n| tekst::mag_tikfout_zijn(&naam, n))
                .min_by_key(|n| tekst::afstand(&naam, n));
            let regeltekst = self.regels.get(regel - 1).copied().unwrap_or("");
            let m = if let (Some(goed), Some(nieuw)) =
                (lijkt_op.clone(), lijkt_op.as_ref().and_then(|g| vervang_naam(regeltekst, &naam, g)))
            {
                Melding::vraag(regel, tekst)
                    .met_voorstel(format!("Bedoel je {}? Dan wordt de regel: {}", goed, nieuw), Wijziging::VervangRegel { regel, nieuw })
            } else if let Some(Uitdrukking::Naam(_)) = toon {
                let rest = tekst::normaliseer(regeltekst);
                let rest = rest.split_once(' ').map(|x| x.1).unwrap_or("").to_string();
                let nieuw = format!("Toon de tekst {}", rest);
                Melding::vraag(regel, tekst).met_voorstel(
                    format!("Bedoel je dat ik de tekst {} moet tonen? Dan wordt de regel: {}", rest, nieuw),
                    Wijziging::VervangRegel { regel, nieuw },
                )
            } else {
                let nieuw = format!("Onthoud {} is 0", naam);
                Melding::vraag(regel, tekst).met_voorstel(
                    format!("Zal ik erboven zetten: {}?", nieuw),
                    Wijziging::VoegInVoor { regel, nieuw },
                )
            };
            self.meld(m);
        }
    }

    fn controleer_aantal(&mut self, regel: usize, u: &Uitdrukking, in_knop: bool) {
        let mut namen = Vec::new();
        u.namen(&mut namen);
        self.controleer_namen(regel, namen, in_knop, None);
        match self.type_van(u) {
            Err(t) => self.meld(Melding::fout(regel, t)),
            Ok(t @ (Type::Tekst | Type::Waarheid)) => self.meld(Melding::fout(
                regel,
                format!("Hoe vaak moet ik het doen? Hier verwacht ik een getal, maar {} is {}.", kort(u), type_naam(t)),
            )),
            Ok(_) => {}
        }
        if let Some(g) = vast_getal(u) {
            if g < 0.0 || g.fract() != 0.0 {
                self.meld(Melding::fout(regel, "Het aantal keer moet een heel getal zijn, zoals 0, 1 of 10."));
            } else if g > MAX_HERHALINGEN {
                self.meld(Melding::fout(
                    regel,
                    "Zo vaak herhalen mag niet. Het kan hoogstens een miljoen keer, anders loopt het programma vast.",
                ));
            }
        }
    }

    fn controleer_voorwaarde(&mut self, regel: usize, v: &Voorwaarde, in_knop: bool) {
        let mut namen = Vec::new();
        v.namen(&mut namen);
        self.controleer_namen(regel, namen, in_knop, None);
        match v {
            Voorwaarde::IsWaar(u) => match self.type_van(u) {
                Err(t) => self.meld(Melding::fout(regel, t)),
                Ok(Type::Getal) | Ok(Type::Tekst) => self.meld(Melding::fout(
                    regel,
                    format!(
                        "Hier verwacht ik iets dat waar of niet waar is. Maar {} is {}. Schrijf bijvoorbeeld: Als teller is groter dan 5.",
                        kort(u),
                        type_naam(self.type_van(u).unwrap())
                    ),
                )),
                Ok(_) => {}
            },
            Voorwaarde::Vergelijk(a, verg, b) => {
                let (ta, tb) = match (self.type_van(a), self.type_van(b)) {
                    (Ok(ta), Ok(tb)) => (ta, tb),
                    (Err(t), _) | (_, Err(t)) => return self.meld(Melding::fout(regel, t)),
                };
                let groter_kleiner = !matches!(verg, Vergelijking::Gelijk | Vergelijking::NietGelijk);
                if groter_kleiner {
                    for (deel, t) in [(a, ta), (b, tb)] {
                        if matches!(t, Type::Tekst | Type::Waarheid) {
                            return self.meld(Melding::fout(
                                regel,
                                format!("Groter of kleiner kan ik alleen bij getallen bepalen. Maar {} is {}.", kort(deel), type_naam(t)),
                            ));
                        }
                    }
                } else if ta != Type::Onbekend && tb != Type::Onbekend && ta != tb {
                    self.meld(Melding::vraag(
                        regel,
                        format!(
                            "Je vergelijkt {} ({}) met {} ({}). Die zijn nooit gelijk, dus deze keuze heeft geen zin. Wat bedoel je hier?",
                            kort(a),
                            type_naam(ta),
                            kort(b),
                            type_naam(tb)
                        ),
                    ));
                }
            }
        }
    }

    fn controleer_lijst(&mut self, lijst: &[Opdracht], in_knop: bool) {
        for o in lijst {
            let r = o.regel;
            match &o.soort {
                Soort::Opmerking(_) | Soort::MaakVenster { .. } | Soort::MaakKnop { .. } | Soort::Installatie(_) => {}
                Soort::Toon(u) => {
                    let mut namen = Vec::new();
                    u.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, Some(u));
                    if let Err(t) = self.type_van(u) {
                        self.meld(Melding::fout(r, t));
                    }
                }
                Soort::Onthoud { naam, waarde, .. } | Soort::Verander { naam, waarde } => {
                    let mut namen = Vec::new();
                    waarde.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    if matches!(o.soort, Soort::Verander { .. }) {
                        self.controleer_namen(r, vec![naam.clone()], in_knop, None);
                    }
                    match self.type_van(waarde) {
                        Err(t) => self.meld(Melding::fout(r, t)),
                        Ok(nieuw) => {
                            if let Some(&(oud, eerste)) = self.types.get(naam) {
                                if eerste != r && oud != Type::Onbekend && nieuw != Type::Onbekend && oud != nieuw {
                                    self.meld(Melding::vraag(
                                        r,
                                        format!(
                                            "{} is op regel {} {}, maar hier maak je er {} van. Dat spreekt elkaar tegen. Wat bedoel je?",
                                            naam,
                                            eerste,
                                            type_naam(oud),
                                            type_naam(nieuw)
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
                Soort::TelOp { naam, waarde } | Soort::TrekAf { naam, waarde } => {
                    let mut namen = vec![naam.clone()];
                    waarde.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    let doel = self.types.get(naam).map(|t| t.0).unwrap_or(Type::Onbekend);
                    match self.type_van(waarde) {
                        Err(t) => self.meld(Melding::fout(r, t)),
                        Ok(t) => {
                            for (wat, tt) in [(naam.clone(), doel), (kort(waarde), t)] {
                                if matches!(tt, Type::Tekst | Type::Waarheid) {
                                    self.meld(Melding::fout(
                                        r,
                                        format!("Optellen en aftrekken kan alleen met getallen. Maar {} is {}.", wat, type_naam(tt)),
                                    ));
                                    break;
                                }
                            }
                        }
                    }
                }
                Soort::Als { voorwaarde, dan, anders, .. } => {
                    self.controleer_voorwaarde(r, voorwaarde, in_knop);
                    self.controleer_lijst(dan, in_knop);
                    self.controleer_lijst(anders, in_knop);
                }
                Soort::Herhaal { aantal, blok, .. } => {
                    self.controleer_aantal(r, aantal, in_knop);
                    self.controleer_lijst(blok, in_knop);
                }
                Soort::HerhaalZolang { voorwaarde, hoogstens, blok, .. } => {
                    self.controleer_voorwaarde(r, voorwaarde, in_knop);
                    self.controleer_aantal(r, hoogstens, in_knop);
                    self.controleer_lijst(blok, in_knop);
                }
                Soort::BijKnop { blok, .. } => self.controleer_lijst(blok, true),
            }
        }
    }
}

/// Vervangt een naam (één of meer woorden) in een regel door een andere naam.
fn vervang_naam(regel: &str, oud: &str, nieuw: &str) -> Option<String> {
    let norm = tekst::normaliseer(regel);
    let woorden: Vec<&str> = norm.split(' ').collect();
    let oud_woorden: Vec<&str> = oud.split(' ').collect();
    let schoon = |w: &str| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '_').to_lowercase();
    for i in 0..woorden.len() {
        if i + oud_woorden.len() <= woorden.len()
            && oud_woorden.iter().enumerate().all(|(j, o)| schoon(woorden[i + j]) == *o)
        {
            let mut uit: Vec<String> = woorden[..i].iter().map(|s| s.to_string()).collect();
            uit.push(nieuw.to_string());
            uit.extend(woorden[i + oud_woorden.len()..].iter().map(|s| s.to_string()));
            return Some(uit.join(" "));
        }
    }
    None
}

pub(crate) fn controleer(p: &Programma, regels: &[&str]) -> Vec<Melding> {
    let mut c = Controle { regels, meldingen: Vec::new(), types: HashMap::new(), onthoud: Vec::new(), al_gemeld: HashSet::new() };

    // Ronde 1: wat wordt er onthouden, en wat voor soort waarde is het?
    fn verzamel(c: &mut Controle, lijst: &[Opdracht], los: bool) {
        for o in lijst {
            match &o.soort {
                Soort::Onthoud { naam, waarde, .. } => {
                    if !c.types.contains_key(naam) {
                        let t = c.type_van(waarde).unwrap_or(Type::Onbekend);
                        c.types.insert(naam.clone(), (t, o.regel));
                    }
                    c.onthoud.push((naam.clone(), o.regel, los));
                }
                Soort::Als { dan, anders, .. } => {
                    verzamel(c, dan, false);
                    verzamel(c, anders, false);
                }
                Soort::Herhaal { blok, .. } | Soort::HerhaalZolang { blok, .. } | Soort::BijKnop { blok, .. } => {
                    verzamel(c, blok, false)
                }
                _ => {}
            }
        }
    }
    verzamel(&mut c, &p.opdrachten, true);

    // Twee keer onthouden in het programma zelf: dubbel.
    let mut eerder: HashMap<String, usize> = HashMap::new();
    for (naam, regel, los) in c.onthoud.clone() {
        if !los {
            continue;
        }
        if let Some(&eerste) = eerder.get(&naam) {
            let bron = p.alle().iter().find_map(|o| match &o.soort {
                Soort::Onthoud { bron, .. } if o.regel == regel => Some(bron.clone()),
                _ => None,
            });
            let nieuw = format!("Verander {} in {}", naam, bron.unwrap_or_default());
            c.meld(
                Melding::vraag(regel, format!("Je onthoudt {} al op regel {}. Twee keer onthouden is dubbel.", naam, eerste))
                    .met_voorstel(
                        format!("Bedoel je dat de waarde verandert? Dan wordt de regel: {}", nieuw),
                        Wijziging::VervangRegel { regel, nieuw },
                    ),
            );
        } else {
            eerder.insert(naam, regel);
        }
    }

    // Vensters, knoppen en wat de knoppen doen.
    let alle = p.alle();
    let vensters: Vec<usize> =
        alle.iter().filter(|o| matches!(o.soort, Soort::MaakVenster { .. })).map(|o| o.regel).collect();
    let knoppen: Vec<(String, usize)> = alle
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::MaakKnop { naam } => Some((naam.clone(), o.regel)),
            _ => None,
        })
        .collect();
    let acties: Vec<(String, usize)> = alle
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::BijKnop { naam, .. } => Some((naam.clone(), o.regel)),
            _ => None,
        })
        .collect();

    if let Some((naam, regel)) = knoppen.iter().find(|(_, r)| !vensters.iter().any(|v| v < r)) {
        let nieuw = "Maak een venster met als titel Mijn programma".to_string();
        c.meld(
            Melding::vraag(*regel, format!("Je maakt de knop {}, maar er is nog geen venster om hem in te zetten.", naam))
                .met_voorstel(
                    format!("Zal ik er eerst een venster voor maken, met de regel: {}?", nieuw),
                    Wijziging::VoegInVoor { regel: *regel, nieuw },
                ),
        );
    }
    for (i, (naam, regel)) in knoppen.iter().enumerate() {
        if let Some((_, eerste)) = knoppen[..i].iter().find(|(n, _)| n.to_lowercase() == naam.to_lowercase()) {
            c.meld(Melding::vraag(
                *regel,
                format!(
                    "Er is al een knop met de naam {}, op regel {}. Twee knoppen met dezelfde naam kan ik niet uit elkaar houden. Welke naam moet deze knop krijgen?",
                    naam, eerste
                ),
            ));
        }
    }
    for (i, (naam, regel)) in acties.iter().enumerate() {
        if let Some((_, eerste)) = acties[..i].iter().find(|(n, _)| n.to_lowercase() == naam.to_lowercase()) {
            c.meld(Melding::vraag(
                *regel,
                format!(
                    "Je beschrijft twee keer wat de knop {} doet: op regel {} en op regel {}. Dat spreekt elkaar tegen. Welke van de twee bedoel je? Haal de andere weg, of zet alles in één blok.",
                    naam, eerste, regel
                ),
            ));
            continue;
        }
        if knoppen.iter().any(|(n, _)| n.to_lowercase() == naam.to_lowercase()) {
            continue;
        }
        let lijkt_op = knoppen
            .iter()
            .filter(|(n, _)| tekst::mag_tikfout_zijn(&naam.to_lowercase(), &n.to_lowercase()))
            .min_by_key(|(n, _)| tekst::afstand(naam, n));
        let regeltekst = regels.get(regel - 1).copied().unwrap_or("");
        let m = match lijkt_op {
            Some((goed, _)) => {
                let nieuw = tekst::normaliseer(regeltekst).replacen(naam.as_str(), goed, 1);
                Melding::vraag(*regel, format!("Er is geen knop met de naam {}.", naam)).met_voorstel(
                    format!("Bedoel je de knop {}? Dan wordt de regel: {}", goed, nieuw),
                    Wijziging::VervangRegel { regel: *regel, nieuw },
                )
            }
            None => {
                let nieuw = format!("Maak een knop met als naam {}", naam);
                Melding::vraag(*regel, format!("Er is geen knop met de naam {}.", naam)).met_voorstel(
                    format!("Zal ik die knop maken, met de regel: {}?", nieuw),
                    Wijziging::VoegInVoor { regel: *regel, nieuw },
                )
            }
        };
        c.meld(m);
    }

    controleer_installatie(&mut c, p);

    // Ronde 2: elke opdracht nalopen.
    c.controleer_lijst(&p.opdrachten, false);
    c.meldingen
}

/// Controleert de zinnen over het installatiebestand en de licentie.
fn controleer_installatie(c: &mut Controle, p: &Programma) {
    let los: Vec<usize> = p.opdrachten.iter().map(|o| o.regel).collect();
    let zinnen: Vec<(usize, &InstallatieZin)> = p
        .alle()
        .into_iter()
        .filter_map(|o| match &o.soort {
            Soort::Installatie(z) => Some((o.regel, z)),
            _ => None,
        })
        .collect();
    for (r, _) in &zinnen {
        if !los.contains(r) {
            c.meld(Melding::fout(*r, "Zinnen over de installatie en de licentie moeten los staan, niet binnen een keuze, herhaling of knop."));
        }
    }
    let Some(&(eerste, _)) = zinnen.first() else { return };
    let namen: Vec<usize> = zinnen.iter().filter(|(_, z)| matches!(z, InstallatieZin::Naam(_))).map(|(r, _)| *r).collect();
    if namen.is_empty() {
        let nieuw = "Maak een installatiebestand met als naam Mijn programma".to_string();
        c.meld(
            Melding::vraag(eerste, "Je beschrijft de installatie, maar het installatiebestand heeft nog geen naam.").met_voorstel(
                format!("Zal ik erboven zetten: {}?", nieuw),
                Wijziging::VoegInVoor { regel: eerste, nieuw },
            ),
        );
    }
    let dubbel = |c: &mut Controle, wat: &str, regels: Vec<usize>| {
        if regels.len() > 1 {
            c.meld(Melding::vraag(
                regels[1],
                format!(
                    "Je beschrijft {} twee keer: op regel {} en op regel {}. Dat spreekt elkaar tegen. Welke bedoel je? Haal de andere weg.",
                    wat, regels[0], regels[1]
                ),
            ));
        }
    };
    let regels_van = |f: &dyn Fn(&InstallatieZin) -> bool| -> Vec<usize> {
        zinnen.iter().filter(|(_, z)| f(z)).map(|(r, _)| *r).collect()
    };
    dubbel(c, "de naam van het installatiebestand", namen.clone());
    dubbel(c, "de versie", regels_van(&|z| matches!(z, InstallatieZin::Versie(_))));
    dubbel(c, "de maker", regels_van(&|z| matches!(z, InstallatieZin::Maker(_))));
    dubbel(c, "het licentiebestand", regels_van(&|z| matches!(z, InstallatieZin::LicentieBestand(_))));
    let rechten: Vec<(usize, Rechten)> = zinnen
        .iter()
        .filter_map(|(r, z)| match z {
            InstallatieZin::Rechten(x) => Some((*r, *x)),
            _ => None,
        })
        .collect();
    if rechten.len() > 1 && rechten.iter().any(|(_, x)| *x != rechten[0].1) {
        c.meld(Melding::vraag(
            rechten[1].0,
            format!(
                "Regel {} en regel {} zeggen iets anders over de beheerdersrechten. Dat spreekt elkaar tegen. Welke bedoel je?",
                rechten[0].0, rechten[1].0
            ),
        ));
    }
    let tekstregels = regels_van(&|z| matches!(z, InstallatieZin::LicentieRegel(_)));
    let bestand = regels_van(&|z| matches!(z, InstallatieZin::LicentieBestand(_)));
    if let (Some(t), Some(b)) = (tekstregels.first(), bestand.first()) {
        c.meld(Melding::vraag(
            *t.max(b),
            format!(
                "Je schrijft de licentie op regel {} zelf, en op regel {} zeg je dat hij in een bestand staat. Welke van de twee bedoel je?",
                t, b
            ),
        ));
    }
}
