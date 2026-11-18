//! # Praten met Jaap's Programmeertaal (eerste ontwikkelstap, versie 0.2)
//!
//! Hier praat je in gewone Nederlandse zinnen met het programma, zoals met een assistent.
//! Er zit GEEN AI en geen taalmodel in: alleen vaste regels die hier staan.
//!
//! Wat dit onderdeel doet:
//! - het ziet het verschil tussen een vraag, een idee, een uitleg en een opdracht;
//! - het onthoudt het gesprek: welk programma, welke onderdelen, wat er is afgesproken;
//! - het begrijpt verwijzingen zoals "dat programma", "die knop", "maak dat groter" en "ga maar verder";
//! - het herstelt dicteerfouten (zoals "tekstpak" voor "tekstvak") of vraagt het vriendelijk na;
//! - het stelt één vraag tegelijk en vertelt in gewone taal wat het denkt dat je wilt.
//!
//! Wat het NIET doet (afspraak voor deze stap): bestanden maken of veranderen, code aanpassen,
//! programma's bouwen of starten, iets installeren of instellingen veranderen. Het geeft alleen tekst terug.

use crate::opdrachten::{self, Begrip};
use crate::wensen::Voorstel;

/// Wat voor soort zin je zei.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Soort {
    Vraag,
    Idee,
    Uitleg,
    Opdracht,
    /// Een antwoord op een vraag van de assistent (ja, nee, of wat er gevraagd werd).
    Antwoord,
    /// Niet begrepen. De assistent gokt niet.
    Onduidelijk,
}

/// Het antwoord van de assistent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reactie {
    pub soort: Soort,
    pub tekst: String,
}

/// Een onderdeel van het programma waar je over praat, zoals "een knop Opslaan".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Onderdeel {
    /// Bijvoorbeeld "knop" of "tekstvak".
    pub soort: String,
    /// Bijvoorbeeld "Opslaan".
    pub naam: Option<String>,
    /// Bijvoorbeeld "voor je verhaal".
    pub doel: Option<String>,
    pub groot: bool,
}

impl Onderdeel {
    /// "een knop Opslaan", "een groot tekstvak voor je verhaal".
    pub fn beschrijving(&self) -> String {
        let mut s = String::from("een ");
        if self.groot {
            s.push_str("groot ");
        }
        s.push_str(&self.soort);
        if let Some(n) = &self.naam {
            s.push(' ');
            s.push_str(n);
        }
        if let Some(d) = &self.doel {
            s.push(' ');
            s.push_str(d);
        }
        s
    }

    /// "de knop Opslaan", "het tekstvak voor je verhaal".
    pub fn kort(&self) -> String {
        let mut s = format!("{} {}{}", lidwoord(&self.soort), if self.groot { "grote " } else { "" }, self.soort);
        if let Some(n) = &self.naam {
            s.push(' ');
            s.push_str(n);
        } else if let Some(d) = &self.doel {
            s.push(' ');
            s.push_str(d);
        }
        s
    }
}

/// Waar de assistent op wacht na een vraag.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Wacht {
    /// "Klopt dat?"
    Bevestiging,
    /// "Bedoel je tekstvak?" De hele zin wordt opnieuw gelezen als het antwoord ja is.
    Dicteer { fout: String, goed: String, zin: String },
    /// "Welk onderdeel wil je anders hebben?" Met de verandering als die al bekend is.
    WelkOnderdeel { wijziging: Option<String> },
    /// "Wat wil je anders aan de knop Opslaan?"
    HoeAnders { onderdeel: String },
    /// "Zal ik dat idee bij je opdracht zetten?"
    Idee { idee: String },
    /// "Bedoel je een nieuw programma, of hoort dit bij het programma waar we het over hadden?"
    NieuwOfHuidig { zin: String },
}

/// Het gesprek. Alles wat je vertelt en wat er is afgesproken, blijft hierin bewaard.
#[derive(Debug, Clone, Default)]
pub struct Gesprek {
    /// Het programma waar we het over hebben, bijvoorbeeld "een klein programma".
    pub programma: Option<String>,
    pub onderdelen: Vec<Onderdeel>,
    /// Wat het programma moet kunnen, bijvoorbeeld "afspelen".
    pub kunnen: Vec<String>,
    /// Veranderingen aan onderdelen, bijvoorbeeld "de knop Opslaan groter".
    pub wijzigingen: Vec<String>,
    /// Losse wensen die bij de opdracht horen.
    pub wensen: Vec<String>,
    /// Ideeën die (nog) niet bij de opdracht horen.
    pub ideeen: Vec<String>,
    /// Wat je over jezelf of je programma vertelde.
    pub uitleg: Vec<String>,
    /// Heb je gezegd dat de samenvatting klopt?
    pub bevestigd: bool,
    /// Het onderdeel waar het laatst over ging ("die knop", "maak dat groter").
    laatste: Option<usize>,
    wacht: Option<Wacht>,
    /// Alles wat er gezegd is: (jij, assistent).
    pub geschiedenis: Vec<(String, String)>,
}

/// Vaste zin die zegt dat er niets wordt uitgevoerd.
pub const NIETS_UITGEVOERD: &str = "In deze proefstap voer ik niets uit: ik maak, bouw en verander nog niets. Ik vertel alleen wat ik denk dat je bedoelt.";

impl Gesprek {
    pub fn nieuw() -> Self {
        Self::default()
    }

    /// Wat je zegt. Geeft het antwoord van de assistent. Voert nooit iets uit.
    pub fn zeg(&mut self, zin: &str) -> Reactie {
        let r = self.verwerk(zin);
        self.geschiedenis.push((zin.trim().to_string(), r.tekst.clone()));
        r
    }

    /// Wacht de assistent op een antwoord van jou?
    pub fn wacht_op_antwoord(&self) -> bool {
        self.wacht.is_some()
    }

    /// Het hele gesprek als tekst, om voor te lezen of na te lezen.
    pub fn als_tekst(&self) -> String {
        let mut s = String::new();
        for (jij, ik) in &self.geschiedenis {
            s.push_str("Jij: ");
            s.push_str(jij);
            s.push('\n');
            s.push_str("Assistent: ");
            s.push_str(ik);
            s.push_str("\n\n");
        }
        s
    }

    /// "een klein programma met een tekstvak en een knop Opslaan".
    pub fn samenvatting(&self) -> String {
        let mut s = self.programma.clone().unwrap_or_else(|| "een programma".to_string());
        let delen: Vec<String> = self.onderdelen.iter().map(|o| o.beschrijving()).collect();
        if !delen.is_empty() {
            s.push_str(" met ");
            s.push_str(&opsomming(&delen));
        }
        if !self.kunnen.is_empty() {
            s.push_str(if delen.is_empty() { " voor: " } else { ", voor: " });
            s.push_str(&opsomming(&self.kunnen));
        }
        if !self.wijzigingen.is_empty() {
            s.push_str(". Daarbij: ");
            s.push_str(&opsomming(&self.wijzigingen));
        }
        if !self.wensen.is_empty() {
            s.push_str(". Ook: ");
            s.push_str(&opsomming(&self.wensen));
        }
        s
    }

    fn heeft_opdracht(&self) -> bool {
        self.programma.is_some() || !self.onderdelen.is_empty() || !self.kunnen.is_empty()
    }

