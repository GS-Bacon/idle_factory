#!/usr/bin/env node
/**
 * Generate Test Matrix from Game Spec
 *
 * Parses game source files to extract:
 * - All GameActions
 * - All InputStates / UIContext
 * - Valid state transitions
 * - Expected UI elements per state
 * - Cursor behavior per state
 *
 * Outputs:
 * - tests/generated/*.toml - Scenario test files
 * - tests/generated/test_matrix.json - Full test matrix
 */

const fs = require('fs');
const path = require('path');

const PROJECT_DIR = path.join(__dirname, '..');
const SRC_DIR = path.join(PROJECT_DIR, 'src');
const OUTPUT_DIR = path.join(PROJECT_DIR, 'tests', 'generated');

// Parse GameAction enum from input/mod.rs
function parseGameActions(content) {
    const actions = [];
    const match = content.match(/pub enum GameAction \{([\s\S]*?)\}/);
    if (match) {
        const body = match[1];
        const lines = body.split('\n');
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed && !trimmed.startsWith('//') && !trimmed.startsWith('#')) {
                const actionMatch = trimmed.match(/^(\w+),?$/);
                if (actionMatch) {
                    actions.push(actionMatch[1]);
                }
            }
        }
    }
    return actions;
}

// Parse UIContext enum from components/ui_state.rs
function parseUIContexts(content) {
    const contexts = [];
    const match = content.match(/pub enum UIContext \{([\s\S]*?)\}/);
    if (match) {
        const body = match[1];
        const lines = body.split('\n');
        for (const line of lines) {
            const trimmed = line.trim();
            if (trimmed && !trimmed.startsWith('//') && !trimmed.startsWith('#')) {
                // Match simple variants and variants with parameters
                const contextMatch = trimmed.match(/^(\w+)(?:\([^)]*\))?,?$/);
                if (contextMatch) {
                    contexts.push(contextMatch[1]);
                }
            }
        }
    }
    return contexts;
}

// Define complete state transitions
function defineStateTransitions() {
    return {
        Gameplay: {
            ToggleInventory: { to: 'Inventory', cursor: 'unlocked' },
            ToggleGlobalInventory: { to: 'GlobalInventory', cursor: 'unlocked' },
            OpenCommand: { to: 'CommandInput', cursor: 'unlocked' },
            CloseUI: { to: 'PauseMenu', cursor: 'unlocked' },
            // Actions that don't change state
            MoveForward: { to: 'Gameplay', cursor: 'locked' },
            MoveBackward: { to: 'Gameplay', cursor: 'locked' },
            MoveLeft: { to: 'Gameplay', cursor: 'locked' },
            MoveRight: { to: 'Gameplay', cursor: 'locked' },
            Jump: { to: 'Gameplay', cursor: 'locked' },
            PrimaryAction: { to: 'Gameplay', cursor: 'locked' },
            SecondaryAction: { to: 'Gameplay', cursor: 'locked' }, // or Machine if targeting
            RotateBlock: { to: 'Gameplay', cursor: 'locked' },
            Hotbar1: { to: 'Gameplay', cursor: 'locked' },
            Hotbar2: { to: 'Gameplay', cursor: 'locked' },
            Hotbar3: { to: 'Gameplay', cursor: 'locked' },
            Hotbar4: { to: 'Gameplay', cursor: 'locked' },
            Hotbar5: { to: 'Gameplay', cursor: 'locked' },
            Hotbar6: { to: 'Gameplay', cursor: 'locked' },
            Hotbar7: { to: 'Gameplay', cursor: 'locked' },
            Hotbar8: { to: 'Gameplay', cursor: 'locked' },
            Hotbar9: { to: 'Gameplay', cursor: 'locked' },
        },
        Inventory: {
            ToggleInventory: { to: 'Gameplay', cursor: 'locked' },
            CloseUI: { to: 'Gameplay', cursor: 'locked' },
        },
        GlobalInventory: {
            ToggleGlobalInventory: { to: 'Gameplay', cursor: 'locked' },
            CloseUI: { to: 'Gameplay', cursor: 'locked' },
        },
        CommandInput: {
            CloseUI: { to: 'Gameplay', cursor: 'locked' },
        },
        PauseMenu: {
            CloseUI: { to: 'Gameplay', cursor: 'locked' },
            // Settings button click -> Settings (would need UI click action)
        },
        Settings: {
            CloseUI: { to: 'PauseMenu', cursor: 'unlocked' },
        },
        Machine: {
            CloseUI: { to: 'Gameplay', cursor: 'locked' },
            ToggleInventory: { to: 'Gameplay', cursor: 'locked' }, // E closes machine UI
        },
    };
}

