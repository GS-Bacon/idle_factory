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

const WS_URL = 'ws://127.0.0.1:9877';
const TIMEOUT = 5000;

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
            let value;

            // Handle inline table: { key = "value", ... }
            if (rawValue.startsWith('{') && rawValue.endsWith('}')) {
                value = {};
                const inner = rawValue.slice(1, -1).trim();
                // Parse key = value pairs inside {}
                const pairs = inner.split(/,\s*(?=\w+\s*=)/);
                for (const pair of pairs) {
                    const pairMatch = pair.match(/(\w+)\s*=\s*(.+)/);
                    if (pairMatch) {
                        const [, pKey, pVal] = pairMatch;
                        // Remove quotes and parse value
                        const cleanVal = pVal.trim().replace(/^["']|["']$/g, '');
                        value[pKey] = isNaN(cleanVal) ? cleanVal : Number(cleanVal);
                    }
                }
            } else if (rawValue.startsWith('"') || rawValue.startsWith("'")) {
                // String value
                value = rawValue.replace(/^["']|["']$/g, '');
            } else if (rawValue === 'true' || rawValue === 'false') {
                // Boolean value
                value = rawValue === 'true';
            } else if (!isNaN(rawValue)) {
                // Number value
                value = Number(rawValue);
            } else {
                value = rawValue;
            }

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

    await new Promise((resolve, reject) => {
        ws.on('open', resolve);
        ws.on('error', reject);
    });

    console.log('Connected to game');

    // Reset game state to Gameplay before running scenario (unless skip_reset is set)
    if (!scenario.skip_reset) {
        try {
            await send('test.set_ui_state', { state: 'Gameplay' });
            // Wait for state to settle
            await new Promise(r => setTimeout(r, 100));
        } catch (e) {
            console.log(`Warning: Could not reset state: ${e.message}`);
        }
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

                case 'compare_position':
                    const current = await send('test.get_state', {});
                    const initial = variables._lastState;
                    if (!initial) {
                        console.log(`Compare position: no saved state (call get_state first) ✗`);
                        failed++;
                        break;
                    }
                    const tolerance = params.tolerance || 0.1;
                    const unchanged =
                        Math.abs(current.player_position[0] - initial.player_position[0]) < tolerance &&
                        Math.abs(current.player_position[1] - initial.player_position[1]) < tolerance &&
                        Math.abs(current.player_position[2] - initial.player_position[2]) < tolerance;
                    if (unchanged === (params.expect === 'unchanged')) {
                        console.log(`Position check: ${params.expect} ✓`);
                        passed++;
                    } else {
                        console.log(`Position check: expected ${params.expect}, got ${unchanged ? 'unchanged' : 'changed'} ✗`);
                        failed++;
                    }
                    break;

                case 'set_ui_state':
                    await send('test.set_ui_state', { state: params.state });
                    console.log(`Set UI state: ${params.state} ✓`);
                    break;

                case 'get_input_state':
                    const inputState = await send('test.get_input_state', {});
                    variables._lastInputState = inputState;
                    console.log(`Get input state: block=${inputState.allows_block_actions} move=${inputState.allows_movement} cam=${inputState.allows_camera} hotbar=${inputState.allows_hotbar} ✓`);
                    break;

                case 'get_events':
                    const eventsResult = await send('test.get_events', {});
                    variables._lastEvents = eventsResult.events;
                    console.log(`Get events: ${eventsResult.events.length} recorded ✓`);
                    break;

                case 'clear_events':
                    const clearResult = await send('test.clear_events', {});
                    console.log(`Clear events: ${clearResult.cleared} cleared ✓`);
                    break;

                case 'assert_input':
                    // Assert on input state flags
                    // params: { flag: "allows_block_actions", expect: true }
                    const is = await send('test.get_input_state', {});
                    const flagVal = is[params.flag];
                    // Convert params.expect to boolean if it's a string
                    const expectBool = typeof params.expect === 'string'
                        ? params.expect === 'true'
                        : params.expect;
                    if (flagVal === expectBool) {
                        console.log(`Assert input: ${params.flag} == ${expectBool} ✓`);
                        passed++;
                    } else {
                        console.log(`Assert input: ${params.flag} == ${expectBool} ✗`);
                        console.log(`    Expected: ${expectBool}`);
                        console.log(`    Actual: ${flagVal}`);
                        failed++;
                    }
                    break;

                case 'assert_events':
                    // Assert on event count or latest event type
                    // params: { count: 2 } or { latest_type: "BlockBroken" }
                    const evts = await send('test.get_events', {});
                    if (params.count !== undefined) {
                        if (evts.events.length === params.count) {
                            console.log(`Assert events: count == ${params.count} ✓`);
                            passed++;
                        } else {
                            console.log(`Assert events: count == ${params.count} ✗`);
                            console.log(`    Expected: ${params.count}`);
                            console.log(`    Actual: ${evts.events.length}`);
                            failed++;
                        }
                    } else if (params.latest_type) {
                        const latest = evts.events[evts.events.length - 1];
                        if (latest && latest.type === params.latest_type) {
                            console.log(`Assert events: latest == ${params.latest_type} ✓`);
                            passed++;
                        } else {
                            console.log(`Assert events: latest == ${params.latest_type} ✗`);
                            console.log(`    Expected: ${params.latest_type}`);
                            console.log(`    Actual: ${latest ? latest.type : 'none'}`);
                            failed++;
                        }
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

                case 'assert_ui':
                    const uiResult = await send('test.get_ui_elements', {});
                    const element = uiResult.elements.find(e => e.id === params.element_id);
                    const actualValue = element ? element[params.property] : undefined;
                    const expectValue = typeof params.expect === 'string'
                        ? params.expect === 'true'
                        : params.expect;

                    if (element && actualValue === expectValue) {
                        console.log(`  Assert UI: ${params.element_id}.${params.property} == ${expectValue} ✓`);
                        passed++;
                    } else {
                        console.log(`  Assert UI: ${params.element_id}.${params.property} == ${expectValue} ✗`);
                        console.log(`      Expected: ${expectValue}`);
                        console.log(`      Actual: ${actualValue !== undefined ? actualValue : 'element not found'}`);
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
