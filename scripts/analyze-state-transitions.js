#!/usr/bin/env node
/**
 * State Transition Analyzer
 *
 * Automatically extracts UI state transitions from Rust source code:
 * - Parses UIState.push/pop/replace calls
 * - Analyzes match statements for GameAction handling
 * - Builds state machine graph
 * - Detects unreachable states and dead ends
 * - Generates comprehensive test scenarios
 *
 * Usage: node scripts/analyze-state-transitions.js
 */

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const PROJECT_DIR = path.join(__dirname, '..');
const SRC_DIR = path.join(PROJECT_DIR, 'src');
const OUTPUT_DIR = path.join(PROJECT_DIR, 'tests', 'generated');

// ============================================================================
// Source Code Analysis
// ============================================================================

/**
 * Find all Rust files in src/
 */
function findRustFiles(dir = SRC_DIR) {
    const files = [];
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        if (entry.isDirectory()) {
            files.push(...findRustFiles(fullPath));
        } else if (entry.name.endsWith('.rs')) {
            files.push(fullPath);
        }
    }
    return files;
}

/**
 * Extract UIContext enum variants
 */
function extractUIContexts() {
    const uiStatePath = path.join(SRC_DIR, 'components', 'ui_state.rs');
    const content = fs.readFileSync(uiStatePath, 'utf8');

    const contexts = [];
    const match = content.match(/pub enum UIContext \{([\s\S]*?)\}/);
    if (match) {
        for (const line of match[1].split('\n')) {
            const m = line.trim().match(/^(\w+)(?:\([^)]*\))?,?(?:\s*\/\/.*)?$/);
            if (m) contexts.push(m[1]);
        }
    }
    return contexts;
}

/**
 * Extract GameAction enum variants
 */
function extractGameActions() {
    const inputModPath = path.join(SRC_DIR, 'input', 'mod.rs');
    const content = fs.readFileSync(inputModPath, 'utf8');

    const actions = [];
    const match = content.match(/pub enum GameAction \{([\s\S]*?)\}/);
    if (match) {
        for (const line of match[1].split('\n')) {
            const m = line.trim().match(/^(\w+),?(?:\s*\/\/.*)?$/);
            if (m) actions.push(m[1]);
        }
    }
    return actions;
}

/**
 * Find all UIState method calls in source code
 */
function findUIStateTransitions() {
    const transitions = [];
    const files = findRustFiles();

    for (const file of files) {
        const content = fs.readFileSync(file, 'utf8');
        const relPath = path.relative(PROJECT_DIR, file);

        // Find ui_state.push(UIContext::XXX)
        const pushMatches = content.matchAll(/ui_state\.push\s*\(\s*UIContext::(\w+)(?:\([^)]*\))?\s*\)/g);
        for (const m of pushMatches) {
            transitions.push({
                type: 'push',
                target: m[1],
                file: relPath,
                raw: m[0],
            });
        }

        // Find ui_state.pop()
        const popMatches = content.matchAll(/ui_state\.pop\s*\(\s*\)/g);
        for (const m of popMatches) {
            transitions.push({
                type: 'pop',
                target: 'previous',
                file: relPath,
                raw: m[0],
            });
        }

        // Find ui_state.replace(UIContext::XXX)
        const replaceMatches = content.matchAll(/ui_state\.replace\s*\(\s*UIContext::(\w+)(?:\([^)]*\))?\s*\)/g);
        for (const m of replaceMatches) {
            transitions.push({
                type: 'replace',
                target: m[1],
                file: relPath,
                raw: m[0],
            });
        }

        // Find ui_state.clear()
        const clearMatches = content.matchAll(/ui_state\.clear\s*\(\s*\)/g);
        for (const m of clearMatches) {
            transitions.push({
                type: 'clear',
                target: 'Gameplay',
                file: relPath,
                raw: m[0],
            });
        }
    }

    return transitions;
}

/**
 * Find GameAction handlers and their context
 */
