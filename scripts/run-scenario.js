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

// Simple TOML parser (basic, for scenarios only)
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

        const match = trimmed.match(/^(\w+)\s*=\s*(.+)$/);
        if (match) {
            const [, key, value] = match;
            const cleanValue = value.replace(/^["']|["']$/g, '');
            if (currentStep) {
                currentStep[key] = isNaN(cleanValue) ? cleanValue : Number(cleanValue);
            } else {
                result[key] = cleanValue;
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
        process.stdout.write(`  Step ${i + 1}: `);

        try {
            switch (step.action) {
                case 'input':
                    await send('test.send_input', { action: step.key });
                    console.log(`Input: ${step.key} ✓`);
                    break;

                case 'wait':
                    await new Promise(r => setTimeout(r, step.ms));
                    console.log(`Wait: ${step.ms}ms ✓`);
                    break;

                case 'assert':
                    // Replace variables in condition
                    let condition = step.condition;
                    for (const [varName, varValue] of Object.entries(variables)) {
                        condition = condition.replace(`$${varName}`, JSON.stringify(varValue));
                    }
                    const result = await send('test.assert', { condition });
                    if (result.success) {
                        console.log(`Assert: ${step.condition} ✓`);
                        passed++;
                    } else {
                        console.log(`Assert: ${step.condition} ✗`);
                        console.log(`    Expected: ${result.expected}`);
                        console.log(`    Actual: ${result.actual}`);
                        failed++;
                    }
                    break;

                case 'save':
                    const state = await send('test.get_state', {});
                    variables[step.variable] = state[step.field];
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
