#!/usr/bin/env python3
"""
API Wiki Generator (Bilingual: English/Japanese)

Extracts /// doc comments from Rust source files and generates Markdown
for GitHub Wiki. Supports `# ja` section for Japanese translations.

Usage:
    python scripts/generate-wiki.py [--check-only] [--lang=en|ja|all]

Options:
    --check-only    Only check for missing documentation, don't generate files
    --lang=LANG     Generate only specified language (en, ja, or all). Default: all
"""

import os
import re
import sys
from pathlib import Path
from dataclasses import dataclass, field
from typing import Optional

# Paths
SCRIPT_DIR = Path(__file__).parent
PROJECT_ROOT = SCRIPT_DIR.parent
HANDLERS_DIR = PROJECT_ROOT / "src" / "modding" / "handlers"
WASM_API_DIR = PROJECT_ROOT / "src" / "modding" / "wasm" / "api"
OUTPUT_DIR = PROJECT_ROOT / "wiki"


@dataclass
class ApiDoc:
    """Documentation for a single API method/function"""
    name: str
    description: str = ""
    description_ja: str = ""
    params: list[str] = field(default_factory=list)
    response: str = ""
    response_ja: str = ""
    returns: str = ""
    returns_ja: str = ""
    file: str = ""
    line: int = 0

    def has_required_docs(self, api_type: str) -> tuple[bool, list[str]]:
        """Check if this API has all required documentation"""
        missing = []

        if not self.description.strip():
            missing.append("description (/// comment)")

        if api_type == "websocket":
            # Accept either # Response or # Returns for WebSocket APIs
            if not self.response.strip() and not self.returns.strip():
                missing.append("# Response or # Returns section with JSON example")
        elif api_type == "wasm":
            if not self.returns.strip():
                missing.append("# Returns section")

        return len(missing) == 0, missing

    def get_response_json(self, lang: str = "en") -> str:
        """Get response JSON from either response or returns field"""
        if lang == "ja":
            content = self.response_ja.strip() or self.returns_ja.strip()
            # Fall back to English if no Japanese
            if not content:
                content = self.response.strip() or self.returns.strip()
        else:
            content = self.response.strip() or self.returns.strip()

        # Remove markdown code fence if present
        lines = content.split("\n")
        result_lines = []
        for line in lines:
            stripped = line.strip()
            # Skip code fence markers
            if stripped.startswith("```"):
                continue
            result_lines.append(line)
        return "\n".join(result_lines).strip()

    def get_description(self, lang: str = "en") -> str:
        """Get description for specified language"""
        if lang == "ja":
            return self.description_ja.strip() or self.description.strip()
        return self.description.strip()

    def get_returns(self, lang: str = "en") -> str:
        """Get returns for specified language"""
        if lang == "ja":
            return self.returns_ja.strip() or self.returns.strip()
        return self.returns.strip()


def parse_doc_comment(lines: list[str], start_idx: int) -> tuple[str, str, dict[str, str]]:
    """
    Parse a Rust doc comment starting at start_idx.
    Returns (description_en, description_ja, sections) where sections is a dict of section_name -> content

    Format:
        /// English description
        ///
        /// # ja
        /// Japanese description
        ///
        /// # Response
        /// JSON example
        ///
        /// # ja Response
        /// Japanese JSON example
    """
    description_en_lines = []
    description_ja_lines = []
    sections: dict[str, str] = {}
    current_section: Optional[str] = None
    section_lines: list[str] = []
    in_ja_description = False

    idx = start_idx
    while idx < len(lines):
        line = lines[idx]
        stripped = line.strip()

        # Check if this is a doc comment line
        if stripped.startswith("///"):
            content = stripped[3:].strip() if len(stripped) > 3 else ""

            # Check for section header
            if content.startswith("# "):
                # Save previous section
                if current_section:
                    sections[current_section] = "\n".join(section_lines).strip()

                section_name = content[2:].strip()

                # Check if this is the `# ja` marker for Japanese description
                if section_name == "ja":
                    in_ja_description = True
                    current_section = None
                    section_lines = []
                else:
                    in_ja_description = False
                    current_section = section_name
                    section_lines = []
            elif current_section:
                section_lines.append(content)
            elif in_ja_description:
                description_ja_lines.append(content)
            else:
                description_en_lines.append(content)

            idx += 1
        elif stripped.startswith("//!"):
            # Module-level doc, skip
            idx += 1
        elif stripped == "" or stripped.startswith("//"):
            idx += 1
        else:
            break

    # Save last section
    if current_section:
        sections[current_section] = "\n".join(section_lines).strip()

    description_en = "\n".join(description_en_lines).strip()
    description_ja = "\n".join(description_ja_lines).strip()
    return description_en, description_ja, sections


