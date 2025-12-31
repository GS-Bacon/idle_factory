import { test, expect, Page } from '@playwright/test';

// タイムアウト設定
test.setTimeout(60000);

// ゲームのキャンバス要素を取得してクリック（ポインターロック取得）
async function clickCanvas(page: Page) {
  // 常にキャンバスをクリック（オーバーレイがあっても透過してクリック）
  await page.click('canvas', { force: true });
  await page.waitForTimeout(500);
}

// キーを送信
async function sendKey(page: Page, key: string) {
  await page.keyboard.press(key);
  await page.waitForTimeout(200);
}

// マウスクリック（左=0, 右=2）
async function sendClick(page: Page, button: 'left' | 'right') {
  await page.mouse.click(400, 300, { button });
  await page.waitForTimeout(200);
}

// スクリーンショットを撮影
async function takeScreenshot(page: Page, name: string) {
  await page.screenshot({
    path: `screenshots/e2e/wasm_${name}.png`,
    fullPage: false
  });
}

// ゲーム起動を待つ
async function waitForGameStart(page: Page) {
  // ゲームのキャンバスが表示されるまで待つ
  await page.waitForSelector('canvas', { timeout: 30000 });
  // ゲームの初期化を待つ
  await page.waitForTimeout(5000);
}

// チュートリアルを閉じる
async function dismissTutorial(page: Page) {
  await sendKey(page, 'Space');
  await page.waitForTimeout(1000);
}

// ゲームの初期化（共通処理）
async function initGame(page: Page) {
  await page.goto('/');
  await waitForGameStart(page);
  await dismissTutorial(page);
  await clickCanvas(page);
  await page.waitForTimeout(500);
}

test.describe('ゲーム基本操作', () => {
  test('チュートリアル表示と閉じる', async ({ page }) => {
    await page.goto('/');
    await waitForGameStart(page);
    await takeScreenshot(page, '00_game_loaded');

    // チュートリアルが表示されていることを確認（スクリーンショットで目視確認）
    await takeScreenshot(page, '01_tutorial_shown');

    // スペースキーでチュートリアルを閉じる
    await dismissTutorial(page);
    await takeScreenshot(page, '01_tutorial_dismissed');
  });

  test('ポインターロック取得', async ({ page }) => {
    await page.goto('/');
    await waitForGameStart(page);
    await dismissTutorial(page);

    // キャンバスをクリックしてポインターロック取得
    await clickCanvas(page);
    await takeScreenshot(page, '02_pointer_locked');
  });
});

test.describe('ホットバー選択', () => {
  test('キー1-9でホットバースロット選択', async ({ page }) => {
    await initGame(page);

    for (let i = 1; i <= 9; i++) {
      await sendKey(page, `Digit${i}`);
      await takeScreenshot(page, `hotbar_slot_${i}`);
    }
  });
});

test.describe('ブロック設置', () => {
  test('採掘機を設置（スロット1）', async ({ page }) => {
    await initGame(page);

    // スロット1を選択（採掘機）
    await sendKey(page, 'Digit1');
    await takeScreenshot(page, 'miner_selected');

    // 右クリックで設置
    await sendClick(page, 'right');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'miner_placed');
  });

  test('コンベアを設置（スロット2）', async ({ page }) => {
    await initGame(page);

    // スロット2を選択（コンベア）
    await sendKey(page, 'Digit2');
    await takeScreenshot(page, 'conveyor_selected');

    // 右クリックで設置
    await sendClick(page, 'right');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'conveyor_placed');
  });

  test('粉砕機を設置（スロット3）', async ({ page }) => {
    await initGame(page);

    await sendKey(page, 'Digit3');
    await takeScreenshot(page, 'crusher_selected');

    await sendClick(page, 'right');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'crusher_placed');
  });

  test('精錬炉を設置（スロット4）', async ({ page }) => {
    await initGame(page);

    await sendKey(page, 'Digit4');
    await takeScreenshot(page, 'furnace_selected');

    await sendClick(page, 'right');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'furnace_placed');
  });
});

