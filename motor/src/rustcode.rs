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
    /// Binnen een eigen stukje: de namen die alleen in dat stukje bestaan.
    lokaal: Option<HashMap<String, usize>>,
    /// De namen van de eigen stukjes, op volgorde.
    stukjes: Vec<String>,
}

/// Alle namen die binnen een blok worden onthouden (voor de namen van een stukje).
fn onthouden_in(lijst: &[Opdracht], uit: &mut Vec<String>) {
    for o in lijst {
        match &o.soort {
            Soort::Onthoud { naam, .. } | Soort::VoorElk { naam, .. } | Soort::Tel { naam, .. } => {
                if !uit.contains(naam) {
                    uit.push(naam.clone());
                }
            }
            _ => {}
        }
        let mut kopie = o.soort.clone();
        for blok in kopie.blokken_mut() {
            onthouden_in(blok, uit);
        }
    }
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

    /// Waar een naam staat: in het stukje (P::L) of in het hele programma (P::G).
    fn plek(&mut self, naam: &str) -> String {
        if let Some(i) = self.lokaal.as_ref().and_then(|l| l.get(naam)) {
            return format!("P::L({})", i);
        }
        format!("P::G({})", self.naam_nr(naam))
    }

    fn aanroep(&mut self, naam: &str, invoer: &[Uitdrukking], r: usize) -> String {
        let nr = self.stukjes.iter().position(|n| n == naam).unwrap_or(0);
        let args: Vec<String> = invoer.iter().map(|i| self.uitdrukking(i, r)).collect();
        format!("{{ let invoer = vec![{}]; stukje_{}(s, invoer, {})? }}", args.join(", "), nr, r)
    }

    fn uitdrukking(&mut self, u: &Uitdrukking, r: usize) -> String {
        match u {
            Uitdrukking::Vast(Waarde::Getal(g)) => format!("W::G({:?}f64)", g),
            Uitdrukking::Vast(Waarde::Tekst(t)) => format!("W::T({}.to_string())", rust_tekst(t)),
            Uitdrukking::Vast(Waarde::Waarheid(b)) => format!("W::B({})", b),
            Uitdrukking::Naam(n) => {
                let p = self.plek(n);
                format!("s.lees({}, {})?", p, r)
            }
            Uitdrukking::LegeLijst => "W::L(Vec::new())".into(),
            Uitdrukking::Ingebouwd { wat, invoer } => {
                let args: Vec<String> = invoer.iter().map(|i| self.uitdrukking(i, r)).collect();
                if wat.nummer() >= 12 {
                    format!("functie({}, vec![{}], {})?", wat.nummer(), args.join(", "), r)
                } else if wat.nummer() >= 9 {
                    format!("{{ let invoer = vec![{}]; s.tabel_waarde({}, invoer, {})? }}", args.join(", "), wat.nummer(), r)
                } else {
                    format!("ingebouwd({}, vec![{}], {})?", wat.nummer(), args.join(", "), r)
                }
            }
            Uitdrukking::Aantal(a) => {
                let a = self.uitdrukking(a, r);
                format!("aantal_in({}, {})?", a, r)
            }
            Uitdrukking::Element(a, b) => {
                let a = self.uitdrukking(a, r);
                let b = self.uitdrukking(b, r);
                format!("element({}, {}, {})?", a, b, r)
            }
            Uitdrukking::Aanroep { naam, invoer } => {
                let a = self.aanroep(naam, invoer, r);
                format!("uitkomst_van({}, {}, {})?", a, rust_tekst(naam), r)
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
            Voorwaarde::En(a, b) => {
                let a = self.voorwaarde(a, r);
                let b = self.voorwaarde(b, r);
                format!("({} && {})", a, b)
            }
            Voorwaarde::Of(a, b) => {
                let a = self.voorwaarde(a, r);
                let b = self.voorwaarde(b, r);
                format!("({} || {})", a, b)
            }
            Voorwaarde::Bevat(a, b) => {
                let a = self.uitdrukking(a, r);
                let b = self.uitdrukking(b, r);
                format!("bevat({}, {}, {})?", a, b, r)
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
                Soort::VulKeuzelijst { lijst, waarde } => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(waarde, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.vul_keuzelijst({}, w, {})?; }}", u, rust_tekst(lijst), r));
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
                    let i = self.plek(naam);
                    let u = self.uitdrukking(waarde, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.zet({}, w); }}", u, i));
                }
                Soort::VoegToeAanLijst { waarde, lijst } | Soort::HaalUitLijst { waarde, lijst } => {
                    self.opmerking(diepte, r);
                    let i = self.plek(lijst);
                    let u = self.uitdrukking(waarde, r);
                    let f = if matches!(o.soort, Soort::VoegToeAanLijst { .. }) { "voeg_toe" } else { "haal_uit" };
                    self.regel(diepte, &format!("{{ let w = {}; s.{}({}, w, {})?; }}", u, f, i, r));
                }
                Soort::SchrijfBestand { waarde, pad, toevoegen } => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(waarde, r);
                    let p = self.uitdrukking(pad, r);
                    self.regel(diepte, &format!("{{ let w = {}; let p = {}; schrijf_bestand(p, w, {}, {})?; }}", w, p, toevoegen, r));
                }
                Soort::Piep { hoogte, duur } => {
                    self.opmerking(diepte, r);
                    let h = self.uitdrukking(hoogte, r);
                    let d = self.uitdrukking(duur, r);
                    self.regel(diepte, &format!("{{ let h = {}; let d = {}; s.piep(h, d, {})?; }}", h, d, r));
                }
                Soort::SpeelGeluid { pad, wachten, richting } => {
                    self.opmerking(diepte, r);
                    let p = self.uitdrukking(pad, r);
                    self.regel(diepte, &format!("{{ let p = {}; s.speel(p, {}, {}, {})?; }}", p, wachten, richting, r));
                }
                Soort::Zip { inpakken, map, zip } => {
                    self.opmerking(diepte, r);
                    let m = self.uitdrukking(map, r);
                    let z = self.uitdrukking(zip, r);
                    self.regel(diepte, &format!("{{ let m = {}; let z = {}; zip({}, m, z, {})?; }}", m, z, inpakken, r));
                }
                Soort::Zeg(u) => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(u, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.zeg(w); }}", u));
                }
                Soort::Pictogram(_) => self.opmerking(diepte, r),
                Soort::LeesVoor(u) => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(u, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.lees_voor(w); }}", u));
                }
                Soort::StopVoorlezen => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, "s.stop_voorlezen();");
                }
                Soort::Volume(u) => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(u, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.volume(w, {})?; }}", u, r));
                }
                Soort::MaakTabblad { naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.tabblad({});", rust_tekst(naam)));
                }
                Soort::KiesTabblad { naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.kies_tabblad({}, {})?;", rust_tekst(naam), r));
                }
                Soort::MaakMenu { naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.menu({});", rust_tekst(naam)));
                }
                Soort::MaakMenuregel { naam } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.menuregel({});", rust_tekst(naam)));
                }
                Soort::BestandActie { soort, van, naar } => {
                    self.opmerking(diepte, r);
                    let v = self.uitdrukking(van, r);
                    let n = match naar {
                        Some(n) => format!("Some({})", self.uitdrukking(n, r)),
                        None => "None".into(),
                    };
                    self.regel(diepte, &format!("{{ let v = {}; let n = {}; bestand_actie({}, v, n, {})?; }}", v, n, soort, r));
                }
                Soort::StopGeluid => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, "s.stop_geluid();");
                }
                Soort::Sneltoets { knop, toets } => {
                    self.opmerking(diepte, r);
                    self.regel(
                        diepte,
                        &format!(
                            "s.sneltoets({}, {}, {}, {}, {}, {}, {})?;",
                            rust_tekst(knop),
                            toets.code,
                            toets.ctrl,
                            toets.alt,
                            toets.shift,
                            rust_tekst(&toets.naam),
                            r
                        ),
                    );
                }
                Soort::Kleur { achtergrond, kleur, rgb } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.kleur({}, ({}, {}, {}), {});", achtergrond, rgb.0, rgb.1, rgb.2, rust_tekst(kleur)));
                }
                Soort::Handleiding { .. } => self.opmerking(diepte, r),
                Soort::Document { soort, waarde, pad } => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(waarde, r);
                    let p = self.uitdrukking(pad, r);
                    self.regel(diepte, &format!("{{ let w = {}; let p = {}; schrijf_document({}, p, w, {})?; }}", w, p, soort, r));
                }
                Soort::Verstuur { waarde, adres } => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(waarde, r);
                    let a = self.uitdrukking(adres, r);
                    self.regel(diepte, &format!("{{ let w = {}; let a = {}; verstuur(a, w, {})?; }}", w, a, r));
                }
                Soort::Download { adres, pad } => {
                    self.opmerking(diepte, r);
                    let a = self.uitdrukking(adres, r);
                    let p = self.uitdrukking(pad, r);
                    self.regel(diepte, &format!("{{ let a = {}; let p = {}; download(a, p, {})?; }}", a, p, r));
                }
                Soort::Tabel { tabel, actie } => {
                    self.opmerking(diepte, r);
                    let t = rust_tekst(tabel);
                    let regel = match actie {
                        TabelActie::Kolom(k) => format!("s.tabel_kolom({}, {}, {})?;", t, rust_tekst(k), r),
                        TabelActie::Rij(w) => {
                            let a: Vec<String> = w.iter().map(|u| self.uitdrukking(u, r)).collect();
                            format!("{{ let rij = vec![{}]; s.tabel_rij({}, rij, {})?; }}", a.join(", "), t, r)
                        }
                        TabelActie::HaalGekozenWeg => format!("s.tabel_haal_weg({}, {})?;", t, r),
                        TabelActie::Leeg => format!("s.tabel_leeg({}, {})?;", t, r),
                        TabelActie::Sorteer(k) => format!("s.tabel_sorteer({}, {}, {})?;", t, rust_tekst(k), r),
                        TabelActie::Opslaan(p) => {
                            let p = self.uitdrukking(p, r);
                            format!("{{ let p = {}; s.tabel_csv({}, p, true, {})?; }}", p, t, r)
                        }
                        TabelActie::Inlezen(p) => {
                            let p = self.uitdrukking(p, r);
                            format!("{{ let p = {}; s.tabel_csv({}, p, false, {})?; }}", p, t, r)
                        }
                    };
                    self.regel(diepte, &regel);
                }
                Soort::Wacht(u) => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(u, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.wacht(w, {})?; }}", w, r));
                }
                Soort::StartProgramma(u) | Soort::OpenBestand(u) => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(u, r);
                    let programma = matches!(o.soort, Soort::StartProgramma(_));
                    self.regel(diepte, &format!("{{ let w = {}; s.open(w, {}, {})?; }}", w, programma, r));
                }
                Soort::Venstergedrag(g) => {
                    self.opmerking(diepte, r);
                    let regel = match g {
                        Gedrag::Schermvullend => "s.scherm_gedrag(0, \"\", \"\");".to_string(),
                        Gedrag::VraagBijAfsluiten(v) => format!("s.scherm_gedrag(1, {}, \"\");", rust_tekst(v)),
                        Gedrag::MaarEenKeerOpen => "s.scherm_gedrag(2, \"\", \"\");".to_string(),
                        Gedrag::Tip { knop, tekst } => format!("s.scherm_gedrag(3, {}, {});", rust_tekst(knop), rust_tekst(tekst)),
                    };
                    self.regel(diepte, &regel);
                }
                Soort::Radio(z) => {
                    self.opmerking(diepte, r);
                    let regel = match z {
                        None => format!("s.radio(None, \"\", {})?;", r),
                        Some((naam, u)) => {
                            let a = self.uitdrukking(u, r);
                            format!("{{ let a = {}; s.radio(Some(a), {}, {})?; }}", a, rust_tekst(naam), r)
                        }
                    };
                    self.regel(diepte, &regel);
                }
                Soort::Reservekopie(p) => {
                    self.opmerking(diepte, r);
                    let p = self.uitdrukking(p, r);
                    self.regel(diepte, &format!("{{ let p = {}; reservekopie(p, {})?; }}", p, r));
                }
                Soort::QrCode { waarde, pad } => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(waarde, r);
                    let p = self.uitdrukking(pad, r);
                    self.regel(diepte, &format!("{{ let w = {}; let p = {}; maak_qr(w, p, {})?; }}", w, p, r));
                }
                Soort::Werkbalk(namen) => {
                    self.opmerking(diepte, r);
                    let n: Vec<String> = namen.iter().map(|x| rust_tekst(x)).collect();
                    self.regel(diepte, &format!("s.werkbalk(&[{}]);", n.join(", ")));
                }
                Soort::Opdrachtvak => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, "s.opdrachtvak();");
                }
                Soort::BeginOpname => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.begin_opname({})?;", r));
                }
                Soort::StopOpname(p) => {
                    self.opmerking(diepte, r);
                    let p = self.uitdrukking(p, r);
                    self.regel(diepte, &format!("{{ let p = {}; s.stop_opname(p, {})?; }}", p, r));
                }
                Soort::MaakAfbeelding { pad, beschrijving } => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.afbeelding({}, {}, {})?;", rust_tekst(pad), rust_tekst(beschrijving), r));
                }
                Soort::ZoekUpdates => {
                    self.opmerking(diepte, r);
                    self.regel(diepte, &format!("s.zoek_updates({})?;", r));
                }
                Soort::MaakMap(p) => {
                    self.opmerking(diepte, r);
                    let p = self.uitdrukking(p, r);
                    self.regel(diepte, &format!("{{ let p = {}; maak_map(p, {})?; }}", p, r));
                }
                Soort::Logboek(w) => {
                    self.opmerking(diepte, r);
                    let w = self.uitdrukking(w, r);
                    self.regel(diepte, &format!("{{ let w = {}; logboek(&w.tekst()); }}", w));
                }
                Soort::Doe(Uitdrukking::Aanroep { naam, invoer }) => {
                    self.opmerking(diepte, r);
                    let a = self.aanroep(naam, invoer, r);
                    self.regel(diepte, &format!("{};", a));
                }
                Soort::Doe(_) | Soort::Stukje { .. } => {}
                Soort::GeefUitkomst(u) => {
                    self.opmerking(diepte, r);
                    let u = self.uitdrukking(u, r);
                    self.regel(diepte, &format!("{{ let w = {}; return Ok(Some(w)); }}", u));
                }
                Soort::VoorElk { naam, lijst, blok, einde } => {
                    self.opmerking(diepte, r);
                    let l = self.uitdrukking(lijst, r);
                    let p = self.plek(naam);
                    self.regel(diepte, &format!("for w in lijst_van({}, {})? {{", l, r));
                    self.regel(diepte + 1, &format!("s.stap({})?;", r));
                    self.regel(diepte + 1, &format!("s.zet({}, w);", p));
                    self.lijst(blok, diepte + 1);
                    self.regel(diepte, "}");
                    self.opmerking(diepte, *einde);
                }
                Soort::Tel { naam, van, tot, blok, einde } => {
                    self.opmerking(diepte, r);
                    let a = self.uitdrukking(van, r);
                    let b = self.uitdrukking(tot, r);
                    let p = self.plek(naam);
                    self.regel(diepte, &format!("for i in telling({}, {}, {})? {{", a, b, r));
                    self.regel(diepte + 1, &format!("s.stap({})?;", r));
                    self.regel(diepte + 1, &format!("s.zet({}, W::G(i as f64));", p));
                    self.lijst(blok, diepte + 1);
                    self.regel(diepte, "}");
                    self.opmerking(diepte, *einde);
                }
                Soort::Verander { naam, waarde } => {
                    self.opmerking(diepte, r);
                    let i = self.plek(naam);
                    let u = self.uitdrukking(waarde, r);
                    self.regel(diepte, &format!("{{ let w = {}; s.verander({}, w, {})?; }}", u, i, r));
                }
                Soort::TelOp { naam, waarde } | Soort::TrekAf { naam, waarde } => {
                    self.opmerking(diepte, r);
                    let i = self.plek(naam);
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
    let stukjes: Vec<(&String, &Vec<String>, &Vec<Opdracht>, usize, usize)> = p
        .opdrachten
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::Stukje { naam, invoer, blok, einde, .. } => Some((naam, invoer, blok, o.regel, *einde)),
            _ => None,
        })
        .collect();
    let mut m = Maker {
        namen: HashMap::new(),
        namenlijst: Vec::new(),
        acties: Vec::new(),
        regels,
        uit: String::new(),
        lokaal: None,
        stukjes: stukjes.iter().map(|s| s.0.clone()).collect(),
    };

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

    // De eigen stukjes.
    let rustcode: Vec<Option<String>> = p
        .opdrachten
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::Stukje { rust, .. } => Some(rust.clone()),
            _ => None,
        })
        .collect();
    for (k, (naam, invoer, blok, regel, einde)) in stukjes.iter().enumerate() {
        if let Some(Some(code)) = rustcode.get(k) {
            m.uit.push_str(&format!("/// Het Rust-stukje {} (eigen Rust-code, regel {} tot {}).\n", naam, regel, einde));
            m.uit.push_str(&format!("#[allow(unused_mut, unused_variables)]\nfn stukje_{}(s: &mut Staat, invoer: Vec<W>, r: usize) -> R<Option<W>> {{\n", k));
            m.uit.push_str("    let mut invoer = invoer.into_iter();\n");
            for i in invoer.iter() {
                let id: String = i.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
                m.uit.push_str(&format!("    let mut {}: W = invoer.next().unwrap_or(W::T(String::new()));\n", id));
            }
            m.uit.push_str("    // Rust-stukje: begin van de eigen Rust-code\n");
            m.uit.push_str("    let uit: W = {\n");
            for l in code.lines() {
                m.uit.push_str(&format!("        {}\n", l));
            }
            m.uit.push_str("    };\n    // Rust-stukje: einde van de eigen Rust-code\n    Ok(Some(uit))\n}\n\n");
            continue;
        }
        let mut lokale: Vec<String> = invoer.to_vec();
        onthouden_in(blok, &mut lokale);
        m.lokaal = Some(lokale.iter().enumerate().map(|(i, n)| (n.clone(), i)).collect());
        let namen: Vec<String> = lokale.iter().map(|n| rust_tekst(n)).collect();
        m.uit.push_str(&format!("const LOKAAL_{}: &[&str] = &[{}];\n\n", k, namen.join(", ")));
        m.uit.push_str(&format!("/// Het stukje {}.\n", naam));
        m.uit.push_str(&format!(
            "fn stukje_{0}(s: &mut Staat, invoer: Vec<W>, r: usize) -> R<Option<W>> {{\n    s.begin_stukje(LOKAAL_{0}, invoer, r)?;\n    let uit = stukje_{0}_binnen(s);\n    s.einde_stukje();\n    uit\n}}\n\n",
            k
        ));
        m.uit.push_str(&format!("fn stukje_{}_binnen(s: &mut Staat) -> R<Option<W>> {{\n", k));
        m.opmerking(1, *regel);
        m.lijst(blok, 1);
        m.opmerking(1, *einde);
        m.uit.push_str("    Ok(None)\n}\n\n");
        m.lokaal = None;
    }

    // De handleiding (F1).
    let mut handleiding = Vec::new();
    let mut handleiding_bestand = String::new();
    for o in &p.opdrachten {
        if let Soort::Handleiding { regel, bestand } = &o.soort {
            if let Some(t) = regel {
                handleiding.push(t.clone());
            }
            if let Some(b) = bestand {
                handleiding_bestand = b.clone();
            }
        }
    }
    m.uit.push_str(&format!("const HANDLEIDING: &str = {};\n", rust_tekst(&handleiding.join("\n"))));
    m.uit.push_str(&format!("const HANDLEIDING_BESTAND: &str = {};\n", rust_tekst(&handleiding_bestand)));
    let privacy: Vec<String> = p
        .opdrachten
        .iter()
        .filter_map(|o| match &o.soort {
            Soort::Installatie(InstallatieZin::PrivacyRegel(t)) => Some(t.clone()),
            _ => None,
        })
        .collect();
    m.uit.push_str(&format!("const PRIVACY: &str = {};\n", rust_tekst(&privacy.join("\n"))));
    m.uit.push_str("const PICTOGRAM: &[u8] = &[];\n");
    let mut versie = String::new();
    let mut update_adres = String::new();
    let mut programmanaam = String::new();
    for o in &p.opdrachten {
        match &o.soort {
            Soort::Installatie(InstallatieZin::Versie(v)) => versie = v.clone(),
            Soort::Installatie(InstallatieZin::UpdateAdres(a)) => update_adres = a.clone(),
            Soort::Installatie(InstallatieZin::Naam(n)) => programmanaam = n.clone(),
            _ => {}
        }
    }
    m.uit.push_str(&format!("const VERSIE: &str = {};\n", rust_tekst(&versie)));
    m.uit.push_str(&format!("const UPDATE_ADRES: &str = {};\n", rust_tekst(&update_adres)));
    m.uit.push_str(&format!("const PROGRAMMANAAM: &str = {};\n", rust_tekst(&programmanaam)));
    let zenders: Vec<String> = crate::taal::ZENDERS.iter().map(|(n, u, a)| format!("({}, {}, {})", rust_tekst(n), rust_tekst(u), rust_tekst(a))).collect();
    m.uit.push_str(&format!("const ZENDERLIJST: &[(&str, &str, &str)] = &[{}];\n", zenders.join(", ")));

    let namen: Vec<String> = m.namenlijst.iter().map(|n| rust_tekst(n)).collect();
    let acties: Vec<String> = m.acties.iter().map(|n| rust_tekst(n)).collect();
    m.uit.push_str(&format!("const NAMEN: &[&str] = &[{}];\n", namen.join(", ")));
    m.uit.push_str(&format!("const ACTIES: &[&str] = &[{}];\n\n", acties.join(", ")));
    m.uit.push_str("fn main() {\n    // Je zinnen blijven zo in het programma staan (anders haalt de linker ze weg).\n    std::hint::black_box(JOUW_ZINNEN);\n    start(NAMEN, ACTIES, hoofd, bij_knop);\n}\n");
    // Je eigen zinnen gaan mee in het programma, zodat je ze later weer kunt openen.
    let zinnen = format!("{}{}{}", crate::zinnen_begin(), regels.join("\n"), crate::zinnen_einde());
    m.uit.push_str(&format!(
        "\n/// Je eigen zinnen, zodat Jaap's Programmeertaal ze later weer uit dit programma kan halen.\n#[used]\nstatic JOUW_ZINNEN: &str = {};\n",
        rust_tekst(&zinnen)
    ));

    format!("{}{}", LOOPTIJD, m.uit)
}

/// Alleen het deel dat van jouw zinnen is gemaakt (zonder de looptijd). Om te laten zien.
pub fn alleen_programma(volledig: &str) -> &str {
    match volledig.find("// ==================== Jouw programma") {
        Some(p) => &volledig[p..],
        None => volledig,
    }
}