// Define expected UI elements per state
function defineExpectedUIElements() {
    return {
        Gameplay: {
            visible: ['Hotbar', 'Crosshair'],
            notVisible: ['InventoryPanel', 'PauseMenuPanel', 'SettingsPanel', 'MachineUIPanel', 'CommandInputField'],
        },
        Inventory: {
            visible: ['InventoryPanel', 'EquipmentSlots', 'CraftingList'],
            notVisible: ['Crosshair', 'PauseMenuPanel', 'CommandInputField'],
        },
        GlobalInventory: {
            visible: ['GlobalInventoryPanel'],
            notVisible: ['Crosshair', 'PauseMenuPanel'],
        },
        PauseMenu: {
            visible: ['PauseMenuPanel', 'ResumeButton', 'SettingsButton', 'QuitButton'],
            notVisible: ['Crosshair', 'InventoryPanel'],
        },
        Settings: {
            visible: [
                'SettingsPanel', 'BackButton',
                'Slider_MouseSensitivity', 'Slider_ViewDistance', 'Slider_Fov',
                'Slider_MasterVolume', 'Slider_SfxVolume', 'Slider_MusicVolume',
                'Toggle_VSync', 'Toggle_Fullscreen', 'Toggle_InvertY',
            ],
            notVisible: ['PauseMenuPanel', 'Crosshair'],
        },
        CommandInput: {
            visible: ['CommandInputField'],
            notVisible: ['InventoryPanel', 'PauseMenuPanel'],
        },
        Machine: {
            visible: ['MachineUIPanel', 'InputSlots', 'OutputSlots'],
            notVisible: ['Crosshair', 'InventoryPanel'],
        },
    };
}

// Define cursor behavior per state
function defineCursorBehavior() {
    return {
        Gameplay: { locked: true, visible: false, inWindow: true },
        Inventory: { locked: false, visible: true, inWindow: true },
        GlobalInventory: { locked: false, visible: true, inWindow: true },
        PauseMenu: { locked: false, visible: true, inWindow: true },
        Settings: { locked: false, visible: true, inWindow: true },
        CommandInput: { locked: false, visible: true, inWindow: true },
        Machine: { locked: false, visible: true, inWindow: true },
    };
}

// Define how to reach each state from Gameplay
function defineStateSetup() {
    return {
        Gameplay: [], // Already there
        Inventory: [{ action: 'ToggleInventory' }],
        GlobalInventory: [{ action: 'ToggleGlobalInventory' }],
        CommandInput: [{ action: 'OpenCommand' }],
        PauseMenu: [{ action: 'CloseUI' }], // ESC opens pause menu from gameplay
        Settings: [
            { action: 'CloseUI' }, // ESC -> PauseMenu
            // Note: Settings requires clicking a button, which we can't do via GameAction
        ],
        Machine: [], // Requires placing and right-clicking a machine - complex setup
    };
}