def extract_websocket_apis() -> list[ApiDoc]:
    """Extract WebSocket API documentation from handlers/"""
    apis: list[ApiDoc] = []

    # First, get method names from mod.rs route_request
    mod_rs = HANDLERS_DIR / "mod.rs"
    if not mod_rs.exists():
        print(f"Warning: {mod_rs} not found")
        return apis

    with open(mod_rs, "r") as f:
        content = f.read()

    # Extract method -> handler mappings
    # Pattern: "method.name" => module::handler_function(...)
    method_pattern = re.compile(r'"([a-z_]+\.[a-z_]+)"\s*=>\s*([a-z_]+)::([a-z_]+)\s*\(')
    methods = method_pattern.findall(content)

    # Map: handler_file -> [(method_name, handler_func), ...]
    file_handlers: dict[str, list[tuple[str, str]]] = {}
    for method, module, func in methods:
        if module not in file_handlers:
            file_handlers[module] = []
        file_handlers[module].append((method, func))

    # Process each handler file
    for module, handlers in file_handlers.items():
        # Map module name to file
        if module == "mod_handlers":
            file_path = HANDLERS_DIR / "mod_handlers.rs"
        else:
            file_path = HANDLERS_DIR / f"{module}.rs"

        if not file_path.exists():
            print(f"Warning: {file_path} not found")
            continue

        with open(file_path, "r") as f:
            lines = f.readlines()

        for method_name, func_name in handlers:
            # Find the function
            func_pattern = re.compile(rf"pub\s+fn\s+{func_name}\s*\(")

            for i, line in enumerate(lines):
                if func_pattern.search(line):
                    # Look backwards for doc comment
                    doc_start = i - 1
                    while doc_start >= 0 and (
                        lines[doc_start].strip().startswith("///") or
                        lines[doc_start].strip() == ""
                    ):
                        doc_start -= 1
                    doc_start += 1

                    desc_en, desc_ja, sections = parse_doc_comment(lines, doc_start)

                    api = ApiDoc(
                        name=method_name,
                        description=desc_en.split("\n")[0] if desc_en else "",
                        description_ja=desc_ja.split("\n")[0] if desc_ja else "",
                        response=sections.get("Response", ""),
                        response_ja=sections.get("ja Response", ""),
                        returns=sections.get("Returns", ""),
                        returns_ja=sections.get("ja Returns", ""),
                        file=str(file_path.relative_to(PROJECT_ROOT)),
                        line=i + 1,
                    )
                    apis.append(api)
                    break

    return apis


def extract_wasm_apis() -> list[ApiDoc]:
    """Extract WASM host function documentation from wasm/api/"""
    apis: list[ApiDoc] = []

    if not WASM_API_DIR.exists():
        print(f"Warning: {WASM_API_DIR} not found")
        return apis

    # Look for host_ functions in .rs files
    for rs_file in WASM_API_DIR.glob("*.rs"):
        if rs_file.name == "mod.rs":
            continue

        with open(rs_file, "r") as f:
            lines = f.readlines()

        # Pattern for host functions: pub fn host_xxx or fn host_xxx
        func_pattern = re.compile(r"(?:pub\s+)?fn\s+(host_[a-z_]+)\s*\(([^)]*)\)")

        for i, line in enumerate(lines):
            match = func_pattern.search(line)
            if match:
                func_name = match.group(1)
                params = match.group(2)

                # Look backwards for doc comment
                doc_start = i - 1
                while doc_start >= 0 and (
                    lines[doc_start].strip().startswith("///") or
                    lines[doc_start].strip() == ""
                ):
                    doc_start -= 1
                doc_start += 1

                desc_en, desc_ja, sections = parse_doc_comment(lines, doc_start)

                api = ApiDoc(
                    name=func_name,
                    description=desc_en.split("\n")[0] if desc_en else "",
                    description_ja=desc_ja.split("\n")[0] if desc_ja else "",
                    params=[p.strip() for p in params.split(",") if p.strip()],
                    returns=sections.get("Returns", ""),
                    returns_ja=sections.get("ja Returns", ""),
                    file=str(rs_file.relative_to(PROJECT_ROOT)),
                    line=i + 1,
                )
                apis.append(api)

    return apis


