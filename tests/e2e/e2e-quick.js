/**
 * E2E Quick Test - 高速なWASM基本動作確認
 *
 * タイムアウトを短くし、最小限のテストのみ実行
 */

const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');

const SCREENSHOT_DIR = '/home/bacon/idle_factory/screenshots/e2e_wasm';
const GAME_URL = process.env.GAME_URL || 'http://localhost:8080';
const TIMEOUT = 45000; // 45秒
const LOAD_TIMEOUT = 30000; // ロード待機30秒

async function main() {
  const args = process.argv.slice(2);
  const headed = args.includes('--headed');

  console.log('='.repeat(50));
  console.log('E2E Quick Test (WASM)');
  console.log('URL:', GAME_URL);
  console.log('='.repeat(50));

  // ディレクトリ作成
  if (!fs.existsSync(SCREENSHOT_DIR)) {
    fs.mkdirSync(SCREENSHOT_DIR, { recursive: true });
  }

  const browser = await chromium.launch({
    headless: !headed,
    args: ['--no-sandbox', '--disable-setuid-sandbox']
  });

  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 }
  });

  const page = await context.newPage();

  let passed = 0;
  let failed = 0;
  const results = [];

  try {
    // 1. ページ読み込み
    console.log('\n[1/5] ページ読み込み...');
    const startTime = Date.now();
    await page.goto(GAME_URL, { timeout: LOAD_TIMEOUT });

    // WASMロード待機（Loading/Downloading表示が消えるまで）
    try {
      await page.waitForFunction(() => {
        const body = document.body.innerText || '';
        return !body.includes('Downloading') && !body.includes('Loading');
      }, { timeout: LOAD_TIMEOUT });
      console.log(`  ✅ ロード完了 (${(Date.now() - startTime) / 1000}s)`);
      passed++;
      results.push({ name: 'load', status: 'pass' });
    } catch (e) {
      console.log(`  ❌ ロードタイムアウト`);
      failed++;
      results.push({ name: 'load', status: 'fail', error: e.message });
    }

    // 2. 初期スクリーンショット
    console.log('\n[2/5] 初期画面確認...');
    await page.waitForTimeout(2000);
    await page.screenshot({ path: path.join(SCREENSHOT_DIR, '01_initial.png') });
    console.log('  ✅ スクリーンショット取得');
    passed++;
    results.push({ name: 'screenshot', status: 'pass' });

    // 3. ゲームアクティベート
    console.log('\n[3/5] ゲームアクティベート...');
    await page.mouse.click(640, 360);
    await page.waitForTimeout(500);
    await page.screenshot({ path: path.join(SCREENSHOT_DIR, '02_activated.png') });
    console.log('  ✅ クリック実行');
    passed++;
    results.push({ name: 'activate', status: 'pass' });

    // 4. 基本操作テスト
    console.log('\n[4/5] 基本操作テスト...');
    try {
      // インベントリ開閉
      await page.keyboard.press('KeyE');
      await page.waitForTimeout(500);
      await page.screenshot({ path: path.join(SCREENSHOT_DIR, '03_inventory.png') });
      await page.keyboard.press('KeyE');
      await page.waitForTimeout(300);

      // デバッグHUD
      await page.keyboard.press('F3');
      await page.waitForTimeout(300);
      await page.screenshot({ path: path.join(SCREENSHOT_DIR, '04_debug.png') });

      console.log('  ✅ 基本操作成功');
      passed++;
      results.push({ name: 'basic_ops', status: 'pass' });
    } catch (e) {
      console.log(`  ❌ 基本操作失敗: ${e.message}`);
      failed++;
      results.push({ name: 'basic_ops', status: 'fail', error: e.message });
    }

    // 5. 最終スクリーンショット
    console.log('\n[5/5] 最終確認...');
    await page.screenshot({ path: path.join(SCREENSHOT_DIR, '05_final.png') });
    console.log('  ✅ 最終スクリーンショット取得');
    passed++;
    results.push({ name: 'final', status: 'pass' });

  } catch (e) {
    console.error('\n❌ Fatal error:', e.message);
    await page.screenshot({ path: path.join(SCREENSHOT_DIR, 'error.png') }).catch(() => {});
    failed++;
    results.push({ name: 'fatal', status: 'fail', error: e.message });
  }

  await browser.close();

  // 結果出力
  console.log('\n' + '='.repeat(50));
  console.log('RESULTS');
  console.log('='.repeat(50));
  console.log(`Passed: ${passed}`);
  console.log(`Failed: ${failed}`);
  console.log(`Total: ${passed + failed}`);
  console.log(`Screenshots: ${SCREENSHOT_DIR}`);

  // JSON結果ファイル
  const resultFile = path.join(SCREENSHOT_DIR, 'results.json');
  fs.writeFileSync(resultFile, JSON.stringify({
    passed,
    failed,
    total: passed + failed,
    results,
    timestamp: new Date().toISOString()
  }, null, 2));

  process.exit(failed > 0 ? 1 : 0);
}

main().catch(e => {
  console.error('Fatal:', e);
  process.exit(1);
});
