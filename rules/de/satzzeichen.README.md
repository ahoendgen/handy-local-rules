# Satzzeichen (de/satzzeichen.json)

Wandelt gesprochene deutsche Satzzeichen-Wörter in die entsprechenden Symbole um.

## Beispiele

| Eingabe           | Ausgabe |
| ----------------- | ------- |
| Punkt             | .       |
| Komma             | ,       |
| Fragezeichen      | ?       |
| Ausrufezeichen    | !       |
| Klammer auf       | (       |
| Eckige Klammer zu | ]       |
| Anführungszeichen | "       |
| Schrägstrich      | /       |
| Klammeraffe       | @       |
| Neue Zeile        | \n      |

## Regeln (28 Stück)

| Kategorie     | Wörter                                                                                  |
| ------------- | --------------------------------------------------------------------------------------- |
| Satzzeichen   | Punkt, Komma, Fragezeichen, Ausrufezeichen, Doppelpunkt, Semikolon/Strichpunkt          |
| Klammern      | Klammer auf/zu, Eckige Klammer auf/zu, Geschweifte Klammer auf/zu                       |
| Anführung     | Anführungszeichen                                                                       |
| Striche       | Bindestrich, Gedankenstrich                                                             |
| Sonderzeichen | Schrägstrich, Backslash, At-Zeichen/Klammeraffe, Unterstrich, Sternchen, Raute/Hashtag  |
| Symbole       | Und-Zeichen/Kaufmanns-Und, Prozent, Euro, Dollar                                        |
| Formatierung  | Neue Zeile, Neuer Absatz                                                                |
| Cleanup       | Auslassungspunkte (drei Punkte/Punkt Punkt Punkt), redundantes "Punkt" nach Satzzeichen |

## Funktionsweise

Reine **Regex-Regeln** mit Wortgrenzen (`\b`) und case-insensitive Matching. Unterstützt Umlaut-Alternativen (ü/ue, ä/ae).

**Priorität 100-110** — Klammern und Auslassungspunkte (110) laufen vor einfachen Satzzeichen (100).

## Aktivierung

Standardmäßig aktiv. In `config.json` unter `rules_paths` eintragen:

```json
{
  "rules_paths": ["rules/de/satzzeichen.json"]
}
```