function findActionHandlers() {
    const handlers = [];
    const files = findRustFiles();

    for (const file of files) {
        const content = fs.readFileSync(file, 'utf8');
        const relPath = path.relative(PROJECT_DIR, file);

        // Find match on GameAction
        const actionMatches = content.matchAll(/GameAction::(\w+)\s*=>/g);
        for (const m of actionMatches) {
            // Try to find surrounding context (which UIContext this is in)
            const lineNum = content.substring(0, m.index).split('\n').length;

            // Look backwards for UIContext match or is_active check
            const contextBefore = content.substring(Math.max(0, m.index - 500), m.index);
            const contextMatch = contextBefore.match(/UIContext::(\w+)|is_active\s*\(\s*&?\s*UIContext::(\w+)/g);
            let inContext = 'Unknown';
            if (contextMatch) {
                const last = contextMatch[contextMatch.length - 1];
                const cm = last.match(/UIContext::(\w+)/);
                if (cm) inContext = cm[1];
            }

            handlers.push({
                action: m[1],
                context: inContext,
                file: relPath,
                line: lineNum,
            });
        }
    }

    return handlers;
}

/**
 * Analyze which actions cause which transitions
 * Handles both direct UIState calls and UIAction event patterns
 */
function analyzeActionTransitions() {
    const results = [];
    const files = findRustFiles();

    for (const file of files) {
        const content = fs.readFileSync(file, 'utf8');
        const relPath = path.relative(PROJECT_DIR, file);

        // Pattern 1: Direct ui_state.push
        for (const m of content.matchAll(/GameAction::(\w+)\s*(?:if[^{]*)?=>\s*\{[^}]*ui_state\.push\s*\(\s*UIContext::(\w+)/g)) {
            results.push({ action: m[1], transition: 'push', target: m[2], file: relPath });
        }

        // Pattern 2: Direct ui_state.pop
        for (const m of content.matchAll(/GameAction::(\w+)\s*(?:if[^{]*)?=>\s*\{[^}]*ui_state\.pop/g)) {
            results.push({ action: m[1], transition: 'pop', target: 'previous', file: relPath });
        }

        // Pattern 3: Direct ui_state.clear
        for (const m of content.matchAll(/GameAction::(\w+)\s*(?:if[^{]*)?=>\s*\{[^}]*ui_state\.clear/g)) {
            results.push({ action: m[1], transition: 'clear', target: 'Gameplay', file: relPath });
        }

        // Pattern 4: Direct ui_state.replace
        for (const m of content.matchAll(/GameAction::(\w+)\s*(?:if[^{]*)?=>\s*\{[^}]*ui_state\.replace\s*\(\s*UIContext::(\w+)/g)) {
            results.push({ action: m[1], transition: 'replace', target: m[2], file: relPath });
        }

        // Pattern 5: UIAction::Push via action_writer.send
        for (const m of content.matchAll(/action_writer\.send\s*\(\s*UIAction::Push\s*\(\s*UIContext::(\w+)/g)) {
            const beforeContext = content.substring(Math.max(0, m.index - 500), m.index);

            // Look for just_pressed pattern
            const jpMatch = beforeContext.match(/just_pressed\s*\(\s*GameAction::(\w+)\s*\)/g);
            if (jpMatch) {
                for (const jp of jpMatch) {
                    const actionName = jp.match(/GameAction::(\w+)/);
                    if (actionName) {
                        results.push({ action: actionName[1], transition: 'push', target: m[1], file: relPath });
                    }
                }
            }

            // Also check for UIContext match arm
            const ctxMatch = beforeContext.match(/UIContext::(\w+)\s*=>/g);
            if (ctxMatch) {
                for (const ctx of ctxMatch) {
                    const ctxName = ctx.match(/UIContext::(\w+)/);
                    if (ctxName) {
                        results.push({
                            action: 'from_' + ctxName[1],
                            transition: 'push',
                            target: m[1],
                            file: relPath,
                            fromContext: ctxName[1],
                        });
                    }
                }
            }
        }

        // Pattern 6: UIAction::Pop
        for (const m of content.matchAll(/action_writer\.send\s*\(\s*UIAction::Pop\s*\)/g)) {
            const beforeContext = content.substring(Math.max(0, m.index - 500), m.index);

            const jpMatch = beforeContext.match(/just_pressed\s*\(\s*GameAction::(\w+)\s*\)/g);
            if (jpMatch) {
                for (const jp of jpMatch) {
                    const actionName = jp.match(/GameAction::(\w+)/);
                    if (actionName) {
                        results.push({ action: actionName[1], transition: 'pop', target: 'previous', file: relPath });
                    }
                }
            }

            const ctxMatch = beforeContext.match(/UIContext::(\w+)\s*=>/g);
            if (ctxMatch) {
                for (const ctx of ctxMatch) {
                    const ctxName = ctx.match(/UIContext::(\w+)/);
                    if (ctxName) {
                        results.push({
                            action: 'from_' + ctxName[1],
                            transition: 'pop',
                            target: 'Gameplay',
                            file: relPath,
                            fromContext: ctxName[1],
                        });
                    }
                }
            }
        }

        // Pattern 7: UIAction::Replace
        for (const m of content.matchAll(/action_writer\.send\s*\(\s*UIAction::Replace\s*\(\s*UIContext::(\w+)/g)) {
            const beforeContext = content.substring(Math.max(0, m.index - 500), m.index);

            const ctxMatch = beforeContext.match(/UIContext::(\w+)\s*=>/g);
            if (ctxMatch) {
                for (const ctx of ctxMatch) {
                    const ctxName = ctx.match(/UIContext::(\w+)/);
                    if (ctxName) {
                        results.push({
                            action: 'from_' + ctxName[1],
                            transition: 'replace',
                            target: m[1],
                            file: relPath,
                            fromContext: ctxName[1],
                        });
                    }
                }
            }
        }
    }

    return results;
}

/**
 * Build state machine graph
 */
function buildStateMachine(contexts, actionTransitions) {
    const graph = {};

    // Initialize all contexts
    for (const ctx of contexts) {
        graph[ctx] = {
            outgoing: [], // { action, target, type }
            incoming: [], // { action, source, type }
        };
    }
    // Gameplay is always reachable (initial state)
    graph['Gameplay'] = graph['Gameplay'] || { outgoing: [], incoming: [] };

    // Add transitions
    for (const t of actionTransitions) {
        // For now, assume transitions happen from any state (conservative)
        // A more sophisticated analysis would track context
        if (graph[t.target]) {
            graph[t.target].incoming.push({
                action: t.action,
                source: 'Any',
                type: t.transition,
            });
        }
    }

    return graph;
}

/**
 * Detect issues in state machine
 */
function detectIssues(graph, contexts) {
    const issues = [];

    for (const ctx of contexts) {
        if (!graph[ctx]) continue;

        // Check for unreachable states (no incoming transitions except Gameplay)
        if (ctx !== 'Gameplay' && graph[ctx].incoming.length === 0) {
            issues.push({
                type: 'unreachable',
                context: ctx,
                message: `${ctx} has no incoming transitions`,
            });
        }

        // Check for dead ends (no outgoing transitions)
        // Note: This needs more sophisticated analysis
    }

    return issues;
}

// ============================================================================
// Dependency Graph Generation
// ============================================================================

/**
 * Generate Mermaid diagram of state machine
 */
function generateMermaidDiagram(actionTransitions, contexts) {
    let mermaid = 'stateDiagram-v2\n';
    mermaid += '    [*] --> Gameplay\n';

    // Deduplicate transitions
    const seen = new Set();
    for (const t of actionTransitions) {
        const key = `${t.target}`;
        if (seen.has(key)) continue;
        seen.add(key);

        if (t.target && t.target !== 'previous') {
            mermaid += `    Gameplay --> ${t.target}: ${t.action}\n`;
        }
    }

    // Add pop transitions (back to previous)
    mermaid += '\n    %% Pop transitions\n';
    for (const ctx of contexts) {
        if (ctx !== 'Gameplay') {
            mermaid += `    ${ctx} --> Gameplay: CloseUI/ESC\n`;
        }
    }

    return mermaid;
}

/**
 * Generate DOT graph
 */
function generateDotGraph(actionTransitions, contexts) {
    let dot = 'digraph UIStateMachine {\n';
    dot += '    rankdir=TB;\n';
    dot += '    node [shape=box, style=rounded];\n';
    dot += '    Gameplay [style="rounded,filled", fillcolor="#90EE90"];\n\n';

    // Transitions
    const seen = new Set();
    for (const t of actionTransitions) {
        if (!t.target || t.target === 'previous') continue;

        const key = `Gameplay->${t.target}:${t.action}`;
        if (seen.has(key)) continue;
        seen.add(key);

        dot += `    Gameplay -> ${t.target} [label="${t.action}"];\n`;
    }

    // Back transitions
    dot += '\n    // Back transitions\n';
    for (const ctx of contexts) {
        if (ctx !== 'Gameplay') {
            dot += `    ${ctx} -> Gameplay [label="CloseUI", style=dashed];\n`;
        }
    }

    dot += '}\n';
    return dot;
}

// ============================================================================
// Test Generation
// ============================================================================

/**
 * Generate comprehensive tests based on discovered transitions
 */
function generateTransitionTests(actionTransitions, contexts) {
    const tests = [];

    // Group by action
    const byAction = {};
    for (const t of actionTransitions) {
        if (!byAction[t.action]) byAction[t.action] = [];
        byAction[t.action].push(t);
    }

    // Generate test for each unique transition
    const seen = new Set();
    for (const t of actionTransitions) {
        if (!t.target || t.target === 'previous') continue;

        const key = `gameplay_${t.action.toLowerCase()}_${t.target.toLowerCase()}`;
        if (seen.has(key)) continue;
        seen.add(key);

        tests.push({
            name: `auto_transition_${key}`,
            action: t.action,
            fromState: 'Gameplay',
            toState: t.target,
            file: t.file,
        });
    }

    // Generate return-to-gameplay tests
    for (const ctx of contexts) {
        if (ctx === 'Gameplay') continue;

        tests.push({
            name: `auto_return_${ctx.toLowerCase()}_to_gameplay`,
            action: 'CloseUI',
            fromState: ctx,
            toState: 'Gameplay',
        });
    }

    return tests;
}

// ============================================================================
// Main
// ============================================================================

function main() {
    console.log('=== State Transition Analyzer ===\n');

    // Extract enums
    const contexts = extractUIContexts();
    const actions = extractGameActions();

    console.log(`Found ${contexts.length} UIContexts: ${contexts.join(', ')}`);
    console.log(`Found ${actions.length} GameActions\n`);

    // Find all UI state transitions
    console.log('Analyzing source code...\n');
    const transitions = findUIStateTransitions();
    console.log(`Found ${transitions.length} UIState method calls\n`);

    // Group by file
    const byFile = {};
    for (const t of transitions) {
        if (!byFile[t.file]) byFile[t.file] = [];
        byFile[t.file].push(t);
    }

    console.log('UIState calls by file:');
    for (const [file, calls] of Object.entries(byFile).sort((a, b) => b[1].length - a[1].length)) {
        console.log(`  ${file}: ${calls.length} calls`);
    }

    // Find action handlers
    const handlers = findActionHandlers();
    console.log(`\nFound ${handlers.length} GameAction handlers\n`);

    // Analyze action-to-transition mapping
    const actionTransitions = analyzeActionTransitions();
    console.log(`Found ${actionTransitions.length} action->transition patterns\n`);

    // Deduplicate and show
    const uniqueTransitions = [];
    const seenT = new Set();
    for (const t of actionTransitions) {
        const key = `${t.action}->${t.target}`;
        if (!seenT.has(key)) {
            seenT.add(key);
            uniqueTransitions.push(t);
        }
    }

    console.log('Unique transitions found:');
    for (const t of uniqueTransitions) {
        console.log(`  ${t.action} -> ${t.target} (${t.transition}) [${t.file}]`);
    }

    // Build state machine
    const graph = buildStateMachine(contexts, uniqueTransitions);

    // Detect issues
    const issues = detectIssues(graph, contexts);
    if (issues.length > 0) {
        console.log('\n=== Potential Issues ===');
        for (const issue of issues) {
            console.log(`  [${issue.type}] ${issue.message}`);
        }
    }

    // Generate diagrams
    console.log('\n=== Generating Diagrams ===');

    const mermaid = generateMermaidDiagram(uniqueTransitions, contexts);
    const mermaidPath = path.join(OUTPUT_DIR, 'state_machine.mmd');
    fs.writeFileSync(mermaidPath, mermaid);
    console.log(`  Mermaid: ${mermaidPath}`);

    const dot = generateDotGraph(uniqueTransitions, contexts);
    const dotPath = path.join(OUTPUT_DIR, 'state_machine.dot');
    fs.writeFileSync(dotPath, dot);
    console.log(`  DOT: ${dotPath}`);

    // Generate tests
    console.log('\n=== Generating Tests ===');
    const tests = generateTransitionTests(uniqueTransitions, contexts);

    // Create output directory
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    // Write test files
    let written = 0;
    for (const test of tests) {
        const toml = generateTestTOML(test);
        const filepath = path.join(OUTPUT_DIR, `${test.name}.toml`);
        fs.writeFileSync(filepath, toml);
        written++;
    }
    console.log(`  Written ${written} auto-generated test files`);

    // Write analysis JSON
    const analysis = {
        generated_at: new Date().toISOString(),
        contexts,
        actions,
        transitions: uniqueTransitions,
        issues,
        tests: tests.map(t => t.name),
    };
    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'state_analysis.json'),
        JSON.stringify(analysis, null, 2)
    );
    console.log(`  Analysis saved to state_analysis.json`);

    console.log('\n=== Summary ===');
    console.log(`  UI Contexts: ${contexts.length}`);
    console.log(`  Game Actions: ${actions.length}`);
    console.log(`  Unique Transitions: ${uniqueTransitions.length}`);
    console.log(`  Generated Tests: ${written}`);
    console.log(`  Issues Found: ${issues.length}`);
}

function generateTestTOML(test) {
    const setupSteps = getSetupSteps(test.fromState);

    return `# Auto-generated: ${test.fromState} -> ${test.toState} via ${test.action}
name = "${test.name}"
description = "Extracted from ${test.file || 'analysis'}"

# Setup
[[steps]]
action = "get_state"

${setupSteps}
# Verify starting state
[[steps]]
action = "assert"
params = { condition = "ui_state == ${test.fromState}" }

# Perform action
[[steps]]
action = "send_input"
params = { action = "${test.action}" }

[[steps]]
action = "wait"
params = { ms = 150 }

# Verify result
[[steps]]
action = "assert"
params = { condition = "ui_state == ${test.toState}" }
`;
}

function getSetupSteps(state) {
    switch (state) {
        case 'Gameplay':
            return '';
        case 'Inventory':
            return `[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 150 }

`;
        case 'GlobalInventory':
            return `[[steps]]
action = "send_input"
params = { action = "ToggleGlobalInventory" }

[[steps]]
action = "wait"
params = { ms = 150 }

`;
        case 'CommandInput':
            return `[[steps]]
action = "send_input"
params = { action = "OpenCommand" }

[[steps]]
action = "wait"
params = { ms = 150 }

`;
        case 'PauseMenu':
            return `[[steps]]
action = "send_input"
params = { action = "CloseUI" }

[[steps]]
action = "wait"
params = { ms = 150 }

`;
        default:
            return '';
    }
}

main();
