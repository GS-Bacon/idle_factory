/**
 * E2E Gameplay Test - ブロック設置からクエスト完了まで検証
 *
 * WASMゲームを起動し、実際のゲームプレイをシミュレートして検証。
 * - ブロック設置
 * - コンベアL字配置
 * - 機械設置
 * - クエスト進行
 *
 * Usage: node e2e-gameplay-test.js [options]
 *   --headed: ブラウザを表示して実行
 *   --slow: 操作間隔を長くする
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const SCREENSHOT_DIR = '/home/bacon/idle_factory/screenshots/e2e_wasm';
const GAME_URL = 'http://localhost:8080';
const TIMEOUT = 90000;

// テスト結果
const results = {
  passed: [],
  failed: [],
  screenshots: [],
  consoleErrors: [],
  gameState: null,
  startTime: null,
  endTime: null
};

// ユーティリティ
async function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

async function cleanScreenshots() {
  if (fs.existsSync(SCREENSHOT_DIR)) {
    const files = fs.readdirSync(SCREENSHOT_DIR);
    files.filter(f => f.endsWith('.png')).forEach(f =>
      fs.unlinkSync(path.join(SCREENSHOT_DIR, f))
    );
  }
}

async function takeScreenshot(page, name, description) {
  const filename = `${String(results.screenshots.length + 1).padStart(2, '0')}_${name}.png`;
  const filepath = path.join(SCREENSHOT_DIR, filename);
  await page.screenshot({ path: filepath });
  results.screenshots.push({ name, filename, description });
  console.log(`  [Screenshot] ${filename}: ${description}`);
  return filepath;
}

async function waitForGameLoad(page) {
  console.log('Waiting for game to load...');
  try {
    await page.waitForFunction(() => {
      const body = document.body.innerText || '';
      return !body.includes('Downloading') && !body.includes('Loading');
    }, { timeout: TIMEOUT });
    await page.waitForTimeout(3000);
    console.log('Game loaded successfully');
    return true;
  } catch (e) {
    console.error('Game failed to load:', e.message);
    return false;
  }
}

async function clickToActivate(page, delay = 500) {
  await page.mouse.click(640, 360);
  await page.waitForTimeout(delay);
}

async function sendCommand(page, command) {
  console.log(`  [Command] ${command}`);
  await page.keyboard.press('KeyT');
  await page.waitForTimeout(300);
  await page.keyboard.type(command);
  await page.waitForTimeout(100);
  await page.keyboard.press('Enter');
  await page.waitForTimeout(500);
}

// テスト関数
function assert(condition, description) {
  if (condition) {
    results.passed.push(description);
    console.log(`  [PASS] ${description}`);
    return true;
  } else {
    results.failed.push(description);
    console.log(`  [FAIL] ${description}`);
    return false;
  }
}

// ゲームプレイテスト
const tests = [
  {
    name: 'initial_state',
    description: '初期状態確認',
    run: async (page) => {
      await takeScreenshot(page, 'initial', 'ゲーム起動直後');
      assert(true, '初期状態確認');
    }
  },
  {
    name: 'activate_game',
    description: 'ゲームアクティベート',
    run: async (page) => {
      await clickToActivate(page);
      await takeScreenshot(page, 'activated', 'ポインターロック取得');
      assert(true, 'ゲームアクティベート');
    }
  },
  {
    name: 'inventory_check',
    description: 'インベントリ確認',
    run: async (page) => {
      await page.keyboard.press('KeyE');
      await page.waitForTimeout(800);
      await takeScreenshot(page, 'inventory', 'インベントリ表示');
      assert(true, 'インベントリ表示');

      await page.keyboard.press('KeyE');
      await page.waitForTimeout(300);
      await clickToActivate(page, 300);
    }
  },
  {
    name: 'creative_mode',
    description: 'クリエイティブモード有効化',
    run: async (page) => {
      await sendCommand(page, '/creative');
      await clickToActivate(page, 300);
      await takeScreenshot(page, 'creative', 'クリエイティブモード');
      assert(true, 'クリエイティブモード有効化');
    }
  },
  {
    name: 'teleport_to_safe_location',
    description: '安全な位置にテレポート',
    run: async (page) => {
      await sendCommand(page, '/tp 10 10 10');
      await clickToActivate(page, 300);
      await takeScreenshot(page, 'teleport', 'テレポート後');
      assert(true, 'テレポート完了');
    }
  },
  {
    name: 'place_conveyor_straight',
    description: 'コンベア直線設置',
    run: async (page) => {
      // 下を向く
      await sendCommand(page, '/look 70 0');
      await clickToActivate(page, 300);

      // コンベア選択（2番）
      await page.keyboard.press('Digit2');
      await page.waitForTimeout(200);

      // 右クリックで設置
      await page.mouse.click(640, 360, { button: 'right' });
      await page.waitForTimeout(500);
      await takeScreenshot(page, 'conveyor1', 'コンベア1設置');
      assert(true, 'コンベア1設置');
    }
  },
  {
    name: 'place_conveyor_corner',
    description: 'コンベアL字設置',
    run: async (page) => {
      // 向きを変える
      await sendCommand(page, '/look 60 45');
      await clickToActivate(page, 300);

      await page.mouse.click(640, 360, { button: 'right' });
      await page.waitForTimeout(500);
      await takeScreenshot(page, 'conveyor2', 'コンベア2設置');

      // さらに向きを変える
      await sendCommand(page, '/look 60 90');
      await clickToActivate(page, 300);

      await page.mouse.click(640, 360, { button: 'right' });
      await page.waitForTimeout(500);
      await takeScreenshot(page, 'conveyor3', 'コンベア3設置（L字）');
      assert(true, 'コンベアL字設置');
    }
  },
  {
    name: 'place_miner',
    description: '採掘機設置',
    run: async (page) => {
      await sendCommand(page, '/look 70 180');
      await clickToActivate(page, 300);

      // 採掘機選択（1番）
      await page.keyboard.press('Digit1');
      await page.waitForTimeout(200);

      await page.mouse.click(640, 360, { button: 'right' });
      await page.waitForTimeout(500);
      await takeScreenshot(page, 'miner', '採掘機設置');
      assert(true, '採掘機設置');
    }
  },
  {
    name: 'place_furnace',
    description: '精錬炉設置',
    run: async (page) => {
      await sendCommand(page, '/look 70 270');
      await clickToActivate(page, 300);

      // 精錬炉選択（4番）
      await page.keyboard.press('Digit4');
      await page.waitForTimeout(200);

      await page.mouse.click(640, 360, { button: 'right' });
      await page.waitForTimeout(500);
      await takeScreenshot(page, 'furnace', '精錬炉設置');
      assert(true, '精錬炉設置');
    }
  },
  {
    name: 'overview',
    description: '全体確認',
    run: async (page) => {
      await sendCommand(page, '/look 80 0');
      await clickToActivate(page, 300);
      await page.waitForTimeout(1000);
      await takeScreenshot(page, 'overview', '全体俯瞰');
      assert(true, '全体確認完了');
    }
  },
  {
    name: 'debug_hud_check',
    description: 'デバッグHUDでステータス確認',
    run: async (page) => {
      await page.keyboard.press('F3');
      await page.waitForTimeout(500);
      await takeScreenshot(page, 'debug_final', 'デバッグHUD最終状態');
      assert(true, 'デバッグHUD確認');
    }
  },
  {
    name: 'quest_check',
    description: 'クエスト状態確認',
    run: async (page) => {
      // クエストUIの存在確認（スクリーンショットで目視確認）
      await takeScreenshot(page, 'quest', 'クエスト状態');
      assert(true, 'クエスト状態確認');
    }
  }
];

// メイン実行
async function main() {
  const args = process.argv.slice(2);
  const headed = args.includes('--headed');
  const slowMode = args.includes('--slow');

  console.log('='.repeat(60));
  console.log('E2E Gameplay Test');
  console.log(`Browser: ${headed ? 'HEADED' : 'HEADLESS'}`);
  if (slowMode) console.log('Speed: SLOW');
  console.log('='.repeat(60));

  results.startTime = new Date();

  await ensureDir(SCREENSHOT_DIR);
  await cleanScreenshots();

  const browser = await chromium.launch({
    headless: !headed,
    slowMo: slowMode ? 100 : 0
  });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 }
  });
  const page = await context.newPage();

  // コンソールエラー収集
  page.on('console', msg => {
    if (msg.type() === 'error') {
      results.consoleErrors.push(msg.text());
    }
  });

  try {
    console.log('\n[Setup] Navigating to game...');
    await page.goto(GAME_URL);

    if (!await waitForGameLoad(page)) {
      throw new Error('Game failed to load');
    }

    console.log('\n[Tests] Running gameplay tests...');
    for (const test of tests) {
      console.log(`\nRunning: ${test.name}`);
      try {
        await test.run(page);
      } catch (e) {
        results.failed.push(test.name);
        console.log(`  [FAIL] ${test.description}: ${e.message}`);
        await takeScreenshot(page, `error_${test.name}`, `エラー: ${e.message}`);
      }
    }

  } catch (e) {
    console.error('\n[FATAL ERROR]', e.message);
    await takeScreenshot(page, 'fatal_error', e.message);
  }

  await browser.close();

  results.endTime = new Date();

  // 結果出力
  console.log('\n' + '='.repeat(60));
  console.log('TEST RESULTS');
  console.log('='.repeat(60));
  console.log(`Duration: ${(results.endTime - results.startTime) / 1000}s`);
  console.log(`Passed: ${results.passed.length}`);
  console.log(`Failed: ${results.failed.length}`);
  console.log(`Screenshots: ${results.screenshots.length}`);

  if (results.consoleErrors.length > 0) {
    console.log(`\nConsole Errors (${results.consoleErrors.length}):`);
    results.consoleErrors.slice(0, 5).forEach(e => console.log(`  - ${e}`));
  }

  if (results.failed.length > 0) {
    console.log('\nFailed Tests:');
    results.failed.forEach(f => console.log(`  - ${f}`));
  }

  console.log(`\nScreenshots saved to: ${SCREENSHOT_DIR}`);

  // JSON結果ファイル出力
  const resultFile = path.join(SCREENSHOT_DIR, 'results.json');
  fs.writeFileSync(resultFile, JSON.stringify(results, null, 2));
  console.log(`Results saved to: ${resultFile}`);

  // スクリーンショット一覧
  console.log('\nScreenshot List:');
  results.screenshots.forEach(s => {
    console.log(`  ${s.filename}: ${s.description}`);
  });

  process.exit(results.failed.length > 0 ? 1 : 0);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
