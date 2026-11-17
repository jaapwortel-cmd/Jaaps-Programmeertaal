//! Vertelt in gewone taal wat de motor van je zinnen heeft gemaakt.

use crate::getallen;
use crate::taal::*;

fn uitdrukking(u: &Uitdrukking) -> String {
    match u {
        Uitdrukking::Vast(Waarde::Getal(g)) => format!("het getal {}", getallen::schrijf(*g)),
        Uitdrukking::Vast(Waarde::Tekst(t)) => format!("de tekst \"{}\"", t),
        Uitdrukking::Vast(Waarde::Waarheid(true)) => "waar".into(),
        Uitdrukking::Vast(Waarde::Waarheid(false)) => "niet waar".into(),
        Uitdrukking::Naam(n) => format!("wat er in {} staat", n),
        Uitdrukking::Onderdeel { naam, als_getal: false } => format!("wat er in het onderdeel {} staat", naam),
        Uitdrukking::Onderdeel { naam, als_getal: true } => format!("het getal in het onderdeel {}", naam),
        Uitdrukking::Negatief(a) => format!("min {}", uitdrukking(a)),
        Uitdrukking::LegeLijst => "een lege lijst".into(),
        Uitdrukking::Ingebouwd { wat, invoer } => {
            let a = |i: usize| invoer.get(i).map(uitdrukking).unwrap_or_default();
            match wat {
                Ingebouwd::Inhoud => format!("de tekst in het bestand {}", a(0)),
                Ingebouwd::Regels => format!("de regels van het bestand {} (als lijst)", a(0)),
                Ingebouwd::Bestaat => format!("of het bestand {} bestaat", a(0)),
                Ingebouwd::LogboekPlek => "de plek van het logboek".into(),
                Ingebouwd::Willekeurig => format!("een willekeurig getal van {} tot en met {}", a(0), a(1)),
                Ingebouwd::Tijd => "de tijd van dit moment".into(),
                Ingebouwd::Datum => "de datum van vandaag".into(),
            }
        }
        Uitdrukking::Aantal(a) => format!("het aantal dingen in {}", uitdrukking(a)),
        Uitdrukking::Element(a, b) => format!("ding nummer {} uit {}", uitdrukking(b), uitdrukking(a)),
        Uitdrukking::Aanroep { naam, invoer } if invoer.is_empty() => format!("de uitkomst van het stukje {}", naam),
        Uitdrukking::Aanroep { naam, invoer } => format!(
            "de uitkomst van het stukje {} met als invoer {}",
            naam,
            invoer.iter().map(uitdrukking).collect::<Vec<_>>().join(" en ")
        ),
        Uitdrukking::Bewerking(a, t, b) => {
            let woord = match t {
                Teken::EnDan => "met daarachter",
                t => t.woord(),
            };
            format!("({} {} {})", uitdrukking(a), woord, uitdrukking(b))
        }
    }
}

fn voorwaarde(v: &Voorwaarde) -> String {
    match v {
        Voorwaarde::Vergelijk(a, verg, b) => format!("{} {} {}", uitdrukking(a), verg.woorden(), uitdrukking(b)),
        Voorwaarde::IsWaar(a) => format!("{} waar is", uitdrukking(a)),
        Voorwaarde::Bevat(a, b) => format!("{} {} bevat", uitdrukking(a), uitdrukking(b)),
    }
}