def validate_docs(websocket_apis: list[ApiDoc], wasm_apis: list[ApiDoc]) -> list[str]:
    """Validate that all APIs have required documentation. Returns list of errors."""
    errors: list[str] = []

    for api in websocket_apis:
        valid, missing = api.has_required_docs("websocket")
        if not valid:
            errors.append(f"WebSocket API '{api.name}' ({api.file}:{api.line}) missing: {', '.join(missing)}")

    for api in wasm_apis:
        valid, missing = api.has_required_docs("wasm")
        if not valid:
            errors.append(f"WASM function '{api.name}' ({api.file}:{api.line}) missing: {', '.join(missing)}")

    return errors


def generate_websocket_md(apis: list[ApiDoc], lang: str = "en") -> str:
    """Generate WebSocket-API.md content"""
    if lang == "ja":
        lines = [
            "# WebSocket API リファレンス",
            "",
            "Mod連携用 JSON-RPC 2.0 API",
            "",
            "**接続先**: `ws://127.0.0.1:9877`",
            "",
            "## メソッド一覧",
            "",
        ]
    else:
        lines = [
            "# WebSocket API Reference",
            "",
            "JSON-RPC 2.0 API for Mod integration.",
            "",
            "**Connection**: `ws://127.0.0.1:9877`",
            "",
            "## Methods",
            "",
        ]

    # Deduplicate by method name (keep first occurrence)
    seen_names: set[str] = set()
    unique_apis: list[ApiDoc] = []
    for api in apis:
        if api.name not in seen_names:
            seen_names.add(api.name)
            unique_apis.append(api)

    # Group by category (first part of method name)
    categories: dict[str, list[ApiDoc]] = {}
    for api in sorted(unique_apis, key=lambda x: x.name):
        category = api.name.split(".")[0]
        if category not in categories:
            categories[category] = []
        categories[category].append(api)

    for category, cat_apis in sorted(categories.items()):
        lines.append(f"### {category.title()}")
        lines.append("")

        for api in cat_apis:
            lines.append(f"#### `{api.name}`")
            lines.append("")
            description = api.get_description(lang)
            if description:
                lines.append(description)
                lines.append("")

            response_json = api.get_response_json(lang)
            if response_json:
                if lang == "ja":
                    lines.append("**レスポンス:**")
                else:
                    lines.append("**Response:**")
                lines.append("```json")
                lines.append(response_json)
                lines.append("```")
                lines.append("")

    return "\n".join(lines)


def generate_wasm_md(apis: list[ApiDoc], lang: str = "en") -> str:
    """Generate WASM-Host-Functions.md content"""
    if lang == "ja":
        lines = [
            "# WASMホスト関数リファレンス",
            "",
            "Core Mod (WASM) で使用可能なホスト関数",
            "",
            "## 関数一覧",
            "",
            "| 関数 | パラメータ | 戻り値 | 説明 |",
            "|------|------------|--------|------|",
        ]
    else:
        lines = [
            "# WASM Host Functions Reference",
            "",
            "Host functions available to Core Mods (WASM).",
            "",
            "## Functions",
            "",
            "| Function | Parameters | Returns | Description |",
            "|----------|------------|---------|-------------|",
        ]

    for api in sorted(apis, key=lambda x: x.name):
        params = ", ".join(api.params) if api.params else "-"
        returns = api.get_returns(lang).split("\n")[0] if api.get_returns(lang) else "-"
        desc = api.get_description(lang) or "-"
        lines.append(f"| `{api.name}` | `{params}` | {returns} | {desc} |")

    lines.append("")
    if lang == "ja":
        lines.append("## 詳細")
    else:
        lines.append("## Details")
    lines.append("")

    for api in sorted(apis, key=lambda x: x.name):
        lines.append(f"### `{api.name}`")
        lines.append("")
        description = api.get_description(lang)
        if description:
            lines.append(description)
            lines.append("")
        if api.params:
            if lang == "ja":
                lines.append("**パラメータ:**")
            else:
                lines.append("**Parameters:**")
            for param in api.params:
                lines.append(f"- `{param}`")
            lines.append("")
        returns = api.get_returns(lang)
        if returns:
            if lang == "ja":
                lines.append("**戻り値:**")
            else:
                lines.append("**Returns:**")
            lines.append(returns)
            lines.append("")

    return "\n".join(lines)


