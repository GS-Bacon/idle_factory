# Test Assets

Minimal assets for CI/E2E testing.

## Contents

- `items.yaml` - Basic test items
- `recipes.yaml` - Simple test recipes

## Usage

Copy to `assets/data/` when running tests in headless mode:
```bash
cp -r assets/test/* assets/data/
```

Or use `--test-assets` flag (when implemented).