    fn verwerk(&mut self, zin: &str) -> Reactie {
        let zin = zin.trim();
        if zin.is_empty() {
            return reactie(Soort::Onduidelijk, "Ik heb niets gehoord. Typ of dicteer wat je wilt, en druk op Enter.");
        }
        let zin = &zeker_herstellen(zin);
        if let Some(w) = self.wacht.take() {
            if let Some(r) = self.antwoord_op(w, zin) {
                return r;
            }
            // Geen antwoord op de vraag: dan telt wat je nu zegt (je laatste duidelijke bedoeling).
        }
        self.gewone_zin(zin)
    }

    fn gewone_zin(&mut self, zin: &str) -> Reactie {
        let t = laag(zin);
        if is_verder(&t) {
            return self.verder();
        }
        if bevat_een(&t, &["begin opnieuw", "opnieuw beginnen", "vergeet alles", "nieuw gesprek", "begin maar opnieuw"]) {
            *self = Gesprek { geschiedenis: std::mem::take(&mut self.geschiedenis), ..Gesprek::default() };
            return reactie(Soort::Opdracht, "Goed, we beginnen opnieuw. Ik ben alles van dit gesprek vergeten. Wat wil je maken?");
        }
        if bevat_een(
            &t,
            &[
                "waar waren we",
                "waar zijn we gebleven",
                "wat hebben we afgesproken",
                "wat heb ik gezegd",
                "wat heb ik gevraagd",
                "wat weet je al",
                "samenvatting",
                "wat ik bedoel",
                "begrijp je me",
                "snap je me",
                "wat ga je maken",
                "wat zit erin",
                "wat zit er al in",
            ],
        ) {
            return self.vertel_stand();
        }
        if is_idee(&t) {
            let idee = zin.to_string();
            self.ideeen.push(idee.clone());
            self.wacht = Some(Wacht::Idee { idee });
            return reactie(Soort::Idee, "Dat is een idee. Ik onthoud het, maar ik voer het nog niet uit. Zal ik het bij je opdracht zetten?");
        }
        if is_vraag(zin, &t) && !is_verzoek(&t) {
            return self.vraag(zin, &t);
        }
        if is_uitleg(&t) {
            self.uitleg.push(zin.to_string());
            let tekst = match in_je_vorm(&t) {
                Some(j) => format!("Dank je. Dat onthoud ik: {}.", j),
                None => "Dank je. Dat onthoud ik.".to_string(),
            };
            return reactie(Soort::Uitleg, &tekst);
        }
        self.opdracht(zin)
    }

    /// "Ga maar verder": verder vanaf de laatste afspraak, niet opnieuw beginnen.
    fn verder(&mut self) -> Reactie {
        if !self.heeft_opdracht() {
            return reactie(
                Soort::Opdracht,
                "Ik weet niet waar we gebleven waren: we hebben nog niets afgesproken. Vertel me wat je wilt maken, dan beginnen we daar.",
            );
        }
        self.bevestigd = true;
        let tekst = format!(
            "Goed, we gaan verder vanaf wat we hebben afgesproken: {}. De volgende stap is: van deze afspraak de zinnen voor je programma maken. Dat doe ik in deze proefstap nog niet. Wil je nog iets veranderen of erbij hebben, zeg het dan.",
            self.samenvatting()
        );
        reactie(Soort::Opdracht, &tekst)
    }

    fn vertel_stand(&self) -> Reactie {
        if !self.heeft_opdracht() {
            return reactie(Soort::Vraag, "We hebben nog niets afgesproken. Vertel me wat je wilt maken.");
        }
        let mut s = format!("Ik begrijp je opdracht nu als: {}.", self.samenvatting());
        if !self.bevestigd {
            s.push_str(" Je hebt nog niet gezegd of dat klopt.");
        }
        if !self.ideeen.is_empty() {
            s.push_str(&format!(" Je had ook {} als idee genoemd.", if self.ideeen.len() == 1 { "één ding".to_string() } else { format!("{} dingen", self.ideeen.len()) }));
        }
        reactie(Soort::Vraag, &s)
    }

    /// Een antwoord op een vraag van de assistent. None = het is geen antwoord daarop.
    fn antwoord_op(&mut self, w: Wacht, zin: &str) -> Option<Reactie> {
        let (jn, rest) = ja_nee(zin);
        match w {
            Wacht::Bevestiging => match jn {
                Some(true) => {
                    self.bevestigd = true;
                    if inhoud(&rest) {
                        return Some(self.gewone_zin(&rest));
                    }
                    Some(reactie(
                        Soort::Antwoord,
                        &format!(
                            "Fijn. Dan is dit afgesproken: {}. Wil je nog iets erbij, zeg het dan. Of zeg: ga maar verder.",
                            self.samenvatting()
                        ),
                    ))
                }
                Some(false) => {
                    self.bevestigd = false;
                    if inhoud(&rest) {
                        return Some(self.gewone_zin(&rest));
                    }
                    Some(reactie(Soort::Antwoord, "Wat klopt er niet? Zeg bijvoorbeeld: geen knop, maar een menu."))
                }
                None => None,
            },
            Wacht::Dicteer { fout, goed, zin: oud } => match jn {
                Some(true) => {
                    let beter = vervang_woord(&oud, &fout, &goed);
                    let mut r = self.gewone_zin(&beter);
                    r.tekst = format!("Goed, je bedoelt {}. {}", goed, r.tekst);
                    Some(r)
                }
                Some(false) => Some(reactie(
                    Soort::Antwoord,
                    &format!("Wat bedoelde je dan met \"{}\"? Zeg het nog eens op een andere manier.", fout),
                )),
                None => None,
            },
            Wacht::WelkOnderdeel { wijziging } => {
                let i = self.zoek_onderdeel(&laag(zin))?;
                self.laatste = Some(i);
                match wijziging {
                    Some(wat) => Some(self.wijzig(i, &wat)),
                    None => {
                        let kort = self.onderdelen[i].kort();
                        self.wacht = Some(Wacht::HoeAnders { onderdeel: kort.clone() });
                        Some(reactie(
                            Soort::Antwoord,
                            &format!("Wat wil je anders aan {}? Bijvoorbeeld groter, een andere naam of een andere kleur.", kort),
                        ))
                    }
                }
            }
            Wacht::HoeAnders { onderdeel } => {
                if jn.is_some() && !inhoud(&rest) {
                    return None;
                }
                let wat = je_vorm_vrij(zin);
                let i = self.onderdelen.iter().position(|o| o.kort() == onderdeel)?;
                Some(self.wijzig(i, &wat))
            }
            Wacht::Idee { idee } => match jn {
                Some(true) => {
                    self.ideeen.retain(|x| *x != idee);
                    let schoon = zonder_idee_woorden(&idee);
                    let r = self.opdracht(&schoon);
                    if r.soort == Soort::Onduidelijk {
                        self.wensen.push(je_vorm_vrij(&schoon));
                        return Some(reactie(
                            Soort::Antwoord,
                            &format!("Goed, ik heb het als wens bij je opdracht gezet. Ik begrijp je opdracht nu als: {}.", self.samenvatting()),
                        ));
                    }
                    Some(r)
                }
                Some(false) => Some(reactie(Soort::Antwoord, "Goed, dan bewaar ik het alleen als idee. Het komt niet in je opdracht.")),
                None => None,
            },
            Wacht::NieuwOfHuidig { zin: oud } => {
                let t = laag(zin);
                if bevat_een(&t, &["nieuw", "ander", "opnieuw", "apart"]) {
                    *self = Gesprek { geschiedenis: std::mem::take(&mut self.geschiedenis), ..Gesprek::default() };
                    return Some(self.opdracht(&oud));
                }
                if jn == Some(true) || bevat_een(&t, &["zelfde", "hetzelfde", "huidig", "dit programma", "dat programma", "erbij", "hoort bij", "hoort erbij"]) {
                    let mut r = self.voeg_toe(&oud, false);
                    if r.soort == Soort::Onduidelijk {
                        r = reactie(Soort::Antwoord, &format!("Goed. Ik begrijp je opdracht nu als: {}.", self.samenvatting()));
                    }
                    return Some(r);
                }
                None
            }
        }
    }

