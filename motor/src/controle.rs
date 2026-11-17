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
    Lijst,
    Onbekend,
}

fn type_naam(t: Type) -> &'static str {
    match t {
        Type::Getal => "een getal",
        Type::Tekst => "tekst",
        Type::Waarheid => "waar of niet waar",
        Type::Lijst => "een lijst",
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
    programma: &'a Programma,
    meldingen: Vec<Melding>,
    types: HashMap<String, (Type, usize)>,
    /// (naam, regel, staat los in het programma)
    onthoud: Vec<(String, usize, bool)>,
    al_gemeld: HashSet<(usize, String)>,
    /// (naam, soort, regel) van elk onderdeel op het scherm.
    onderdelen: Vec<(String, OnderdeelSoort, usize)>,
    /// De namen van alle tabbladen.
    tabbladen: Vec<String>,
}

fn onderdelen_in(u: &Uitdrukking, uit: &mut Vec<(String, bool)>) {
    match u {
        Uitdrukking::Onderdeel { naam, als_getal } => uit.push((naam.clone(), *als_getal)),
        Uitdrukking::Bewerking(a, _, b) => {
            onderdelen_in(a, uit);
            onderdelen_in(b, uit);
        }
        Uitdrukking::Negatief(a) | Uitdrukking::Aantal(a) => onderdelen_in(a, uit),
        Uitdrukking::Element(a, b) => {
            onderdelen_in(a, uit);
            onderdelen_in(b, uit);
        }
        Uitdrukking::Aanroep { invoer, .. } | Uitdrukking::Ingebouwd { invoer, .. } => {
            for i in invoer {
                onderdelen_in(i, uit);
            }
        }
        _ => {}
    }
}

fn onderdelen_in_voorwaarde(v: &Voorwaarde, uit: &mut Vec<(String, bool)>) {
    match v {
        Voorwaarde::Vergelijk(a, _, b) => {
            onderdelen_in(a, uit);
            onderdelen_in(b, uit);
        }
        Voorwaarde::IsWaar(a) => onderdelen_in(a, uit),
        Voorwaarde::Bevat(a, b) => {
            onderdelen_in(a, uit);
            onderdelen_in(b, uit);
        }
        Voorwaarde::En(a, b) | Voorwaarde::Of(a, b) => {
            onderdelen_in_voorwaarde(a, uit);
            onderdelen_in_voorwaarde(b, uit);
        }
    }
}

