# Numbers (de/zahlen.json)

Converts spoken German number words into their digit equivalents.

## Examples

| Input                                 | Output           |
| ------------------------------------- | ---------------- |
| dreiunddreißig                        | 33               |
| neunundneunzig Luftballons            | 99 Luftballons   |
| drei Komma eins vier                  | 3,14             |
| null Komma fünf                       | 0,5              |
| eintausenddreihundertsiebenunddreißig | 1337             |
| Äpfel und Birnen                      | Äpfel und Birnen |

## How it works

Uses a **shell rule** with the Python library [text2num](https://github.com/allo-media/text2num), which recognizes and converts arbitrarily large German numbers (including decimals with "Komma").

**Priority 105** — runs before punctuation rules (100) so that "Komma" is recognized as a decimal separator before being converted to ",".

## Prerequisites

1. **Python 3.13** (or <=3.13) must be installed
2. **text2num** installed in a venv:

```bash
/opt/homebrew/bin/python3.13 -m venv ~/.handy-local-rules/venv
~/.handy-local-rules/venv/bin/pip install text2num
```

3. **Shell rules** must be enabled in `config.json`:

```json
{
  "enable_shell_rules": true
}
```

## Notes

- Standalone words like "eins" or "null" are only converted in context (e.g. "null Komma fünf" -> "0,5"), not in isolation
- "und" is correctly left as a conjunction (no false positives)
- Latency: ~26ms per transformation