    fn vraag(&mut self, zin: &str, t: &str) -> Reactie {
        // Wat is een ...?
        if bevat_een(t, &["wat is een", "wat is het", "wat betekent", "wat zijn"]) {
            for (woorden, soort, _) in SOORTEN {
                if woorden.iter().any(|w| heeft(t, w)) {
                    return reactie(Soort::Vraag, uitleg_soort(soort));
                }
            }
        }
        if bevat_een(t, &["wat kun je", "wat kan je", "wat kan jij", "wat kun jij", "wie ben je", "wat doe je", "hoe werk je", "wat moet ik zeggen"]) {
            return reactie(
                Soort::Vraag,
                "Ik begrijp gewone zinnen over het programma dat je wilt maken. Zeg bijvoorbeeld: maak een klein programma waar ik tekst in kan typen. Daarna kun je dingen erbij zetten, zoals: zet er ook een knop Opslaan bij. Ik vertel steeds wat ik denk dat je bedoelt. In deze proefstap voer ik nog niets uit.",
            );
        }
        if bevat_een(t, &["heb je", "is het", "ben je"]) && bevat_een(t, &["gemaakt", "gedaan", "gebouwd", "uitgevoerd", "veranderd", "opgeslagen", "klaar", "aangepast"]) {
            return reactie(Soort::Vraag, &format!("Nee. {}", NIETS_UITGEVOERD));
        }
        // Kan ik een knop maken?
        if bevat_een(t, &["kan ik", "kun ik", "mag ik", "kan het", "is het mogelijk", "lukt het"]) {
            for (woorden, soort, _) in SOORTEN {
                if woorden.iter().any(|w| heeft(t, w)) {
                    return reactie(
                        Soort::Vraag,
                        &format!("Ja, {} {} kan. Wil je er een, zeg dan bijvoorbeeld: zet er {} bij.", lidwoord_een(soort), soort, voorbeeld(soort)),
                    );
                }
            }
        }
        // Vragen die de rest van het programma al kent (uitleg uit Jaaps eigen zinnen, wat nog niet kan).
        match opdrachten::begrijp_alles(zin, &[]) {
            opdrachten::Antwoord::Taal(Voorstel::Uitleg(u)) => return reactie(Soort::Vraag, &u),
            opdrachten::Antwoord::Taal(Voorstel::NogNiet(n)) => return reactie(Soort::Vraag, &n),
            opdrachten::Antwoord::Editor(Begrip::Doe { functie, .. }) => {
                let a = opdrachten::actie(functie);
                let toets = if a.toets.is_empty() { String::new() } else { format!(" (sneltoets {})", a.toets) };
                return reactie(
                    Soort::Vraag,
                    &format!("Dat gaat met: {}{}. Je kunt dat in het venster Opdracht geven (Ctrl+E) zeggen. Hier in het gesprek voer ik het niet uit.", kleine_letter(a.naam), toets),
                );
            }
            _ => {}
        }
        reactie(
            Soort::Onduidelijk,
            "Dat weet ik niet. Ik wil niets verzinnen. Vraag het op een andere manier, of kijk in de handleiding (F1).",
        )
    }

    /// Een opdracht: iets maken, erbij zetten, veranderen of weghalen.
    fn opdracht(&mut self, zin: &str) -> Reactie {
        let t = laag(zin);

        // Dicteerfouten: "tekstpak" bedoelt waarschijnlijk "tekstvak".
        let (zin_goed, notitie) = match dicteerfout(zin) {
            Some((fout, goed)) => {
                let andere_context = self.heeft_opdracht() || !soorten_in(&laag(&vervang_woord(zin, &fout, ""))).is_empty() || is_programma(&t);
                if andere_context {
                    (vervang_woord(zin, &fout, &goed), Some(format!("Ik lees \"{}\" als \"{}\". ", fout, goed)))
                } else {
                    self.wacht = Some(Wacht::Dicteer { fout: fout.clone(), goed: goed.clone(), zin: zin.to_string() });
                    return reactie(Soort::Opdracht, &format!("Ik hoorde \"{}\". Bedoel je {}?", fout, goed));
                }
            }
            None => (zin.to_string(), None),
        };
        let zin = zin_goed.as_str();
        let t = laag(zin);
        let mut r = self.opdracht_schoon(zin, &t);
        if let Some(n) = notitie {
            r.tekst = n + &r.tekst;
        }
        r
    }

