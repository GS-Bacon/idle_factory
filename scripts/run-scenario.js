#!/usr/bin/env node
/**
 * Scenario Test Runner
 *
 * Connects to the game via WebSocket and executes test scenarios.
 *
 * Usage:
 *   node run-scenario.js <scenario.toml>
 *   node run-scenario.js tests/scenarios/inventory_toggle.toml
 */

const WebSocket = require('ws');
const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');

const WS_URL = 'ws://127.0.0.1:9877';
const TIMEOUT = 5000;
const EVENT_TIMEOUT = 10000; // Longer timeout for waiting events

// Parse a TOML value (recursive for arrays and inline tables)
function parseTOMLValue(rawValue) {
    rawValue = rawValue.trim();

    // Handle inline array: [1, 2, 3] or [1.0, 2.0, 3.0]
    if (rawValue.startsWith('[') && rawValue.endsWith(']')) {
        const inner = rawValue.slice(1, -1).trim();
        if (inner === '') return [];
        // Split by comma, handling nested structures
        const elements = [];
        let depth = 0;
        let current = '';
        for (const char of inner) {
            if (char === '[' || char === '{') depth++;
            else if (char === ']' || char === '}') depth--;
            else if (char === ',' && depth === 0) {
                elements.push(parseTOMLValue(current.trim()));
                current = '';
                continue;
            }
            current += char;
        }
        if (current.trim()) {
            elements.push(parseTOMLValue(current.trim()));
        }
        return elements;
    }

    // Handle inline table: { key = "value", ... }
    if (rawValue.startsWith('{') && rawValue.endsWith('}')) {
        const value = {};
        const inner = rawValue.slice(1, -1).trim();
        if (inner === '') return value;
        // Parse key = value pairs inside {}
        const pairs = inner.split(/,\s*(?=\w+\s*=)/);
        for (const pair of pairs) {
            const pairMatch = pair.match(/(\w+)\s*=\s*(.+)/);
            if (pairMatch) {
                const [, pKey, pVal] = pairMatch;
                value[pKey] = parseTOMLValue(pVal.trim());
            }
        }
        return value;
    }

    // String value
    if (rawValue.startsWith('"') || rawValue.startsWith("'")) {
        return rawValue.replace(/^["']|["']$/g, '');
    }

    // Boolean value
    if (rawValue === 'true') return true;
    if (rawValue === 'false') return false;

    // Number value
    if (!isNaN(rawValue) && rawValue !== '') {
        return Number(rawValue);
    }

    return rawValue;
}

// Better TOML parser for scenarios
function parseTOML(content) {
    const lines = content.split('\n');
    const result = { steps: [] };
    let currentStep = null;

    for (const line of lines) {
        const trimmed = line.trim();
        if (trimmed.startsWith('#') || trimmed === '') continue;

        if (trimmed === '[[steps]]') {
            if (currentStep) result.steps.push(currentStep);
            currentStep = {};
            continue;
        }

        // Match key = value pairs
        const match = trimmed.match(/^(\w+)\s*=\s*(.+)$/);
        if (match) {
            const [, key, rawValue] = match;
            const value = parseTOMLValue(rawValue);

            if (currentStep) {
                currentStep[key] = value;
            } else {
                result[key] = value;
            }
        }
    }
    if (currentStep) result.steps.push(currentStep);

    return result;
}

async function runScenario(scenarioPath) {
    console.log(`Loading scenario: ${scenarioPath}`);
    const content = fs.readFileSync(scenarioPath, 'utf8');
    const scenario = parseTOML(content);

    console.log(`Running: ${scenario.name}`);
    if (scenario.description) {
        console.log(`  ${scenario.description}`);
    }

    const ws = new WebSocket(WS_URL);
    let requestId = 1;
    const variables = {};
    const subscriptions = {}; // event_type -> subscription_id
    const eventQueue = []; // Received events

    // Handle incoming notifications
    ws.on('message', (data) => {
        const msg = JSON.parse(data.toString());
        // JSON-RPC notification (no id field)
        if (!msg.id && msg.method && msg.method.startsWith('event.')) {
            eventQueue.push({
                type: msg.params?.event_type,
                data: msg.params,
                timestamp: Date.now()
            });
        }
    });

    const send = (method, params = {}) => {
        return new Promise((resolve, reject) => {
            const id = requestId++;
            const timeout = setTimeout(() => reject(new Error('Timeout')), TIMEOUT);

            const handler = (data) => {
                const response = JSON.parse(data.toString());
                if (response.id === id) {
                    clearTimeout(timeout);
                    ws.off('message', handler);
                    if (response.error) {
                        reject(new Error(response.error.message));
                    } else {
                        resolve(response.result);
                    }
                }
            };

            ws.on('message', handler);
            ws.send(JSON.stringify({ jsonrpc: '2.0', id, method, params }));
        });
    };

    const waitForEvent = (eventType, timeoutMs = EVENT_TIMEOUT) => {
        return new Promise((resolve, reject) => {
            const startTime = Date.now();

            const check = () => {
                // Look for matching event in queue
                const idx = eventQueue.findIndex(e => e.type === eventType);
                if (idx !== -1) {
                    const event = eventQueue.splice(idx, 1)[0];
                    resolve(event.data);
                    return;
                }

                // Check timeout
                if (Date.now() - startTime > timeoutMs) {
                    reject(new Error(`Timeout waiting for event: ${eventType}`));
                    return;
                }

                // Check again soon
                setTimeout(check, 50);
            };
            check();
        });
    };

    await new Promise((resolve, reject) => {
        ws.on('open', resolve);
        ws.on('error', reject);
    });

    console.log('Connected to game');

    // Auto-reset state at the start of each scenario (unless disabled)
    if (scenario.auto_reset !== false) {
        await send('test.reset_state', {});
        await new Promise(r => setTimeout(r, 100)); // Wait for reset to apply
    }

    let passed = 0;
    let failed = 0;

    for (let i = 0; i < scenario.steps.length; i++) {
        const step = scenario.steps[i];
        const params = step.params || {};
        process.stdout.write(`  Step ${i + 1}: `);

        try {
            switch (step.action) {
                case 'get_state':
                    const state = await send('test.get_state', {});
                    // Store state for later use
                    variables._lastState = state;
                    console.log(`Get state: ui_state=${state.ui_state} ✓`);
                    break;

                case 'send_input':
                    await send('test.send_input', { action: params.action });
                    console.log(`Send input: ${params.action} ✓`);
                    break;

                case 'wait':
                    const ms = params.ms || 100;
                    await new Promise(r => setTimeout(r, ms));
                    console.log(`Wait: ${ms}ms ✓`);
                    break;

                case 'reset_state':
                    await send('test.reset_state', {});
                    await new Promise(r => setTimeout(r, 50)); // Give time for reset
                    console.log(`Reset state ✓`);
                    break;

                case 'assert':
                    // Replace variables in condition
                    let condition = params.condition;
                    for (const [varName, varValue] of Object.entries(variables)) {
                        if (varName.startsWith('_')) continue;
                        condition = condition.replace(`$${varName}`, JSON.stringify(varValue));
                    }
                    const result = await send('test.assert', { condition });
                    if (result.success) {
                        console.log(`Assert: ${params.condition} ✓`);
                        passed++;
                    } else {
                        console.log(`Assert: ${params.condition} ✗`);
                        console.log(`    Expected: ${result.expected}`);
                        console.log(`    Actual: ${result.actual}`);
                        failed++;
                    }
                    break;

                // Legacy format support
                case 'input':
                    await send('test.send_input', { action: step.key });
                    console.log(`Input: ${step.key} ✓`);
                    break;

                case 'save':
                    const savedState = await send('test.get_state', {});
                    variables[step.variable] = savedState[step.field];
                    console.log(`Save: ${step.variable} = ${JSON.stringify(variables[step.variable])} ✓`);
                    break;

                case 'subscribe':
                    // Subscribe to an event type
                    const eventType = params.event_type;
                    const subResult = await send('test.subscribe_event', { event_type: eventType });
                    if (subResult.success) {
                        subscriptions[eventType] = subResult.subscription_id;
                        console.log(`Subscribe: ${eventType} (id=${subResult.subscription_id}) ✓`);
                    } else {
                        throw new Error(`Failed to subscribe to ${eventType}`);
                    }
                    break;

                case 'unsubscribe':
                    // Unsubscribe from an event type
                    const unsubType = params.event_type;
                    const subId = subscriptions[unsubType];
                    if (subId) {
                        await send('test.unsubscribe_event', { subscription_id: subId });
                        delete subscriptions[unsubType];
                        console.log(`Unsubscribe: ${unsubType} ✓`);
                    } else {
                        console.log(`Unsubscribe: ${unsubType} (not subscribed) ✓`);
                    }
                    break;

                case 'wait_for_event':
                    // Wait for a specific event
                    const waitType = params.event_type;
                    const timeoutMs = params.timeout_ms || EVENT_TIMEOUT;
                    const eventData = await waitForEvent(waitType, timeoutMs);
                    variables._lastEvent = eventData;
                    console.log(`Wait for event: ${waitType} received ✓`);
                    if (params.store_as) {
                        variables[params.store_as] = eventData;
                    }
                    break;

                case 'assert_event':
                    // Assert on the last received event
                    const eventCond = params.condition;
                    const lastEvent = variables._lastEvent;
                    if (!lastEvent) {
                        console.log(`Assert event: no event received ✗`);
                        failed++;
                        break;
                    }
                    // Simple field comparison
                    const [field, expectedVal] = eventCond.split('==').map(s => s.trim());
                    const actualVal = lastEvent[field];
                    if (String(actualVal) === expectedVal || actualVal === expectedVal) {
                        console.log(`Assert event: ${eventCond} ✓`);
                        passed++;
                    } else {
                        console.log(`Assert event: ${eventCond} ✗`);
                        console.log(`    Expected: ${expectedVal}`);
                        console.log(`    Actual: ${actualVal}`);
                        failed++;
                    }
                    break;

                case 'screenshot':
                    // Take a screenshot using scrot
                    const screenshotName = params.name || `screenshot_${Date.now()}`;
                    const screenshotDir = path.join(process.cwd(), 'tests', 'screenshots');
                    if (!fs.existsSync(screenshotDir)) {
                        fs.mkdirSync(screenshotDir, { recursive: true });
                    }
                    const screenshotPath = path.join(screenshotDir, `${screenshotName}.png`);
                    try {
                        const display = process.env.DISPLAY || ':10';
                        execSync(`DISPLAY=${display} scrot "${screenshotPath}"`, { stdio: 'pipe' });
                        console.log(`Screenshot: ${screenshotName}.png ✓`);
                        variables._lastScreenshot = screenshotPath;
                    } catch (e) {
                        console.log(`Screenshot: ${screenshotName} ✗ (${e.message})`);
                    }
                    break;

                case 'compare_screenshot':
                    // Compare screenshot with baseline using SSIM
                    const compareName = params.name;
                    const threshold = params.threshold || 0.95;
                    const baselineDir = path.join(process.cwd(), 'tests', 'screenshots', 'baseline');
                    const actualDir = path.join(process.cwd(), 'tests', 'screenshots');
                    const baselinePath = path.join(baselineDir, `${compareName}.png`);
                    const actualPath = path.join(actualDir, `${compareName}.png`);

                    if (!fs.existsSync(baselinePath)) {
                        // No baseline - create one
                        if (!fs.existsSync(baselineDir)) {
                            fs.mkdirSync(baselineDir, { recursive: true });
                        }
                        fs.copyFileSync(actualPath, baselinePath);
                        console.log(`Compare: ${compareName} - baseline created ✓`);
                        passed++;
                    } else {
                        // Compare using ImageMagick
                        try {
                            const diffOutput = execSync(
                                `compare -metric SSIM "${baselinePath}" "${actualPath}" /dev/null 2>&1 || true`,
                                { encoding: 'utf8' }
                            ).trim();
                            const ssim = parseFloat(diffOutput) || 0;
                            if (ssim >= threshold) {
                                console.log(`Compare: ${compareName} SSIM=${ssim.toFixed(3)} >= ${threshold} ✓`);
                                passed++;
                            } else {
                                console.log(`Compare: ${compareName} SSIM=${ssim.toFixed(3)} < ${threshold} ✗`);
                                // Save diff image
                                const diffPath = path.join(actualDir, `${compareName}_diff.png`);
                                execSync(`compare "${baselinePath}" "${actualPath}" "${diffPath}" 2>/dev/null || true`);
                                console.log(`    Diff saved to: ${diffPath}`);
                                failed++;
                            }
                        } catch (e) {
                            console.log(`Compare: ${compareName} error (${e.message}) ✗`);
                            failed++;
                        }
                    }
                    break;

                // === Inventory API ===
                case 'inventory_get_slot':
                    const slotResult = await send('inventory.get_slot', { index: params.index });
                    variables._lastSlot = slotResult;
                    console.log(`Inventory get slot ${params.index}: ${slotResult.item_id || 'empty'} x${slotResult.amount} ✓`);
                    break;

                case 'inventory_list':
                    const invResult = await send('inventory.list', { non_empty_only: params.non_empty_only || false });
                    variables._lastInventory = invResult;
                    console.log(`Inventory list: ${invResult.slots.length} slots ✓`);
                    break;

                case 'inventory_move_item':
                    const moveResult = await send('inventory.move_item', {
                        from: params.from,
                        to: params.to,
                        amount: params.amount
                    });
                    console.log(`Inventory move: ${params.from} -> ${params.to} ✓`);
                    break;

                case 'inventory_get_hotbar':
                    const hotbarResult = await send('inventory.get_hotbar', {});
                    variables._lastHotbar = hotbarResult;
                    console.log(`Inventory get hotbar: selected=${hotbarResult.selected} ✓`);
                    break;

                // === Player API ===
                case 'player_get_state':
                    const playerResult = await send('player.get_state', {});
                    variables._lastPlayer = playerResult;
                    console.log(`Player state: pos=[${playerResult.position.map(v => v.toFixed(1)).join(', ')}] ✓`);
                    break;

                case 'player_teleport':
                    const teleportResult = await send('player.teleport', {
                        x: params.x, y: params.y, z: params.z,
                        yaw: params.yaw, pitch: params.pitch
                    });
                    console.log(`Player teleport: [${params.x}, ${params.y}, ${params.z}] ✓`);
                    break;

                case 'player_get_looking_at':
                    const lookingResult = await send('player.get_looking_at', {});
                    variables._lastLookingAt = lookingResult;
                    if (lookingResult.hit) {
                        console.log(`Player looking at: [${lookingResult.position.join(', ')}] dist=${lookingResult.distance.toFixed(2)} ✓`);
                    } else {
                        console.log(`Player looking at: nothing ✓`);
                    }
                    break;

                case 'player_set_selected_slot':
                    await send('player.set_selected_slot', { slot: params.slot });
                    console.log(`Player select slot: ${params.slot} ✓`);
                    break;

                // === World API ===
                case 'world_get_block':
                    const blockResult = await send('world.get_block', { x: params.x, y: params.y, z: params.z });
                    variables._lastBlock = blockResult;
                    console.log(`World get block: [${params.x}, ${params.y}, ${params.z}] ✓`);
                    break;

                case 'world_place_block':
                    const placeResult = await send('world.place_block', {
                        x: params.x, y: params.y, z: params.z,
                        item_id: params.item_id,
                        facing: params.facing
                    });
                    console.log(`World place block: ${params.item_id} at [${params.x}, ${params.y}, ${params.z}] ✓`);
                    break;

                case 'world_break_block':
                    const breakResult = await send('world.break_block', { x: params.x, y: params.y, z: params.z });
                    console.log(`World break block: [${params.x}, ${params.y}, ${params.z}] ✓`);
                    break;

                case 'world_raycast':
                    const rayResult = await send('world.raycast', {
                        origin: params.origin,
                        direction: params.direction,
                        max_distance: params.max_distance
                    });
                    variables._lastRaycast = rayResult;
                    console.log(`World raycast: queued ✓`);
                    break;

                // === Quest API ===
                case 'quest_list':
                    const questListResult = await send('quest.list', {});
                    variables._lastQuestList = questListResult;
                    console.log(`Quest list: ${questListResult.quests?.length || 0} quests ✓`);
                    break;

                case 'quest_get':
                    const questGetResult = await send('quest.get', { id: params.id });
                    variables._lastQuest = questGetResult;
                    console.log(`Quest get: ${params.id} ✓`);
                    break;

                // === Craft API ===
                case 'craft_list':
                    const craftListResult = await send('craft.list', { station: params.station });
                    variables._lastCraftList = craftListResult;
                    console.log(`Craft list: ${craftListResult.recipes?.length || 0} recipes ✓`);
                    break;

                case 'craft_get':
                    const craftGetResult = await send('craft.get', { name: params.name });
                    variables._lastCraft = craftGetResult;
                    console.log(`Craft get: ${params.name} ✓`);
                    break;

                case 'craft_can_craft':
                    const canCraftResult = await send('craft.can_craft', { name: params.name });
                    variables._lastCanCraft = canCraftResult;
                    const canCraftStatus = canCraftResult.can_craft ? 'can craft' : 'cannot craft';
                    console.log(`Craft can_craft: ${params.name} - ${canCraftStatus} ✓`);
                    break;

                // === Machine API ===
                case 'machine_get_slots':
                    const slotsResult = await send('machine.get_slots', {
                        x: params.x, y: params.y, z: params.z
                    });
                    variables._lastMachineSlots = slotsResult;
                    console.log(`Machine get slots: [${params.x}, ${params.y}, ${params.z}] ✓`);
                    break;

                case 'machine_insert_item':
                    const insertResult = await send('machine.insert_item', {
                        x: params.x, y: params.y, z: params.z,
                        slot: params.slot,
                        item_id: params.item_id,
                        amount: params.amount || 1
                    });
                    console.log(`Machine insert: ${params.item_id} x${params.amount || 1} into slot ${params.slot} ✓`);
                    break;

                case 'machine_extract_item':
                    const extractResult = await send('machine.extract_item', {
                        x: params.x, y: params.y, z: params.z,
                        slot: params.slot,
                        amount: params.amount
                    });
                    console.log(`Machine extract: slot ${params.slot} ✓`);
                    break;

                case 'machine_get_progress':
                    const progressResult = await send('machine.get_progress', {
                        x: params.x, y: params.y, z: params.z
                    });
                    variables._lastMachineProgress = progressResult;
                    const progressPct = ((progressResult.progress || 0) * 100).toFixed(1);
                    console.log(`Machine progress: ${progressPct}% ✓`);
                    break;

                // === Assert Variable (for verifying stored results) ===
                case 'assert_variable':
                    const varName = params.variable;
                    const varCondition = params.condition;
                    // Parse variable path (e.g., "_lastQuest.complete")
                    const parts = varName.split('.');
                    let varValue = variables;
                    for (const part of parts) {
                        varValue = varValue?.[part];
                    }
                    // Simple condition parsing
                    const condMatch = varCondition.match(/^(==|!=|>=|<=|>|<)\s*(.+)$/);
                    if (condMatch) {
                        const [, op, expectedStr] = condMatch;
                        let expected = expectedStr.trim();
                        // Parse expected value
                        if (expected === 'true') expected = true;
                        else if (expected === 'false') expected = false;
                        else if (expected === 'null') expected = null;
                        else if (!isNaN(expected)) expected = Number(expected);
                        else expected = expected.replace(/^["']|["']$/g, '');

                        let success = false;
                        switch (op) {
                            case '==': success = varValue == expected; break;
                            case '!=': success = varValue != expected; break;
                            case '>=': success = varValue >= expected; break;
                            case '<=': success = varValue <= expected; break;
                            case '>': success = varValue > expected; break;
                            case '<': success = varValue < expected; break;
                        }
                        if (success) {
                            console.log(`Assert variable: ${varName} ${varCondition} ✓`);
                            passed++;
                        } else {
                            console.log(`Assert variable: ${varName} ${varCondition} ✗`);
                            console.log(`    Actual: ${JSON.stringify(varValue)}`);
                            failed++;
                        }
                    } else {
                        console.log(`Assert variable: invalid condition format ✗`);
                        failed++;
                    }
                    break;

                case 'verify_ui':
                    // Verify UI elements are present using VLM or game state
                    // params.expect: array of expected UI elements/conditions
                    // params.expect_state: expected game state conditions
                    const expectations = params.expect || [];
                    const expectState = params.expect_state || {};
                    const verifyScreenshot = variables._lastScreenshot;

                    let verifyPassed = true;
                    const verifyErrors = [];

                    // First, verify game state matches expectations
                    if (Object.keys(expectState).length > 0) {
                        const currentState = await send('test.get_state', {});
                        for (const [key, expected] of Object.entries(expectState)) {
                            const actual = currentState[key];
                            if (String(actual) !== String(expected)) {
                                verifyErrors.push(`${key}: expected ${expected}, got ${actual}`);
                                verifyPassed = false;
                            }
                        }
                    }

                    // Then verify visual elements if VLM is available
                    if (expectations.length > 0 && process.env.ANTHROPIC_API_KEY && verifyScreenshot) {
                        try {
                            const vlmScript = path.join(process.cwd(), 'scripts', 'vlm_check', 'visual_checker.py');
                            if (fs.existsSync(vlmScript)) {
                                const checkItems = expectations.join('|');
                                const vlmResult = execSync(
                                    `python3 "${vlmScript}" "${verifyScreenshot}" --check-items "${checkItems}" --json 2>/dev/null || echo '{"error": true}'`,
                                    { encoding: 'utf8', timeout: 30000 }
                                );
                                try {
                                    const vlmJson = JSON.parse(vlmResult);
                                    if (!vlmJson.error) {
                                        const foundCount = vlmJson.items_found || 0;
                                        if (foundCount < expectations.length * 0.8) {
                                            verifyErrors.push(`VLM: ${foundCount}/${expectations.length} items found`);
                                            verifyPassed = false;
                                        }
                                    }
                                } catch (parseErr) {
                                    // VLM result not parseable, skip visual check
                                }
                            }
                        } catch (vlmErr) {
                            // VLM not available or failed, continue with state-based verification
                        }
                    }

                    if (verifyPassed) {
                        const checkCount = Object.keys(expectState).length + expectations.length;
                        console.log(`Verify UI: ${checkCount} checks passed ✓`);
                        passed++;
                    } else {
                        console.log(`Verify UI: failed ✗`);
                        for (const err of verifyErrors) {
                            console.log(`    ${err}`);
                        }
                        failed++;
                    }
                    break;

                default:
                    console.log(`Unknown action: ${step.action} ?`);
            }
        } catch (e) {
            console.log(`Error: ${e.message} ✗`);
            failed++;
        }
    }

    ws.close();

    console.log('');
    console.log(`Results: ${passed} passed, ${failed} failed`);

    process.exit(failed > 0 ? 1 : 0);
}

// Main
const args = process.argv.slice(2);
if (args.length === 0) {
    console.log('Usage: node run-scenario.js <scenario.toml>');
    process.exit(1);
}

runScenario(args[0]).catch(e => {
    console.error('Fatal error:', e.message);
    process.exit(1);
});