impl<'a> Controle<'a> {
    fn type_van(&self, u: &Uitdrukking) -> Result<Type, String> {
        match u {
            Uitdrukking::Vast(Waarde::Getal(_)) => Ok(Type::Getal),
            Uitdrukking::Vast(Waarde::Tekst(_)) => Ok(Type::Tekst),
            Uitdrukking::Vast(Waarde::Waarheid(_)) => Ok(Type::Waarheid),
            Uitdrukking::Naam(n) => Ok(self.types.get(n).map(|t| t.0).unwrap_or(Type::Onbekend)),
            Uitdrukking::Onderdeel { als_getal: true, .. } => Ok(Type::Getal),
            Uitdrukking::LegeLijst => Ok(Type::Lijst),
            Uitdrukking::Aanroep { .. } => Ok(Type::Onbekend),
            Uitdrukking::Ingebouwd { wat, invoer } => {
                for i in invoer {
                    self.type_van(i)?;
                }
                Ok(match wat {
                    Ingebouwd::Inhoud | Ingebouwd::LogboekPlek | Ingebouwd::Tijd | Ingebouwd::Datum | Ingebouwd::Webpagina | Ingebouwd::WebTekst => {
                        Type::Tekst
                    }
                    Ingebouwd::Regels | Ingebouwd::GekozenRij | Ingebouwd::RijNummer => Type::Lijst,
                    Ingebouwd::AantalRijen => Type::Getal,
                    Ingebouwd::Bestaat => Type::Waarheid,
                    Ingebouwd::Willekeurig => {
                        for i in invoer {
                            if matches!(self.type_van(i)?, Type::Tekst | Type::Waarheid | Type::Lijst) {
                                return Err("Een willekeurig getal kan alleen tussen twee getallen.".into());
                            }
                        }
                        Type::Getal
                    }
                })
            }
            Uitdrukking::Aantal(a) => match self.type_van(a)? {
                Type::Lijst | Type::Onbekend => Ok(Type::Getal),
                t => Err(format!("Het aantal kan ik alleen tellen in een lijst. Maar {} is {}.", kort(a), type_naam(t))),
            },
            Uitdrukking::Element(a, b) => {
                match self.type_van(a)? {
                    Type::Lijst | Type::Onbekend => {}
                    t => return Err(format!("Iets uit een lijst halen kan alleen bij een lijst. Maar {} is {}.", kort(a), type_naam(t))),
                }
                match self.type_van(b)? {
                    Type::Getal | Type::Onbekend => Ok(Type::Onbekend),
                    t => Err(format!("Het nummer in de lijst moet een getal zijn, maar {} is {}.", kort(b), type_naam(t))),
                }
            }
            Uitdrukking::Onderdeel { naam, .. } => Ok(match self.onderdeel_soort(naam) {
                Some(OnderdeelSoort::Selectievakje) => Type::Waarheid,
                Some(_) => Type::Tekst,
                None => Type::Onbekend,
            }),
            Uitdrukking::Negatief(a) => {
                let t = self.type_van(a)?;
                if matches!(t, Type::Tekst | Type::Waarheid | Type::Lijst) {
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
                    if matches!(t, Type::Tekst | Type::Waarheid | Type::Lijst) {
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

    fn onderdeel_soort(&self, naam: &str) -> Option<OnderdeelSoort> {
        self.onderdelen.iter().find(|(n, _, _)| n.to_lowercase() == naam.to_lowercase()).map(|o| o.1)
    }

    /// Bestaat het onderdeel, en is het van een soort die hier past?
    fn controleer_onderdeel(&mut self, regel: usize, naam: &str, toegestaan: &[OnderdeelSoort], wat: &str) {
        if !self.al_gemeld.insert((regel, format!("onderdeel:{}", naam.to_lowercase()))) {
            return;
        }
        match self.onderdeel_soort(naam) {
            Some(soort) if toegestaan.is_empty() || toegestaan.contains(&soort) => {}
            Some(soort) => self.meld(Melding::fout(
                regel,
                format!("{} is een {}. {}", naam, soort.woord(), wat),
            )),
            None => {
                let tekst = format!("Er is geen tekstvak, label, selectievakje of keuzelijst met de naam {}.", naam);
                let lijkt_op = self
                    .onderdelen
                    .iter()
                    .map(|o| o.0.clone())
                    .filter(|n| tekst::mag_tikfout_zijn(&naam.to_lowercase(), &n.to_lowercase()))
                    .min_by_key(|n| tekst::afstand(naam, n));
                let regeltekst = tekst::normaliseer(self.regels.get(regel - 1).copied().unwrap_or(""));
                let m = match lijkt_op {
                    Some(goed) if regeltekst.contains(naam) => {
                        let nieuw = regeltekst.replacen(naam, &goed, 1);
                        Melding::vraag(regel, tekst).met_voorstel(
                            format!("Bedoel je {}? Dan wordt de regel: {}", goed, nieuw),
                            Wijziging::VervangRegel { regel, nieuw },
                        )
                    }
                    _ => Melding::vraag(regel, format!("{} Welk onderdeel bedoel je? Maak het eerst, bijvoorbeeld met: Maak een tekstvak met als naam {}.", tekst, naam)),
                };
                self.meld(m);
            }
        }
    }

    fn controleer_onderdelen_in(&mut self, regel: usize, lijst: Vec<(String, bool)>) {
        for (naam, als_getal) in lijst {
            let toegestaan: &[OnderdeelSoort] = if als_getal {
                &[OnderdeelSoort::Tekstvak, OnderdeelSoort::GrootTekstvak, OnderdeelSoort::Label]
            } else {
                &[]
            };
            self.controleer_onderdeel(regel, &naam, toegestaan, "Daar kan ik geen getal uit lezen.");
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
        let mut od = Vec::new();
        onderdelen_in_voorwaarde(v, &mut od);
        self.controleer_onderdelen_in(regel, od);
        let mut namen = Vec::new();
        v.namen(&mut namen);
        self.controleer_namen(regel, namen, in_knop, None);
        match v {
            Voorwaarde::En(a, b) | Voorwaarde::Of(a, b) => {
                self.controleer_voorwaarde(regel, a, in_knop);
                self.controleer_voorwaarde(regel, b, in_knop);
            }
            Voorwaarde::Bevat(a, b) => {
                match self.type_van(a) {
                    Err(t) => self.meld(Melding::fout(regel, t)),
                    Ok(Type::Lijst) | Ok(Type::Onbekend) => {}
                    Ok(t) => self.meld(Melding::fout(regel, format!("\"bevat\" kan alleen bij een lijst. Maar {} is {}.", kort(a), type_naam(t)))),
                }
                if let Err(t) = self.type_van(b) {
                    self.meld(Melding::fout(regel, t));
                }
            }
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
                        if matches!(t, Type::Tekst | Type::Waarheid | Type::Lijst) {
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
                Soort::VoegToeAanLijst { waarde, lijst } | Soort::HaalUitLijst { waarde, lijst } => {
                    let mut od = Vec::new();
                    onderdelen_in(waarde, &mut od);
                    self.controleer_onderdelen_in(r, od);
                    let mut namen = Vec::new();
                    waarde.namen(&mut namen);
                    if let Uitdrukking::Naam(n) = waarde {
                        if !self.is_bekend(n, r, in_knop) && !self.onthoud.iter().any(|o| tekst::mag_tikfout_zijn(n, &o.0)) {
                            let regeltekst = self.regels.get(r - 1).copied().unwrap_or("");
                            if let Some(nieuw) = aanhalen(regeltekst, n) {
                                self.al_gemeld.insert((r, n.clone()));
                                self.meld(Melding::vraag(r, format!("Je gebruikt \"{}\", maar je hebt nog niet gezegd wat {} is.", n, n)).met_voorstel(
                                    format!("Bedoel je de tekst {}? Dan wordt de regel: {}", n, nieuw),
                                    Wijziging::VervangRegel { regel: r, nieuw },
                                ));
                            }
                        }
                    }
                    self.controleer_namen(r, namen, in_knop, None);
                    self.controleer_namen(r, vec![lijst.clone()], in_knop, None);
                    if let Err(t) = self.type_van(waarde) {
                        self.meld(Melding::fout(r, t));
                    }
                    let tl = self.types.get(lijst).map(|t| t.0).unwrap_or(Type::Onbekend);
                    if !matches!(tl, Type::Lijst | Type::Onbekend) {
                        self.meld(Melding::fout(
                            r,
                            format!("{} is {}, geen lijst. Maak eerst een lijst met: Maak een lijst met als naam {}.", lijst, type_naam(tl), lijst),
                        ));
                    }
                }
                Soort::VoorElk { lijst, blok, .. } => {
                    let mut namen = Vec::new();
                    lijst.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    match self.type_van(lijst) {
                        Ok(Type::Lijst) | Ok(Type::Onbekend) => {}
                        Ok(t) => self.meld(Melding::fout(r, format!("Voor elk ding in ... kan alleen bij een lijst. Maar {} is {}.", kort(lijst), type_naam(t)))),
                        Err(t) => self.meld(Melding::fout(r, t)),
                    }
                    self.controleer_lijst(blok, in_knop);
                }
                Soort::Tel { van, tot, blok, .. } => {
                    self.controleer_aantal(r, van, in_knop);
                    self.controleer_aantal(r, tot, in_knop);
                    if let (Some(a), Some(b)) = (vast_getal(van), vast_getal(tot)) {
                        if b - a > MAX_HERHALINGEN {
                            self.meld(Melding::fout(r, "Zo ver tellen mag niet. Het kan hoogstens een miljoen keer, anders loopt het programma vast."));
                        }
                    }
                    self.controleer_lijst(blok, in_knop);
                }
                Soort::Stukje { blok, .. } => self.controleer_lijst(blok, true),
                Soort::SchrijfBestand { waarde, pad, .. } | Soort::Document { waarde, pad, .. } => {
                    if let (Soort::Document { soort, .. }, Uitdrukking::Vast(Waarde::Tekst(t))) = (&o.soort, pad) {
                        let eind = if *soort == 0 { ".docx" } else { ".pdf" };
                        if !t.to_lowercase().ends_with(eind) {
                            self.meld(Melding::fout(r, format!("Dit bestand moet eindigen op {}, maar hier staat {}. Bedoel je {}{}?", eind, t, t, eind)));
                        }
                    }
                    for u in [waarde, pad] {
                        let mut od = Vec::new();
                        onderdelen_in(u, &mut od);
                        self.controleer_onderdelen_in(r, od);
                        let mut namen = Vec::new();
                        u.namen(&mut namen);
                        self.controleer_namen(r, namen, in_knop, None);
                        if let Err(t) = self.type_van(u) {
                            self.meld(Melding::fout(r, t));
                        }
                    }
                }
                Soort::Verstuur { waarde: a, adres: b } | Soort::Download { adres: a, pad: b } => {
                    for u in [a, b] {
                        let mut namen = Vec::new();
                        u.namen(&mut namen);
                        self.controleer_namen(r, namen, in_knop, None);
                        if let Err(t) = self.type_van(u) {
                            self.meld(Melding::fout(r, t));
                        }
                    }
                    let adres = if let Soort::Verstuur { adres, .. } = &o.soort { adres } else { a };
                    if let Uitdrukking::Vast(Waarde::Tekst(t)) = adres {
                        if !t.starts_with("http://") && !t.starts_with("https://") {
                            self.meld(Melding::fout(r, format!("Een internetadres begint met https:// of http://, maar hier staat {}.", t)));
                        }
                    }
                }
                Soort::Tabel { tabel, actie } => {
                    self.controleer_onderdeel(r, tabel, &[OnderdeelSoort::Tabel], "Dit kan alleen bij een tabel.");
                    let mut uitdr: Vec<&Uitdrukking> = Vec::new();
                    match actie {
                        TabelActie::Rij(w) => uitdr.extend(w.iter()),
                        TabelActie::Opslaan(p) | TabelActie::Inlezen(p) => uitdr.push(p),
                        _ => {}
                    }
                    for u in uitdr {
                        let mut namen = Vec::new();
                        u.namen(&mut namen);
                        self.controleer_namen(r, namen, in_knop, None);
                        if let Err(t) = self.type_van(u) {
                            self.meld(Melding::fout(r, t));
                        }
                    }
                }
                Soort::ZoekUpdates => {
                    let inst = crate::installatie::uit_programma(self.programma);
                    let heeft_adres = self.programma.opdrachten.iter().any(|o| matches!(o.soort, Soort::Installatie(InstallatieZin::UpdateAdres(_))));
                    if !heeft_adres || inst.as_ref().map_or(true, |i| i.versie.is_empty()) {
                        self.meld(Melding::vraag(
                            r,
                            "Om updates te zoeken heb ik twee regels nodig: De versie is ... en De updates staan op het adres ... . Welke versie en welk adres bedoel je?",
                        ));
                    }
                }
                Soort::Doe(u) | Soort::GeefUitkomst(u) | Soort::MaakMap(u) | Soort::Logboek(u) => {
                    let mut od = Vec::new();
                    onderdelen_in(u, &mut od);
                    self.controleer_onderdelen_in(r, od);
                    let mut namen = Vec::new();
                    u.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    if let Err(t) = self.type_van(u) {
                        self.meld(Melding::fout(r, t));
                    }
                }
                Soort::Piep { hoogte, duur } => {
                    for (u, laag, hoog, wat) in [(hoogte, 37.0, 32767.0, "De hoogte van een piep"), (duur, 1.0, 10000.0, "De duur van een piep")] {
                        let mut namen = Vec::new();
                        u.namen(&mut namen);
                        self.controleer_namen(r, namen, in_knop, None);
                        match self.type_van(u) {
                            Err(t) => self.meld(Melding::fout(r, t)),
                            Ok(Type::Tekst | Type::Waarheid | Type::Lijst) => self.meld(Melding::fout(r, format!("{} moet een getal zijn.", wat))),
                            _ => {}
                        }
                        if let Some(g) = vast_getal(u) {
                            if g < laag || g > hoog {
                                self.meld(Melding::fout(
                                    r,
                                    format!("{} moet tussen {} en {} liggen.", wat, crate::getallen::schrijf(laag), crate::getallen::schrijf(hoog)),
                                ));
                            }
                        }
                    }
                }
                Soort::SpeelGeluid { pad, .. } => {
                    let mut namen = Vec::new();
                    pad.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    if let Uitdrukking::Vast(Waarde::Tekst(t)) = pad {
                        let laag = t.to_lowercase();
                        if ![".wav", ".mp3", ".wma", ".m4a", ".aac", ".mid", ".midi"].iter().any(|e| laag.ends_with(e)) {
                            self.meld(Melding::fout(
                                r,
                                format!("Ik kan WAV, MP3, WMA, M4A, AAC en MIDI afspelen. {} is een ander soort bestand.", t),
                            ));
                        }
                    }
                }
                Soort::Zip { map, zip, .. } => {
                    for u in [map, zip] {
                        let mut namen = Vec::new();
                        u.namen(&mut namen);
                        self.controleer_namen(r, namen, in_knop, None);
                    }
                    if let Uitdrukking::Vast(Waarde::Tekst(t)) = zip {
                        if !t.to_lowercase().ends_with(".zip") {
                            self.meld(Melding::fout(r, format!("Een zipbestand eindigt op .zip, maar hier staat {}. Bedoel je {}.zip?", t, t)));
                        }
                    }
                }
                Soort::MaakMenu { .. } | Soort::MaakMenuregel { .. } | Soort::StopVoorlezen | Soort::MaakTabblad { .. } => {}
                Soort::KiesTabblad { naam } => {
                    if !self.tabbladen.iter().any(|t| t.to_lowercase() == naam.to_lowercase()) {
                        let lijkt = self.tabbladen.iter().find(|t| tekst::mag_tikfout_zijn(&naam.to_lowercase(), &t.to_lowercase())).cloned();
                        let m = match lijkt {
                            Some(goed) => {
                                let nieuw = format!("Ga naar het tabblad {}", goed);
                                Melding::vraag(r, format!("Er is geen tabblad met de naam {}.", naam))
                                    .met_voorstel(format!("Bedoel je: {}?", nieuw), Wijziging::VervangRegel { regel: r, nieuw })
                            }
                            None => Melding::vraag(r, format!("Er is geen tabblad met de naam {}. Welk tabblad bedoel je?", naam)),
                        };
                        self.meld(m);
                    }
                }
                Soort::Volume(u) => {
                    let mut namen = Vec::new();
                    u.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    match self.type_van(u) {
                        Err(t) => self.meld(Melding::fout(r, t)),
                        Ok(Type::Tekst | Type::Waarheid | Type::Lijst) => self.meld(Melding::fout(r, "Het volume is een getal van 0 tot 100.")),
                        _ => {}
                    }
                    if let Some(g) = vast_getal(u) {
                        if !(0.0..=100.0).contains(&g) {
                            self.meld(Melding::fout(r, "Het volume is een getal van 0 tot 100."));
                        }
                    }
                }
                Soort::BestandActie { van, naar, .. } => {
                    let mut lijst = vec![van];
                    if let Some(n) = naar {
                        lijst.push(n);
                    }
                    for u in lijst {
                        let mut namen = Vec::new();
                        u.namen(&mut namen);
                        self.controleer_namen(r, namen, in_knop, None);
                        if let Err(t) = self.type_van(u) {
                            self.meld(Melding::fout(r, t));
                        }
                    }
                }
                Soort::Zeg(u) | Soort::LeesVoor(u) => {
                    if let Uitdrukking::Naam(n) = u {
                        if !self.is_bekend(n, r, in_knop) && !self.onthoud.iter().any(|o| tekst::mag_tikfout_zijn(n, &o.0)) {
                            if let Some(nieuw) = aanhalen(self.regels.get(r - 1).copied().unwrap_or(""), n) {
                                self.al_gemeld.insert((r, n.clone()));
                                self.meld(Melding::vraag(r, format!("Je gebruikt \"{}\", maar je hebt nog niet gezegd wat {} is.", n, n)).met_voorstel(
                                    format!("Bedoel je de tekst {}? Dan wordt de regel: {}", n, nieuw),
                                    Wijziging::VervangRegel { regel: r, nieuw },
                                ));
                            }
                        }
                    }
                    let mut namen = Vec::new();
                    u.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    if let Err(t) = self.type_van(u) {
                        self.meld(Melding::fout(r, t));
                    }
                }
                Soort::Pictogram(b) => {
                    if !b.to_lowercase().ends_with(".ico") {
                        self.meld(Melding::fout(r, format!("Een pictogram is een .ico-bestand, maar hier staat {}.", b)));
                    }
                }
                Soort::StopGeluid | Soort::Sneltoets { .. } | Soort::Kleur { .. } | Soort::Handleiding { .. } => {}
                Soort::MaakOnderdeel { .. } => {}
                Soort::VoegKeuzeToe { keuze, lijst } => {
                    let mut namen = Vec::new();
                    keuze.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    let mut od = Vec::new();
                    onderdelen_in(keuze, &mut od);
                    self.controleer_onderdelen_in(r, od);
                    self.controleer_onderdeel(
                        r,
                        lijst,
                        &[OnderdeelSoort::Keuzelijst, OnderdeelSoort::Uitklaplijst],
                        "Een keuze toevoegen kan alleen bij een keuzelijst of een uitklaplijst.",
                    );
                }
                Soort::ZetIn { waarde, naam } => {
                    let mut namen = Vec::new();
                    waarde.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, None);
                    let mut od = Vec::new();
                    onderdelen_in(waarde, &mut od);
                    self.controleer_onderdelen_in(r, od);
                    if let Err(t) = self.type_van(waarde) {
                        self.meld(Melding::fout(r, t));
                    }
                    self.controleer_onderdeel(
                        r,
                        naam,
                        &[OnderdeelSoort::Tekstvak, OnderdeelSoort::GrootTekstvak, OnderdeelSoort::Label],
                        "Tekst erin zetten kan alleen bij een tekstvak of een label.",
                    );
                }
                Soort::Vink { naam, .. } => {
                    self.controleer_onderdeel(r, naam, &[OnderdeelSoort::Selectievakje], "Aan- en uitvinken kan alleen bij een selectievakje.")
                }
                Soort::Focus { naam } => self.controleer_onderdeel(r, naam, &[], ""),
                Soort::Toon(u) => {
                    let mut od = Vec::new();
                    onderdelen_in(u, &mut od);
                    self.controleer_onderdelen_in(r, od);
                    let mut namen = Vec::new();
                    u.namen(&mut namen);
                    self.controleer_namen(r, namen, in_knop, Some(u));
                    if let Err(t) = self.type_van(u) {
                        self.meld(Melding::fout(r, t));
                    }
                }
                Soort::Onthoud { naam, waarde, .. } | Soort::Verander { naam, waarde } => {
                    let mut od = Vec::new();
                    onderdelen_in(waarde, &mut od);
                    self.controleer_onderdelen_in(r, od);
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
                                if matches!(tt, Type::Tekst | Type::Waarheid | Type::Lijst) {
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
    let mut c = Controle {
        regels,
        programma: p,
        meldingen: Vec::new(),
        types: HashMap::new(),
        onthoud: Vec::new(),
        al_gemeld: HashSet::new(),
        tabbladen: p
            .alle()
            .iter()
            .filter_map(|o| match &o.soort {
                Soort::MaakTabblad { naam } => Some(naam.clone()),
                _ => None,
            })
            .collect(),
        onderdelen: p
            .alle()
            .iter()
            .filter_map(|o| match &o.soort {
                Soort::MaakOnderdeel { soort, naam } => Some((naam.clone(), *soort, o.regel)),
                _ => None,
            })
            .collect(),
    };

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
                Soort::VoorElk { naam, blok, .. } | Soort::Tel { naam, blok, .. } => {
                    let t = if matches!(o.soort, Soort::Tel { .. }) { Type::Getal } else { Type::Onbekend };
                    c.types.entry(naam.clone()).or_insert((t, o.regel));
                    c.onthoud.push((naam.clone(), o.regel, false));
                    verzamel(c, blok, false)
                }
                Soort::Stukje { invoer, blok, .. } => {
                    for i in invoer {
                        c.types.entry(i.clone()).or_insert((Type::Onbekend, o.regel));
                        c.onthoud.push((i.clone(), o.regel, false));
                    }
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
            Soort::MaakKnop { naam } | Soort::MaakMenuregel { naam } => Some((naam.clone(), o.regel)),
            _ => None,
        })
        .collect();
    let mut op_scherm: Vec<(String, usize)> = alle
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::MaakKnop { naam }
            | Soort::MaakOnderdeel { naam, .. }
            | Soort::MaakMenu { naam }
            | Soort::MaakTabblad { naam }
            | Soort::MaakMenuregel { naam } => Some((naam.clone(), o.regel)),
            _ => None,
        })
        .collect();
    op_scherm.sort_by_key(|x| x.1);
    let acties: Vec<(String, usize)> = alle
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::BijKnop { naam, .. } => Some((naam.clone(), o.regel)),
            _ => None,
        })
        .collect();

    if let Some((naam, regel)) = op_scherm.iter().find(|(_, r)| !vensters.iter().any(|v| v < r)) {
        let nieuw = "Maak een venster met als titel Mijn programma".to_string();
        c.meld(
            Melding::vraag(*regel, format!("Je maakt {}, maar er is nog geen venster om het in te zetten.", naam))
                .met_voorstel(
                    format!("Zal ik er eerst een venster voor maken, met de regel: {}?", nieuw),
                    Wijziging::VoegInVoor { regel: *regel, nieuw },
                ),
        );
    }
    for (i, (naam, regel)) in op_scherm.iter().enumerate() {
        if let Some((_, eerste)) = op_scherm[..i].iter().find(|(n, _)| n.to_lowercase() == naam.to_lowercase()) {
            c.meld(Melding::vraag(
                *regel,
                format!(
                    "Er is al een onderdeel met de naam {}, op regel {}. Twee onderdelen met dezelfde naam kan ik niet uit elkaar houden. Welke naam moet dit krijgen?",
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
    controleer_stukjes(&mut c, p);
    controleer_scherm(&mut c, p);
    // Een menuregel hoort in een menu.
    let mut menu_gezien = false;
    let mut gesorteerd = p.alle();
    gesorteerd.sort_by_key(|o| o.regel);
    for o in gesorteerd {
        match &o.soort {
            Soort::MaakVenster { .. } => menu_gezien = false,
            Soort::MaakMenu { .. } => menu_gezien = true,
            Soort::MaakMenuregel { naam } if !menu_gezien => {
                let nieuw = "Maak een menu met als naam Bestand".to_string();
                c.meld(
                    Melding::vraag(o.regel, format!("Je maakt de menuregel {}, maar er is nog geen menu om hem in te zetten.", naam))
                        .met_voorstel(format!("Zal ik er eerst een menu voor maken, met de regel: {}?", nieuw), Wijziging::VoegInVoor { regel: o.regel, nieuw }),
                );
                break;
            }
            _ => {}
        }
    }

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
    dubbel(c, "het bestand van de privacyverklaring", regels_van(&|z| matches!(z, InstallatieZin::PrivacyBestand(_))));
    dubbel(c, "de naam van het certificaat", regels_van(&|z| matches!(z, InstallatieZin::Onderteken(_))));
    let p_tekst = regels_van(&|z| matches!(z, InstallatieZin::PrivacyRegel(_)));
    let p_bestand = regels_van(&|z| matches!(z, InstallatieZin::PrivacyBestand(_)));
    if let (Some(t), Some(b)) = (p_tekst.first(), p_bestand.first()) {
        c.meld(Melding::vraag(
            *t.max(b),
            format!(
                "Je schrijft de privacyverklaring op regel {} zelf, en op regel {} zeg je dat hij in een bestand staat. Welke van de twee bedoel je?",
                t, b
            ),
        ));
    }
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

/// Zet de naam (uit de regel) tussen aanhalingstekens: Voeg melk toe ... wordt Voeg "melk" toe ...
fn aanhalen(regel: &str, naam: &str) -> Option<String> {
    let norm = tekst::normaliseer(regel);
    let woorden: Vec<&str> = norm.split(' ').collect();
    let oud: Vec<&str> = naam.split(' ').collect();
    for i in 0..woorden.len() {
        if i + oud.len() <= woorden.len() && oud.iter().enumerate().all(|(j, o)| woorden[i + j].to_lowercase() == *o) {
            let mut uit: Vec<String> = woorden[..i].iter().map(|s| s.to_string()).collect();
            uit.push(format!("\"{}\"", woorden[i..i + oud.len()].join(" ")));
            uit.extend(woorden[i + oud.len()..].iter().map(|s| s.to_string()));
            return Some(uit.join(" "));
        }
    }
    None
}

/// Eigen stukjes: bestaan ze, krijgen ze genoeg invoer, en zijn de namen uniek?
fn controleer_stukjes(c: &mut Controle, p: &Programma) {
    let stukjes: Vec<(String, Vec<String>, usize)> = p
        .alle()
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::Stukje { naam, invoer, .. } => Some((naam.clone(), invoer.clone(), o.regel)),
            _ => None,
        })
        .collect();
    for (i, (naam, _, regel)) in stukjes.iter().enumerate() {
        if let Some((_, _, eerste)) = stukjes[..i].iter().find(|s| &s.0 == naam) {
            c.meld(Melding::vraag(
                *regel,
                format!("Er is al een stukje met de naam {}, op regel {}. Dat spreekt elkaar tegen. Welke naam moet dit stukje krijgen?", naam, eerste),
            ));
        }
        if let Some((_, r, _)) = c.onthoud.iter().find(|o| &o.0 == naam).cloned() {
            c.meld(Melding::vraag(
                *regel,
                format!("{} is al de naam van iets dat je onthoudt (regel {}). Een stukje moet een andere naam hebben.", naam, r),
            ));
        }
    }
    for o in p.alle() {
        let mut aanroepen = Vec::new();
        for u in o.soort.uitdrukkingen() {
            u.aanroepen(&mut aanroepen);
        }
        for (naam, aantal) in aanroepen {
            match stukjes.iter().find(|s| s.0 == naam) {
                Some((_, invoer, _)) if invoer.len() == aantal => {}
                Some((_, invoer, _)) => c.meld(Melding::fout(
                    o.regel,
                    if invoer.is_empty() {
                        format!("Het stukje {} heeft geen invoer nodig, maar je geeft er {}.", naam, aantal)
                    } else {
                        format!(
                            "Het stukje {} heeft {} invoer nodig ({}), maar je geeft er {}. Schrijf bijvoorbeeld: Doe {} met {}.",
                            naam,
                            invoer.len(),
                            invoer.join(" en "),
                            aantal,
                            naam,
                            invoer.iter().map(|_| "1").collect::<Vec<_>>().join(" en ")
                        )
                    },
                )),
                None => {
                    let lijkt_op = stukjes
                        .iter()
                        .map(|s| s.0.clone())
                        .filter(|s| tekst::mag_tikfout_zijn(&naam, s))
                        .min_by_key(|s| tekst::afstand(&naam, s));
                    let regeltekst = c.regels.get(o.regel - 1).copied().unwrap_or("");
                    let m = match lijkt_op.as_ref().and_then(|g| vervang_naam(regeltekst, &naam, g).map(|n| (g.clone(), n))) {
                        Some((goed, nieuw)) => Melding::vraag(o.regel, format!("Er is geen stukje met de naam {}.", naam)).met_voorstel(
                            format!("Bedoel je {}? Dan wordt de regel: {}", goed, nieuw),
                            Wijziging::VervangRegel { regel: o.regel, nieuw },
                        ),
                        None => Melding::vraag(
                            o.regel,
                            format!(
                                "Er is geen stukje met de naam {}. Welk stukje bedoel je? Een stukje maak je met: Maak een stukje met als naam {}.",
                                naam, naam
                            ),
                        ),
                    };
                    c.meld(m);
                }
            }
        }
    }
}

/// Sneltoetsen, kleuren (is de tekst goed te lezen?) en de handleiding.
fn controleer_scherm(c: &mut Controle, p: &Programma) {
    let mut alle = p.alle();
    alle.sort_by_key(|o| o.regel);
    let knoppen: Vec<String> = alle
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::MaakKnop { naam } | Soort::MaakMenuregel { naam } => Some(naam.to_lowercase()),
            _ => None,
        })
        .collect();
    let mut toetsen: Vec<(String, String, usize)> = Vec::new();
    // Kleuren per venster: (achtergrond, tekst). Standaard: lichtgrijs met zwarte tekst.
    let standaard = ((240u8, 240u8, 240u8), (0u8, 0u8, 0u8));
    let mut kleuren: Option<((u8, u8, u8), (u8, u8, u8))> = None;
    let mut kleurnamen = ("lichtgrijs".to_string(), "zwart".to_string());
    let mut handleiding_regel = None;
    let mut handleiding_bestand = None;
    let mut laatste_kleur: Option<(usize, bool, (u8, u8, u8), (u8, u8, u8), (String, String))> = None;
    let los: Vec<usize> = p.opdrachten.iter().map(|o| o.regel).collect();
    for o in &alle {
        let r = o.regel;
        match &o.soort {
            Soort::MaakVenster { .. } => {
                if let Some(k) = laatste_kleur.take() {
                    controleer_contrast(c, k);
                }
                kleuren = Some(standaard);
                kleurnamen = ("lichtgrijs".into(), "zwart".into());
            }
            Soort::Sneltoets { knop, toets } => {
                if !knoppen.contains(&knop.to_lowercase()) {
                    c.meld(Melding::vraag(r, format!("Er is geen knop met de naam {}. Welke knop moet de sneltoets {} krijgen?", knop, toets.naam)));
                }
                if let Some((andere, _, eerder)) = toetsen.iter().find(|t| t.1 == toets.naam && t.0 != knop.to_lowercase()) {
                    c.meld(Melding::vraag(
                        r,
                        format!(
                            "De sneltoets {} hoort al bij de knop {} (regel {}). Twee knoppen met dezelfde sneltoets spreken elkaar tegen. Welke sneltoets moet deze knop krijgen?",
                            toets.naam, andere, eerder
                        ),
                    ));
                }
                toetsen.push((knop.to_lowercase(), toets.naam.clone(), r));
            }
            Soort::Kleur { achtergrond, kleur, rgb } => {
                let Some((mut bg, mut fg)) = kleuren else {
                    let nieuw = "Maak een venster met als titel Mijn programma".to_string();
                    c.meld(Melding::vraag(r, "Je kiest een kleur, maar er is nog geen venster.").met_voorstel(
                        format!("Zal ik er eerst een venster voor maken, met de regel: {}?", nieuw),
                        Wijziging::VoegInVoor { regel: r, nieuw },
                    ));
                    continue;
                };
                if *achtergrond {
                    bg = *rgb;
                    kleurnamen.0 = kleur.clone();
                } else {
                    fg = *rgb;
                    kleurnamen.1 = kleur.clone();
                }
                kleuren = Some((bg, fg));
                laatste_kleur = Some((r, *achtergrond, bg, fg, kleurnamen.clone()));
            }
            Soort::Handleiding { regel, bestand } => {
                if !los.contains(&r) {
                    c.meld(Melding::fout(r, "Zinnen over de handleiding moeten los staan, niet binnen een blok."));
                }
                if regel.is_some() {
                    handleiding_regel.get_or_insert(r);
                }
                if bestand.is_some() {
                    if let Some(eerder) = handleiding_bestand {
                        c.meld(Melding::vraag(r, format!("Je noemt twee keer een bestand voor de handleiding (ook op regel {}). Welke bedoel je?", eerder)));
                    }
                    handleiding_bestand.get_or_insert(r);
                }
            }
            _ => {}
        }
    }
    if let Some(k) = laatste_kleur.take() {
        controleer_contrast(c, k);
    }
    if let (Some(a), Some(b)) = (handleiding_regel, handleiding_bestand) {
        c.meld(Melding::vraag(
            a.max(b),
            format!("Je schrijft de handleiding op regel {} zelf, en op regel {} zeg je dat hij in een bestand staat. Welke van de twee bedoel je?", a, b),
        ));
    }
}

/// Is de tekst goed te lezen op de achtergrond? Gecontroleerd bij de laatste kleurregel van een venster.
fn controleer_contrast(c: &mut Controle, k: (usize, bool, (u8, u8, u8), (u8, u8, u8), (String, String))) {
    let (r, achtergrond, bg, fg, (bgnaam, fgnaam)) = k;
    let ct = contrast(bg, fg);
    if ct >= 4.5 {
        return;
    }
    let beter = if contrast(bg, (0, 0, 0)) >= contrast(bg, (255, 255, 255)) { "zwart" } else { "wit" };
    let tekst = format!(
        "Tekst in {} op een achtergrond van {} is slecht te lezen. Het contrast is {}; goed is 4,5 of meer.",
        fgnaam,
        bgnaam,
        crate::getallen::schrijf((ct * 10.0).round() / 10.0)
    );
    let nieuw = format!("Geef de tekst de kleur {}", beter);
    let m = if achtergrond {
        Melding::vraag(r, tekst).met_voorstel(format!("Zal ik eronder zetten: {}?", nieuw), Wijziging::VoegInVoor { regel: r + 1, nieuw })
    } else {
        Melding::vraag(r, tekst).met_voorstel(
            format!("Zal ik er {} van maken? Dan wordt de regel: {}", beter, nieuw),
            Wijziging::VervangRegel { regel: r, nieuw },
        )
    };
    c.meld(m);
}