    fn opdracht_schoon(&mut self, zin: &str, t: &str) -> Reactie {
        // Weghalen.
        if bevat_een(t, &[" haal ", " verwijder", " zonder ", "mag weg", "moet weg", "hoeft niet", "niet meer", " weg "]) || begint_met(t, "geen ") {
            if let Some(i) = self.zoek_onderdeel(t) {
                let o = self.onderdelen.remove(i);
                self.laatste = None;
                return reactie(
                    Soort::Opdracht,
                    &format!("Dan hoort {} er niet meer bij. Ik begrijp je opdracht nu als: {}.", o.kort(), self.samenvatting()),
                );
            }
            if let Some(i) = self.kunnen.iter().position(|k| t.contains(k.split_whitespace().last().unwrap_or(k))) {
                let k = self.kunnen.remove(i);
                return reactie(Soort::Opdracht, &format!("Dan hoort {} er niet meer bij. Ik begrijp je opdracht nu als: {}.", k, self.samenvatting()));
            }
        }

        // Veranderen: "maak dat groter", "maak de knop rood", "maak dat anders".
        if let Some(wat) = eigenschap(t) {
            let genoemd = self.zoek_onderdeel(t);
            let verwijst = bevat_een(t, &[" dat ", " het ", " hem ", " die ", " deze ", " dit "]);
            let nieuw_onderdeel = genoemd.is_none() && !soorten_in(t).is_empty();
            if !nieuw_onderdeel {
                if wat == "anders" {
                    // Te vaag: eerst vragen welk onderdeel. Niet zelf kiezen.
                    if let Some(i) = genoemd {
                        let kort = self.onderdelen[i].kort();
                        self.laatste = Some(i);
                        self.wacht = Some(Wacht::HoeAnders { onderdeel: kort.clone() });
                        return reactie(Soort::Opdracht, &format!("Wat wil je anders aan {}? Bijvoorbeeld groter, een andere naam of een andere kleur.", kort));
                    }
                    return self.vraag_welk_onderdeel(None);
                }
                let doel = genoemd.or(if verwijst { self.laatste } else { None }).or(if self.onderdelen.len() == 1 { Some(0) } else { None });
                match doel {
                    Some(i) => return self.wijzig(i, &wat),
                    None if verwijst || self.heeft_opdracht() => return self.vraag_welk_onderdeel(Some(wat)),
                    None => {}
                }
            }
        }

        // Een nieuw programma terwijl we het al over een ander hadden?
        if is_programma(t) && self.heeft_opdracht() {
            if bevat_een(t, &["nieuw programma", "nieuwe programma", "ander programma", "andere programma", "nieuwe app", "andere app"]) {
                *self = Gesprek { geschiedenis: std::mem::take(&mut self.geschiedenis), ..Gesprek::default() };
                let mut r = self.voeg_toe(zin, true);
                r.tekst = format!("Goed, een nieuw programma. {}", r.tekst);
                return r;
            }
            if !bevat_een(t, &[" dat programma", " het programma", " dit programma", " mijn programma", " er ", " ook "]) {
                self.wacht = Some(Wacht::NieuwOfHuidig { zin: zin.to_string() });
                return reactie(
                    Soort::Opdracht,
                    &format!("Bedoel je een nieuw programma, of hoort dit bij {}? Zeg: nieuw, of: hetzelfde.", self.samenvatting()),
                );
            }
        }

        let r = self.voeg_toe(zin, false);
        if r.soort != Soort::Onduidelijk {
            return r;
        }

        // Een opdracht voor de editor (bijvoorbeeld "maak er een exe van"): eerlijk zeggen dat het hier niet gebeurt.
        match opdrachten::begrijp_alles(zin, &[]) {
            opdrachten::Antwoord::Editor(Begrip::Doe { functie, .. }) => {
                let a = opdrachten::actie(functie);
                let toets = if a.toets.is_empty() { "Ctrl+E".to_string() } else { a.toets.to_string() };
                return reactie(
                    Soort::Opdracht,
                    &format!("Je wilt: {}. Dat voer ik hier niet uit. {} Wil je het echt doen, gebruik dan {}.", kleine_letter(a.naam), NIETS_UITGEVOERD, toets),
                );
            }
            opdrachten::Antwoord::Taal(Voorstel::NogNiet(n)) => return reactie(Soort::Opdracht, &n),
            opdrachten::Antwoord::Taal(Voorstel::Uitleg(u)) => return reactie(Soort::Uitleg, &u),
            _ => {}
        }
        reactie(
            Soort::Onduidelijk,
            &format!(
                "Dat begrijp ik niet helemaal: \"{}\". Ik wil niet raden. Kun je het anders zeggen? Bijvoorbeeld: maak een knop Opslaan.",
                zin.trim()
            ),
        )
    }

    fn vraag_welk_onderdeel(&mut self, wijziging: Option<String>) -> Reactie {
        if self.onderdelen.is_empty() {
            return reactie(
                Soort::Opdracht,
                "Wat wil je anders hebben? We hebben nog geen onderdelen afgesproken. Zeg bijvoorbeeld: maak de knop Opslaan groter.",
            );
        }
        let namen: Vec<String> = self.onderdelen.iter().map(|o| o.kort()).collect();
        let vb = if namen.len() == 1 { format!("Bedoel je {}?", namen[0]) } else { format!("Bijvoorbeeld {}.", of_opsomming(&namen)) };
        let vraag = match &wijziging {
            Some(w) => format!("Welk onderdeel wil je {} hebben? {}", w, vb),
            None => format!("Welk onderdeel wil je anders hebben? {}", vb),
        };
        self.wacht = Some(Wacht::WelkOnderdeel { wijziging });
        reactie(Soort::Opdracht, &vraag)
    }

    fn wijzig(&mut self, i: usize, wat: &str) -> Reactie {
        let o = &mut self.onderdelen[i];
        self.laatste = Some(i);
        let tekst;
        if let Some(n) = wat.strip_prefix("naam ") {
            let oud = o.kort();
            o.naam = Some(hoofdletter(n));
            tekst = format!("Dan krijgt {} de naam {}.", oud, hoofdletter(n));
        } else {
            let w = format!("{} {}", o.kort(), wat);
            tekst = format!("Dan wordt {}.", w);
            self.wijzigingen.retain(|x| !x.starts_with(&o.kort()) || !zelfde_soort_wijziging(x, wat));
            self.wijzigingen.push(w);
        }
        reactie(Soort::Opdracht, &format!("{} Ik begrijp je opdracht nu als: {}.", tekst, self.samenvatting()))
    }

    /// Welk bestaand onderdeel noem je? ("de knop", "die knop", "het tekstvak", "Opslaan")
    fn zoek_onderdeel(&self, t: &str) -> Option<usize> {
        // Op naam.
        for (i, o) in self.onderdelen.iter().enumerate() {
            if let Some(n) = &o.naam {
                if heeft(t, &n.to_lowercase()) {
                    return Some(i);
                }
            }
        }
        // Op soort.
        let soorten = soorten_in(t);
        for s in soorten {
            let passend: Vec<usize> = self.onderdelen.iter().enumerate().filter(|(_, o)| o.soort == s).map(|(i, _)| i).collect();
            match passend.len() {
                0 => {}
                1 => return Some(passend[0]),
                _ => {
                    if let Some(l) = self.laatste {
                        if passend.contains(&l) {
                            return Some(l);
                        }
                    }
                }
            }
        }
        None
    }

