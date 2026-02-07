# Words (de/woerter.json)

Replaces commonly spoken words with their correct German equivalent.

## Examples

| Input | Output |
| ----- | ------ |
| Yeah  | Ja     |

## How it works

Pure **regex rules** with word boundaries and case-insensitive matching. Primarily for anglicisms that occur during dictation.

**Priority 100**

## Activation

Add to `rules_paths` in `config.json`:

```json
{
  "rules_paths": ["rules/de/woerter.json"]
}
```
