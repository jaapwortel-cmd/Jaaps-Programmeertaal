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
                Ingebouwd::Webpagina => format!("wat er op de webpagina {} staat", a(0)),
                Ingebouwd::WebTekst => format!("de leesbare tekst van de webpagina {}", a(0)),
                Ingebouwd::GekozenRij => format!("de gekozen rij van de tabel {} (als lijst)", a(0)),
                Ingebouwd::AantalRijen => format!("het aantal rijen in de tabel {}", a(0)),
                Ingebouwd::RijNummer => format!("rij nummer {} van de tabel {} (als lijst)", a(1), a(0)),
                Ingebouwd::Wortel => format!("de wortel van {}", a(0)),
                Ingebouwd::Macht => format!("{} tot de macht {}", a(0), a(1)),
                Ingebouwd::Afgerond if matches!(invoer.get(1), Some(Uitdrukking::Vast(Waarde::Getal(g))) if *g == 0.0) => {
                    format!("{} afgerond op een heel getal", a(0))
                }
                Ingebouwd::Afgerond => format!("{} afgerond op {} cijfers achter de komma", a(0), a(1)),
                Ingebouwd::Rest => format!("de rest bij het delen van {} door {}", a(0), a(1)),
                Ingebouwd::Absoluut => format!("{} zonder min ervoor", a(0)),
                Ingebouwd::AlsGetal => format!("{} als getal", a(0)),
                Ingebouwd::AlsTekst => format!("{} als tekst", a(0)),
                Ingebouwd::Lengte => format!("het aantal tekens van {}", a(0)),
                Ingebouwd::Hoofdletters => format!("{} in hoofdletters", a(0)),
                Ingebouwd::KleineLetters => format!("{} in kleine letters", a(0)),
                Ingebouwd::EersteTekens => format!("de eerste {} tekens van {}", a(1), a(0)),
                Ingebouwd::LaatsteTekens => format!("de laatste {} tekens van {}", a(1), a(0)),
                Ingebouwd::Woorden => format!("de woorden van {} (als lijst)", a(0)),
                Ingebouwd::Gesplitst => format!("{} in stukken gehakt bij {} (als lijst)", a(0), a(1)),
                Ingebouwd::Samengevoegd => format!("de lijst {} aan elkaar geplakt met {} ertussen", a(0), a(1)),
                Ingebouwd::Vervangen => format!("{} waarin {} vervangen is door {}", a(0), a(1), a(2)),
                Ingebouwd::ZonderSpaties => format!("{} zonder spaties aan het begin en eind", a(0)),
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
        Voorwaarde::En(a, b) => format!("{} en {}", voorwaarde(a), voorwaarde(b)),
        Voorwaarde::Of(a, b) => format!("{} of {}", voorwaarde(a), voorwaarde(b)),
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
            Soort::Document { soort, waarde, pad } => format!(
                "schrijf {} in het {}-bestand {} (een nieuw bestand; wat er stond, verdwijnt).",
                uitdrukking(waarde),
                if *soort == 0 { "Word" } else { "PDF" },
                uitdrukking(pad)
            ),
            Soort::Verstuur { waarde, adres } => format!("verstuur {} naar het internetadres {}.", uitdrukking(waarde), uitdrukking(adres)),
            Soort::Download { adres, pad } => format!("download het bestand van {} en bewaar het als {}.", uitdrukking(adres), uitdrukking(pad)),
            Soort::Tabel { tabel, actie } => match actie {
                TabelActie::Kolom(k) => format!("voeg de kolom {} toe aan de tabel {}.", k, tabel),
                TabelActie::Rij(w) => format!(
                    "voeg een rij toe aan de tabel {}, met: {}.",
                    tabel,
                    w.iter().map(uitdrukking).collect::<Vec<_>>().join(", ")
                ),
                TabelActie::HaalGekozenWeg => format!("haal de gekozen rij weg uit de tabel {}.", tabel),
                TabelActie::Leeg => format!("maak de tabel {} leeg (de kolommen blijven).", tabel),
                TabelActie::Sorteer(k) => format!("sorteer de tabel {} op de kolom {}.", tabel, k),
                TabelActie::Opslaan(p) => format!("sla de tabel {} op als CSV-bestand {} (Excel kan dat openen).", tabel, uitdrukking(p)),
                TabelActie::Inlezen(p) => format!("lees de tabel {} in uit het CSV-bestand {}.", tabel, uitdrukking(p)),
            },
            Soort::BeginOpname => "begin met opnemen van de microfoon.".into(),
            Soort::Wacht(u) => format!("wacht {} seconden.", uitdrukking(u)),
            Soort::StartProgramma(u) => format!("start het programma {}.", uitdrukking(u)),
            Soort::OpenBestand(u) => format!("open {} met het programma dat erbij hoort.", uitdrukking(u)),
            Soort::StopOpname(p) => format!("stop met opnemen en bewaar de opname als WAV-bestand {}.", uitdrukking(p)),
            Soort::MaakAfbeelding { pad, beschrijving } => {
                format!("zet de afbeelding {} in het laatste venster. De schermlezer leest voor: {}.", pad, beschrijving)
            }
            Soort::ZoekUpdates => "kijk op internet of er een nieuwe versie is, en vraag of die geïnstalleerd mag worden.".into(),
            Soort::MaakMap(p) => format!("maak de map {}.", uitdrukking(p)),
            Soort::Piep { hoogte, duur } => format!("laat een piep horen van {} hertz, {} milliseconden lang.", uitdrukking(hoogte), uitdrukking(duur)),
            Soort::SpeelGeluid { pad, wachten } => format!(
                "speel het geluid {} af{}.",
                uitdrukking(pad),
                if *wachten { " en wacht tot het klaar is" } else { " (het programma gaat meteen verder)" }
            ),
            Soort::StopGeluid => "stop het geluid.".into(),
            Soort::LeesVoor(u) => format!("lees {} voor met de stem van Windows.", uitdrukking(u)),
            Soort::StopVoorlezen => "stop met voorlezen.".into(),
            Soort::Volume(u) => format!("zet het volume op {} (van 0 tot 100).", uitdrukking(u)),
            Soort::MaakTabblad { naam } => format!("maak een tabblad in het laatste venster. De naam is: {}. Wat hierna komt, staat op dit tabblad.", naam),
            Soort::KiesTabblad { naam } => format!("ga naar het tabblad {}.", naam),
            Soort::MaakMenu { naam } => format!("maak een menu in de menubalk van het laatste venster. De naam is: {}.", naam),
            Soort::MaakMenuregel { naam } => format!("maak een regel in het laatste menu. De naam is: {}.", naam),
            Soort::BestandActie { soort: 0, van, naar } => format!("kopieer {} naar {}.", uitdrukking(van), naar.as_ref().map(uitdrukking).unwrap_or_default()),
            Soort::BestandActie { soort: 1, van, naar } => format!("verplaats {} naar {}.", uitdrukking(van), naar.as_ref().map(uitdrukking).unwrap_or_default()),
            Soort::BestandActie { van, .. } => format!("gooi {} weg (naar de Prullenbak, zodat je het terug kunt halen).", uitdrukking(van)),
            Soort::Zeg(u) => format!("laat de schermlezer {} zeggen, zonder meldingsvenster.", uitdrukking(u)),
            Soort::Pictogram(b) => format!("het pictogram van het programma is {}.", b),
            Soort::Zip { inpakken: true, map, zip } => format!("pak de map {} in tot het zipbestand {}.", uitdrukking(map), uitdrukking(zip)),
            Soort::Zip { inpakken: false, map, zip } => format!("pak het zipbestand {} uit in de map {}.", uitdrukking(zip), uitdrukking(map)),
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
            Soort::Stukje { naam, invoer, einde, rust: Some(code), .. } => {
                let met = if invoer.is_empty() { String::new() } else { format!(", met als invoer {}", invoer.join(" en ")) };
                uit.push(format!(
                    "{}Regel {}: een Rust-stukje met de naam {}{}. Het doet wat de eigen Rust-code zegt ({}), tot regel {}.",
                    insp,
                    r,
                    naam,
                    met,
                    match code.lines().count() { 1 => "1 regel".to_string(), n => format!("{} regels", n) },
                    einde
                ));
                continue;
            }
            Soort::Stukje { naam, invoer, blok, einde, .. } => {
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
                InstallatieZin::PrivacyRegel(t) => format!("in de privacyverklaring staat: {}", t),
                InstallatieZin::PrivacyBestand(b) => format!("de privacyverklaring staat in het bestand {}.", b),
                InstallatieZin::UpdateAdres(a) => format!("de updates staan op het adres {}.", a),
                InstallatieZin::Onderteken(n) => format!("onderteken het programma met een eigen certificaat op naam van {}.", n),
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
