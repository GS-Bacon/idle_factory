const { chromium } = require('playwright');

(async () => {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext({ viewport: { width: 1280, height: 720 } });
  const page = await context.newPage();

  console.log('Navigating to game...');
  await page.goto('http://localhost:8080');

  // Wait for WASM to load (wait for canvas to appear and loading to finish)
  console.log('Waiting for game to load...');
  try {
    // Wait until "Downloading" text disappears (max 60 seconds)
    await page.waitForFunction(() => {
      const body = document.body.innerText;
      return !body.includes('Downloading');
    }, { timeout: 60000 });

    // Give it extra time to render
    await page.waitForTimeout(3000);

    console.log('Game loaded, taking screenshot...');
  } catch (e) {
    console.log('Timeout waiting for load, taking screenshot anyway...');
  }

  await page.screenshot({ path: '/home/bacon/idle_factory/screenshots/verify/wasm_game.png' });
  console.log('Screenshot saved to wasm_game.png');

  // Also capture console errors
  page.on('console', msg => {
    if (msg.type() === 'error') {
      console.log('Console error:', msg.text());
    }
  });

  await browser.close();
})();
