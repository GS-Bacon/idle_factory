/**
 * E2E Visual Test - 統合ビジュアルテスト
 *
 * WASMゲームを起動し、各種操作を実行してスクリーンショットを撮影。
 * AIがスクリーンショットを確認して視覚的異常を検出する。
 *
 * Usage: node e2e-visual-test.js [--quick|--full]
 *   --quick: 基本テストのみ（デフォルト）
 *   --full: 全テスト実行
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const SCREENSHOT_DIR = '/home/bacon/idle_factory/screenshots/e2e';
const GAME_URL = 'http://localhost:8080';
const TIMEOUT = 60000;

// テスト結果
const results = {
  passed: [],
  failed: [],
  screenshots: [],
  consoleErrors: [],
  startTime: null,
  endTime: null
};

// ユーティリティ関数
async function ensureDir(dir) {
  if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
  }
}

async function cleanScreenshots() {
  if (fs.existsSync(SCREENSHOT_DIR)) {
    const files = fs.readdirSync(SCREENSHOT_DIR);
    files.forEach(f => fs.unlinkSync(path.join(SCREENSHOT_DIR, f)));
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
    await page.waitForTimeout(2000);
    console.log('Game loaded successfully');
    return true;
  } catch (e) {
    console.error('Game failed to load:', e.message);
    return false;
  }
}

async function clickToActivate(page) {
  await page.mouse.click(640, 360);
  await page.waitForTimeout(300);
}

// テストケース定義
const tests = {
  // 基本テスト（--quick）
  basic: [
    {
      name: 'initial_state',
      description: '初期状態 - ゲーム起動直後',
      run: async (page) => {
        await takeScreenshot(page, 'initial', 'ゲーム起動直後の状態');
      }
    },
    {
      name: 'activated',
      description: 'クリックでアクティベート',
      run: async (page) => {
        await clickToActivate(page);
        await takeScreenshot(page, 'activated', 'ポインターロック取得後');
      }
    },
    {
      name: 'inventory_open',
      description: 'Eキーでインベントリ開く',
      run: async (page) => {
        await page.keyboard.press('KeyE');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'inventory_open', 'インベントリUI表示');
      }
    },
    {
      name: 'inventory_close',
      description: 'ESCでインベントリ閉じる',
      run: async (page) => {
        await page.keyboard.press('Escape');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'inventory_close', 'インベントリUI閉じた後');
      }
    },
    {
      name: 'hotbar_selection',
      description: 'ホットバー選択（1-9キー）',
      run: async (page) => {
        await clickToActivate(page);
        for (let i = 1; i <= 9; i++) {
          await page.keyboard.press(`Digit${i}`);
          await page.waitForTimeout(100);
        }
        await takeScreenshot(page, 'hotbar', 'ホットバー選択後');
      }
    },
    {
      name: 'debug_hud',
      description: 'F3でデバッグHUD表示',
      run: async (page) => {
        await page.keyboard.press('F3');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'debug_hud', 'デバッグHUD表示中');
        // トグルオフ
        await page.keyboard.press('F3');
        await page.waitForTimeout(300);
      }
    }
  ],

  // 拡張テスト（--full）
  extended: [
    {
      name: 'command_input',
      description: 'Tキーでコマンド入力',
      run: async (page) => {
        await clickToActivate(page);
        await page.keyboard.press('KeyT');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'command_input', 'コマンド入力欄表示');
        // ESCでキャンセル
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
      }
    },
    {
      name: 'creative_mode',
      description: '/creativeでクリエイティブモード',
      run: async (page) => {
        await clickToActivate(page);
        // コマンド入力
        await page.keyboard.press('KeyT');
        await page.waitForTimeout(300);
        await page.keyboard.type('/creative');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'creative_mode', 'クリエイティブモード有効');
      }
    },
    {
      name: 'creative_inventory',
      description: 'クリエイティブインベントリ',
      run: async (page) => {
        await page.keyboard.press('KeyE');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'creative_inventory', 'クリエイティブインベントリUI');
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
      }
    },
    {
      name: 'movement',
      description: 'WASD移動',
      run: async (page) => {
        await clickToActivate(page);
        // 前進
        await page.keyboard.down('KeyW');
        await page.waitForTimeout(500);
        await page.keyboard.up('KeyW');
        // 左移動
        await page.keyboard.down('KeyA');
        await page.waitForTimeout(300);
        await page.keyboard.up('KeyA');
        await takeScreenshot(page, 'movement', '移動後の位置');
      }
    },
    {
      name: 'block_break',
      description: '左クリックでブロック破壊',
      run: async (page) => {
        await clickToActivate(page);
        // 下を向く
        await page.mouse.move(640, 500, { steps: 5 });
        await page.waitForTimeout(200);
        // 左クリック（破壊）
        await page.mouse.click(640, 500, { button: 'left' });
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'block_break', 'ブロック破壊後');
      }
    },
    {
      name: 'block_place',
      description: '右クリックでブロック設置',
      run: async (page) => {
        await page.keyboard.press('Digit1');
        await page.waitForTimeout(100);
        await page.mouse.click(640, 400, { button: 'right' });
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'block_place', 'ブロック設置後');
      }
    },
    {
      name: 'machine_ui',
      description: '機械UIテスト（精錬炉）',
      run: async (page) => {
        // 精錬炉の方を向いて右クリック
        await page.keyboard.down('KeyD');
        await page.waitForTimeout(300);
        await page.keyboard.up('KeyD');
        await page.mouse.click(640, 360, { button: 'right' });
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'machine_ui', '機械UI（もし開いていれば）');
        await page.keyboard.press('Escape');
        await page.waitForTimeout(300);
      }
    },
    {
      name: 'survival_mode',
      description: '/survivalでサバイバルモード',
      run: async (page) => {
        await clickToActivate(page);
        await page.keyboard.press('KeyT');
        await page.waitForTimeout(300);
        await page.keyboard.type('/survival');
        await page.keyboard.press('Enter');
        await page.waitForTimeout(500);
        await takeScreenshot(page, 'survival_mode', 'サバイバルモード');
      }
    },
    {
      name: 'final_state',
      description: '最終状態',
      run: async (page) => {
        await clickToActivate(page);
        await page.keyboard.press('F3');
        await page.waitForTimeout(300);
        await takeScreenshot(page, 'final', 'テスト完了後の状態（デバッグHUD表示）');
      }
    }
  ]
};

// メイン実行
async function main() {
  const args = process.argv.slice(2);
  const fullTest = args.includes('--full');

  console.log('='.repeat(60));
  console.log('E2E Visual Test');
  console.log(`Mode: ${fullTest ? 'FULL' : 'QUICK'}`);
  console.log('='.repeat(60));

  results.startTime = new Date();

  await ensureDir(SCREENSHOT_DIR);
  await cleanScreenshots();

  const browser = await chromium.launch({ headless: true });
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
    // ゲームロード
    console.log('\n[Setup] Navigating to game...');
    await page.goto(GAME_URL);

    if (!await waitForGameLoad(page)) {
      throw new Error('Game failed to load');
    }

    // 基本テスト実行
    console.log('\n[Basic Tests]');
    for (const test of tests.basic) {
      console.log(`\nRunning: ${test.name}`);
      try {
        await test.run(page);
        results.passed.push(test.name);
        console.log(`  [PASS] ${test.description}`);
      } catch (e) {
        results.failed.push({ name: test.name, error: e.message });
        console.log(`  [FAIL] ${test.description}: ${e.message}`);
        await takeScreenshot(page, `error_${test.name}`, `エラー: ${e.message}`);
      }
    }

    // 拡張テスト実行（--full時のみ）
    if (fullTest) {
      console.log('\n[Extended Tests]');
      for (const test of tests.extended) {
        console.log(`\nRunning: ${test.name}`);
        try {
          await test.run(page);
          results.passed.push(test.name);
          console.log(`  [PASS] ${test.description}`);
        } catch (e) {
          results.failed.push({ name: test.name, error: e.message });
          console.log(`  [FAIL] ${test.description}: ${e.message}`);
          await takeScreenshot(page, `error_${test.name}`, `エラー: ${e.message}`);
        }
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
    if (results.consoleErrors.length > 5) {
      console.log(`  ... and ${results.consoleErrors.length - 5} more`);
    }
  }

  if (results.failed.length > 0) {
    console.log('\nFailed Tests:');
    results.failed.forEach(f => console.log(`  - ${f.name}: ${f.error}`));
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

  // 終了コード
  process.exit(results.failed.length > 0 ? 1 : 0);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(1);
});