def generate_home_md(lang: str = "en") -> str:
    """Generate Home.md content"""
    if lang == "ja":
        return """# Mod API ドキュメント

ゲームのMod API ドキュメントへようこそ。

[English](Home) | **日本語**

## API種別

| 種別 | 説明 | リンク |
|------|------|--------|
| **WebSocket API** | Script Mod用 JSON-RPC 2.0 API | [WebSocket API](WebSocket-API-ja) |
| **WASMホスト関数** | Core Mod用ホスト関数 | [WASMホスト関数](WASM-Host-Functions-ja) |

## はじめに

### Script Mod (WebSocket)

```javascript
const ws = new WebSocket("ws://127.0.0.1:9877");

ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "game.version",
    params: {}
  }));
};

ws.onmessage = (event) => {
  console.log(JSON.parse(event.data));
};
```

### Core Mod (WASM)

利用可能な関数は [WASMホスト関数](WASM-Host-Functions-ja) を参照してください。

---

*ソースコードから自動生成*
"""
    else:
        return """# Mod API Documentation

Welcome to the Mod API documentation for the game.

**English** | [日本語](Home-ja)

## API Types

| Type | Description | Link |
|------|-------------|------|
| **WebSocket API** | JSON-RPC 2.0 API for Script Mods | [WebSocket API](WebSocket-API) |
| **WASM Host Functions** | Host functions for Core Mods | [WASM Host Functions](WASM-Host-Functions) |

## Getting Started

### Script Mod (WebSocket)

```javascript
const ws = new WebSocket("ws://127.0.0.1:9877");

ws.onopen = () => {
  ws.send(JSON.stringify({
    jsonrpc: "2.0",
    id: 1,
    method: "game.version",
    params: {}
  }));
};

ws.onmessage = (event) => {
  console.log(JSON.parse(event.data));
};
```

### Core Mod (WASM)

See [WASM Host Functions](WASM-Host-Functions) for available functions.

---

*Auto-generated from source code.*
"""


def main():
    check_only = "--check-only" in sys.argv

    # Parse --lang option
    lang_option = "all"
    for arg in sys.argv:
        if arg.startswith("--lang="):
            lang_option = arg.split("=")[1]

    print("Extracting API documentation...")

    websocket_apis = extract_websocket_apis()
    print(f"  Found {len(websocket_apis)} WebSocket API methods")

    wasm_apis = extract_wasm_apis()
    print(f"  Found {len(wasm_apis)} WASM host functions")

    # Validate
    print("\nValidating documentation...")
    errors = validate_docs(websocket_apis, wasm_apis)

    if errors:
        print(f"\n❌ Found {len(errors)} documentation errors:\n")
        for error in errors:
            print(f"  - {error}")
        print("\nPlease add the missing documentation and try again.")
        sys.exit(1)

    print("✅ All APIs have required documentation")

    if check_only:
        print("\n--check-only mode, skipping file generation")
        return

    # Generate output
    print(f"\nGenerating Wiki pages (lang={lang_option})...")
    OUTPUT_DIR.mkdir(exist_ok=True)

    languages = ["en", "ja"] if lang_option == "all" else [lang_option]

    for lang in languages:
        suffix = "-ja" if lang == "ja" else ""

        # Home.md
        home_path = OUTPUT_DIR / f"Home{suffix}.md"
        with open(home_path, "w") as f:
            f.write(generate_home_md(lang))
        print(f"  Generated {home_path}")

        # WebSocket-API.md
        ws_path = OUTPUT_DIR / f"WebSocket-API{suffix}.md"
        with open(ws_path, "w") as f:
            f.write(generate_websocket_md(websocket_apis, lang))
        print(f"  Generated {ws_path}")

        # WASM-Host-Functions.md
        wasm_path = OUTPUT_DIR / f"WASM-Host-Functions{suffix}.md"
        with open(wasm_path, "w") as f:
            f.write(generate_wasm_md(wasm_apis, lang))
        print(f"  Generated {wasm_path}")

    print("\n✅ Done!")


if __name__ == "__main__":
    main()