// Generate all scenario types
function generateScenarios(transitions, uiElements, cursorBehavior, stateSetup) {
    const scenarios = [];

    // 1. State transition scenarios
    for (const [fromState, actions] of Object.entries(transitions)) {
        for (const [action, result] of Object.entries(actions)) {
            if (result.to !== fromState) { // Only transitions that change state
                scenarios.push({
                    type: 'transition',
                    name: `transition_${fromState.toLowerCase()}_${action.toLowerCase()}_to_${result.to.toLowerCase()}`,
                    description: `${fromState} + ${action} → ${result.to}`,
                    fromState,
                    action,
                    toState: result.to,
                    expectedCursor: result.cursor,
                });
            }
        }
    }

    // 2. UI element visibility scenarios
    for (const [state, elements] of Object.entries(uiElements)) {
        const setup = stateSetup[state] || [];
        scenarios.push({
            type: 'ui_visibility',
            name: `ui_visibility_${state.toLowerCase()}`,
            description: `Verify UI elements in ${state} state`,
            state,
            setup, // Steps to reach this state
            visible: elements.visible,
            notVisible: elements.notVisible,
        });
    }

    // 3. Cursor behavior scenarios
    for (const [state, behavior] of Object.entries(cursorBehavior)) {
        const setup = stateSetup[state] || [];
        scenarios.push({
            type: 'cursor',
            name: `cursor_${state.toLowerCase()}`,
            description: `Verify cursor behavior in ${state} state`,
            state,
            setup, // Steps to reach this state
            cursorLocked: behavior.locked,
            cursorVisible: behavior.visible,
        });
    }

    // 4. Action coverage scenarios (verify all actions work in their expected states)
    const actionsInGameplay = [
        'MoveForward', 'MoveBackward', 'MoveLeft', 'MoveRight', 'Jump',
        'PrimaryAction', 'SecondaryAction', 'RotateBlock',
        'Hotbar1', 'Hotbar2', 'Hotbar3', 'Hotbar4', 'Hotbar5',
        'Hotbar6', 'Hotbar7', 'Hotbar8', 'Hotbar9',
    ];
    for (const action of actionsInGameplay) {
        scenarios.push({
            type: 'action_coverage',
            name: `action_gameplay_${action.toLowerCase()}`,
            description: `${action} works in Gameplay`,
            state: 'Gameplay',
            action,
            expectStateChange: false,
        });
    }

    return scenarios;
}

// Generate TOML for transition scenario
function generateTransitionTOML(scenario) {
    return `# Auto-generated: ${scenario.description}
name = "${scenario.name}"
description = "${scenario.description}"

# Start in ${scenario.fromState}
[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.fromState}" }

# Perform action: ${scenario.action}
[[steps]]
action = "send_input"
params = { action = "${scenario.action}" }

[[steps]]
action = "wait"
params = { ms = 150 }

# Verify transition to ${scenario.toState}
[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.toState}" }

# Verify cursor state
[[steps]]
action = "assert"
params = { condition = "cursor_locked == ${scenario.expectedCursor === 'locked'}" }
`;
}

// Generate setup steps TOML
function generateSetupSteps(setup, targetState) {
    if (!setup || setup.length === 0) {
        return '';
    }

    let steps = `# Setup: Navigate to ${targetState}
`;
    for (const step of setup) {
        steps += `[[steps]]
action = "send_input"
params = { action = "${step.action}" }

[[steps]]
action = "wait"
params = { ms = 150 }

`;
    }
    return steps;
}

// Generate TOML for UI visibility scenario
function generateUIVisibilityTOML(scenario) {
    let steps = `# Auto-generated: ${scenario.description}
name = "${scenario.name}"
description = "${scenario.description}"

# Initial state check
[[steps]]
action = "get_state"

`;
    // Add setup steps to reach target state
    steps += generateSetupSteps(scenario.setup, scenario.state);

    steps += `# Verify state is ${scenario.state}
[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.state}" }

# Take screenshot for visual verification
[[steps]]
action = "screenshot"
params = { name = "${scenario.name}" }

`;
    // Add assertions for visible elements
    for (const element of scenario.visible) {
        steps += `# Verify ${element} is visible
[[steps]]
action = "assert"
params = { condition = "has_ui_element == ${element}" }

`;
    }

    // Add assertions for not-visible elements
    for (const element of scenario.notVisible) {
        steps += `# Verify ${element} is NOT visible
[[steps]]
action = "assert"
params = { condition = "no_ui_element == ${element}" }

`;
    }

    return steps;
}

