#!/usr/bin/env node
/**
 * WASM Console Log Capture Script
 *
 * Captures browser console logs from the WASM version of the game.
 * Used for debugging and bug investigation.
 *
 * Usage:
 *   node capture-wasm-logs.js [duration_seconds] [output_file]
 *
 * Examples:
 *   node capture-wasm-logs.js              # Capture for 30 seconds
 *   node capture-wasm-logs.js 60           # Capture for 60 seconds
 *   node capture-wasm-logs.js 60 my.log    # Capture to specific file
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const GAME_URL = 'http://10.13.1.1:8080';
const DEFAULT_DURATION = 30; // seconds
const LOGS_DIR = path.join(__dirname, 'logs');
const FREEZE_THRESHOLD_MS = 3000; // 3 seconds without requestAnimationFrame = freeze

async function captureWasmLogs(durationSeconds, outputFile) {
    // Ensure logs directory exists
    if (!fs.existsSync(LOGS_DIR)) {
        fs.mkdirSync(LOGS_DIR, { recursive: true });
    }

    const timestamp = new Date().toISOString().replace(/[:.]/g, '-');
    const logFile = outputFile || path.join(LOGS_DIR, `wasm_${timestamp}.log`);

    const logs = [];
    let lastFrameTime = Date.now();
    let freezeCount = 0;
    let panicDetected = false;

    console.log(`🎮 Starting WASM log capture`);
    console.log(`   URL: ${GAME_URL}`);
    console.log(`   Duration: ${durationSeconds} seconds`);
    console.log(`   Output: ${logFile}`);
    console.log('');

    const browser = await chromium.launch({
        headless: true,
        args: ['--no-sandbox', '--disable-gpu']
    });

    try {
        const context = await browser.newContext({
            viewport: { width: 1280, height: 720 }
        });
        const page = await context.newPage();

        // Capture all console messages
        page.on('console', msg => {
            const timestamp = new Date().toISOString();
            const type = msg.type().toUpperCase();
            const text = msg.text();

            const entry = `[${timestamp}] [${type}] ${text}`;
            logs.push(entry);

            // Detect WASM panic
            if (text.includes('panicked at') || text.includes('wasm-bindgen') && text.includes('error')) {
                panicDetected = true;
                console.log(`\n💀 WASM PANIC: ${text}`);
            }
            // Detect memory issues
            else if (text.includes('out of memory') || text.includes('OOM') || text.includes('memory access')) {
                console.log(`\n🧠 MEMORY ERROR: ${text}`);
            }
            // Also print to stdout for real-time monitoring
            else if (type === 'ERROR') {
                console.log(`❌ ${text}`);
            } else if (type === 'WARNING') {
                console.log(`⚠️  ${text}`);
            } else if (text.includes('BLOCK') || text.includes('MACHINE') || text.includes('QUEST')) {
                // Highlight game events
                console.log(`🎯 ${text}`);
            }
        });

        // Capture page errors
        page.on('pageerror', error => {
            const timestamp = new Date().toISOString();
            const entry = `[${timestamp}] [PAGEERROR] ${error.message}`;
            logs.push(entry);
            console.log(`💥 PAGE ERROR: ${error.message}`);
        });

        // Capture request failures
        page.on('requestfailed', request => {
            const timestamp = new Date().toISOString();
            const entry = `[${timestamp}] [REQFAIL] ${request.url()} - ${request.failure()?.errorText}`;
            logs.push(entry);
            console.log(`🔴 Request failed: ${request.url()}`);
        });

        console.log('📡 Connecting to game...');

        // Navigate to the game
        await page.goto(GAME_URL, {
            waitUntil: 'domcontentloaded',
            timeout: 30000
        });

        console.log('✅ Connected! Capturing logs...');
        console.log('');

        // Inject frame monitoring script
        await page.evaluate(() => {
            window.__frameTimestamps = [];
            window.__lastFrameTime = Date.now();
            window.__freezeEvents = [];

            const originalRAF = window.requestAnimationFrame;
            window.requestAnimationFrame = function(callback) {
                const now = Date.now();
                const delta = now - window.__lastFrameTime;

                // Detect freeze (>3 seconds between frames)
                if (delta > 3000) {
                    window.__freezeEvents.push({
                        timestamp: new Date().toISOString(),
                        duration: delta,
                        type: 'FREEZE'
                    });
                    console.warn(`FREEZE DETECTED: ${delta}ms between frames`);
                }

                window.__lastFrameTime = now;
                window.__frameTimestamps.push(now);

                // Keep only last 60 timestamps for FPS calculation
                if (window.__frameTimestamps.length > 60) {
                    window.__frameTimestamps.shift();
                }

                return originalRAF.call(window, callback);
            };
        });

        // Wait for the specified duration, checking for issues periodically
        const startTime = Date.now();
        const endTime = startTime + (durationSeconds * 1000);

        while (Date.now() < endTime) {
            const remaining = Math.ceil((endTime - Date.now()) / 1000);

            // Check for freeze events and FPS
            const stats = await page.evaluate(() => {
                const freezes = window.__freezeEvents || [];
                const timestamps = window.__frameTimestamps || [];
                let fps = 0;

                if (timestamps.length >= 2) {
                    const duration = (timestamps[timestamps.length - 1] - timestamps[0]) / 1000;
                    if (duration > 0) {
                        fps = Math.round((timestamps.length - 1) / duration);
                    }
                }

                // Clear freeze events after reporting
                window.__freezeEvents = [];

                return { freezes, fps, frameCount: timestamps.length };
            });

            // Log any freeze events
            for (const freeze of stats.freezes) {
                const entry = `[${freeze.timestamp}] [FREEZE] Duration: ${freeze.duration}ms`;
                logs.push(entry);
                freezeCount++;
                console.log(`\n🥶 FREEZE DETECTED: ${freeze.duration}ms`);
            }

            process.stdout.write(`\r⏱️  ${remaining}s remaining... (${logs.length} logs, FPS: ${stats.fps}, freezes: ${freezeCount})`);
            await new Promise(r => setTimeout(r, 1000));
        }

        // Final stats
        const finalStats = await page.evaluate(() => ({
            totalFrames: window.__frameTimestamps?.length || 0
        }));

        console.log('\n');

    } catch (error) {
        console.error(`\n❌ Error: ${error.message}`);
        logs.push(`[ERROR] Capture failed: ${error.message}`);
    } finally {
        await browser.close();
    }

    // Write logs to file
    const logContent = [
        `# WASM Log Capture`,
        `# URL: ${GAME_URL}`,
        `# Duration: ${durationSeconds} seconds`,
        `# Captured at: ${new Date().toISOString()}`,
        `# Total entries: ${logs.length}`,
        ``,
        ...logs
    ].join('\n');

    fs.writeFileSync(logFile, logContent);

    console.log(`✅ Logs saved to ${logFile}`);
    console.log(`   Total entries: ${logs.length}`);

    // Also create/update latest symlink
    const latestLink = path.join(LOGS_DIR, 'wasm_latest.log');
    try {
        if (fs.existsSync(latestLink)) {
            fs.unlinkSync(latestLink);
        }
        fs.symlinkSync(path.basename(logFile), latestLink);
    } catch (e) {
        // Symlink might fail on some systems, ignore
    }

    // Summary
    const errors = logs.filter(l => l.includes('[ERROR]') || l.includes('[PAGEERROR]')).length;
    const warnings = logs.filter(l => l.includes('[WARNING]')).length;
    const freezes = logs.filter(l => l.includes('[FREEZE]')).length;
    const gameEvents = logs.filter(l =>
        l.includes('BLOCK') || l.includes('MACHINE') || l.includes('QUEST')
    ).length;

    console.log('');
    console.log('═══════════════════════════════════════════════════════');
    console.log('📊 WASM Diagnostics Summary:');
    console.log('───────────────────────────────────────────────────────');
    console.log(`   Total logs:    ${logs.length}`);
    console.log(`   Errors:        ${errors}`);
    console.log(`   Warnings:      ${warnings}`);
    console.log(`   Freezes:       ${freezes}`);
    console.log(`   Game events:   ${gameEvents}`);
    console.log(`   Panic detected: ${panicDetected ? '❌ YES' : '✅ No'}`);
    console.log('═══════════════════════════════════════════════════════');

    if (panicDetected) {
        console.log('\n⚠️  WASM panic was detected! Check the log file for details.');
    }
    if (freezes > 0) {
        console.log(`\n⚠️  ${freezes} freeze event(s) detected! Game may have performance issues.`);
    }

    return { logFile, logs, errors, warnings, freezes, panicDetected };
}

// Main
const duration = parseInt(process.argv[2]) || DEFAULT_DURATION;
const outputFile = process.argv[3];

captureWasmLogs(duration, outputFile).catch(console.error);
