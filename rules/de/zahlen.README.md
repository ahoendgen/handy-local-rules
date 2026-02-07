# Zahlen (de/zahlen.json)

Wandelt gesprochene deutsche Zahlwörter in Ziffern um.

## Beispiele

| Eingabe                               | Ausgabe          |
| ------------------------------------- | ---------------- |
| dreiunddreißig                        | 33               |
| neunundneunzig Luftballons            | 99 Luftballons   |
| drei Komma eins vier                  | 3,14             |
| null Komma fünf                       | 0,5              |
| eintausenddreihundertsiebenunddreißig | 1337             |
| Äpfel und Birnen                      | Äpfel und Birnen |

## Funktionsweise

Nutzt eine **Shell-Rule** mit der Python-Bibliothek [text2num](https://github.com/allo-media/text2num), die beliebig große deutsche Zahlen erkennt und umwandelt (inkl. Dezimalzahlen mit "Komma").

**Priorität 105** — läuft vor den Satzzeichen-Regeln (100), damit "Komma" als Dezimaltrennzeichen erkannt wird, bevor es zu "," umgewandelt wird.

## Voraussetzungen

1. **Python 3.13** (oder <=3.13) muss installiert sein
2. **text2num** in einem venv installiert:

```bash
/opt/homebrew/bin/python3.13 -m venv ~/.handy-local-rules/venv
~/.handy-local-rules/venv/bin/pip install text2num
```

3. **Shell-Rules** müssen in der `config.json` aktiviert sein:

```json
{
  "enable_shell_rules": true
}
```

## Hinweise

- Standalone-Wörter wie "eins" oder "null" werden nur im Kontext umgewandelt (z.B. "null Komma fünf" -> "0,5"), nicht isoliert
- "und" wird korrekt als Konjunktion belassen (kein false positive)
- Latenz: ~26ms pro Transformation