// Generate TOML for cursor behavior scenario
function generateCursorTOML(scenario) {
    let steps = `# Auto-generated: ${scenario.description}
name = "${scenario.name}"
description = "${scenario.description}"

# Initial state check
[[steps]]
action = "get_state"

`;
    // Add setup steps to reach target state
    steps += generateSetupSteps(scenario.setup, scenario.state);

    steps += `# Verify we're in ${scenario.state}
[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.state}" }

# Verify cursor locked state
[[steps]]
action = "assert"
params = { condition = "cursor_locked == ${scenario.cursorLocked}" }

# Verify cursor visibility
[[steps]]
action = "assert"
params = { condition = "cursor_visible == ${scenario.cursorVisible}" }
`;
    return steps;
}

// Generate TOML for action coverage scenario
function generateActionCoverageTOML(scenario) {
    return `# Auto-generated: ${scenario.description}
name = "${scenario.name}"
description = "${scenario.description}"

# Verify starting state is ${scenario.state}
[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.state}" }

# Perform action: ${scenario.action}
[[steps]]
action = "send_input"
params = { action = "${scenario.action}" }

[[steps]]
action = "wait"
params = { ms = 50 }

# Verify state unchanged (action executed without error)
[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.state}" }
`;
}

// Main
function main() {
    console.log('Generating comprehensive test matrix...\n');

    // Parse source files
    const inputModPath = path.join(SRC_DIR, 'input', 'mod.rs');
    const uiStatePath = path.join(SRC_DIR, 'components', 'ui_state.rs');

    const inputModContent = fs.readFileSync(inputModPath, 'utf8');
    const uiStateContent = fs.readFileSync(uiStatePath, 'utf8');

    const gameActions = parseGameActions(inputModContent);
    const uiContexts = parseUIContexts(uiStateContent);

    console.log(`Found ${gameActions.length} GameActions`);
    console.log(`Found ${uiContexts.length} UIContexts: ${uiContexts.join(', ')}\n`);

    // Build test definitions
    const transitions = defineStateTransitions();
    const uiElements = defineExpectedUIElements();
    const cursorBehavior = defineCursorBehavior();
    const stateSetup = defineStateSetup();

    // Generate scenarios
    const scenarios = generateScenarios(transitions, uiElements, cursorBehavior, stateSetup);

    console.log(`Generated ${scenarios.length} test scenarios:\n`);
    const byType = {};
    for (const s of scenarios) {
        byType[s.type] = (byType[s.type] || 0) + 1;
    }
    for (const [type, count] of Object.entries(byType)) {
        console.log(`  ${type}: ${count}`);
    }
    console.log('');

    // Create output directory
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    // Write scenario files
    const written = new Set();
    for (const scenario of scenarios) {
        let content;
        switch (scenario.type) {
            case 'transition':
                content = generateTransitionTOML(scenario);
                break;
            case 'ui_visibility':
                content = generateUIVisibilityTOML(scenario);
                break;
            case 'cursor':
                content = generateCursorTOML(scenario);
                break;
            case 'action_coverage':
                content = generateActionCoverageTOML(scenario);
                break;
            default:
                continue;
        }

        const filename = `${scenario.name}.toml`;
        if (!written.has(filename)) {
            const filepath = path.join(OUTPUT_DIR, filename);
            fs.writeFileSync(filepath, content);
            written.add(filename);
        }
    }
    console.log(`Written ${written.size} scenario files to tests/generated/\n`);

    // Write full matrix as JSON
    const matrix = {
        gameActions,
        uiContexts,
        transitions,
        uiElements,
        cursorBehavior,
        scenarios: scenarios.map(s => ({
            type: s.type,
            name: s.name,
            description: s.description,
        })),
    };

    const matrixPath = path.join(OUTPUT_DIR, 'test_matrix.json');
    fs.writeFileSync(matrixPath, JSON.stringify(matrix, null, 2));
    console.log(`Written test_matrix.json with ${scenarios.length} scenarios`);

    // Summary
    console.log('\n=== Summary ===');
    console.log(`Actions: ${gameActions.length}`);
    console.log(`UI Contexts: ${uiContexts.length}`);
    console.log(`Total Scenarios: ${scenarios.length}`);
    console.log(`\nRun scenarios with: ./scripts/run-all-scenarios.sh`);
}

main();
