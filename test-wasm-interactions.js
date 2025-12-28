const { chromium } = require('playwright');
const fs = require('fs');

const SCREENSHOT_DIR = '/home/bacon/idle_factory/screenshots/verify';

async function waitForGameLoad(page) {
  console.log('Waiting for game to load...');
  await page.waitForFunction(() => {
    const body = document.body.innerText;
    return !body.includes('Downloading');
  }, { timeout: 60000 });
  await page.waitForTimeout(2000);
  console.log('Game loaded!');
}

async function takeScreenshot(page, name) {
  const path = `${SCREENSHOT_DIR}/${name}.png`;
  await page.screenshot({ path });
  console.log(`Screenshot: ${name}.png`);
  return path;
}

async function main() {
  // Clean old screenshots
  const files = fs.readdirSync(SCREENSHOT_DIR);
  files.filter(f => f.startsWith('test_')).forEach(f => {
    fs.unlinkSync(`${SCREENSHOT_DIR}/${f}`);
  });

  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 }
  });
  const page = await context.newPage();

  // Collect console errors
  const errors = [];
  page.on('console', msg => {
    if (msg.type() === 'error') {
      errors.push(msg.text());
    }
  });

  try {
    console.log('\n=== WASM Interaction Test ===\n');

    // Navigate and wait for load
    await page.goto('http://localhost:8080');
    await waitForGameLoad(page);

    // Test 1: Initial state
    console.log('\n[Test 1] Initial state');
    await takeScreenshot(page, 'test_01_initial');

    // Test 2: Click to activate (pointer lock)
    console.log('\n[Test 2] Click to activate game');
    await page.mouse.click(640, 360);
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_02_activated');

    // Test 3: Press C for Creative Mode
    console.log('\n[Test 3] Press C - Creative Mode');
    await page.keyboard.press('KeyC');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_03_creative_mode');

    // Test 4: Press E for Inventory UI (in creative mode shows creative inventory)
    console.log('\n[Test 4] Press E - Inventory/Creative UI');
    await page.keyboard.press('KeyE');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_04_inventory_ui');

    // Test 5: Press ESC to close UI
    console.log('\n[Test 5] Press ESC - Close UI');
    await page.keyboard.press('Escape');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_05_esc_close');

    // Test 6: Click to resume
    console.log('\n[Test 6] Click to resume');
    await page.mouse.click(640, 360);
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_06_resumed');

    // Test 7: Press number keys 1-5 for hotbar selection
    console.log('\n[Test 7] Press 1-5 - Hotbar selection');
    for (let i = 1; i <= 5; i++) {
      await page.keyboard.press(`Digit${i}`);
      await page.waitForTimeout(200);
    }
    await takeScreenshot(page, 'test_07_hotbar_selection');

    // Test 8: Press F3 for Debug HUD
    console.log('\n[Test 8] Press F3 - Debug HUD');
    await page.keyboard.press('F3');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_08_debug_hud');

    // Test 9: Move with WASD
    console.log('\n[Test 9] Move with WASD');
    await page.keyboard.press('KeyW');
    await page.waitForTimeout(300);
    await page.keyboard.press('KeyA');
    await page.waitForTimeout(300);
    await takeScreenshot(page, 'test_09_movement');

    // Test 10: Look around with mouse (simulated)
    console.log('\n[Test 10] Mouse look');
    await page.mouse.move(640, 360);
    await page.mouse.move(700, 300, { steps: 10 });
    await page.waitForTimeout(300);
    await takeScreenshot(page, 'test_10_mouse_look');

    // Test 11: Left click (block break attempt)
    console.log('\n[Test 11] Left click - Block break');
    await page.mouse.click(640, 400, { button: 'left' });
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_11_left_click');

    // Test 12: Right click (block place attempt)
    console.log('\n[Test 12] Right click - Block place');
    await page.keyboard.press('Digit1'); // Select first slot
    await page.waitForTimeout(200);
    await page.mouse.click(640, 400, { button: 'right' });
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_12_right_click');

    // Test 13: Press F3 again to toggle off
    console.log('\n[Test 13] Press F3 - Toggle debug off');
    await page.keyboard.press('F3');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_13_debug_off');

    // Test 14: Press C to toggle creative off
    console.log('\n[Test 14] Press C - Creative mode off');
    await page.keyboard.press('KeyC');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'test_14_creative_off');

    // Test 15: Final state
    console.log('\n[Test 15] Final state');
    await takeScreenshot(page, 'test_15_final');

    // Summary
    console.log('\n=== Test Summary ===');
    console.log(`Screenshots saved: ${SCREENSHOT_DIR}/test_*.png`);

    if (errors.length > 0) {
      console.log('\nConsole Errors:');
      errors.forEach(e => console.log(`  - ${e}`));
    } else {
      console.log('\nNo console errors detected');
    }

  } catch (e) {
    console.error('Test error:', e.message);
    await takeScreenshot(page, 'test_error');
  }

  await browser.close();

  // List screenshots
  console.log('\n=== Screenshots ===');
  const screenshots = fs.readdirSync(SCREENSHOT_DIR).filter(f => f.startsWith('test_'));
  screenshots.sort().forEach(f => console.log(f));
}

main().catch(console.error);
