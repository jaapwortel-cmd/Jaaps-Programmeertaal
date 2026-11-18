# Jaap's Programmeertaal

Programmeren met gewone Nederlandse zinnen. Je typt of dicteert zinnen, en het programma maakt er een echt Windows-programma van. Ook een installatiebestand, met een eigen licentie als je dat wilt.

**Er zit geen AI in.** Het programma werkt alleen met vaste regels die erin gebouwd zijn. Kent het een zin niet, dan gokt het niet. Het zegt wat het niet begrijpt en stelt één vraag. Of het doet één voorstel waar je ja of nee op zegt.

Dit is versie 0.1.0, een **proefversie**.

---

## 1. Wat het programma doet

1. Je schrijft je zinnen in de editor, of in een gewoon TXT-bestand.
2. Het programma **ontleedt** elke zin: wat bedoel je?
3. Het **controleert** of de zinnen samen kloppen.
4. Het maakt er **Rust-code** van. Bij elke regel Rust-code staat jouw zin erboven.
5. De **Rust-compiler** in het programma bouwt er een .exe van.

Na elke stap kun je zien wat het programma van je zinnen heeft gemaakt: met F6 in gewone taal, en met Shift+F6 als Rust-code.

De motor (de onderdelen die de zinnen lezen, controleren en vertalen) staat los van het scherm. Hij zit in de map `motor`. Zo kun je hem later in Bouwmaatje inbouwen.

## 2. Hoe je begint

1. Download het installatiebestand bij de GitHub-uitgave **proef** (Releases): `Jaaps Programmeertaal 0.1.0 installatie.exe`.
2. Start het. Je hoort eerst de vraag voor wie je installeert: alleen voor jou (geen beheerder nodig) of voor alle gebruikers (wel beheerder nodig).
3. Start Jaap's Programmeertaal. Je komt in het vak Opdrachten.
4. Schrijf of dicteer het voorbeeld uit punt 3 en druk op F5.

Het installatiebestand is niet ondertekend. Windows SmartScreen kan dan zeggen dat het een onbekend programma is. Kies dan "Meer informatie" en "Toch uitvoeren".

## 3. Het voorbeeld

```
Maak een venster met als titel Begroeting
Maak een knop met als naam Welkom
Als ik op de knop Welkom druk
Toon de tekst Hallo
Einde van de knop
```

Druk op F5. Er komt een venster met de titel Begroeting. De knop Welkom heeft de focus. Druk op Enter of op de spatiebalk: je hoort "Hallo". Escape sluit het venster.

## 4. De toetsen

| Toets | Wat het doet |
|---|---|
| Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+Shift+S | Nieuw, openen, opslaan, opslaan als (TXT) |
| F7 | Controleren (testen) |
| F5 | Uitvoeren: bouwen en starten |
| F9 | Alleen deze regel proberen (of je selectie) |
| F6 | Laat zien wat ik van je zinnen maak, in gewone taal |
| Shift+F6 | Laat de Rust-code zien |
| F8 | Bouw een .exe-bestand |
| Ctrl+F8 | Bouw een installatiebestand (met licentie) |
| Ctrl+F6 | Vertel wat er op het scherm komt (vensters, onderdelen, kleuren, Tab-volgorde, tips) |
| Ctrl+L | Op welke regel sta ik? |
| F4 | Naar het vak Meldingen |
| Tab | Wisselen tussen Opdrachten en Meldingen |
| F1 | Handleiding (Escape sluit hem) |

## 5. De zinnen die werken

Hoofdletters maken niet uit. Een punt aan het eind mag. De volledige uitleg staat in de handleiding (`docs/Handleiding.txt`, in het programma met F1).

**Opmerkingen (doen niets)**
- `Opmerking: hier begint het rekenen`
- `Commentaar dit is voor mij`
- `// ook dit doet niets`

**Vensters en knoppen**
- `Maak een venster met als titel Begroeting` (ook: `met de titel`)
- `Maak een knop met als naam Welkom` (ook: `met de naam`)
- `Als ik op de knop Welkom druk` ... `Einde van de knop` (ook: `klik`, `Einde knop`)

