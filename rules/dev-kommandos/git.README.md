# Dev Commands (dev-kommandos/git.json)

Voice triggers for developer commands. Recognizes spoken shortcuts and converts them into commands.

## Commands

| Voice input | Output          | Description                    |
| ----------- | --------------- | ------------------------------ |
| AC / A C    | add and commit  | Git add & commit               |
| SAC / S A C | /add-and-commit | Slash command for add & commit |
| Käsekuchen  | /add-and-commit | Voice trigger (code word)      |
| Streichholz | cy              | Voice trigger (code word)      |

## How it works

**Regex rules** with `stop_on_match: true` — when a command is recognized, further rule processing stops. The input must exactly match the pattern (anchors `^...$`) so that normal text is not accidentally transformed.

**Priority 200-300** — highest priority, checked before all other rules.

## Activation

Add to `rules_paths` in `config.json`:

```json
{
  "rules_paths": ["rules/dev-kommandos/git.json"]
}
```