    /// Zet nieuwe onderdelen en wat het programma moet kunnen in de opdracht.
    fn voeg_toe(&mut self, zin: &str, nieuw: bool) -> Reactie {
        let t = laag(zin);
        let eerst = !self.heeft_opdracht() || nieuw;
        let mut nieuwe_delen: Vec<String> = Vec::new();
        let mut opmerkingen: Vec<String> = Vec::new();
        let mut programma_genoemd = false;

        if let Some(p) = programma_van(&t) {
            programma_genoemd = true;
            if self.programma.is_none() || nieuw {
                self.programma = Some(p);
            }
        }

        let nog_een = bevat_een(&t, &["nog een", "tweede", "nog één", "andere knop", "extra knop"]);
        for deel in delen_van(zin) {
            for o in onderdelen_in(&deel) {
                let bestaand = self.onderdelen.iter().position(|x| x.soort == o.soort && (x.naam == o.naam || (o.naam.is_none() && !nog_een) || (x.naam.is_none() && o.naam.is_some() && !nog_een)));
                match bestaand {
                    Some(i) if !nog_een => {
                        let x = &mut self.onderdelen[i];
                        let wie = hoofdletter(&format!("{} {}", lidwoord(&x.soort), x.soort));
                        let mut veranderd = Vec::new();
                        if x.naam.is_none() && o.naam.is_some() {
                            x.naam = o.naam.clone();
                            veranderd.push(format!("heet nu {}", o.naam.clone().unwrap_or_default()));
                        }
                        if o.doel.is_some() && x.doel != o.doel {
                            x.doel = o.doel.clone();
                            veranderd.push(format!("is nu {}", o.doel.clone().unwrap_or_default()));
                        }
                        if o.groot && !x.groot {
                            x.groot = true;
                            veranderd.push("wordt groot".to_string());
                        }
                        self.laatste = Some(i);
                        if !veranderd.is_empty() {
                            opmerkingen.push(format!("{} {}.", wie, opsomming(&veranderd)));
                        } else {
                            opmerkingen.push(format!("{} had je al genoemd.", hoofdletter(&self.onderdelen[i].kort())));
                        }
                    }
                    _ => {
                        nieuwe_delen.push(o.beschrijving());
                        self.onderdelen.push(o);
                        self.laatste = Some(self.onderdelen.len() - 1);
                    }
                }
            }
        }
        let mut nieuwe_kunnen = Vec::new();
        for k in kunnen_in(&t) {
            if self.onderdelen.iter().any(|o| o.naam.as_deref().map(|n| n.to_lowercase()) == Some(k.clone())) {
                continue;
            }
            if !self.kunnen.contains(&k) {
                self.kunnen.push(k.clone());
                nieuwe_kunnen.push(k);
            }
        }

        if nieuwe_delen.is_empty() && nieuwe_kunnen.is_empty() && opmerkingen.is_empty() && !programma_genoemd {
            return reactie(Soort::Onduidelijk, "");
        }
        if nieuwe_delen.is_empty() && nieuwe_kunnen.is_empty() && opmerkingen.is_empty() {
            // Alleen "maak een programma": vraag wat erin moet (één vraag, met voorbeelden).
            self.bevestigd = false;
            return reactie(
                Soort::Opdracht,
                &format!(
                    "Je wilt dus {} maken. Wat moet erin komen, of wat moet het kunnen? Bijvoorbeeld: een tekstvak om in te typen, of een knop Opslaan.",
                    self.samenvatting()
                ),
            );
        }

        if eerst && (!nieuwe_delen.is_empty() || !nieuwe_kunnen.is_empty()) {
            self.bevestigd = false;
            self.wacht = Some(Wacht::Bevestiging);
            return reactie(
                Soort::Opdracht,
                &format!("Je wilt dus {}. Klopt dat? Ik maak nog niets: ik wil eerst zeker weten dat ik je goed begrijp.", self.samenvatting()),
            );
        }

        let mut s = String::new();
        if !nieuwe_delen.is_empty() {
            s.push_str(&format!("Dan komt er ook {} bij. ", opsomming(&nieuwe_delen)));
        }
        if !nieuwe_kunnen.is_empty() {
            s.push_str(&format!("Dan hoort {} er ook bij. ", opsomming(&nieuwe_kunnen)));
        }
        for o in opmerkingen {
            s.push_str(&o);
            s.push(' ');
        }
        s.push_str(&format!("Ik begrijp je opdracht nu als: {}.", self.samenvatting()));
        reactie(Soort::Opdracht, &s)
    }
}

// ------------------------------------------------------------------ kennis

/// Soorten onderdelen: (woorden die je kunt zeggen, de soort, lidwoord). Langere woorden eerst.
const SOORTEN: &[(&[&str], &str, &str)] = &[
    (&["selectievakje", "vinkje", "aankruisvakje", "checkbox"], "selectievakje", "het"),
    (&["uitklaplijst", "dropdown", "combobox"], "uitklaplijst", "de"),
    (&["keuzelijst", "listbox"], "keuzelijst", "de"),
    (&["menuregel", "menu-item", "menuitem"], "menuregel", "de"),
    (&["menubalk", "menu"], "menu", "het"),
    (&["tabblad", "tabbladen"], "tabblad", "het"),
    (&["tabel"], "tabel", "de"),
    (&["tekstvak", "tekstveld", "invoervak", "invulvak", "invoerveld", "editveld"], "tekstvak", "het"),
    (&["label", "opschrift"], "label", "het"),
    (&["knop", "knoppen", "knopje", "button"], "knop", "de"),
    (&["venster", "scherm"], "venster", "het"),
];

/// Zinnetjes waarmee je een tekstvak bedoelt zonder het woord te noemen.
const TYPEN: &[&str] = &["tekst in kan typen", "tekst kan typen", "tekst typen", "tekst intypen", "in kan typen", "kan typen", "kan schrijven", "tekst in kan schrijven"];

/// Wat een programma kan: (werkwoord, hoe het in de samenvatting komt).
const KUNNEN: &[(&str, &str)] = &[
    ("openen", "openen"),
    ("afspelen", "afspelen"),
    ("pauzeren", "pauzeren"),
    ("stoppen", "stoppen"),
    ("opslaan", "opslaan"),
    ("bewaren", "opslaan"),
    ("afdrukken", "afdrukken"),
    ("printen", "afdrukken"),
    ("zoeken", "zoeken"),
    ("voorlezen", "voorlezen"),
    ("rekenen", "rekenen"),
    ("optellen", "optellen"),
    ("opnemen", "opnemen"),
    ("kopiëren", "kopiëren"),
    ("plakken", "plakken"),
    ("vertalen", "vertalen"),
];

/// Dicteerfouten die altijd fout zijn (losse stukken die aan elkaar horen).
const ZEKER: &[(&str, &str)] = &[
    ("tekst vak", "tekstvak"),
    ("tekst veld", "tekstveld"),
    ("op slaan", "opslaan"),
    ("menu balk", "menubalk"),
    ("tab blad", "tabblad"),
    ("selectie vakje", "selectievakje"),
    ("keuze lijst", "keuzelijst"),
    ("uitklap lijst", "uitklaplijst"),
    ("pro gramma", "programma"),
];

/// Dicteerfouten die de spraakherkenning vaak maakt. Die worden met de rest van het gesprek gecontroleerd.
const VERHASPELD: &[(&str, &str)] = &[
    ("tekstpak", "tekstvak"),
    ("tekstvat", "tekstvak"),
    ("teksvak", "tekstvak"),
    ("tekstvaak", "tekstvak"),
    ("tekstfak", "tekstvak"),
    ("knoop", "knop"),
    ("knob", "knop"),
    ("venstr", "venster"),
];

/// Gewone woorden die op een onderdeel lijken maar het niet zijn.
const GEEN_FOUT: &[&str] = &["tekst", "teksten", "venster", "vensters", "programmeur", "programmeren", "programmeertaal", "tabellen", "labels"];

fn lidwoord(soort: &str) -> &'static str {
    SOORTEN.iter().find(|x| x.1 == soort).map(|x| x.2).unwrap_or("de")
}

fn lidwoord_een(_soort: &str) -> &'static str {
    "een"
}

fn voorbeeld(soort: &str) -> String {
    match soort {
        "knop" => "een knop Opslaan".into(),
        "tekstvak" => "een tekstvak voor je naam".into(),
        "label" => "een label met je naam".into(),
        "menu" => "een menu Bestand".into(),
        s => format!("een {}", s),
    }
}

