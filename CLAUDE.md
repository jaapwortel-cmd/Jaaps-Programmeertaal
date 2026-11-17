# Jaap's Programmeertaal

Programmeertaal in gewone Nederlandse zinnen, gemaakt voor Jaap Wortel. Rust, zonder extra pakketten.

## Met Jaap praten

- Nederlands, eenvoudige taal (B1), met "je" en "jij".
- Jaap is blind, werkt met JAWS, NVDA en dicteren. Korte uitleg, duidelijke stappen. Jaap schrijft zelf geen code.
- Bij een keuze die alleen Jaap kan maken: één duidelijke vraag, en wachten. Niet gokken.
- Eerlijk zijn over wat wel en niet werkt; niets mooier maken dan het is.

## De belangrijkste regels

- GEEN AI in het programma (geen taalmodel, geen modelverbinding). Test `motor/tests/geen_ai.rs` controleert dat.
- Onbekende zin: niet gokken, één vraag of één ja/nee-voorstel. Tekst verandert pas na "ja".
- Tegenstrijdige of niet-aansluitende zinnen: stoppen en vragen.
- Alles toetsenbord en schermlezer: alleen gewone Windows-onderdelen (menubalk, EDIT, BUTTON, MessageBox).
- Geen andere Rust-pakketten: Windows-functies rechtstreeks (extern "system").

## Opbouw

- `motor`: tekst -> ontleden -> controle -> rustcode (+ looptijd/looptijd.rs) -> bouwen. Los van het scherm (voor Bouwmaatje).
- `motor/src/installatie.rs` + `motor/looptijd/installatie.rs`: installatiebestand en licentie (Jaaps wens 17 nov 2026).
- `editor`: Windows-editor (F5, F7, F8, Ctrl+F8, F9, F6, Shift+F6, F1).
- De Rust-compiler (1.97.0, x86_64-pc-windows-gnu, zonder cargo/rustdoc/lld) gaat mee in de map `rust` van de installatie
  (Jaaps keuze 17 nov 2026: "meteen A"). Bouwen met `-C link-self-contained=yes`.
- Nieuwe zinnen: in ontleden.rs, controle.rs, beschrijf.rs, rustcode.rs (+ looptijd), tests in motor/tests, handleiding
  (docs/Handleiding.txt) en README (lijst met zinnen en "Wat nog niet werkt") bijwerken, en Wijzigingen.txt.

## Bouwen en testen

- `cargo test --workspace` (tests bouwen echte programma's met rustc).
- Windows-build vanaf Linux: `cargo build --release --target x86_64-pc-windows-gnu --config 'target.x86_64-pc-windows-gnu.linker="x86_64-w64-mingw32-gcc"'`.
- Wine + Xvfb werkt voor proeven (ook de Windows-rustc met link-self-contained).
- Bouwstraat: `.github/workflows/bouwen.yml`; installatiebestand bij de uitgave "proef" (te groot voor de tak exe-builds).
- Bouwminuten sparen (zoals bij Jaaps andere programma's): wijzigingen verzamelen en samen pushen.

## Werkafspraken

- Niets veranderen aan Jaaps andere repositories. Zichtbaarheid van deze repository niet veranderen.
- Geen wachtwoorden, sleutels of persoonlijke gegevens in de code.
- Wensenlijst: zie de opdracht in README ("Wat nog niet werkt"). "Motorschrijver": eerst voorbeelden aan Jaap vragen.