**Tekstvakken, labels, selectievakjes en keuzelijsten**
- `Maak een tekstvak met als naam Naam`, `Maak een groot tekstvak met als naam Verhaal`
- `Maak een label met als naam Antwoord`, `Maak een selectievakje met als naam Geluid aan`
- `Maak een keuzelijst met als naam Kleur`, `Maak een uitklaplijst met als naam Maat`
- `Voeg de keuze Rood toe aan de keuzelijst Kleur`
- `Zet de tekst Hallo in het label Antwoord`, `Maak het tekstvak Naam leeg`, `Zet de focus op het tekstvak Naam`
- `Vink het selectievakje Geluid aan aan` (of `uit`)
- Lezen: `de tekst in het tekstvak Naam`, `het getal in het tekstvak Leeftijd`, `de keuze in de keuzelijst Kleur`, `het selectievakje Geluid aan`
- Kiezen: `Als het selectievakje Geluid aan is aangevinkt`, `Als het tekstvak Naam is leeg`
- Voorbeeld: `voorbeelden/Rekenhulp.txt` (twee getallen, plus of keer, uitkomst in een label)

**Tekst tonen**
- `Toon de tekst Hallo`
- `Toon teller`, `Toon de waarde van teller`, `Toon de uitkomst van 3 plus 4`
- `Toon "Je bent" en dan leeftijd en dan "jaar"`

**Onthouden en veranderen**
- `Onthoud teller is 0`, `Onthoud dat teller 0 is`, `Onthoud mijn naam is "Jaap"`, `Onthoud klaar is waar`
- `Verander teller in 5`, `Verander teller naar 5`, `Zet teller op 5`
- `Tel 1 op bij teller`, `Verhoog teller met 1`, `Trek 2 af van teller`, `Verlaag teller met 2`

**Rekenen**: `plus`, `min`, `keer` (of `maal`), `gedeeld door`. Keer en gedeeld door gaan eerst. Getallen in cijfers (`42`, `3,5`) of in woorden (`tien`, `eenentwintig`, `honderd`).

**Kiezen**
- `Als teller is groter dan 5` ... `Anders` ... `Einde als`
- Vergelijkingen: `is groter dan`, `is kleiner dan`, `is gelijk aan` (of `is`), `is niet gelijk aan` (of `is niet`), `is minstens`, `is hoogstens`. Ook: `groter is dan`, `is meer dan`, `is minder dan`.
- `Als klaar` (iets dat waar of niet waar is)

**Herhalen (altijd met een grens)**
- `Herhaal 10 keer` ... `Einde herhaal`
- `Herhaal zolang teller is kleiner dan 10, hoogstens 100 keer` ... `Einde herhaal`

**Tabbladen**: `Maak een tabblad met als naam Algemeen` (wat daarna komt, staat op dat tabblad), `Ga naar het tabblad Geluid`. Wisselen met Ctrl+Tab.

**Menubalk**: `Maak een menu met als naam Bestand`, `Maak een menuregel met als naam Opslaan`, `Als ik de menuregel Opslaan kies` ... `Einde van de knop`, `Geef de menuregel Opslaan de sneltoets Ctrl+S`.

**Twee voorwaarden**: `Als a is groter dan 1 en naam is "Jaap"`, `Als dag is "zaterdag" of dag is "zondag"`.

**Voorlezen en volume**: `Lees voor "Goedemorgen"`, `Stop met voorlezen`, `Zet het volume op 50`.

**Kopiëren, verplaatsen, weggooien**: `Kopieer het bestand a.txt naar b.txt`, `Verplaats het bestand a.txt naar Oud/a.txt`, `Gooi het bestand a.txt weg` (naar de Prullenbak).

