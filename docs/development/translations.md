# Translations

## Adding a new language

To auto translate, simple run:

```bash
uv run scripts/build_translations.py --languages en ru pl es ar
```

Providing the desired translations after `--languages`. To add new text, the old translations (`.qm`) must be removed.
