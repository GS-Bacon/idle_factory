#!/usr/bin/env node
/**
 * UI Preview Generator
 *
 * Converts TOML style definitions to CSS and generates HTML preview files.
 *
 * Usage:
 *   node scripts/ui-preview.js [styles.toml] [--output dir]
 *   node scripts/ui-preview.js mods/base/ui/styles.toml
 *   node scripts/ui-preview.js --help
 */

const fs = require('fs');
const path = require('path');

// Simple TOML parser for our specific use case
function parseToml(content) {
    const result = { classes: {} };
    let currentSection = null;
    let currentClass = null;

    const lines = content.split('\n');

    for (const line of lines) {
        const trimmed = line.trim();

        // Skip empty lines and comments
        if (!trimmed || trimmed.startsWith('#')) continue;

        // Section header: [classes.name]
        const sectionMatch = trimmed.match(/^\[classes\.([^\]]+)\]$/);
        if (sectionMatch) {
            currentClass = sectionMatch[1];
            result.classes[currentClass] = {};
            continue;
        }

        // Key-value pairs
        if (currentClass) {
            const kvMatch = trimmed.match(/^(\w+)\s*=\s*(.+)$/);
            if (kvMatch) {
                const key = kvMatch[1];
                let value = kvMatch[2].trim();

                // Parse value
                if (value.startsWith('"') && value.endsWith('"')) {
                    // String value
                    value = value.slice(1, -1);
                } else if (value.startsWith('{')) {
                    // Inline table (border spec)
                    value = parseInlineTable(value);
                }

                result.classes[currentClass][key] = value;
            }
        }
    }

    return result;
}

function parseInlineTable(str) {
    // Parse { key = "value", key2 = "value2" }
    const result = {};
    const content = str.slice(1, -1); // Remove { }

    // Match key = "value" pairs
    const pairRegex = /(\w+)\s*=\s*"([^"]+)"/g;
    let match;
    while ((match = pairRegex.exec(content)) !== null) {
        result[match[1]] = match[2];
    }

    return result;
}

// Convert style object to CSS
function styleToCss(className, style) {
    const cssProperties = [];

    if (style.width) cssProperties.push(`width: ${style.width}`);
    if (style.height) cssProperties.push(`height: ${style.height}`);
    if (style.background) cssProperties.push(`background-color: ${style.background}`);
    if (style.color) cssProperties.push(`color: ${style.color}`);
    if (style.font_size) cssProperties.push(`font-size: ${style.font_size}`);
    if (style.padding) cssProperties.push(`padding: ${style.padding}`);
    if (style.margin) cssProperties.push(`margin: ${style.margin}`);
    if (style.margin_bottom) cssProperties.push(`margin-bottom: ${style.margin_bottom}`);
    if (style.border_radius) cssProperties.push(`border-radius: ${style.border_radius}`);

    // Border
    if (style.border && typeof style.border === 'object') {
        const border = style.border;
        if (border.width && border.color) {
            cssProperties.push(`border: ${border.width} solid ${border.color}`);
        }
        if (border.radius) {
            cssProperties.push(`border-radius: ${border.radius}`);
        }
    }

    return `.${className} {\n  ${cssProperties.join(';\n  ')};\n}`;
}

// Generate complete CSS from styles
function generateCss(styles) {
    const cssBlocks = [];

    // Base styles
    cssBlocks.push(`
/* Generated from TOML - Do not edit directly */
* {
  box-sizing: border-box;
  margin: 0;
  padding: 0;
}

body {
  font-family: 'Noto Sans JP', sans-serif;
  background-color: #0a0a0a;
  color: #ffffff;
  padding: 20px;
}

.preview-section {
  margin-bottom: 40px;
}

.preview-section h2 {
  color: #ff8700;
  margin-bottom: 16px;
  font-size: 18px;
}

.preview-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: flex-start;
}

.preview-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.preview-label {
  font-size: 12px;
  color: #888888;
}
`);

    // Convert each class
    for (const [className, style] of Object.entries(styles.classes)) {
        cssBlocks.push(styleToCss(className, style));
    }

    return cssBlocks.join('\n\n');
}