**Tabellen**: `Maak een tabel met als naam Uitgaven`, `Voeg de kolom Bedrag toe aan de tabel Uitgaven`, `Voeg de rij "Brood" en 2,50 toe aan de tabel Uitgaven`, `Haal de gekozen rij weg uit ...`, `Maak de tabel ... leeg`, `Sorteer de tabel ... op de kolom ...`, `Sla de tabel ... op in het bestand x.csv`, `Lees de tabel ... uit het bestand x.csv`; lezen: `de gekozen rij van de tabel ...`, `rij nummer 2 van de tabel ...`, `het aantal rijen in de tabel ...`.

**Internet**: `de tekst van de webpagina https://...`, `de inhoud van de webpagina https://...`, `Verstuur "x" naar het adres https://...`, `Download het bestand van het adres https://... naar a.mp3`. Updates: `De updates staan op het adres https://...` en `Zoek naar updates` (versiebestand met `versie 1.1` en de link naar het installatiebestand).

**Opnemen**: `Begin met opnemen`, `Stop met opnemen en sla het op in het bestand opname.wav`.

**Afbeeldingen**: `Maak een afbeelding van het bestand hond.jpg met de beschrijving Mijn hond in het park` (PNG, JPG, BMP, GIF, TIFF).

**Word en PDF**: `Schrijf "..." in het Word-bestand brief.docx`, `Schrijf "..." in het PDF-bestand brief.pdf` (alleen maken, nog niet lezen).

**Lijsten**
- `Maak een lijst met als naam boodschappen`, `Voeg "melk" toe aan de lijst boodschappen`, `Haal "melk" weg uit de lijst boodschappen`, `Maak de lijst boodschappen leeg`
- Lezen: `de lijst boodschappen`, `het aantal in de lijst boodschappen`, `het eerste uit ...`, `het laatste uit ...`, `nummer 2 uit de lijst boodschappen`
- `Als de lijst boodschappen bevat "melk"`
- `Herhaal voor elk ding in de lijst boodschappen` ... `Einde herhaal`
- `Herhaal voor elk getal van 1 tot 10` ... `Einde herhaal`

**Eigen stukjes (een blok met een naam, invoer en uitkomst)**
- `Maak een stukje met als naam Verdubbel, met als invoer getal` ... `Einde van het stukje`
- `Geef als uitkomst getal keer 2`
- `Doe Groet`, `Doe Verdubbel met 5`, `Toon de uitkomst van Optellen met 3 en 4`
- Voorbeeld: `voorbeelden/Lijsten en stukjes.txt`

**Bestanden, mappen en logboek** (zonder volledig pad: in Documenten)
- `Schrijf "Hallo" in het bestand notities.txt`, `Voeg "Nog een regel" toe aan het bestand notities.txt`
- `Maak een map met als naam Mijn notities`
- `de inhoud van het bestand notities.txt`, `de regels van het bestand notities.txt` (lijst)
- `Als het bestand notities.txt bestaat` (of `bestaat niet`)
- `Schrijf in het logboek "Gestart"` (met datum en tijd; fouten komen er vanzelf in), `de plek van het logboek`

**Tijd, datum en toeval**: `de tijd`, `de datum`, `een willekeurig getal van 1 tot 6`

**Geluid**: `Piep`, `Speel een piep van 440 hertz en 300 milliseconden`, `Speel het geluid deurbel.wav` (ook: `af en wacht`), `Stop het geluid` (WAV, MP3, WMA, M4A, AAC, MIDI)

**Schermlezer laten zeggen**: `Zeg Het bestand is opgeslagen`, `Laat de schermlezer zeggen "Klaar"` (via UI Automation-meldingen, zonder venster)

**Pictogram**: `Het pictogram staat in het bestand logo.ico` (titelbalk, taakbalk, snelkoppelingen; de .exe in de Verkenner nog niet)

**ZIP**: `Pak de map Foto's in tot het zipbestand Foto's.zip`, `Pak het zipbestand Foto's.zip uit in de map Foto's`

**Sneltoetsen, kleuren en handleiding (F1) in je eigen programma**
- `Geef de knop Welkom de sneltoets Ctrl+W`
- `Geef het venster de kleur donkerblauw`, `Geef de tekst de kleur wit` (met controle of het goed leesbaar is)
- `In de handleiding staat: ...` of `De handleiding staat in het bestand Handleiding.txt`

