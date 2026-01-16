#!/bin/bash
# UI Preview Generator
# Converts TOML style definitions to HTML preview
#
# Usage:
#   ./scripts/ui-preview.sh [styles.toml]
#   ./scripts/ui-preview.sh mods/base/ui/styles.toml

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

node scripts/ui-preview.js "$@"