fn uitleg_soort(soort: &str) -> &'static str {
    match soort {
        "tekstvak" => "Een tekstvak is een vak waarin je tekst typt, zoals je naam of een verhaal. Je schermlezer noemt het meestal bewerken of invoerveld.",
        "knop" => "Een knop is iets waar je op drukt om iets te laten gebeuren, zoals Opslaan. Met Tab kom je erop, met Spatie of Enter druk je erop.",
        "label" => "Een label is een stukje tekst in het venster, bijvoorbeeld de naam van een vak. Je kunt er niets in typen.",
        "selectievakje" => "Een selectievakje is een vakje dat aan of uit staat. Met Spatie zet je het aan of uit.",
        "keuzelijst" => "Een keuzelijst is een lijst waarin je met de pijltjes iets kiest.",
        "uitklaplijst" => "Een uitklaplijst is een lijst die openklapt. Met Alt en pijltje omlaag klap je hem open.",
        "menu" => "Een menu is de balk bovenaan met bijvoorbeeld Bestand en Help. Je opent hem met Alt.",
        "menuregel" => "Een menuregel is een keuze in een menu, zoals Opslaan in het menu Bestand.",
        "tabblad" => "Een tabblad is een bladzijde in een venster. Met Control Tab ga je naar het volgende tabblad.",
        "tabel" => "Een tabel heeft rijen en kolommen, zoals een lijstje in Excel.",
        "venster" => "Een venster is het scherm van je programma, met een titel bovenaan.",
        _ => "Dat is een onderdeel van een programma.",
    }
}

// ------------------------------------------------------------------ hulpjes voor zinnen

fn reactie(soort: Soort, tekst: &str) -> Reactie {
    Reactie { soort, tekst: tekst.to_string() }
}

/// Kleine letters, leestekens weg, spaties eromheen (zodat " knop " een heel woord vindt).
fn laag(zin: &str) -> String {
    let s: String = zin
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '\'' || c == '-' || c == 'ë' || c == 'é' { c } else { ' ' })
        .collect();
    format!(" {} ", s.split_whitespace().collect::<Vec<_>>().join(" "))
}

fn heeft(t: &str, woord: &str) -> bool {
    t.contains(&format!(" {} ", woord.trim()))
}

fn bevat_een(t: &str, stukken: &[&str]) -> bool {
    stukken.iter().any(|s| t.contains(s))
}

fn begint_met(t: &str, b: &str) -> bool {
    t.trim_start().starts_with(b)
}

fn inhoud(s: &str) -> bool {
    laag(s).trim().split_whitespace().any(|w| !["en", "dan", "maar", "ook", "nou"].contains(&w))
}

fn opsomming(lijst: &[String]) -> String {
    match lijst.len() {
        0 => String::new(),
        1 => lijst[0].clone(),
        n => format!("{} en {}", lijst[..n - 1].join(", "), lijst[n - 1]),
    }
}

fn of_opsomming(lijst: &[String]) -> String {
    match lijst.len() {
        0 => String::new(),
        1 => lijst[0].clone(),
        n => format!("{}, of {}", lijst[..n - 1].join(", "), lijst[n - 1]),
    }
}

fn hoofdletter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn kleine_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Vervangt een heel woord (zonder op hoofdletters te letten).
fn vervang_woord(zin: &str, oud: &str, nieuw: &str) -> String {
    zin.split_whitespace()
        .map(|w| {
            let kern: String = w.chars().filter(|c| c.is_alphanumeric()).collect();
            if kern.to_lowercase() == oud.to_lowercase() {
                w.to_lowercase().replace(&oud.to_lowercase(), nieuw)
            } else {
                w.to_string()
            }
        })
        .filter(|w| !w.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Dicteerfouten die altijd fout zijn, meteen herstellen.
fn zeker_herstellen(zin: &str) -> String {
    let mut s = zin.to_string();
    for (fout, goed) in ZEKER {
        let laag_s = s.to_lowercase();
        if let Some(p) = laag_s.find(fout) {
            let voor_ok = p == 0 || !laag_s[..p].chars().last().map_or(false, |c| c.is_alphanumeric());
            let eind = p + fout.len();
            let na_ok = eind >= laag_s.len() || !laag_s[eind..].chars().next().map_or(false, |c| c.is_alphanumeric());
            if voor_ok && na_ok {
                s = format!("{}{}{}", &s[..p], goed, &s[eind..]);
            }
        }
    }
    s
}

/// Een woord dat waarschijnlijk verkeerd is gedicteerd: (fout, goed).
fn dicteerfout(zin: &str) -> Option<(String, String)> {
    let t = laag(zin);
    for (fout, goed) in VERHASPELD {
        if heeft(&t, fout) {
            return Some((fout.to_string(), goed.to_string()));
        }
    }
    // Lijkt sterk op een onderdeel (één letter anders), maar is het niet.
    let doelen: Vec<&str> = SOORTEN.iter().flat_map(|s| s.0.iter().copied()).filter(|w| w.chars().count() >= 6).chain(["programma"]).collect();
    for w in t.split_whitespace() {
        if w.chars().count() < 6 || GEEN_FOUT.contains(&w) || !w.chars().all(|c| c.is_alphabetic()) {
            continue;
        }
        if doelen.iter().any(|d| w.starts_with(d) || d.starts_with(w)) {
            continue;
        }
        for d in &doelen {
            if afstand(w, d) == 1 {
                return Some((w.to_string(), d.to_string()));
            }
        }
    }
    None
}

/// Hoeveel letters verschillen (Levenshtein).
fn afstand(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut vorige: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut nu = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let kost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            nu[j] = (vorige[j] + 1).min(nu[j - 1] + 1).min(vorige[j - 1] + kost);
        }
        vorige = nu;
    }
    vorige[b.len()]
}

fn is_verder(t: &str) -> bool {
    let k = t.trim();
    bevat_een(t, &["ga maar verder", "ga verder", "ga nu verder", "ga maar door", "ga door", "ga je gang", "verder gaan", "doorgaan", "volgende stap", "ga maar weer verder"])
        || ["verder", "door", "en nu", "verder maar", "ga maar"].contains(&k)
}

fn is_idee(t: &str) -> bool {
    bevat_een(
        t,
        &[" misschien ", " idee ", "zou leuk zijn", "zou mooi zijn", "zou handig zijn", "zou het niet", " wat als ", "ik denk erover", "ooit ", " later wil ik", " eventueel "],
    )
}

fn zonder_idee_woorden(zin: &str) -> String {
    let mut s = zin.to_string();
    for w in ["misschien", "eventueel", "het zou leuk zijn als", "het zou mooi zijn als", "het zou handig zijn als", "ik heb een idee", "idee"] {
        let l = s.to_lowercase();
        if let Some(p) = l.find(w) {
            s = format!("{}{}", &s[..p], &s[p + w.len()..]);
        }
    }
    s.trim().trim_start_matches([',', ':']).trim().to_string()
}

const VRAAGWOORDEN: &[&str] = &["wat", "wie", "waar", "waarom", "wanneer", "hoe", "welke", "welk", "hoeveel", "kan", "kun", "kunt", "mag", "heb", "is", "zijn", "ben", "weet", "snap", "begrijp", "werkt", "moet"];

fn is_vraag(zin: &str, t: &str) -> bool {
    if zin.trim_end().ends_with('?') {
        return true;
    }
    let eerste = t.split_whitespace().next().unwrap_or("");
    // "moet ik ...?" en "is het ...?" alleen als vraag; "moet er een knop bij" is een opdracht.
    if eerste == "moet" {
        return t.starts_with(" moet ik ") || t.starts_with(" moet je ");
    }
    VRAAGWOORDEN.contains(&eerste) && !t.starts_with(" kan er ") && !t.starts_with(" kun er ")
}

