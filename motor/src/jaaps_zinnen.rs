//! Jaaps eigen zinnen: wat Jaap tegen Claude Code zei om dit programma te laten bouwen, en de
//! wensen uit zijn opdrachtbestand. Elke zin heeft een betekenis die het programma al kent.
//! Lijkt wat je zegt veel op een van deze zinnen, dan weet het programma beter wat je bedoelt.
//!
//! Geen AI: het programma telt welke woorden hetzelfde zijn.
//!
//! De betekenis is een gewone zin die het programma begrijpt (bijvoorbeeld "maak een
//! installatiebestand"), of "uitleg: ..." voor een antwoord op de vraag of iets kan.

/// (Jaaps zin, wat hij betekent)
pub const ZINNEN: &[(&str, &str)] = &[
    // --- Wat Jaap vanavond zei (17 november 2026), letterlijk of bijna letterlijk ---
    ("Ga maar ook verder met stap twee en drie en vier en totdat het helemaal klaar is", "ga verder"),
    ("Ik zei dat je verder moest gaan", "ga verder"),
    ("Ga maar verder met de punten van de lijst", "open mijn wensenlijst"),
    ("Ik zei dat je verder moest gaan met het hele programma alle punten die in dat bestandje staan", "open mijn wensenlijst"),
    ("Het installatiebestand en het compileren moet er ook meteen inzitten", "maak een installatiebestand"),
    ("Maak alvast maar het installatieprogramma zodat ik het op mijn eigen computer kan testen", "maak een installatiebestand"),
    ("Geef mij hier het installatiebestand direct", "maak een installatiebestand"),
    ("Geef het nieuwe installatiebestand zodra het klaar is", "maak een installatiebestand"),
    ("Ik bedoel dat ik zelf met beschrijvingen een installatiebestand kan maken waarin het programma zit", "maak een installatiebestand"),
    ("Ik wil eerst weten of je snapt wat ik wil", "beschrijf mijn programma"),
    ("Heb je nou begrepen wat ik wil", "beschrijf mijn programma"),
    ("Snap je het nog", "beschrijf mijn programma"),
    ("Snap je wat ik bedoel", "beschrijf mijn programma"),
    ("En wat is de rest, kun je dat even opnoemen", "open mijn wensenlijst"),
    ("Wat moet er nog voor het lijstje gebeuren", "open mijn wensenlijst"),
    ("Kun jij nu vijf dingen kiezen die we nu meteen kunnen inbouwen", "open mijn wensenlijst"),
    ("Wat zijn de volgende twee grote dingen", "open mijn wensenlijst"),
    ("Ik moet ook weer programma code kunnen openen van de programma's die ik heb geschreven", "open mijn programma"),
    ("Maar denk je dat ik nu wel een programma kan maken met mijn nieuwe programma", "uitleg: Ja. Schrijf je zinnen, druk F7 om te controleren, F5 om het te proberen en F8 voor een exe. Ctrl+F8 maakt een installatiebestand."),
    ("Ik wil met omschrijvingen een programma kunnen signeren met mijn eigen certificaat", "onderteken het programma"),
    ("Een eigen licentie schrijven met omschrijvingen voor mijn programma's", "uitleg: Dat kan. Schrijf: In de licentie staat: ... (één regel per zin), of: De licentie staat in het bestand Licentie.txt."),
    ("Ik wil een eigen privacyverklaring kunnen schrijven", "uitleg: Dat kan. Schrijf: In de privacyverklaring staat: ... of: De privacyverklaring staat in het bestand Privacy.txt."),
    ("Een tekstbestandje openen met een omschrijving voor een programma waar ik mee bezig ben", "uitleg: Zet bovenaan het bestand: Dit is een omschrijving voor het programma Naam (of: Dit is code voor het programma Naam), en open het met Ctrl+O. Ik vraag dan wat ermee moet."),
    ("Een eigen pictogram voor mijn programma", "ik wil een eigen pictogram"),
    ("Eigen meldingen die je schermlezer uitspreekt", "uitleg: Dat kan met: Zeg \"Opgeslagen\". De schermlezer zegt het dan, zonder meldingsvenster."),
    ("Ook tabbladen inbouwen en maak daar een mooie omschrijving voor", "maak een tabblad"),
    ("Kun je het geluid met MP3 en zip alvast bouwen", "uitleg: Dat kan: Speel het geluid muziek.mp3, en: Pak de map Foto's in tot het zipbestand Foto's.zip."),
    ("Gewone normale mappenstructuur bij het openen met Control O net zoals in Word en Kladblok", "uitleg: Ctrl+O opent het gewone Openen-venster van Windows: Enter op een map gaat erin, Backspace gaat een map hoger."),
    ("Alles wat Rust ook kan, met beschrijvingen, maar ik moet ook mijn eigen beschrijving kunnen bedenken", "uitleg: Je eigen zinnen bedenk je met: Nieuwe zin: Begroet [wie], dan de betekenis, en Einde van de zin. Voor echte Rust-code: Maak een Rust-stukje met als naam ..."),
    ("Alles wat ik tegen jou zeg moet gekoppeld worden aan functies binnen mijn programma", "wat kan ik zeggen"),
    ("Maak een zoekfunctie van dingen die ik zou kunnen zeggen om dingen uit te voeren", "wat kan ik zeggen"),
    ("3D-audio is niet alleen links en rechts, voor of achter mij, maar ook onder en boven mij", "uitleg: Dat kan: Speel het geluid deurbel.wav boven me, of onder me, of links achter me. Werkt met WAV-bestanden."),
    ("Kan ik ook .txt-bestanden openen maar ook maken met het programma", "uitleg: Ja. In de editor: Ctrl+N nieuw, Ctrl+O openen, Ctrl+S opslaan als .txt. In je eigen programma: Schrijf \"Hallo\" in het bestand notities.txt, Toon de inhoud van het bestand notities.txt, of: Open het bestand notities.txt (in Kladblok)."),
    ("Kijk naar Bouwmaatje of je daaruit nog iets kunt halen en optimaliseer dit programma zodat het snel blijft", "uitleg: Het programma controleert je zinnen in een paar milliseconden. Alleen het bouwen van een exe duurt een paar seconden: dat doet de Rust-compiler."),
    ("Alle prompts die ik jou geef en het opdrachtenbestand moet je ook in het programma zetten", "uitleg: Dat is gedaan: je eigen zinnen staan als voorbeelden in het programma. Lijkt wat je zegt erop, dan begrijpt het je beter of vraagt het: Bedoel je zoiets als ...?"),
    ("Ik wil ook prompts kunnen invoeren om internetradio te bouwen of om een website te bouwen, het moet ook echt werken", "uitleg: Dat kan. Internetradio: Speel de radiozender NPO Radio 2, en Stop de radio (zie het voorbeeld Mijn radio.txt). Een website: begin met Maak een website met als titel ..., en druk F5 (zie Mijn radiosite (website).txt)."),
    ("Of mijn audio editor, daar zit ook internetradio in", "uitleg: De zenders komen uit Computerhulp Maatje en Jaaps Audio Editor. Een zender die niet in de lijst staat, zoekt je programma op bij radio-browser.info, net als de Audio Editor."),
    ("3D-audio is niet alleen links en rechts maar ook onder en boven mij", "uitleg: Dat kan: Speel het geluid deurbel.wav boven me, onder me, of links achter me. Werkt met WAV-bestanden."),
    ("Als ik vraag om een radioprogramma te maken, wil ik ook alles zelf kunnen bouwen: vensters, menu, knoppen, en de streaming URL moet ik ergens vandaan kunnen halen", "uitleg: Dat kan. Vul de keuzelijst Zenders met de radiozenders (of: met de radiozenders van het land Nederland). Speel de radiozender die gekozen is in de keuzelijst Zenders. Het adres krijg je met: het adres van de radiozender \"Qmusic\". Zie het voorbeeld Mijn radio.txt."),
    ("Kan ik nu al prompts geven in een geopend projectbestand en gaat het programma dat dan ook echt uitvoeren", "uitleg: Ja. Schrijf je verzoek gewoon als regel in je programma, bijvoorbeeld: zet er een knop Opslaan bij. Druk F7: ik stel de echte zin voor en zet hem erin als je ja zegt. Een opdracht zoals: maak er een exe van, voer ik uit na ja."),
    ("Ga maar verder met de punten van de lijst", "open mijn wensenlijst"),
    ("Ik moet verschillende motoren kunnen maken om dingen te maken, zet er drie voorbeelden in", "uitleg: Dat kan: een motor is een bestand met je eigen zinnen in de map Motoren (menu Help, Mijn motoren). Gebruik hem met: Gebruik de motor Radio. Voorbeelden: Radio, Brief en Quiz."),
    ("Ik wil een website maken zoals jaapwortelproducties.nl", "uitleg: Dat kan: zie het voorbeeld Jaap Wortel Producties (website).txt, met een menu, een knop, een citaat, stappen, een onderkant, eigen kleuren en een pagina niet gevonden."),
    ("Zorg ervoor dat ik het in gewone taal kan vragen, net zoals ik dat bij jou doe", "uitleg: Dat kan: zeg (Ctrl+E) of schrijf in je programma wat je wilt, ook lang en met meer dingen tegelijk. Bijvoorbeeld: maak een venster met de titel Adressen, met een tekstvak Naam en een knop Opslaan, en als ik op Opslaan druk, schrijf de naam in het bestand adressen.txt en zeg opgeslagen."),
    ("Gekocht certificaat mag eraf, want dat ga ik niet doen", "uitleg: Afgesproken: alleen je eigen certificaat. Schrijf: Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel."),
    // --- De wensen uit het opdrachtbestand (punt 3) ---
    ("Variabelen: iets onthouden en later veranderen", "uitleg: Dat kan: Onthoud teller is 0, en: Verander teller naar 5. Of: Tel 1 op bij teller."),
    ("Rekenen en vergelijken, het programma moet dat allemaal voor mij doen", "uitleg: Dat kan: Toon 3 plus 4 keer 2, Toon de wortel van 16, Toon 10 gedeeld door 3 afgerond op 2 decimalen. Vergelijken: Als teller is groter dan 5."),
    ("Keuzes maken als dan anders en opdrachten herhalen met een grens", "uitleg: Dat kan: Als ... (en Anders, Einde als). Herhaal 10 keer, of: Herhaal zolang ..., hoogstens 100 keer."),
    ("Lijsten met meerdere waarden", "uitleg: Dat kan: Maak een lijst met als naam boodschappen, en: Voeg \"melk\" toe aan de lijst boodschappen."),
    ("Eigen stukjes hergebruiken met een naam, invoer en een uitkomst", "uitleg: Dat kan: Maak een stukje met als naam Verdubbel, met als invoer getal. Geef als uitkomst getal keer 2. Einde van het stukje."),
    ("Aanhalingstekens in woorden beschrijven, aanhalingsteken openen en aanhalingsteken sluiten", "uitleg: Dat kan: dicteer aanhalingsteken openen en aanhalingsteken sluiten, dan worden het echte aanhalingstekens."),
    ("Losse opdrachten los proberen zonder een heel programma te maken", "probeer deze regel"),
    ("Vensters, ook meerdere vensters over elkaar", "uitleg: Dat kan: elke zin Maak een venster met als titel ... maakt een nieuw venster."),
    ("Knoppen, keuzelijsten, comboboxen, selectievakjes, tekstvakken, labels, tabellen, menu's en werkbalken", "uitleg: Dat kan allemaal. Bijvoorbeeld: Maak een keuzelijst met als naam Kleur, of: Maak een werkbalk met de knoppen Nieuw, Openen en Opslaan."),
    ("Een tekstverwerker kunnen maken", "maak een tekstverwerker voor me"),
    ("Kleuren instellen en zelf een lay-out kiezen", "uitleg: Kleuren kan: Geef het venster de kleur donkerblauw. Knoppen naast elkaar: Zet de knop Annuleren naast de knop Opslaan. Verder staan onderdelen onder elkaar."),
    ("Vooraf vertellen wat er te zien is en of kleuren bij elkaar passen", "vertel wat er op het scherm komt"),
    ("Volledige controle over het toetsenbord: sneltoetsen, tabvolgorde en focus", "uitleg: Dat kan: Geef de knop Opslaan de sneltoets Ctrl+S, Zet de focus op het tekstvak Naam, en: De Tab-volgorde is Naam, Leeftijd en Opslaan."),
    ("Geluiden aan een programma toevoegen", "uitleg: Dat kan: Speel het geluid deurbel.wav, of: Speel een piep."),
    ("Mono, stereo en 3D-audio, ruimtelijk geluid alsof iets links of rechts of achter me klinkt", "uitleg: Dat kan: Speel het geluid deurbel.wav links (of rechts, achter, voor, boven of onder me). Werkt met WAV-bestanden, eenvoudig gemaakt (zonder HRTF)."),
    ("Geluid opnemen van een microfoon en opnamen opslaan", "uitleg: Dat kan: Begin met opnemen, en: Stop met opnemen en sla het op in het bestand opname.wav."),
    ("Muziek afspelen en eigen muziekspelers bouwen", "uitleg: Afspelen kan: Speel het geluid liedje.mp3, en Stop het geluid. Een hele muziekspeler bouw je met knoppen en een lijst; spoelen en een afspeellijst als onderdeel kunnen nog niet."),
    ("Een echte audio-editor bouwen zoals Goldwave", "kan nog niet: geluid bewerken"),
    ("QR-codes, streepjescodes en barcodes maken en lezen", "uitleg: Maken kan: Maak een QR-code van \"tekst\" in het bestand qr.png, en: Maak een streepjescode van \"12345\" in het bestand code.png. Lezen kan nog niet."),
    ("Foto's en afbeeldingen tonen met een tekst die de schermlezer voorleest", "uitleg: Dat kan: Maak een afbeelding van het bestand hond.jpg met de beschrijving Mijn hond in het park."),
    ("3D-afbeeldingen gebruiken en beschrijven", "kan nog niet: 3D-beeld"),
    ("Tekstbestanden lezen en schrijven", "uitleg: Dat kan: Schrijf \"Hallo\" in het bestand notities.txt, en: Toon de inhoud van het bestand notities.txt."),
    ("Logbestanden schrijven die ook vastleggen wat er fout gaat", "uitleg: Dat kan: Schrijf in het logboek \"Gestart\". Fouten komen er vanzelf in."),
    ("Word-documenten en PDF-bestanden maken en lezen", "uitleg: Dat kan: Schrijf ... in het Word-bestand brief.docx, en: Toon de tekst van het Word-bestand brief.docx. Hetzelfde met een PDF-bestand. Een gescande PDF kan ik niet lezen."),
    ("Mappen maken", "uitleg: Dat kan: Maak een map met als naam Mijn notities."),
    ("ZIP-bestanden maken en uitpakken, en RAR", "uitleg: Zip kan: Pak de map Foto's in tot het zipbestand Foto's.zip. RAR kan niet: inpakken mag niet zonder licentie van RAR."),
    ("Een eigen installatiebestand met en zonder beheerdersrechten", "maak een installatiebestand"),
    ("32-bits en 64-bits programma's", "kan nog niet: 32-bits programma's (64-bits kan wel)"),
    ("Handleidingen in het programma met F1 en Escape om te sluiten", "uitleg: Dat kan: In de handleiding staat: ..., of: De handleiding staat in het bestand Handleiding.txt. In je programma opent F1 de handleiding en Escape sluit hem."),
    ("Een verbinding met een andere computer", "uitleg: Stuur \"hallo\" naar de computer Keuken, en op die computer: Toon het bericht dat binnenkomt."),
    ("Werken met internet: gegevens ophalen en versturen", "uitleg: Dat kan: Toon de tekst van de webpagina https://..., en: Verstuur \"hallo\" naar het adres https://..."),
    ("PHP-scripts en CSS-stijlen schrijven", "uitleg: Op een website: Zet een contactformulier met het e-mailadres ... (PHP), en Voeg CSS toe: \"...\". Een eigen PHP-script schrijven kan nog niet."),
    ("Updates via een Dropbox-link of pCloud-link", "uitleg: Dat kan: De updates staan op het adres https://www.dropbox.com/... en: Zoek naar updates."),
    ("Een spraakherkenner bouwen met microfoons, opnemen en herkennen", "uitleg: Onthoud gezegd is wat ik zeg (spraakherkenning van Windows: Engels en een paar andere talen, niet Nederlands). Opnemen: Begin met opnemen."),
    ("Een programma dat zelf tekst voorleest", "uitleg: Dat kan: Lees voor \"Goedemorgen\". Dat gebruikt de stem van Windows."),
    ("Bepalen welke informatie naar de brailleleesregel gaat", "uitleg: Dat kan: Toon op de brailleleesregel \"Opgeslagen\". Met JAWS gaat het rechtstreeks naar de leesregel; met NVDA als korte melding. Nog niet getest met een echte leesregel."),
    ("Werken met allerlei hardware om apparaten toegankelijk te maken", "uitleg: Een apparaat aan een COM-poort (zoals een Arduino): Stuur \"aan\" naar de poort COM3, en: wat er van de poort COM3 komt."),
    ("Whisper-onderdelen en taalmodellen als mogelijkheid voor mijn programma's", "kan nog niet: Whisper en taalmodellen in je eigen programma (de taal zelf blijft zonder AI)"),
    ("Engelse tekst en Engelse opdrachten begrijpen", "uitleg: Engelse basiszinnen werken, zoals Make a window with the title Hello. Menu Programma, Zet mijn programma om naar het Engels."),
    ("Mijn Nederlandse opdracht naar het Engels omzetten", "zet mijn programma om naar het Engels"),
    ("Het programma onthoudt mijn keuzes en vaste formuleringen", "uitleg: Dat kan: menu Help, Mijn eigen woorden, Mijn eigen zinnen en Mijn eigen opdrachten. Na een verbeterde tikfout vraag ik of ik het mag onthouden."),
    ("Motorschrijver, extensies en drivers", "uitleg: Een motor is een bestand met je eigen zinnen in de map Motoren (menu Help, Mijn motoren). Gebruik hem met: Gebruik de motor Radio. Voorbeelden: Radio, Brief en Quiz."),
    ("Mijn bestaande programma's nabouwen, zoals Bouwmaatje, Echo Worlds, Bestandsbrug en mijn internetradio", "kan nog niet: bestaande programma's in één keer nabouwen (stap voor stap met de zinnen kan wel)"),
    ("Mijn taal moet alles kunnen wat Rust kan", "uitleg: Nog niet alles, maar met een Rust-stukje kun je elke Rust-code toevoegen: Maak een Rust-stukje met als naam ..."),
];

