# Wörter (de/woerter.json)

Ersetzt häufige gesprochene Wörter durch ihre korrekte deutsche Entsprechung.

## Beispiele

| Eingabe | Ausgabe |
| ------- | ------- |
| Yeah    | Ja      |

## Funktionsweise

Reine **Regex-Regeln** mit Wortgrenzen und case-insensitive Matching. Hauptsächlich für Anglizismen, die beim Diktieren auftreten.

**Priorität 100**

## Aktivierung

In `config.json` unter `rules_paths` eintragen:

```json
{
  "rules_paths": ["rules/de/woerter.json"]
}
```
