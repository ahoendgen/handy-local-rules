# Dev-Kommandos (dev-kommandos/git.json)

Voice-Trigger für Entwickler-Befehle. Erkennt gesprochene Kurzkommandos und wandelt sie in Befehle um.

## Kommandos

| Spracheingabe | Ausgabe         | Beschreibung                   |
| ------------- | --------------- | ------------------------------ |
| AC / A C      | add and commit  | Git add & commit               |
| SAC / S A C   | /add-and-commit | Slash-Command für add & commit |
| Käsekuchen    | /add-and-commit | Voice-Trigger (Codewort)       |
| Streichholz   | cy              | Voice-Trigger (Codewort)       |

## Funktionsweise

**Regex-Regeln** mit `stop_on_match: true` — wenn ein Kommando erkannt wird, stoppt die weitere Regelverarbeitung. Die Eingabe muss exakt dem Muster entsprechen (Anker `^...$`), damit normaler Text nicht versehentlich umgewandelt wird.

**Priorität 200-300** — höchste Priorität, wird vor allen anderen Regeln geprüft.

## Aktivierung

In `config.json` unter `rules_paths` eintragen:

```json
{
  "rules_paths": ["rules/dev-kommandos/git.json"]
}
```