/// "Kun je een programma maken?" is een vraag in de vorm, maar een verzoek in bedoeling.
fn is_verzoek(t: &str) -> bool {
    let beleefd = ["kun je ", "kan je ", "kunt u ", "kun jij ", "kan jij ", "wil je ", "wilt u ", "zou je ", "zou jij ", "wil jij ", "mag er ", "kan er ", "kun er ", "moet er "]
        .iter()
        .any(|b| t.trim_start().starts_with(b));
    let doe = bevat_een(t, &[" maken", " maak", " bouwen", " zetten", " toevoegen", " erbij", " erin", " bij ", " veranderen", " weghalen", " groter", " kleiner"])
        || !soorten_in(t).is_empty()
        || is_programma(t);
    beleefd && doe
}

fn is_uitleg(t: &str) -> bool {
    let begin = [" ik ben ", " ik heb ", " ik gebruik ", " ik werk ", " ik zit ", " ik kan niet ", " ik zie ", " mijn ", " het is voor ", " het programma is voor ", " het is bedoeld ", " ter informatie", " voor de duidelijkheid", " even zodat je het weet", " weet je dat "];
    let doe = bevat_een(t, &[" maak", " zet ", " voeg", " bouw", " wil ", " moet ", " graag", " erbij", " haal "]) || !soorten_in(t).is_empty();
    begin.iter().any(|b| t.starts_with(b)) && !doe
}

/// "ik ben blind" -> "je bent blind". Alleen bij zinnen waarvan het zeker goed gaat.
fn in_je_vorm(t: &str) -> Option<String> {
    let k = t.trim();
    let paren = [("ik ben ", "je bent "), ("ik heb ", "je hebt "), ("ik gebruik ", "je gebruikt "), ("ik werk ", "je werkt "), ("ik zit ", "je zit "), ("ik zie ", "je ziet "), ("ik kan niet ", "je kunt niet ")];
    for (a, b) in paren {
        if let Some(rest) = k.strip_prefix(a) {
            return Some(format!("{}{}", b, rest.replace(" mijn ", " je ").replace(" mij ", " je ")));
        }
    }
    if let Some(rest) = k.strip_prefix("mijn ") {
        return Some(format!("je {}", rest));
    }
    None
}

/// Vrije tekst in de je-vorm, voor in een samenvatting.
fn je_vorm_vrij(zin: &str) -> String {
    let s = format!(" {} ", zin.trim().trim_end_matches(['.', '!', '?']));
    s.replace(" mijn ", " je ").replace(" mij ", " jou ").replace(" ik ", " je ").replace(" Ik ", " Je ").trim().to_string()
}

/// (ja/nee, de rest van de zin). "Ja, en ik wil ook kunnen pauzeren" -> (ja, "ik wil ook kunnen pauzeren").
fn ja_nee(zin: &str) -> (Option<bool>, String) {
    let t = laag(zin);
    let k = t.trim();
    let ja = ["ja", "jawel", "jazeker", "klopt", "goed", "prima", "precies", "oké", "oke", "ok", "yes", "zeker", "inderdaad", "correct", "akkoord"];
    let nee = ["nee", "neen", "no", "niet", "fout", "nope"];
    let zinnen_ja = ["dat klopt", "doe maar", "is goed", "dat is goed", "helemaal goed", "ja hoor", "dat is het"];
    let zinnen_nee = ["klopt niet", "dat klopt niet", "niet goed", "dat is niet goed", "nee hoor", "dat is het niet"];
    for z in zinnen_nee {
        if k.starts_with(z) {
            return (Some(false), rest_na(zin, z.split_whitespace().count()));
        }
    }
    for z in zinnen_ja {
        if k.starts_with(z) {
            return (Some(true), rest_na(zin, z.split_whitespace().count()));
        }
    }
    let eerste = k.split_whitespace().next().unwrap_or("");
    if ja.contains(&eerste) {
        return (Some(true), rest_na(zin, 1));
    }
    if nee.contains(&eerste) && (k == eerste || zin.trim_start().to_lowercase().starts_with(&format!("{},", eerste)) || zin.trim_start().to_lowercase().starts_with(&format!("{} ", eerste))) && eerste != "niet" {
        return (Some(false), rest_na(zin, 1));
    }
    (None, zin.to_string())
}

/// De zin zonder de eerste n woorden (en zonder "en" vooraan).
fn rest_na(zin: &str, n: usize) -> String {
    let woorden: Vec<&str> = zin.split_whitespace().skip(n).collect();
    let mut s = woorden.join(" ");
    s = s.trim_start_matches([',', '.', '!', ' ']).to_string();
    for b in ["en ", "En ", "maar ", "Maar "] {
        if let Some(r) = s.strip_prefix(b) {
            s = r.to_string();
        }
    }
    s.trim().to_string()
}

fn is_programma(t: &str) -> bool {
    programma_van(t).is_some()
}

/// "een klein programma", "een rekenmachine".
fn programma_van(t: &str) -> Option<String> {
    let klein = bevat_een(t, &[" klein ", " kleine ", " eenvoudig ", " eenvoudige ", " simpel ", " simpele "]);
    let voor = if klein { "een klein " } else { "een " };
    for (w, naam) in [
        ("programma", "programma"),
        ("programmaatje", "programma"),
        ("app", "programma"),
        ("applicatie", "programma"),
        ("rekenmachine", "rekenmachine"),
        ("spelletje", "spelletje"),
        ("spel", "spelletje"),
        ("muziekspeler", "muziekspeler"),
    ] {
        if heeft(t, w) {
            let mut s = format!("{}{}", if naam == "rekenmachine" || naam == "muziekspeler" { if klein { "een kleine " } else { "een " } } else { voor }, naam);
            if let Some(p) = t.find(" heet ").or_else(|| t.find(" met de naam ")) {
                let na = &t[p..].split_whitespace().skip_while(|x| ["heet", "met", "de", "naam"].contains(x)).next().unwrap_or("").to_string();
                if !na.is_empty() {
                    s.push_str(&format!(" met de naam {}", hoofdletter(na)));
                }
            }
            return Some(s);
        }
    }
    None
}

/// Welke soorten onderdelen worden genoemd (zonder dubbele).
fn soorten_in(t: &str) -> Vec<String> {
    let mut uit: Vec<String> = Vec::new();
    for (woorden, soort, _) in SOORTEN {
        if woorden.iter().any(|w| heeft(t, w) || t.split_whitespace().any(|x| x.ends_with(w) && x.len() > w.len() + 2 && *w == "knop")) && !uit.iter().any(|x| x == soort) {
            uit.push(soort.to_string());
        }
    }
    if uit.is_empty() && TYPEN.iter().any(|z| t.contains(z)) {
        uit.push("tekstvak".to_string());
    }
    uit
}

/// Splitst een zin in stukken bij "en" en komma's: "een knop Opslaan en een tekstvak".
fn delen_van(zin: &str) -> Vec<String> {
    let mut delen = Vec::new();
    let mut nu: Vec<&str> = Vec::new();
    for w in zin.split_whitespace() {
        let kaal = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
        if kaal == "en" {
            delen.push(nu.join(" "));
            nu.clear();
            continue;
        }
        if w.ends_with(',') {
            nu.push(w.trim_end_matches(','));
            delen.push(nu.join(" "));
            nu.clear();
            continue;
        }
        nu.push(w);
    }
    if !nu.is_empty() {
        delen.push(nu.join(" "));
    }
    delen.into_iter().filter(|d| !d.trim().is_empty()).collect()
}