**Blokken sluiten**: `Einde als`, `Einde herhaal`, `Einde van de knop`, `Einde van het stukje`, of alleen `Einde`.

**Installatiebestand** (doet niets als het programma draait)
- `Maak een installatiebestand met als naam Begroeting`
- `De versie is 1.0`
- `De maker is Jaap Wortel`
- `Zet een snelkoppeling in het startmenu`
- `Zet een snelkoppeling op het bureaublad`
- `De installatie mag met en zonder beheerdersrechten` (ook: `alleen zonder`, `alleen met`)
- `Voeg het bestand Uitleg.txt toe aan de installatie`

**Licentie**
- `In de licentie staat: Je mag dit programma gratis gebruiken.` (zo veel regels als je wilt)
- `De licentie staat in het bestand Licentie.txt`

**Privacyverklaring**
- `In de privacyverklaring staat: Dit programma verstuurt geen gegevens.` (zo veel regels als je wilt)
- `De privacyverklaring staat in het bestand Privacy.txt`
- Voorgelezen bij het installeren, als Privacyverklaring.txt bij het programma, en in je programma met Shift+F1.

**Ondertekenen met een eigen certificaat**
- `Onderteken het programma met mijn eigen certificaat op naam van Jaap Wortel`
- Gebeurt bij F8 en Ctrl+F8. De eerste keer vraagt het programma of het het certificaat mag maken (met Windows zelf, geen internet nodig). Menu Programma: "Mijn certificaat vertrouwen op deze computer".
- Eerlijk: op andere computers blijft Windows SmartScreen waarschuwen. Alleen een gekocht certificaat haalt dat weg.

**Een bestand van elders (bijvoorbeeld van je telefoon)**
- Zet bovenaan: `Dit is code voor het programma Rekenhulp` of `Dit is een omschrijving voor het programma Rekenhulp`.
- Open het met Ctrl+O. Je hoort wat erin staat en kiest: toevoegen als code (en meteen controleren), bewaren als notitie (`Rekenhulp notities.txt`), of gewoon openen.
- Je programma's staan standaard in `Documenten\Jaaps Programmeertaal`.

**Engels**
- Engelse basiszinnen werken ook: `Make a window with title Greeting`, `Make a button named Welcome`, `When I press the button Welcome`, `Show the text Hello`, `End of the button`, `Remember x is 1`, `Add 1 to x`, `Subtract 1 from x`, `Change x to 5`, `Set x to 5`, `If x is greater than 2`, `Else`, `End if`, `Repeat 3 times`, `Repeat while ..., at most 100 times`, `End repeat`, `Comment: ...`, `Beep`, `Play the sound ...`.
- Menu Programma, "Zet mijn programma om naar het Engels": met vaste regels. Titels, namen en tekst tussen aanhalingstekens worden niet vertaald; die krijg je in een lijstje.

**Eigen woorden (zelflerend, zonder AI)**
- Na een "Ja" op een tikfoutverbetering vraagt het programma of het dat woord mag onthouden.
- Menu Help, "Mijn eigen woorden": een tekstbestand met regels als `laat zien = toon`.

**Leestekens dicteren**: `aanhalingsteken openen` en `aanhalingsteken sluiten` worden echte aanhalingstekens.

## 6. Wat er gebeurt als iets niet klopt

- Elke melding heeft een regelnummer. De cursor springt naar die regel.
- **Onbekende zin**: "Deze zin ken ik niet. Ik gok niet wat je bedoelt. Welke zin bedoel je?"
- **Tikfout**: "Bedoel je: Maak een venster ...?" Ja of Nee. Je zin verandert alleen bij Ja. Ctrl+Z maakt het ongedaan.
- **Zinnen die elkaar tegenspreken of niet aansluiten**: het programma stopt en vraagt wat je bedoelt. Bijvoorbeeld: twee keer beschrijven wat dezelfde knop doet; een knop zonder venster; iets dat eerst een getal is en later tekst; een naam die nog niet onthouden is; twee verschillende zinnen over beheerdersrechten; de licentie twee keer beschreven.
- **Herhaling zonder grens**: kan niet. Het programma stelt voor om "hoogstens 100 keer" toe te voegen.
- **Fout tijdens het draaien** (zoals delen door nul): een melding met het regelnummer. Een programma stopt na tien miljoen stappen, zodat het nooit vastloopt.