fn lijst_(l: &[Opdracht], diepte: usize, uit: &mut Vec<String>) {
    let insp = "    ".repeat(diepte);
    for o in l {
        let r = o.regel;
        let zin = match &o.soort {
            Soort::Opmerking(t) => format!("een opmerking die niets doet: {}", t),
            Soort::SchrijfBestand { waarde, pad, toevoegen } => format!(
                "{} {} {} het bestand {}.",
                if *toevoegen { "voeg" } else { "schrijf" },
                uitdrukking(waarde),
                if *toevoegen { "als nieuwe regel toe aan" } else { "in (wat er stond, verdwijnt)" },
                uitdrukking(pad)
            ),
            Soort::MaakMap(p) => format!("maak de map {}.", uitdrukking(p)),
            Soort::Piep { hoogte, duur } => format!("laat een piep horen van {} hertz, {} milliseconden lang.", uitdrukking(hoogte), uitdrukking(duur)),
            Soort::SpeelGeluid { pad, wachten } => format!(
                "speel het geluid {} af{}.",
                uitdrukking(pad),
                if *wachten { " en wacht tot het klaar is" } else { " (het programma gaat meteen verder)" }
            ),
            Soort::StopGeluid => "stop het geluid.".into(),
            Soort::Sneltoets { knop, toets } => format!("de knop {} krijgt de sneltoets {}.", knop, toets.naam),
            Soort::Kleur { achtergrond: true, kleur, .. } => format!("de achtergrond van het laatste venster wordt {}.", kleur),
            Soort::Kleur { achtergrond: false, kleur, .. } => format!("de tekst in het laatste venster wordt {}.", kleur),
            Soort::Handleiding { regel: Some(t), .. } => format!("in de handleiding (F1) staat: {}", t),
            Soort::Handleiding { bestand: Some(b), .. } => format!("de handleiding (F1) staat in het bestand {}.", b),
            Soort::Handleiding { .. } => "een handleiding.".into(),
            Soort::Logboek(w) => format!("schrijf {} in het logboek, met de datum en tijd.", uitdrukking(w)),
            Soort::VoegToeAanLijst { waarde, lijst } => format!("voeg {} toe aan de lijst {}.", uitdrukking(waarde), lijst),
            Soort::HaalUitLijst { waarde, lijst } => format!("haal {} weg uit de lijst {}.", uitdrukking(waarde), lijst),
            Soort::Doe(a) => format!("voer uit: {}.", uitdrukking(a).replacen("de uitkomst van ", "", 1)),
            Soort::GeefUitkomst(a) => format!("stop het stukje en geef als uitkomst {}.", uitdrukking(a)),
            Soort::VoorElk { naam, lijst, blok, einde } => {
                uit.push(format!("{}Regel {}: doe dit voor elk ding in {}. Het ding heet dan {}:", insp, r, uitdrukking(lijst), naam));
                lijst_(blok, diepte + 1, uit);
                uit.push(format!("{}Regel {}: einde van de herhaling.", insp, einde));
                continue;
            }
            Soort::Tel { naam, van, tot, blok, einde } => {
                uit.push(format!("{}Regel {}: tel {} van {} tot en met {}, en doe elke keer dit:", insp, r, naam, uitdrukking(van), uitdrukking(tot)));
                lijst_(blok, diepte + 1, uit);
                uit.push(format!("{}Regel {}: einde van de herhaling.", insp, einde));
                continue;
            }
            Soort::Stukje { naam, invoer, blok, einde } => {
                let met = if invoer.is_empty() { String::new() } else { format!(", met als invoer {}", invoer.join(" en ")) };
                uit.push(format!("{}Regel {}: een eigen stukje met de naam {}{}. Dit doet het:", insp, r, naam, met));
                lijst_(blok, diepte + 1, uit);
                uit.push(format!("{}Regel {}: einde van het stukje {}.", insp, einde, naam));
                continue;
            }
            Soort::MaakOnderdeel { soort, naam } => {
                format!("maak een {} in het laatste venster. De naam is: {}.", soort.woord(), naam)
            }
            Soort::VoegKeuzeToe { keuze, lijst } => format!("voeg {} toe aan de keuzelijst {}.", uitdrukking(keuze), lijst),
            Soort::ZetIn { waarde, naam } => format!("zet {} in het onderdeel {}.", uitdrukking(waarde), naam),
            Soort::Vink { naam, aan } => format!("vink het selectievakje {} {}.", naam, if *aan { "aan" } else { "uit" }),
            Soort::Focus { naam } => format!("zet de focus op het onderdeel {}.", naam),
            Soort::Installatie(z) => match z {
                InstallatieZin::Naam(n) => format!("het installatiebestand heet: {} installatie.exe.", n),
                InstallatieZin::Versie(v) => format!("de versie is {}.", v),
                InstallatieZin::Maker(m) => format!("de maker is {}.", m),
                InstallatieZin::Bureaublad => "bij het installeren komt er een snelkoppeling op het bureaublad.".into(),
                InstallatieZin::Startmenu => "bij het installeren komt er een snelkoppeling in het startmenu.".into(),
                InstallatieZin::Rechten(Rechten::MetEnZonder) => {
                    "installeren mag voor jezelf (zonder beheerdersrechten) of voor iedereen (met beheerdersrechten).".into()
                }
                InstallatieZin::Rechten(Rechten::AlleenZonder) => "installeren gaat alleen voor jezelf, zonder beheerdersrechten.".into(),
                InstallatieZin::Rechten(Rechten::AlleenMet) => "installeren gaat voor iedereen, met beheerdersrechten.".into(),
                InstallatieZin::Bestand(b) => format!("het bestand {} gaat mee in de installatie.", b),
                InstallatieZin::LicentieRegel(t) => format!("in de licentie staat: {}", t),
                InstallatieZin::LicentieBestand(b) => format!("de licentie staat in het bestand {}.", b),
            },
            Soort::MaakVenster { titel } => format!("maak een venster. De titel is: {}.", titel),
            Soort::MaakKnop { naam } => format!("maak een knop in het laatste venster. De naam is: {}.", naam),
            Soort::Toon(u) => format!("toon {}.", uitdrukking(u)),
            Soort::Onthoud { naam, waarde, .. } => format!("onthoud {} onder de naam {}.", uitdrukking(waarde), naam),
            Soort::Verander { naam, waarde } => format!("verander {} in {}.", naam, uitdrukking(waarde)),
            Soort::TelOp { naam, waarde } => format!("tel {} op bij {}.", uitdrukking(waarde), naam),
            Soort::TrekAf { naam, waarde } => format!("trek {} af van {}.", uitdrukking(waarde), naam),
            Soort::BijKnop { naam, blok, einde } => {
                uit.push(format!("{}Regel {}: als iemand op de knop {} drukt, doe dan dit:", insp, r, naam));
                lijst_(blok, diepte + 1, uit);
                uit.push(format!("{}Regel {}: einde van wat de knop {} doet.", insp, einde, naam));
                continue;
            }
            Soort::Als { voorwaarde: v, dan, anders, anders_regel, einde } => {
                uit.push(format!("{}Regel {}: kijk of {}. Zo ja, doe dan dit:", insp, r, voorwaarde(v)));
                lijst_(dan, diepte + 1, uit);
                if let Some(a) = anders_regel {
                    uit.push(format!("{}Regel {}: zo nee, doe dan dit:", insp, a));
                    lijst_(anders, diepte + 1, uit);
                }
                uit.push(format!("{}Regel {}: einde van de keuze.", insp, einde));
                continue;
            }
            Soort::Herhaal { aantal, blok, einde } => {
                uit.push(format!("{}Regel {}: doe dit {} keer:", insp, r, uitdrukking(aantal)));
                lijst_(blok, diepte + 1, uit);
                uit.push(format!("{}Regel {}: einde van de herhaling.", insp, einde));
                continue;
            }
            Soort::HerhaalZolang { voorwaarde: v, hoogstens, blok, einde } => {
                uit.push(format!(
                    "{}Regel {}: doe dit zolang {}, maar hoogstens {} keer:",
                    insp,
                    r,
                    voorwaarde(v),
                    uitdrukking(hoogstens)
                ));
                lijst_(blok, diepte + 1, uit);
                uit.push(format!("{}Regel {}: einde van de herhaling.", insp, einde));
                continue;
            }
        };
        uit.push(format!("{}Regel {}: {}", insp, r, zin));
    }
}

/// Een beschrijving in gewone taal, één regel per opdracht.
pub fn beschrijf(p: &Programma) -> String {
    let mut uit = vec!["Zo begrijp ik je programma:".to_string()];
    if p.opdrachten.is_empty() {
        uit.push("Er staan nog geen opdrachten in.".into());
    }
    lijst_(&p.opdrachten, 0, &mut uit);
    uit.join("\n")
}