// Generate HTML preview
function generateHtml(styles, cssContent) {
    const classGroups = {
        'Inventory Slots': ['inventory-slot', 'inventory-slot-selected', 'slot-input', 'slot-output', 'slot-fuel'],
        'Hotbar': ['hotbar-slot', 'hotbar-slot-active'],
        'Panels': ['panel', 'panel-dark', 'machine-ui-panel'],
        'Buttons': ['button', 'button-secondary', 'button-danger'],
        'Typography': ['title', 'subtitle', 'item-count'],
        'Tabs': ['category-tab', 'category-tab-active'],
        'Progress': ['progress-bar', 'progress-bar-fill'],
        'Quest Items': ['quest-item', 'quest-item-complete'],
        'Tooltip': ['tooltip'],
    };

    let sectionsHtml = '';

    for (const [groupName, classNames] of Object.entries(classGroups)) {
        const items = classNames
            .filter(name => styles.classes[name])
            .map(name => {
                const style = styles.classes[name];
                let content = name;

                // Add appropriate content based on class type
                if (name.includes('button')) {
                    content = 'Button';
                } else if (name.includes('title')) {
                    content = 'Title Text';
                } else if (name.includes('subtitle')) {
                    content = 'Subtitle';
                } else if (name.includes('tooltip')) {
                    content = 'Tooltip text here';
                } else if (name.includes('tab')) {
                    content = 'Tab';
                } else if (name.includes('quest')) {
                    content = 'Quest Description';
                } else if (name.includes('count')) {
                    content = '99';
                } else if (name.includes('progress-bar-fill')) {
                    return `
        <div class="preview-item">
          <div class="progress-bar" style="width: 200px;">
            <div class="${name}" style="width: 70%;"></div>
          </div>
          <span class="preview-label">${name}</span>
        </div>`;
                } else if (name.includes('slot') || name.includes('hotbar')) {
                    content = '';
                }

                return `
        <div class="preview-item">
          <div class="${name}">${content}</div>
          <span class="preview-label">${name}</span>
        </div>`;
            })
            .join('');

        if (items) {
            sectionsHtml += `
    <div class="preview-section">
      <h2>${groupName}</h2>
      <div class="preview-grid">
        ${items}
      </div>
    </div>`;
        }
    }

    return `<!DOCTYPE html>
<html lang="ja">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>UI Style Preview</title>
  <link href="https://fonts.googleapis.com/css2?family=Noto+Sans+JP:wght@400;700&display=swap" rel="stylesheet">
  <style>
${cssContent}
  </style>
</head>
<body>
  <h1 style="color: #ff8700; margin-bottom: 32px;">UI Style Preview</h1>
  ${sectionsHtml}
</body>
</html>`;
}

// Main
function main() {
    const args = process.argv.slice(2);

    if (args.includes('--help') || args.includes('-h')) {
        console.log(`
UI Preview Generator

Usage:
  node scripts/ui-preview.js [styles.toml] [--output dir]

Options:
  --output, -o    Output directory (default: ui_preview)
  --help, -h      Show this help

Examples:
  node scripts/ui-preview.js mods/base/ui/styles.toml
  node scripts/ui-preview.js mods/base/ui/styles.toml --output preview
`);
        process.exit(0);
    }

    // Parse arguments
    let inputFile = 'mods/base/ui/styles.toml';
    let outputDir = 'ui_preview';

    for (let i = 0; i < args.length; i++) {
        if (args[i] === '--output' || args[i] === '-o') {
            outputDir = args[++i];
        } else if (!args[i].startsWith('-')) {
            inputFile = args[i];
        }
    }

    // Check input file exists
    if (!fs.existsSync(inputFile)) {
        console.error(`Error: Input file not found: ${inputFile}`);
        process.exit(1);
    }

    // Read and parse TOML
    console.log(`Reading: ${inputFile}`);
    const tomlContent = fs.readFileSync(inputFile, 'utf-8');
    const styles = parseToml(tomlContent);

    console.log(`Found ${Object.keys(styles.classes).length} style classes`);

    // Generate CSS and HTML
    const cssContent = generateCss(styles);
    const htmlContent = generateHtml(styles, cssContent);

    // Create output directory
    if (!fs.existsSync(outputDir)) {
        fs.mkdirSync(outputDir, { recursive: true });
    }

    // Write files
    const htmlPath = path.join(outputDir, 'preview.html');
    const cssPath = path.join(outputDir, 'styles.css');

    fs.writeFileSync(htmlPath, htmlContent);
    fs.writeFileSync(cssPath, cssContent);

    console.log(`Generated: ${htmlPath}`);
    console.log(`Generated: ${cssPath}`);
    console.log(`\nOpen ${htmlPath} in a browser to preview.`);
}

main();
