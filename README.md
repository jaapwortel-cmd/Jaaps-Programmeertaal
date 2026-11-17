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

Vaste bouwmethode: `cargo build --release` met opt-level z, lto, codegen-units 1, panic abort en strip. Een gebouwd programma: rustc met opt-level s, panic abort, strip.

Gemeten in deze sessie (Linux, de Windows-versie gebouwd met mingw, en de Windows-Rust-compiler getest in Wine):

| Wat | Grootte |
|---|---|
| Editor `JaapsProgrammeertaal.exe` (mingw-build) | 564.224 bytes (551 kB) |
| Map `rust` met de Rust-compiler 1.97.0 (zonder cargo, rustdoc, lld) | 520.156.953 bytes (496 MB) |
| Diezelfde map, ingepakt met xz | ongeveer 90 MB |
| `Begroeting.exe`, gebouwd met F8 (klein) | 850.944 bytes (831 kB) |
| `Begroeting.exe`, gebouwd met F5 (snel) | 930.816 bytes (909 kB) |
| `Begroeting installatie.exe` met licentie en extra bestand | 1.707.008 bytes (1,6 MB) |

Bouwtijd in Wine op deze computer (4 kernen): Begroeting snel 0,8 s, klein 1,2 s, installatiebestand 2,6 s. Op jouw computer kan dat anders zijn.

De bouwstraat op GitHub meet de groottes ook op een echte Windows-computer (MSVC-build van de editor, en de grootte van het installatiebestand van Jaap's Programmeertaal). Die staan in de samenvatting van elke bouwronde en in `groottes.txt` bij de uitgave **proef**.

Eerlijk: de editor zelf is klein, maar de Rust-compiler is groot (bijna 500 MB op de schijf). Dat komt omdat je echte programma's moet kunnen compileren. Rust is niet kleiner te krijgen zonder onderdelen weg te laten die nodig zijn.

## 9. Testen en een .exe maken

**Automatische tests** (in de map `motor/tests`): het voorbeeld, elke ondersteunde zin, rekenen, keuzes, herhaling met grens, foutmeldingen met regelnummer, tikfoutvoorstel dat pas na "ja" iets verandert, onbekende zinnen, tegenstrijdige zinnen, TXT openen en opslaan (ook oude Kladblok-bestanden), installatie- en licentiezinnen, een installatiebestand bouwen (op Windows ook stil installeren en verwijderen), en de controle dat er nergens AI, een taalmodel of internet wordt gebruikt.

Uitvoeren (met Rust op de computer):

```
cargo test --workspace
```

Uitslag in deze sessie (Linux): 75 tests, allemaal geslaagd. Op GitHub (echte Windows) slaagden de tests ook, en bouwde de ingebouwde Rust-compiler het voorbeeld dat "Hallo" zegt.

**De opdrachtregel** (`jaaptaal.exe`, staat ook in de installatie):

```
jaaptaal controleer programma.txt
jaaptaal beschrijf programma.txt
jaaptaal rust programma.txt
jaaptaal bouw programma.txt Programma.exe
jaaptaal proef programma.txt --typ "Naam=Jaap" --kies "Kleur=Rood" --vink "Geluid aan" --druk Welkom
jaaptaal installatie programma.txt "Programma installatie.exe"
```

**De bouwstraat** (`.github/workflows/bouwen.yml`) doet op een echte Windows-computer: alle tests, de editor bouwen, de Rust-compiler klaarmaken, het voorbeeld bouwen met de ingebouwde Rust-compiler (zonder de Rust van de computer zelf), het installatiebestand van het voorbeeld bouwen, stil installeren, controleren en verwijderen, de editor opstarten en afsluiten, een toegankelijkheidscontrole met UI Automation, en het installatiebestand van Jaap's Programmeertaal maken, installeren en verwijderen. Het resultaat komt bij de uitgave **proef**.

## 10. Wat er nog met JAWS en NVDA getest moet worden

Ik kan in deze bouwomgeving geen JAWS of NVDA draaien. Ik claim dus niets over hoe het klinkt. Wat ik wel heb gedaan: alleen gewone Windows-onderdelen gebruikt (menubalk, tekstvakken, knoppen, meldingsvensters), en de editor, een gebouwd programma en een installatiebestand bekeken in een nagebootst Windows (Wine).

De lijst met handmatige tests staat in `docs/Handmatige-tests.md`. Belangrijkste punten:

1. Leest de schermlezer bij de start "Opdrachten" als naam van het vak? En "Meldingen en uitvoer" bij het tweede vak?
2. Hoor je elke melding (de vensters met OK of Ja/Nee) helemaal voorgelezen, met het regelnummer?
3. Werkt dicteren (Windows-dicteren, of Spraak naar tekst) in het vak Opdrachten?
4. Hoor je in een gebouwd programma de titel van het venster en de naam van de knop? Werkt Enter en de spatiebalk?
5. Hoor je in het installatiebestand alle vragen, ook de licentie?
6. Zegt Ctrl+L het juiste regelnummer?

## 11. Wat nog niet werkt

Dit staat op je wensenlijst, maar zit nog **niet** in deze versie. Schrijf je zo'n zin, dan zegt het programma dat eerlijk.

**De taal**: rekenen binnen de invoer van een stukje (eerst onthouden); twee voorwaarden met "en" of "of"; Engelse opdrachten en vertalen naar het Engels; het onthouden van jouw eigen formuleringen (zelflerend).

**Scherm**: tabellen, menu's, werkbalken, een echte tekstverwerker (met opslaan, lettertype, zoeken), kleuren, zelf een lay-out kiezen, een lay-outvoorstel, vooraf vertellen wat er op het scherm staat, eigen sneltoetsen en tabvolgorde, aanpassen aan schermgrootte.

**Geluid**: geluiden, mono/stereo/3D, opnemen, muziekspelers, audio-editor.

**Beeld**: afbeeldingen met beschrijving, 3D, QR-codes en streepjescodes.

**Bestanden**: tekstbestanden lezen en schrijven vanuit je programma, logbestanden, DOCX en PDF, mappen, ZIP (RAR: alleen uitpakken is technisch netjes mogelijk, inpakken valt onder de licentie van RAR), 32-bits programma's, een handleiding in je eigen programma (F1).

**Netwerk**: verbinding met een andere computer, internet, PHP en CSS, updates via Dropbox of pCloud.

**Spraak en toegankelijkheid**: eigen meldingen voor schermlezers, braille, spraakherkenning, voorlezen, hardware, Whisper en taalmodellen als onderdeel van jouw programma's (niet van de taal zelf).

**Overig**: extensies en drivers ("motorschrijver"), je bestaande programma's nabouwen, alles wat Rust kan.

**Over de installatie**: een gebouwd installatiebestand is niet ondertekend (daarvoor is een betaald certificaat nodig). Een eigen pictogram (icoon) kan nog niet.

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
