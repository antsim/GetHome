# Työaikalaskuri

Henkilökohtainen macOS-sovellus, joka näyttää menu barissa ja pienessä
kelluvassa ikkunassa, paljonko päivän työajasta on jäljellä. Tiedot haetaan
[Toggl Trackista](https://toggl.com/track/). Kun Togglessa ei ole ajastinta
käynnissä, laskuri pysähtyy ja näytölle tulee teksti **"Tauolla"**.

```
⏱ 3:42:15            menu bar, ajastin käynnissä
⏸ 3:42 Tauolla       ei käynnissä olevaa ajastinta
✓ +0:12              tavoite täynnä, ylityötä 12 min
```

## Ominaisuudet

- Jäljellä oleva aika menu barissa. Päivittyy sekunnin välein myös silloin, kun ikkuna on piilossa.
- Kehyksetön, aina päällä pysyvä ikkuna. Sen voi raahata esimerkiksi ulkoiselle näytölle, ja sijainti säilyy.
- Ajastimen pysäytys ja jatkaminen suoraan sovelluksesta.
- Päiväkohtainen tavoite (oletus ma–pe 7:30, la–su 0 h).
- Valmistumisaika-arvio ("valmis klo 16:05") ja tauon kesto.
- Vaalea ja tumma teema, säädettävä läpinäkyvyys, käynnistys kirjautuessa.
- Toggl API -kutsuja normaalikäytössä alle 20 tunnissa. Rajoitukset, verkkokatkot ja uni käsitellään hallitusti.
- API-token tallennetaan vain macOS:n avainnippuun (Keychain).

## Asennus ja käyttöönotto

Vaatimukset: macOS 11+, [Node.js](https://nodejs.org/) 22+, [Rust](https://rustup.rs/) (stable) ja Xcode Command Line Tools.

```bash
git clone https://github.com/antsim/GetHome.git
cd GetHome
npm install
```

Kehitystilassa ajo:

```bash
npm run dev
```

Julkaisuversion (`.app` ja `.dmg`) rakentaminen:

```bash
npm run bundle
```

Valmis paketti löytyy hakemistosta `src-tauri/target/release/bundle/`.
Sovellus ei ole allekirjoitettu, joten ensimmäisellä käynnistyskerralla se
avataan Finderissa hiiren oikealla painikkeella → *Avaa*.

### API-tokenin asetus

1. Kopioi API-token Toggl Trackin [profiilisivulta](https://track.toggl.com/profile) (kohta *API Token*).
2. Avaa sovelluksen valikosta menu barissa *Asetukset…*.
3. Liitä token ja paina *Tallenna*. Sovellus tarkistaa tokenin ja näyttää nimesi ja workspacen.

Token tallennetaan vain avainnippuun. Sitä ei kirjoiteta tiedostoon eikä
lähetetä mihinkään muualle kuin Toggl API:in.

## Käyttö

| Toiminto | Miten |
|---|---|
| Näytä/piilota ikkuna | Menu bar → *Näytä ikkuna* / *Piilota ikkuna* |
| Pysäytä ajastin / jatka työtä | Ikkunan ⏸/▶-painike tai menu bar |
| Päivitä heti | Menu bar → *Päivitä nyt* (enintään kerran 30 sekunnissa) |
| Asetukset | Menu bar → *Asetukset…* tai ikkunan ⚙ (näkyy hiiren ollessa ikkunan päällä) |
| Siirrä ikkunaa | Raahaa mistä tahansa kohdasta |

Jos ajastin pysäytetään Toggl-sovelluksessa, muutos näkyy enintään neljän
minuutin viiveellä. Tämä johtuu Toggl API:n pyyntörajoista (30 pyyntöä
tunnissa). Tämän sovelluksen kautta tehdyt muutokset näkyvät heti.

## Kehitys

```bash
npm test -- --watch=false     # Angular-testit (Vitest)
npm run format:check          # Prettier
npm run build                 # Angular-build (tarvitaan ennen cargo-komentoja)

cd src-tauri
cargo test                    # Rust-testit (laskenta + mockattu Toggl API)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Linuxilla Rust-koodin kääntäminen vaatii Taurin
[järjestelmäriippuvuudet](https://tauri.app/start/prerequisites/#linux)
(esim. `libwebkit2gtk-4.1-dev`). Sovellusta tuetaan ja testataan vain
macOS:llä.

Arkkitehtuuri ja koodikonventiot on kuvattu tiedostossa
[CLAUDE.md](CLAUDE.md). Alkuperäinen suunnitelma ja toteutuksen tarkennukset
ovat tiedostossa [PLAN.md](PLAN.md).

### Projektin rakenne

```
src/              Angular-käyttöliittymä (kelluva ikkuna ja asetukset)
src-tauri/        Rust-backend: tila, ajastus, Toggl-client, tray
PLAN.md           Toteutussuunnitelma
CLAUDE.md         Arkkitehtuuri ja konventiot
```

## Tietosuoja

- Sovellus keskustelee vain Toggl Trackin API:n kanssa (`api.track.toggl.com`).
- API-token on vain macOS:n avainnipussa.
- Asetukset (tavoitteet, näyttöasetukset sekä Toggl-tilin nimi ja workspace) tallennetaan sovelluksen datahakemistoon (`~/Library/Application Support/io.github.antsim.tyoaikalaskuri/`).

## Lisenssi

[MIT](LICENSE)