## 7. Wat er op je computer nodig is

- Windows 10 of Windows 11, 64-bits.
- Verder niets. De Rust-compiler zit in de installatie (de map `rust`). Je hoeft hem niet apart te installeren.
- Een programma dat je bouwt, werkt op elke 64-bits Windows 10/11-computer, zonder Jaap's Programmeertaal.

## 8. Groottes (gemeten)

Vaste bouwmethode: `cargo build --release` met opt-level z, lto, codegen-units 1, panic abort en strip. Een programma dat je bouwt: rustc met opt-level s, panic abort en strip.

**Gemeten op een echte Windows-computer** (de bouwstraat op GitHub, 17 november 2026, Rust 1.97.0):

| Wat | Grootte |
|---|---|
| Installatiebestand `Jaaps Programmeertaal 0.1.0 installatie.exe` | 94.153.827 bytes (90 MB) |
| Editor `JaapsProgrammeertaal.exe` | 604.160 bytes (590 kB) |
| Opdrachtregel `jaaptaal.exe` | 553.472 bytes (540 kB) |
| Map `rust` (de ingebouwde Rust-compiler) | 520.156.953 bytes (496 MB) op de schijf |
| `Begroeting.exe`, gebouwd met F8 | 858.624 bytes (838 kB) |
| `Begroeting installatie.exe` met licentie en uitlegbestand | 1.714.176 bytes (1,6 MB) |
| Bouwtijd van `Begroeting.exe` (F8) | 1,2 seconden |

Eerlijk: de editor zelf is klein, maar de Rust-compiler is groot (bijna 500 MB op de schijf, 90 MB in het installatiebestand). Dat is de prijs voor echte programma's bouwen zonder iets apart te installeren. Kleiner gaat niet zonder onderdelen weg te laten die nodig zijn.

## 9. Testen en een .exe maken

**Automatische tests** (in de map `motor/tests`): het voorbeeld, elke ondersteunde zin, rekenen, keuzes, herhaling met grens, foutmeldingen met regelnummer, tikfoutvoorstel dat pas na "ja" iets verandert, onbekende zinnen, tegenstrijdige zinnen, TXT openen en opslaan (ook oude Kladblok-bestanden), installatie- en licentiezinnen, een installatiebestand bouwen (op Windows ook stil installeren en verwijderen), en de controle dat er nergens AI, een taalmodel of internet wordt gebruikt.

Uitvoeren (met Rust op de computer):

```
cargo test --workspace
```

Uitslag in deze sessie (Linux): 98 tests (en op Windows ook de test die echt ondertekent), allemaal geslaagd. Op GitHub (echte Windows) slaagden de tests ook, en bouwde de ingebouwde Rust-compiler het voorbeeld dat "Hallo" zegt.

**De opdrachtregel** (`jaaptaal.exe`, staat ook in de installatie):

```
jaaptaal controleer programma.txt
jaaptaal beschrijf programma.txt
jaaptaal scherm programma.txt
jaaptaal rust programma.txt
jaaptaal bouw programma.txt Programma.exe
jaaptaal proef programma.txt --typ "Naam=Jaap" --kies "Kleur=Rood" --vink "Geluid aan" --druk Welkom
jaaptaal installatie programma.txt "Programma installatie.exe"
```

