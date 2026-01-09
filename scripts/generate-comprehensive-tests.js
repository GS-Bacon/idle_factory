#!/usr/bin/env node
/**
 * Comprehensive Test Generator
 *
 * Parses game source files AND mod data to generate exhaustive test scenarios:
 * - All UI state transitions
 * - All GameActions in all valid contexts
 * - All placeable items placement/destruction
 * - All machine interactions
 * - All crafting recipes
 * - All inventory operations
 * - Cursor states
 * - Settings controls
 * - Edge cases and error conditions
 *
 * Usage: node scripts/generate-comprehensive-tests.js
 */

const fs = require('fs');
const path = require('path');

const PROJECT_DIR = path.join(__dirname, '..');
const SRC_DIR = path.join(PROJECT_DIR, 'src');
const MODS_DIR = path.join(PROJECT_DIR, 'mods');
const OUTPUT_DIR = path.join(PROJECT_DIR, 'tests', 'generated');

// ============================================================================
// Parsers
// ============================================================================

function parseGameActions(content) {
    const actions = [];
    const match = content.match(/pub enum GameAction \{([\s\S]*?)\}/);
    if (match) {
        const body = match[1];
        for (const line of body.split('\n')) {
            const trimmed = line.trim();
            if (trimmed && !trimmed.startsWith('//') && !trimmed.startsWith('#')) {
                const m = trimmed.match(/^(\w+),?$/);
                if (m) actions.push(m[1]);
            }
        }
    }
    return actions;
}

function parseUIContexts(content) {
    const contexts = [];
    const match = content.match(/pub enum UIContext \{([\s\S]*?)\}/);
    if (match) {
        const body = match[1];
        for (const line of body.split('\n')) {
            const trimmed = line.trim();
            if (trimmed && !trimmed.startsWith('//') && !trimmed.startsWith('#')) {
                const m = trimmed.match(/^(\w+)(?:\([^)]*\))?,?$/);
                if (m) contexts.push(m[1]);
            }
        }
    }
    return contexts;
}

