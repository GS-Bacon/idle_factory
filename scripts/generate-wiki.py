#!/usr/bin/env python3
"""
API Wiki Generator

Extracts /// doc comments from Rust source files and generates Markdown
for GitHub Wiki. Validates that all APIs have required documentation.

Usage:
    python scripts/generate-wiki.py [--check-only]

Options:
    --check-only    Only check for missing documentation, don't generate files
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
    params: list[str] = field(default_factory=list)
    response: str = ""
    returns: str = ""
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

    def get_response_json(self) -> str:
        """Get response JSON from either response or returns field"""
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


def parse_doc_comment(lines: list[str], start_idx: int) -> tuple[str, dict[str, str]]:
    """
    Parse a Rust doc comment starting at start_idx.
    Returns (description, sections) where sections is a dict of section_name -> content
    """
    description_lines = []
    sections: dict[str, str] = {}
    current_section: Optional[str] = None
    section_lines: list[str] = []

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
                current_section = content[2:].strip()
                section_lines = []
            elif current_section:
                section_lines.append(content)
            else:
                description_lines.append(content)

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

    description = "\n".join(description_lines).strip()
    return description, sections


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

                    description, sections = parse_doc_comment(lines, doc_start)

                    api = ApiDoc(
                        name=method_name,
                        description=description.split("\n")[0] if description else "",
                        response=sections.get("Response", ""),
                        returns=sections.get("Returns", ""),
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

                description, sections = parse_doc_comment(lines, doc_start)

                api = ApiDoc(
                    name=func_name,
                    description=description.split("\n")[0] if description else "",
                    params=[p.strip() for p in params.split(",") if p.strip()],
                    returns=sections.get("Returns", ""),
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


def generate_websocket_md(apis: list[ApiDoc]) -> str:
    """Generate WebSocket-API.md content"""
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
            if api.description:
                lines.append(api.description)
                lines.append("")

            response_json = api.get_response_json()
            if response_json:
                lines.append("**Response:**")
                lines.append("```json")
                lines.append(response_json)
                lines.append("```")
                lines.append("")

    return "\n".join(lines)


def generate_wasm_md(apis: list[ApiDoc]) -> str:
    """Generate WASM-Host-Functions.md content"""
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
        returns = api.returns.split("\n")[0] if api.returns else "-"
        desc = api.description or "-"
        lines.append(f"| `{api.name}` | `{params}` | {returns} | {desc} |")

    lines.append("")
    lines.append("## Details")
    lines.append("")

    for api in sorted(apis, key=lambda x: x.name):
        lines.append(f"### `{api.name}`")
        lines.append("")
        if api.description:
            lines.append(api.description)
            lines.append("")
        if api.params:
            lines.append("**Parameters:**")
            for param in api.params:
                lines.append(f"- `{param}`")
            lines.append("")
        if api.returns:
            lines.append("**Returns:**")
            lines.append(api.returns)
            lines.append("")

    return "\n".join(lines)


def generate_home_md() -> str:
    """Generate Home.md content"""
    return """# Mod API Documentation

Welcome to the Mod API documentation for the game.

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
    print("\nGenerating Wiki pages...")
    OUTPUT_DIR.mkdir(exist_ok=True)

    # Home.md
    home_path = OUTPUT_DIR / "Home.md"
    with open(home_path, "w") as f:
        f.write(generate_home_md())
    print(f"  Generated {home_path}")

    # WebSocket-API.md
    ws_path = OUTPUT_DIR / "WebSocket-API.md"
    with open(ws_path, "w") as f:
        f.write(generate_websocket_md(websocket_apis))
    print(f"  Generated {ws_path}")

    # WASM-Host-Functions.md
    wasm_path = OUTPUT_DIR / "WASM-Host-Functions.md"
    with open(wasm_path, "w") as f:
        f.write(generate_wasm_md(wasm_apis))
    print(f"  Generated {wasm_path}")

    print("\n✅ Done!")


if __name__ == "__main__":
    main()
