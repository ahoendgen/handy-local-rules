# Cleanup (general/cleanup.json)

Bereinigt Text nach der Transformation: entfernt doppelte Satzzeichen, korrigiert Abstände und normalisiert Whitespace.

## Was wird bereinigt?

| Kategorie                    | Beispiel                                                          | Ergebnis        |
| ---------------------------- | ----------------------------------------------------------------- | --------------- |
| Trailing Ellipsis            | Text...                                                           | Text            |
| Komma vor ? / !              | ,?                                                                | ?               |
| Punkt nach ? / !             | ?.                                                                | ?               |
| Punkt + Komma                | ., oder ,.                                                        | .               |
| Sonderzeichen + Punkt        | \_. oder -.                                                       | \_ oder -       |
| Doppelte Zeichen             | !! oder ?? oder ::                                                | ! oder ? oder : |
| Doppelte Punkte              | ..                                                                | .               |
| Ellipsis-Schutz              | ... wird vor Punkt-Cleanup geschützt und danach wiederhergestellt |
| Leerzeichen vor Satzzeichen  | Wort .                                                            | Wort.           |
| Leerzeichen nach Satzzeichen | Wort.Wort                                                         | Wort. Wort      |
| Whitespace                   | Mehrere Leerzeichen -> eins                                       |
| Trim                         | Leerzeichen am Anfang/Ende entfernen                              |

## Funktionsweise

Mix aus **Regex-Regeln** und **Function-Regeln** (normalize_whitespace, trim).

**Prioritäten:**

- 200: Trailing Ellipsis entfernen (Speech-to-Text Artefakt)
- 15: Satzzeichen-Konflikte auflösen
- 10: Doppelte Satzzeichen konsolidieren
- 5: Leerzeichen vor Satzzeichen entfernen
- 4: Leerzeichen nach Satzzeichen sicherstellen
- 3-1: Ellipsis-Schutz (schützen -> doppelte Punkte entfernen -> wiederherstellen)
- 0: Whitespace normalisieren
- -1: Trim

## Aktivierung

In `config.json` unter `rules_paths` eintragen:

```json
{
  "rules_paths": ["rules/general/cleanup.json"]
}
```