**De bouwstraat** (`.github/workflows/bouwen.yml`) doet op een echte Windows-computer: alle tests, de editor bouwen, de Rust-compiler klaarmaken, het voorbeeld bouwen met de ingebouwde Rust-compiler (zonder de Rust van de computer zelf), het installatiebestand van het voorbeeld bouwen, stil installeren, controleren en verwijderen, de editor opstarten en afsluiten, een toegankelijkheidscontrole met UI Automation, en het installatiebestand van Jaap's Programmeertaal maken, installeren en verwijderen. Het resultaat komt bij de uitgave **proef**.

## 10. Wat er nog met JAWS en NVDA getest moet worden

Ik kan in deze bouwomgeving geen JAWS of NVDA draaien. Ik claim dus niets over hoe het klinkt. Wat ik wel heb gedaan: alleen gewone Windows-onderdelen gebruikt (menubalk, tekstvakken, knoppen, meldingsvensters), en de editor, een gebouwd programma en een installatiebestand bekeken in een nagebootst Windows (Wine).

De lijst met handmatige tests staat in `docs/Handmatige-tests.md`. Belangrijkste punten:

De automatische controle op GitHub (UI Automation, dezelfde techniek als NVDA en JAWS) vond: beide vakken hebben een goede naam ("Opdrachten (je programma, één opdracht per regel):" en "Meldingen en uitvoer:"), en de Tab-volgorde loopt goed rond. De menu's kon die controle niet uitklappen. In de nabootsing (Wine) gaan ze wel open met F10 en pijltje omlaag. Het zijn gewone Windows-menu's. Test dit zelf.

1. Leest de schermlezer bij de start "Opdrachten" als naam van het vak? En "Meldingen en uitvoer" bij het tweede vak?
2. Hoor je elke melding (de vensters met OK of Ja/Nee) helemaal voorgelezen, met het regelnummer?
3. Werkt dicteren (Windows-dicteren, of Spraak naar tekst) in het vak Opdrachten?
4. Hoor je in een gebouwd programma de titel van het venster en de naam van de knop? Werkt Enter en de spatiebalk?
5. Hoor je in het installatiebestand alle vragen, ook de licentie?
6. Zegt Ctrl+L het juiste regelnummer?

## 10b. Opdrachten in gewone zinnen

In de editor: **Ctrl+E** om in gewone woorden te zeggen wat je wilt ("maak er een exe van", "ga maar verder", "zet er een knop Opslaan bij"), **Ctrl+F** om te zoeken in alles wat je kunt zeggen. De verzoeken komen uit hoe Jaap tegen Claude Code praat in zijn programma's. Werkt zonder AI: `motor/src/opdrachten.rs` (de editor) en `motor/src/wensen.rs` (zinnen voor je programma). Een programma kan ook een opdrachtvak krijgen: `Geef mijn programma een opdrachtvak` en `Als ik zeg "..."`.

## 11. Wat nog niet werkt

Dit staat op je wensenlijst, maar zit nog **niet** in deze versie. Schrijf je zo'n zin, dan zegt het programma dat eerlijk.

**De taal**: rekenen binnen de invoer van een stukje (eerst onthouden); Engelse zinnen voor lijsten, stukjes, bestanden, tekstvakken en installatie; tekst vertalen.

**Scherm**: werkbalken, een echte tekstverwerker (met opslaan, lettertype, zoeken), zelf een plek en grootte kiezen (onderdelen staan nu altijd onder elkaar), een eigen Tab-volgorde, aanpassen aan schermgrootte, kleuren van knoppen en tekstvakken. (Wel gebouwd: kleuren van venster en tekst met contrastcontrole, sneltoetsen, en een beschrijving van het scherm met tips: Ctrl+F6.)

**Geluid**: OGG en FLAC, stereo/3D, een microfoon kiezen, muziekspelers, audio-editor. (Wel gebouwd: piep, WAV en MP3 afspelen.)

**Beeld**: 3D, QR-codes en streepjescodes lezen. (Wel gebouwd: QR-codes en streepjescodes maken.)

**Bestanden**: DOCX en PDF lezen (ZIP inpakken en uitpakken werkt wel) (RAR: alleen uitpakken is technisch netjes mogelijk, inpakken valt onder de licentie van RAR), 32-bits programma's, een handleiding in je eigen programma (F1).

