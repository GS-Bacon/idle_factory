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