test.describe('ブロック破壊', () => {
  test('地形ブロックを破壊', async ({ page }) => {
    await initGame(page);

    // 左クリックで破壊
    await sendClick(page, 'left');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'terrain_broken');
  });
});

test.describe('UI表示', () => {
  test('Eキーでインベントリ開閉', async ({ page }) => {
    await initGame(page);

    // インベントリを開く
    await sendKey(page, 'KeyE');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'inventory_open');

    // インベントリを閉じる
    await sendKey(page, 'KeyE');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'inventory_closed');
  });

  test('Tキーでコマンド入力', async ({ page }) => {
    await initGame(page);

    // コマンド入力を開く
    await sendKey(page, 'KeyT');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'command_open');

    // ESCで閉じる
    await sendKey(page, 'Escape');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'command_closed');
  });

  test('F3キーでデバッグHUD', async ({ page }) => {
    await initGame(page);

    // デバッグHUDを表示
    await sendKey(page, 'F3');
    await page.waitForTimeout(300);
    await takeScreenshot(page, 'debug_hud_on');

    // デバッグHUDを非表示
    await sendKey(page, 'F3');
    await page.waitForTimeout(300);
    await takeScreenshot(page, 'debug_hud_off');
  });
});

test.describe('ゲームモード', () => {
  test('/creativeでクリエイティブモードに変更', async ({ page }) => {
    await initGame(page);

    // コマンド入力を開く
    await sendKey(page, 'KeyT');
    await page.waitForTimeout(300);

    // コマンドを入力
    await page.keyboard.type('/creative');
    await sendKey(page, 'Enter');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'creative_mode');
  });

  test('/survivalでサバイバルモードに変更', async ({ page }) => {
    await initGame(page);

    // まずクリエイティブモードに
    await sendKey(page, 'KeyT');
    await page.keyboard.type('/creative');
    await sendKey(page, 'Enter');
    await page.waitForTimeout(300);

    // クリックして再度ポインターロック
    await clickCanvas(page);

    // サバイバルモードに戻す
    await sendKey(page, 'KeyT');
    await page.keyboard.type('/survival');
    await sendKey(page, 'Enter');
    await page.waitForTimeout(500);
    await takeScreenshot(page, 'survival_mode');
  });
});

test.describe('コンベアシステム', () => {
  test('コンベアチェーンを設置', async ({ page }) => {
    await initGame(page);

    // クリエイティブモードに（無限アイテム）
    await sendKey(page, 'KeyT');
    await page.keyboard.type('/creative');
    await sendKey(page, 'Enter');
    await page.waitForTimeout(300);
    await clickCanvas(page);

    // コンベア選択
    await sendKey(page, 'Digit2');

    // 3つのコンベアを設置
    for (let i = 0; i < 3; i++) {
      await sendClick(page, 'right');
      await page.waitForTimeout(300);
      // 少し前に移動（Wキー）
      await page.keyboard.down('KeyW');
      await page.waitForTimeout(200);
      await page.keyboard.up('KeyW');
    }

    await takeScreenshot(page, 'conveyor_chain');
  });
});

test.describe('プレイヤー操作', () => {
  test('WASD移動', async ({ page }) => {
    await initGame(page);

    // 前進
    await page.keyboard.down('KeyW');
    await page.waitForTimeout(500);
    await page.keyboard.up('KeyW');
    await takeScreenshot(page, 'move_forward');

    // 後退
    await page.keyboard.down('KeyS');
    await page.waitForTimeout(500);
    await page.keyboard.up('KeyS');
    await takeScreenshot(page, 'move_backward');
  });

  test('インベントリ表示中は移動できない', async ({ page }) => {
    await initGame(page);

    // インベントリを開く
    await sendKey(page, 'KeyE');
    await page.waitForTimeout(300);

    // WASDを押しても移動しない（スクリーンショットで確認）
    await page.keyboard.down('KeyW');
    await page.waitForTimeout(300);
    await page.keyboard.up('KeyW');
    await takeScreenshot(page, 'no_move_during_inventory');

    // インベントリを閉じる
    await sendKey(page, 'KeyE');
  });
});