**Netwerk**: verbinding met een andere computer, eigen PHP-scripts. (Wel gebouwd: contactformulier in PHP en eigen CSS op een website.)

**Spraak en toegankelijkheid**: braille, spraakherkenning, voorlezen, hardware, Whisper en taalmodellen als onderdeel van jouw programma's (niet van de taal zelf).

**Overig**: extensies en drivers ("motorschrijver"), je bestaande programma's nabouwen. (Wel gebouwd: eigen zinnen, Rust-stukjes, reken- en tekstzinnen, wachten, programma's starten, bestanden openen.)

**Opdrachten in gewone zinnen** (Ctrl+E): gebouwd met vaste woorden per opdracht, zonder AI. Dingen die de taal nog niet kan (een venster dat schermvullend start, een vraag bij afsluiten, maar één keer open, reservekopieën, een donkere stand, tooltips) gaan op je wensenlijst.

**Over de installatie**: het pictogram op het .exe-bestand zelf (in de Verkenner) kan nog niet; in de titelbalk, taakbalk en snelkoppelingen wel. (Een gekocht certificaat komt er niet: Jaap kiest voor zijn eigen certificaat.)

## 12. Wat ik heb gebouwd

| Map of bestand | Wat |
|---|---|
| `motor/src/tekst.rs` | Een regel netjes maken: gesproken aanhalingstekens, punt aan het eind, knippen in woorden |
| `motor/src/getallen.rs` | Getallen in cijfers en woorden |
| `motor/src/taal.rs` | Wat er van je zinnen wordt gemaakt (de tussenstap) |
| `motor/src/ontleden.rs` | Alle zinnen die werken, blokken, tikfouten zoeken |
| `motor/src/controle.rs` | Controleren: tegenspraak, ontbrekende namen, rekenen met tekst, grenzen |
| `motor/src/melding.rs` | Meldingen met regelnummer, voorstellen, ja-en-pas-dan-veranderen |
| `motor/src/beschrijf.rs` | In gewone taal vertellen wat er gemaakt is (F6) |
| `motor/src/rustcode.rs` | Rust-code maken (Shift+F6) |
| `motor/looptijd/looptijd.rs` | Wat in elk gebouwd programma meegaat: rekenen, vensters, knoppen |
| `motor/src/installatie.rs`, `motor/looptijd/installatie.rs` | Installatiebestand en licentie |
| `motor/src/bouwen.rs` | De Rust-compiler aanroepen |
| `motor/src/bestand.rs` | TXT lezen en veilig opslaan |
| `motor/src/bin/jaaptaal.rs` | De opdrachtregel |
| `motor/tests/` | De automatische tests |
| `editor/` | Het Windows-programma (editor) |
| `docs/Handleiding.txt` | De handleiding (F1) |
| `docs/Handmatige-tests.md` | Wat je zelf met JAWS en NVDA moet testen |
| `docs/Betere-opdracht.md` | Een uitgebreidere versie van je opdracht, met aanvullingen |
| `voorbeelden/` | Voorbeeldprogramma's |
| `installer/` | Het installatieprogramma van Jaap's Programmeertaal zelf (Inno Setup) |
| `.github/workflows/bouwen.yml` | De bouwstraat |
| `tools/Toegankelijkheid.ps1` | Toegankelijkheidscontrole (hetzelfde script als in je andere programma's) |

Geen enkel extra Rust-pakket: de motor en de editor gebruiken alleen de standaard Rust-bibliotheek en Windows zelf.

**Wat ik niet kon uitvoeren**: JAWS en NVDA (niet beschikbaar hier); echte Windows (alleen Wine, en daarna de bouwstraat op GitHub); de stap "voor alle gebruikers" met de toestemmingsvraag van Windows (UAC), want die heeft een echte Windows nodig; de knoppen van de meldingsvensters in het Nederlands (Wine liet ze in het Engels zien; op een Nederlandse Windows staan er Ja en Nee).