const STOP: &[&str] = &[
    "en", "met", "voor", "om", "die", "dat", "bij", "erbij", "toe", "ook", "onder", "boven", "in", "op", "aan", "erin", "eronder", "erboven", "waarmee",
    "zodat", "er", "maar", "nog", "dan", "graag", "toevoegen", "maken", "zetten", "hebben", "komen", "plaatsen", "de", "het", "een", "naast", "waar",
    "is", "moet", "hoort", "wil", "ik", "je", "mijn", "van", "zijn", "staat", "staan", "komt",
];

/// Onderdelen in een stuk van een zin, met naam en doel.
fn onderdelen_in(deel: &str) -> Vec<Onderdeel> {
    let t = laag(deel);
    let woorden: Vec<String> = deel.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '\'').to_string()).filter(|w| !w.is_empty()).collect();
    let laag_w: Vec<String> = woorden.iter().map(|w| w.to_lowercase()).collect();
    let mut uit = Vec::new();
    for soort in soorten_in(&t) {
        let syn = SOORTEN.iter().find(|s| s.1 == soort).map(|s| s.0).unwrap_or(&[]);
        let plek = laag_w.iter().position(|w| syn.contains(&w.as_str()) || (soort == "knop" && w.ends_with("knop") && w.len() > 6));
        let mut naam = None;
        if let Some(p) = plek {
            // "opslaanknop"
            if soort == "knop" && laag_w[p].ends_with("knop") && laag_w[p].len() > 6 {
                naam = Some(hoofdletter(&laag_w[p][..laag_w[p].len() - 4]));
            }
            // "knop Opslaan" of knop "Opslaan"
            if naam.is_none() {
                if let Some(volgend) = woorden.get(p + 1) {
                    let l = volgend.to_lowercase();
                    if !STOP.contains(&l.as_str()) && !KUNNEN.iter().any(|k| k.0 == l && soort != "knop") && soort != "tekstvak" {
                        naam = Some(hoofdletter(volgend));
                    }
                }
            }
            // "knop om op te slaan"
            if naam.is_none() && soort == "knop" {
                if let Some(om) = laag_w.iter().skip(p).position(|w| w == "om").map(|x| x + p) {
                    if let Some(te) = laag_w.iter().skip(om).position(|w| w == "te").map(|x| x + om) {
                        let mut n: String = laag_w[om + 1..te].iter().filter(|w| !["het", "de", "dit", "dat", "alles"].contains(&w.as_str())).cloned().collect::<Vec<_>>().join("");
                        if let Some(ww) = laag_w.get(te + 1) {
                            n.push_str(ww);
                            naam = Some(hoofdletter(&n));
                        }
                    }
                }
            }
            // "opslaan knop"
            if naam.is_none() && p > 0 {
                let ervoor = &laag_w[p - 1];
                if KUNNEN.iter().any(|k| k.0 == ervoor) {
                    naam = Some(hoofdletter(ervoor));
                }
            }
        }
        // Doel: "voor mijn verhaal".
        let mut doel = None;
        if let Some(v) = laag_w.iter().position(|w| w == "voor") {
            let na: Vec<String> = laag_w[v + 1..]
                .iter()
                .take_while(|w| !["en", "met", "waar", "dat", "die"].contains(&w.as_str()))
                .map(|w| match w.as_str() {
                    "mijn" => "je".to_string(),
                    "mij" => "jou".to_string(),
                    x => x.to_string(),
                })
                .collect();
            if !na.is_empty() && na.len() <= 5 {
                doel = Some(format!("voor {}", na.join(" ")));
            }
        } else if soort == "tekstvak" && TYPEN.iter().any(|z| t.contains(z)) {
            doel = Some("om tekst in te typen".to_string());
        }
        let groot = bevat_een(&t, &[" groot ", " grote ", " meer regels"]) && soort == "tekstvak";
        uit.push(Onderdeel { soort, naam, doel, groot });
    }
    uit
}

/// Wat het programma moet kunnen (alleen in zinnen zonder onderdeel ervoor).
fn kunnen_in(t: &str) -> Vec<String> {
    let mut uit = Vec::new();
    let voorwerp = t
        .split_whitespace()
        .find(|w| w.ends_with("bestanden") || w.ends_with("bestand") || *w == "foto's" || *w == "liedjes" || *w == "muziek")
        .map(|w| if w.ends_with("bestand") { format!("{}en", w) } else { w.to_string() });
    let woorden: Vec<&str> = t.split_whitespace().collect();
    for (i, w) in woorden.iter().enumerate() {
        if let Some((_, k)) = KUNNEN.iter().find(|k| k.0 == *w) {
            // "knop Opslaan" of "opslaan knop": dat is de naam van een knop, niet iets wat het programma kan.
            let bij_knop = woorden.get(i.wrapping_sub(1)).map_or(false, |x| x.contains("knop")) || woorden.get(i + 1).map_or(false, |x| x.contains("knop"));
            if bij_knop {
                continue;
            }
            // "om op te slaan" hoort bij een knop.
            let s = if *k == "openen" { format!("{} openen", voorwerp.clone().unwrap_or_else(|| "bestanden".to_string())) } else { k.to_string() };
            if !uit.contains(&s) {
                uit.push(s);
            }
        }
    }
    uit
}

/// Wat er aan een onderdeel moet veranderen: "groter", "rood", "anders", of "naam X".
fn eigenschap(t: &str) -> Option<String> {
    for w in ["groter", "kleiner", "breder", "smaller", "hoger", "lager", "dikker", "dunner", "duidelijker", "mooier", "donkerder", "lichter", "vetter", "langer", "korter"] {
        if heeft(t, w) {
            return Some(w.to_string());
        }
    }
    for (kleur, _) in crate::taal::KLEUREN {
        if heeft(t, kleur) {
            return Some(kleur.to_string());
        }
    }
    if let Some(p) = t.find(" noem ").or_else(|| t.find(" hernoem ")) {
        let na: Vec<&str> = t[p..].split_whitespace().collect();
        if let Some(q) = na.iter().position(|w| *w == "naar" || *w == "voortaan") {
            if let Some(n) = na.get(q + 1) {
                return Some(format!("naam {}", n));
            }
        }
        if let Some(n) = na.last() {
            if na.len() >= 3 {
                return Some(format!("naam {}", n));
            }
        }
    }
    if bevat_een(t, &[" anders", "verander", "aanpassen", "pas aan", "veranderen"]) {
        return Some("anders".to_string());
    }
    None
}

fn zelfde_soort_wijziging(oud: &str, nieuw: &str) -> bool {
    let tegen = [("groter", "kleiner"), ("breder", "smaller"), ("hoger", "lager"), ("dikker", "dunner"), ("donkerder", "lichter"), ("langer", "korter")];
    let laatste = oud.split_whitespace().last().unwrap_or("");
    laatste == nieuw || tegen.iter().any(|(a, b)| (laatste == *a && nieuw == *b) || (laatste == *b && nieuw == *a)) || (crate::taal::KLEUREN.iter().any(|k| k.0 == laatste) && crate::taal::KLEUREN.iter().any(|k| k.0 == nieuw))
}
