# Punctuation (de/satzzeichen.json)

Converts spoken German punctuation words into their corresponding symbols.

## Examples

| Input             | Output |
| ----------------- | ------ |
| Punkt             | .      |
| Komma             | ,      |
| Fragezeichen      | ?      |
| Ausrufezeichen    | !      |
| Klammer auf       | (      |
| Eckige Klammer zu | ]      |
| Anführungszeichen | "      |
| Schrägstrich      | /      |
| Klammeraffe       | @      |
| Neue Zeile        | \n     |

## Rules (28 total)

| Category      | Words                                                                                  |
| ------------- | -------------------------------------------------------------------------------------- |
| Punctuation   | Punkt, Komma, Fragezeichen, Ausrufezeichen, Doppelpunkt, Semikolon/Strichpunkt         |
| Brackets      | Klammer auf/zu, Eckige Klammer auf/zu, Geschweifte Klammer auf/zu                      |
| Quotation     | Anführungszeichen                                                                      |
| Dashes        | Bindestrich, Gedankenstrich                                                            |
| Special chars | Schrägstrich, Backslash, At-Zeichen/Klammeraffe, Unterstrich, Sternchen, Raute/Hashtag |
| Symbols       | Und-Zeichen/Kaufmanns-Und, Prozent, Euro, Dollar                                       |
| Formatting    | Neue Zeile, Neuer Absatz                                                               |
| Cleanup       | Auslassungspunkte (drei Punkte/Punkt Punkt Punkt), redundant "Punkt" after punctuation |

## How it works

Pure **regex rules** with word boundaries (`\b`) and case-insensitive matching. Supports umlaut alternatives (ü/ue, ä/ae).

**Priority 100-110** — brackets and ellipsis (110) run before simple punctuation (100).

## Activation

Active by default. Add to `rules_paths` in `config.json`:

```json
{
  "rules_paths": ["rules/de/satzzeichen.json"]
}
```
