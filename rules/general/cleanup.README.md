# Cleanup (general/cleanup.json)

Cleans up text after transformation: removes duplicate punctuation, fixes spacing, and normalizes whitespace.

## What gets cleaned up?

| Category                 | Example                                                        | Result       |
| ------------------------ | -------------------------------------------------------------- | ------------ |
| Trailing ellipsis        | Text...                                                        | Text         |
| Comma before ? / !       | ,?                                                             | ?            |
| Period after ? / !       | ?.                                                             | ?            |
| Period + comma           | ., or ,.                                                       | .            |
| Special char + period    | \_. or -.                                                      | \_ or -      |
| Duplicate characters     | !! or ?? or ::                                                 | ! or ? or :  |
| Double periods           | ..                                                             | .            |
| Ellipsis protection      | ... is protected before period cleanup and restored afterwards |
| Space before punctuation | Word .                                                         | Word.        |
| Space after punctuation  | Word.Word                                                      | Word. Word   |
| Whitespace               | Multiple spaces                                                | Single space |
| Trim                     | Leading/trailing whitespace removed                            |

## How it works

Mix of **regex rules** and **function rules** (normalize_whitespace, trim).

**Priorities:**

- 200: Remove trailing ellipsis (speech-to-text artifact)
- 15: Resolve punctuation conflicts
- 10: Consolidate duplicate punctuation
- 5: Remove space before punctuation
- 4: Ensure space after punctuation
- 3-1: Ellipsis protection (protect -> remove double periods -> restore)
- 0: Normalize whitespace
- -1: Trim

## Activation

Add to `rules_paths` in `config.json`:

```json
{
  "rules_paths": ["rules/general/cleanup.json"]
}
```