/// Woorden die niets zeggen over wat je bedoelt.
const NIETS: &[&str] = &[
    "ik", "je", "jij", "de", "het", "een", "en", "of", "dat", "die", "dit", "er", "d'r", "maar", "ook", "nu", "nog", "wel", "even",
    "kan", "kun", "kunt", "moet", "mag", "wil", "zou", "is", "zijn", "van", "voor", "met", "in", "op", "aan", "naar", "als", "dan",
    "me", "mij", "m'n", "mijn", "we", "wat", "hoe", "alle", "alles", "zo", "hè", "he", "hier", "daar", "toch", "nou", "graag",
];

fn woorden(t: &str) -> Vec<String> {
    let t: String = t.to_lowercase().chars().map(|c| if c.is_alphanumeric() || c == '-' { c } else { ' ' }).collect();
    t.split_whitespace()
        .map(|w| match w {
            "stant" | "best" | "stand" => "bestand",
            "exxon" => "exe",
            "error" => "editor",
            "plons" | "promts" | "prompts" => "zinnen",
            w => w,
        })
        .filter(|w| w.chars().count() >= 3 && !NIETS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Lijken twee woorden genoeg op elkaar? Zelfde begin van 6 letters, of gelijk.
fn zelfde(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    let (ac, bc): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    ac.len() >= 6 && bc.len() >= 6 && ac[..6] == bc[..6]
}

/// Hoeveel een zin lijkt op een van Jaaps zinnen (0 tot 1), en welke het meest.
pub fn lijkt_op(zin: &str) -> Option<(f32, &'static str, &'static str)> {
    let a = woorden(zin);
    if a.is_empty() {
        return None;
    }
    let mut beste: Option<(f32, &str, &str)> = None;
    for (jaap, betekenis) in ZINNEN {
        let b = woorden(jaap);
        let gelijk = a.iter().filter(|x| b.iter().any(|y| zelfde(x, y))).count();
        let score = 2.0 * gelijk as f32 / (a.len() + b.len()) as f32;
        if beste.map_or(true, |x| score > x.0) {
            beste = Some((score, jaap, betekenis));
        }
    }
    beste
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jaaps_zinnen_worden_herkend() {
        let (s, _, b) = lijkt_op("ga maar verder met de punten van de lijst").unwrap();
        assert!(s > 0.9);
        assert_eq!(b, "open mijn wensenlijst");
        let (s, _, b) = lijkt_op("kan het ook 3D-audio boven en onder mij").unwrap();
        assert!(s >= 0.4, "{}", s);
        assert!(b.contains("boven me"));
        assert!(lijkt_op("bananen zijn geel").map_or(true, |x| x.0 < 0.4));
    }

    #[test]
    fn elke_betekenis_wordt_begrepen() {
        use crate::opdrachten::{begrijp_alles, Antwoord, Begrip};
        for (jaap, betekenis) in ZINNEN {
            if betekenis.starts_with("uitleg:") || betekenis.starts_with("kan nog niet:") {
                continue;
            }
            let a = begrijp_alles(betekenis, &[]);
            assert!(!matches!(a, Antwoord::Editor(Begrip::Onbekend) | Antwoord::Editor(Begrip::Kies(_))), "{} -> {} gaf {:?}", jaap, betekenis, a);
        }
    }
}