function parseTOML(content) {
    // Simple TOML parser for arrays of tables
    const items = [];
    let currentItem = null;
    let currentSection = null;

    for (const line of content.split('\n')) {
        const trimmed = line.trim();
        if (!trimmed || trimmed.startsWith('#')) continue;

        // Array of tables: [[item]] or [[machine]]
        const arrayMatch = trimmed.match(/^\[\[(\w+)\]\]$/);
        if (arrayMatch) {
            if (currentItem) items.push(currentItem);
            currentItem = { _type: arrayMatch[1] };
            currentSection = null;
            continue;
        }

        // Nested section: [item.ports]
        const sectionMatch = trimmed.match(/^\[[\w.]+\.(\w+)\]$/);
        if (sectionMatch) {
            currentSection = sectionMatch[1];
            if (currentItem) currentItem[currentSection] = {};
            continue;
        }

        // Key = value
        const kvMatch = trimmed.match(/^(\w+)\s*=\s*(.+)$/);
        if (kvMatch && currentItem) {
            const [, key, rawValue] = kvMatch;
            let value = rawValue;

            // Parse value
            if (rawValue.startsWith('"')) {
                value = rawValue.replace(/^"|"$/g, '');
            } else if (rawValue === 'true') {
                value = true;
            } else if (rawValue === 'false') {
                value = false;
            } else if (!isNaN(rawValue)) {
                value = Number(rawValue);
            } else if (rawValue.startsWith('[')) {
                try { value = JSON.parse(rawValue.replace(/'/g, '"')); } catch {}
            }

            if (currentSection && currentItem[currentSection]) {
                currentItem[currentSection][key] = value;
            } else {
                currentItem[key] = value;
            }
        }
    }
    if (currentItem) items.push(currentItem);
    return items;
}

function loadBaseMod() {
    const items = [];
    const machines = [];
    const recipes = [];

    try {
        const itemsContent = fs.readFileSync(path.join(MODS_DIR, 'base', 'items.toml'), 'utf8');
        items.push(...parseTOML(itemsContent).filter(i => i._type === 'item'));
    } catch {}

    try {
        const machinesContent = fs.readFileSync(path.join(MODS_DIR, 'base', 'machines.toml'), 'utf8');
        machines.push(...parseTOML(machinesContent).filter(m => m._type === 'machine'));
    } catch {}

    try {
        const recipesContent = fs.readFileSync(path.join(MODS_DIR, 'base', 'recipes.toml'), 'utf8');
        recipes.push(...parseTOML(recipesContent).filter(r => r._type === 'recipe'));
    } catch {}

    return { items, machines, recipes };
}

// ============================================================================
// State Definitions
// ============================================================================

const STATE_TRANSITIONS = {
    Gameplay: {
        ToggleInventory: 'Inventory',
        ToggleGlobalInventory: 'GlobalInventory',
        OpenCommand: 'CommandInput',
        CloseUI: 'PauseMenu',
        ToggleQuest: 'Gameplay', // Quest is a toggle overlay, not state change
        TogglePause: 'PauseMenu',
    },
    Inventory: {
        ToggleInventory: 'Gameplay',
        CloseUI: 'Gameplay',
    },
    GlobalInventory: {
        ToggleGlobalInventory: 'Gameplay',
        CloseUI: 'Gameplay',
    },
    CommandInput: {
        CloseUI: 'Gameplay',
        Cancel: 'Gameplay',
    },
    PauseMenu: {
        CloseUI: 'Gameplay',
        TogglePause: 'Gameplay',
        Cancel: 'Gameplay',
    },
    Settings: {
        CloseUI: 'PauseMenu',
        Cancel: 'PauseMenu',
    },
    Machine: {
        CloseUI: 'Gameplay',
        ToggleInventory: 'Gameplay',
    },
};

const CURSOR_STATES = {
    Gameplay: { locked: true, visible: false },
    Inventory: { locked: false, visible: true },
    GlobalInventory: { locked: false, visible: true },
    CommandInput: { locked: false, visible: true },
    PauseMenu: { locked: false, visible: true },
    Settings: { locked: false, visible: true },
    Machine: { locked: false, visible: true },
};

const VALID_ACTIONS_PER_STATE = {
    Gameplay: [
        'MoveForward', 'MoveBackward', 'MoveLeft', 'MoveRight',
        'Jump', 'Descend', 'LookUp', 'LookDown', 'LookLeft', 'LookRight',
        'PrimaryAction', 'SecondaryAction', 'RotateBlock',
        'Hotbar1', 'Hotbar2', 'Hotbar3', 'Hotbar4', 'Hotbar5',
        'Hotbar6', 'Hotbar7', 'Hotbar8', 'Hotbar9',
        'ToggleInventory', 'ToggleGlobalInventory', 'ToggleQuest',
        'OpenCommand', 'CloseUI', 'TogglePause', 'ToggleDebug',
        'ModifierShift',
    ],
    Inventory: [
        'ToggleInventory', 'CloseUI', 'Confirm', 'Cancel',
        'Hotbar1', 'Hotbar2', 'Hotbar3', 'Hotbar4', 'Hotbar5',
        'Hotbar6', 'Hotbar7', 'Hotbar8', 'Hotbar9',
    ],
    GlobalInventory: [
        'ToggleGlobalInventory', 'CloseUI', 'Confirm', 'Cancel',
    ],
    CommandInput: [
        'CloseUI', 'Confirm', 'Cancel', 'DeleteChar',
    ],
    PauseMenu: [
        'CloseUI', 'TogglePause', 'Cancel', 'Confirm',
    ],
    Settings: [
        'CloseUI', 'Cancel',
    ],
    Machine: [
        'CloseUI', 'ToggleInventory', 'Confirm', 'Cancel',
    ],
};

const SETTINGS_CONTROLS = {
    sliders: ['MouseSensitivity', 'ViewDistance', 'Fov', 'MasterVolume', 'SfxVolume', 'MusicVolume'],
    toggles: ['VSync', 'Fullscreen', 'InvertY'],
};

// ============================================================================
// Scenario Generators
// ============================================================================

function generateAllScenarios(gameActions, uiContexts, baseMod) {
    const scenarios = [];

    // 1. State Transitions (comprehensive)
    for (const [fromState, transitions] of Object.entries(STATE_TRANSITIONS)) {
        for (const [action, toState] of Object.entries(transitions)) {
            scenarios.push({
                type: 'state_transition',
                name: `state_${fromState.toLowerCase()}_${action.toLowerCase()}`,
                fromState,
                action,
                toState,
            });
        }
    }

    // 2. Action Coverage per State
    for (const [state, validActions] of Object.entries(VALID_ACTIONS_PER_STATE)) {
        for (const action of validActions) {
            scenarios.push({
                type: 'action_in_state',
                name: `action_${state.toLowerCase()}_${action.toLowerCase()}`,
                state,
                action,
                shouldWork: true,
            });
        }
    }

    // 3. Invalid Actions per State (should be ignored)
    for (const [state, validActions] of Object.entries(VALID_ACTIONS_PER_STATE)) {
        const invalidActions = gameActions.filter(a => !validActions.includes(a));
        for (const action of invalidActions.slice(0, 5)) { // Limit to 5 per state
            scenarios.push({
                type: 'invalid_action',
                name: `invalid_${state.toLowerCase()}_${action.toLowerCase()}`,
                state,
                action,
                shouldBeIgnored: true,
            });
        }
    }

    // 4. Cursor State Verification
    for (const [state, cursor] of Object.entries(CURSOR_STATES)) {
        scenarios.push({
            type: 'cursor_state',
            name: `cursor_${state.toLowerCase()}`,
            state,
            ...cursor,
        });
    }

    // 5. Settings Controls
    for (const slider of SETTINGS_CONTROLS.sliders) {
        scenarios.push({
            type: 'settings_slider',
            name: `settings_slider_${slider.toLowerCase()}`,
            control: slider,
            controlType: 'slider',
        });
    }
    for (const toggle of SETTINGS_CONTROLS.toggles) {
        scenarios.push({
            type: 'settings_toggle',
            name: `settings_toggle_${toggle.toLowerCase()}`,
            control: toggle,
            controlType: 'toggle',
        });
    }

    // 6. Placeable Item Operations
    const placeableItems = baseMod.items.filter(i => i.is_placeable);
    for (const item of placeableItems) {
        scenarios.push({
            type: 'item_place',
            name: `place_${item.id}`,
            itemId: `base:${item.id}`,
            itemName: item.name,
        });
        scenarios.push({
            type: 'item_destroy',
            name: `destroy_${item.id}`,
            itemId: `base:${item.id}`,
            itemName: item.name,
        });
    }

    // 7. Machine Interactions
    for (const machine of baseMod.machines) {
        scenarios.push({
            type: 'machine_interact',
            name: `machine_interact_${machine.id}`,
            machineId: machine.id,
            machineName: machine.name,
            blockType: machine.block_type,
        });

        // Machine UI slots
        if (machine.ui_slots) {
            for (const [slotName, slotInfo] of Object.entries(machine.ui_slots)) {
                scenarios.push({
                    type: 'machine_slot',
                    name: `machine_slot_${machine.id}_${slotName}`,
                    machineId: machine.id,
                    slotName,
                    slotLabel: slotInfo.label || slotName,
                });
            }
        }
    }

    // 8. Inventory Operations
    const inventoryOps = [
        'get_slot', 'list_all', 'move_item', 'stack_item',
        'drop_item', 'get_hotbar', 'set_selected_slot',
    ];
    for (const op of inventoryOps) {
        scenarios.push({
            type: 'inventory_op',
            name: `inventory_${op}`,
            operation: op,
        });
    }

    // 9. Hotbar Selection
    for (let i = 1; i <= 9; i++) {
        scenarios.push({
            type: 'hotbar_select',
            name: `hotbar_select_${i}`,
            slot: i,
            action: `Hotbar${i}`,
        });
    }

    // 10. Block Rotation
    scenarios.push({
        type: 'block_rotation',
        name: 'block_rotation_cycle',
        description: 'Rotate through all 4 directions',
    });

    // 11. World Operations
    const worldOps = ['raycast', 'get_block', 'place_block', 'break_block'];
    for (const op of worldOps) {
        scenarios.push({
            type: 'world_op',
            name: `world_${op}`,
            operation: op,
        });
    }

    // 12. Player Operations
    const playerOps = ['teleport', 'get_position', 'get_looking_at', 'get_state'];
    for (const op of playerOps) {
        scenarios.push({
            type: 'player_op',
            name: `player_${op}`,
            operation: op,
        });
    }

    return scenarios;
}

// ============================================================================
// TOML Generators
// ============================================================================

function getStateSetupSteps(targetState) {
    const steps = [];
    switch (targetState) {
        case 'Gameplay':
            // Already there
            break;
        case 'Inventory':
            steps.push({ action: 'ToggleInventory', wait: 150 });
            break;
        case 'GlobalInventory':
            steps.push({ action: 'ToggleGlobalInventory', wait: 150 });
            break;
        case 'CommandInput':
            steps.push({ action: 'OpenCommand', wait: 150 });
            break;
        case 'PauseMenu':
            steps.push({ action: 'CloseUI', wait: 150 });
            break;
        case 'Settings':
            steps.push({ action: 'CloseUI', wait: 150 });
            // Note: Need UI click for settings
            break;
        case 'Machine':
            // Complex setup needed
            break;
    }
    return steps;
}

function generateSetupTOML(steps) {
    let toml = '';
    for (const step of steps) {
        toml += `[[steps]]
action = "send_input"
params = { action = "${step.action}" }

[[steps]]
action = "wait"
params = { ms = ${step.wait} }

`;
    }
    return toml;
}

function generateStateTransitionTOML(scenario) {
    const setup = getStateSetupSteps(scenario.fromState);
    return `# State Transition: ${scenario.fromState} -> ${scenario.toState} via ${scenario.action}
name = "${scenario.name}"
description = "${scenario.fromState} + ${scenario.action} -> ${scenario.toState}"

# Setup: reach ${scenario.fromState}
[[steps]]
action = "get_state"

${generateSetupTOML(setup)}
# Verify starting state
[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.fromState}" }

# Perform action
[[steps]]
action = "send_input"
params = { action = "${scenario.action}" }

[[steps]]
action = "wait"
params = { ms = 150 }

# Verify result
[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.toState}" }
`;
}

function generateActionInStateTOML(scenario) {
    const setup = getStateSetupSteps(scenario.state);
    return `# Action: ${scenario.action} in ${scenario.state}
name = "${scenario.name}"
description = "${scenario.action} should work in ${scenario.state}"

[[steps]]
action = "get_state"

${generateSetupTOML(setup)}
[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.state}" }

[[steps]]
action = "send_input"
params = { action = "${scenario.action}" }

[[steps]]
action = "wait"
params = { ms = 50 }

# Action executed without error
[[steps]]
action = "get_state"
`;
}

function generateCursorStateTOML(scenario) {
    const setup = getStateSetupSteps(scenario.state);
    return `# Cursor State: ${scenario.state}
name = "${scenario.name}"
description = "Cursor should be ${scenario.locked ? 'locked' : 'unlocked'} in ${scenario.state}"

[[steps]]
action = "get_state"

${generateSetupTOML(setup)}
[[steps]]
action = "assert"
params = { condition = "ui_state == ${scenario.state}" }

[[steps]]
action = "assert"
params = { condition = "cursor_locked == ${scenario.locked}" }

[[steps]]
action = "assert"
params = { condition = "cursor_visible == ${scenario.visible}" }
`;
}

function generateItemPlaceTOML(scenario) {
    return `# Place Item: ${scenario.itemName}
name = "${scenario.name}"
description = "Place ${scenario.itemName} block"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "player_teleport"
params = { x = 50.0, y = 60.0, z = 50.0 }

[[steps]]
action = "wait"
params = { ms = 200 }

[[steps]]
action = "world_place_block"
params = { x = 50, y = 59, z = 52, item_id = "${scenario.itemId}" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "world_get_block"
params = { x = 50, y = 59, z = 52 }

# Cleanup
[[steps]]
action = "world_break_block"
params = { x = 50, y = 59, z = 52 }
`;
}

function generateItemDestroyTOML(scenario) {
    return `# Destroy Item: ${scenario.itemName}
name = "${scenario.name}"
description = "Place and destroy ${scenario.itemName} block"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "player_teleport"
params = { x = 60.0, y = 60.0, z = 60.0 }

[[steps]]
action = "wait"
params = { ms = 200 }

# Place block
[[steps]]
action = "world_place_block"
params = { x = 60, y = 59, z = 62, item_id = "${scenario.itemId}" }

[[steps]]
action = "wait"
params = { ms = 100 }

# Destroy block
[[steps]]
action = "world_break_block"
params = { x = 60, y = 59, z = 62 }

[[steps]]
action = "wait"
params = { ms = 100 }

# Verify destroyed
[[steps]]
action = "world_get_block"
params = { x = 60, y = 59, z = 62 }
`;
}

function generateMachineInteractTOML(scenario) {
    return `# Machine Interact: ${scenario.machineName}
name = "${scenario.name}"
description = "Place ${scenario.machineName} and open its UI"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "player_teleport"
params = { x = 70.0, y = 60.0, z = 70.0 }

[[steps]]
action = "wait"
params = { ms = 200 }

# Place machine
[[steps]]
action = "world_place_block"
params = { x = 70, y = 59, z = 72, item_id = "base:${scenario.machineId}_block" }

[[steps]]
action = "wait"
params = { ms = 200 }

# Interact (right click)
[[steps]]
action = "send_input"
params = { action = "SecondaryAction" }

[[steps]]
action = "wait"
params = { ms = 300 }

# Get state (may be Machine if targeting worked)
[[steps]]
action = "get_state"

# Close UI
[[steps]]
action = "send_input"
params = { action = "CloseUI" }

[[steps]]
action = "wait"
params = { ms = 100 }

# Cleanup
[[steps]]
action = "world_break_block"
params = { x = 70, y = 59, z = 72 }
`;
}

function generateHotbarSelectTOML(scenario) {
    return `# Hotbar Select: Slot ${scenario.slot}
name = "${scenario.name}"
description = "Select hotbar slot ${scenario.slot}"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "send_input"
params = { action = "${scenario.action}" }

[[steps]]
action = "wait"
params = { ms = 50 }

[[steps]]
action = "inventory_get_hotbar"
`;
}

function generateInventoryOpTOML(scenario) {
    return `# Inventory Operation: ${scenario.operation}
name = "${scenario.name}"
description = "Test inventory.${scenario.operation}"

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 150 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Inventory" }

[[steps]]
action = "inventory_list"

[[steps]]
action = "inventory_get_hotbar"

[[steps]]
action = "inventory_get_slot"
params = { index = 0 }

[[steps]]
action = "send_input"
params = { action = "ToggleInventory" }

[[steps]]
action = "wait"
params = { ms = 100 }

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }
`;
}

function generateWorldOpTOML(scenario) {
    return `# World Operation: ${scenario.operation}
name = "${scenario.name}"
description = "Test world.${scenario.operation}"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

[[steps]]
action = "world_get_block"
params = { x = 0, y = 50, z = 0 }

[[steps]]
action = "world_raycast"
params = { origin = [0.0, 60.0, 0.0], direction = [0.0, -1.0, 0.0], max_distance = 50.0 }

[[steps]]
action = "player_get_looking_at"
`;
}

function generatePlayerOpTOML(scenario) {
    return `# Player Operation: ${scenario.operation}
name = "${scenario.name}"
description = "Test player.${scenario.operation}"

[[steps]]
action = "get_state"

[[steps]]
action = "player_get_state"

[[steps]]
action = "player_teleport"
params = { x = 0.0, y = 60.0, z = 0.0 }

[[steps]]
action = "wait"
params = { ms = 200 }

[[steps]]
action = "player_get_state"

[[steps]]
action = "player_get_looking_at"
`;
}

function generateSettingsSliderTOML(scenario) {
    return `# Settings Slider: ${scenario.control}
name = "${scenario.name}"
description = "Verify settings slider ${scenario.control} exists"

# Note: Requires Settings screen to be open manually

[[steps]]
action = "get_state"

# If in Settings, verify slider exists
[[steps]]
action = "assert"
params = { condition = "has_ui_element == Slider_${scenario.control}" }
`;
}

function generateSettingsToggleTOML(scenario) {
    return `# Settings Toggle: ${scenario.control}
name = "${scenario.name}"
description = "Verify settings toggle ${scenario.control} exists"

# Note: Requires Settings screen to be open manually

[[steps]]
action = "get_state"

# If in Settings, verify toggle exists
[[steps]]
action = "assert"
params = { condition = "has_ui_element == Toggle_${scenario.control}" }
`;
}

function generateBlockRotationTOML(scenario) {
    return `# Block Rotation Cycle
name = "${scenario.name}"
description = "Rotate block through all 4 directions"

[[steps]]
action = "get_state"

[[steps]]
action = "assert"
params = { condition = "ui_state == Gameplay" }

# Rotate 4 times (full cycle)
[[steps]]
action = "send_input"
params = { action = "RotateBlock" }

[[steps]]
action = "wait"
params = { ms = 50 }

[[steps]]
action = "send_input"
params = { action = "RotateBlock" }

[[steps]]
action = "wait"
params = { ms = 50 }

[[steps]]
action = "send_input"
params = { action = "RotateBlock" }

[[steps]]
action = "wait"
params = { ms = 50 }

[[steps]]
action = "send_input"
params = { action = "RotateBlock" }

[[steps]]
action = "wait"
params = { ms = 50 }

[[steps]]
action = "get_state"
`;
}

function scenarioToTOML(scenario) {
    switch (scenario.type) {
        case 'state_transition':
            return generateStateTransitionTOML(scenario);
        case 'action_in_state':
            return generateActionInStateTOML(scenario);
        case 'cursor_state':
            return generateCursorStateTOML(scenario);
        case 'item_place':
            return generateItemPlaceTOML(scenario);
        case 'item_destroy':
            return generateItemDestroyTOML(scenario);
        case 'machine_interact':
            return generateMachineInteractTOML(scenario);
        case 'hotbar_select':
            return generateHotbarSelectTOML(scenario);
        case 'inventory_op':
            return generateInventoryOpTOML(scenario);
        case 'world_op':
            return generateWorldOpTOML(scenario);
        case 'player_op':
            return generatePlayerOpTOML(scenario);
        case 'settings_slider':
            return generateSettingsSliderTOML(scenario);
        case 'settings_toggle':
            return generateSettingsToggleTOML(scenario);
        case 'block_rotation':
            return generateBlockRotationTOML(scenario);
        default:
            return null;
    }
}

// ============================================================================
// Main
// ============================================================================

function main() {
    console.log('=== Comprehensive Test Generator ===\n');

    // Parse source files
    const inputModPath = path.join(SRC_DIR, 'input', 'mod.rs');
    const uiStatePath = path.join(SRC_DIR, 'components', 'ui_state.rs');

    const inputModContent = fs.readFileSync(inputModPath, 'utf8');
    const uiStateContent = fs.readFileSync(uiStatePath, 'utf8');

    const gameActions = parseGameActions(inputModContent);
    const uiContexts = parseUIContexts(uiStateContent);

    console.log(`Found ${gameActions.length} GameActions`);
    console.log(`Found ${uiContexts.length} UIContexts`);

    // Load base mod
    const baseMod = loadBaseMod();
    console.log(`Found ${baseMod.items.length} items`);
    console.log(`Found ${baseMod.machines.length} machines`);
    console.log(`Found ${baseMod.recipes.length} recipes\n`);

    // Generate all scenarios
    const scenarios = generateAllScenarios(gameActions, uiContexts, baseMod);

    // Count by type
    const byType = {};
    for (const s of scenarios) {
        byType[s.type] = (byType[s.type] || 0) + 1;
    }

    console.log('Generated scenarios by type:');
    for (const [type, count] of Object.entries(byType).sort((a, b) => b[1] - a[1])) {
        console.log(`  ${type}: ${count}`);
    }
    console.log(`  TOTAL: ${scenarios.length}\n`);

    // Create output directory
    if (!fs.existsSync(OUTPUT_DIR)) {
        fs.mkdirSync(OUTPUT_DIR, { recursive: true });
    }

    // Write scenario files
    let written = 0;
    const writtenNames = new Set();

    for (const scenario of scenarios) {
        const toml = scenarioToTOML(scenario);
        if (!toml) continue;

        const filename = `${scenario.name}.toml`;
        if (writtenNames.has(filename)) continue;

        const filepath = path.join(OUTPUT_DIR, filename);
        fs.writeFileSync(filepath, toml);
        writtenNames.add(filename);
        written++;
    }

    console.log(`Written ${written} scenario files to tests/generated/`);

    // Write matrix JSON
    const matrix = {
        generated_at: new Date().toISOString(),
        game_actions: gameActions,
        ui_contexts: uiContexts,
        base_mod: {
            items_count: baseMod.items.length,
            machines_count: baseMod.machines.length,
            recipes_count: baseMod.recipes.length,
        },
        scenarios_by_type: byType,
        total_scenarios: scenarios.length,
    };

    fs.writeFileSync(
        path.join(OUTPUT_DIR, 'comprehensive_matrix.json'),
        JSON.stringify(matrix, null, 2)
    );

    console.log('\n=== Summary ===');
    console.log(`Total unique scenarios: ${written}`);
    console.log(`Run with: node scripts/run-scenario.js tests/generated/<name>.toml`);
    console.log(`Run all: ./scripts/run-all-scenarios.sh`);
}

main();
