/**
 * Factory Data Architect E2E Tests
 *
 * エディタのE2Eテスト。以下の機能をテスト:
 * - 初期起動・セットアップ画面
 * - タブ切り替え
 * - アイテムエディタ
 * - レシピエディタ
 * - クエストエディタ
 * - マルチブロックエディタ
 * - バイオームエディタ
 * - サウンドエディタ
 */
import { test, expect, Page } from '@playwright/test';

// ヘルパー関数
async function takeScreenshot(page: Page, name: string) {
  await page.screenshot({ path: `e2e-screenshots/${name}.png`, fullPage: true });
}

async function selectTab(page: Page, tabName: string) {
  await page.click(`button:has-text("${tabName}")`);
  await page.waitForTimeout(300);
}

// ====================================
// セットアップ画面テスト
// ====================================
test.describe('Setup Screen', () => {
  test('should show setup screen on first launch', async ({ page }) => {
    await page.goto('/');

    // セットアップ画面またはメイン画面のいずれかが表示される
    const hasSetupScreen = await page.locator('.setup-screen').isVisible().catch(() => false);
    const hasMainScreen = await page.locator('.app-header').isVisible().catch(() => false);

    expect(hasSetupScreen || hasMainScreen).toBeTruthy();
    await takeScreenshot(page, '01_initial_screen');
  });
});

// ====================================
// タブナビゲーションテスト
// ====================================
test.describe('Tab Navigation', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // セットアップ画面をスキップ（既にパスが設定されている場合）
    const hasMainScreen = await page.locator('.app-header').isVisible().catch(() => false);
    if (!hasMainScreen) {
      test.skip();
    }
  });

  test('should switch to Items tab', async ({ page }) => {
    await selectTab(page, 'Items');
    await expect(page.locator('.items-tab-layout, .items-list-panel')).toBeVisible();
    await takeScreenshot(page, '02_items_tab');
  });

  test('should switch to Recipes tab', async ({ page }) => {
    await selectTab(page, 'Recipes');
    await expect(page.locator('.recipe-editor, .rf-canvas')).toBeVisible();
    await takeScreenshot(page, '03_recipes_tab');
  });

  test('should switch to Quests tab', async ({ page }) => {
    await selectTab(page, 'Quests');
    // クエストエディタの存在確認
    await page.waitForTimeout(500);
    await takeScreenshot(page, '04_quests_tab');
  });

  test('should switch to Multiblock tab', async ({ page }) => {
    await selectTab(page, 'Multiblock');
    await page.waitForTimeout(500);
    await takeScreenshot(page, '05_multiblock_tab');
  });

  test('should switch to Biomes tab', async ({ page }) => {
    await selectTab(page, 'Biomes');
    await page.waitForTimeout(500);
    await takeScreenshot(page, '06_biomes_tab');
  });

  test('should switch to Sounds tab', async ({ page }) => {
    await selectTab(page, 'Sounds');
    await page.waitForTimeout(500);
    await takeScreenshot(page, '07_sounds_tab');
  });
});

// ====================================
// アイテムエディタテスト
// ====================================
test.describe('Item Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    const hasMainScreen = await page.locator('.app-header').isVisible().catch(() => false);
    if (!hasMainScreen) {
      test.skip();
    }
    await selectTab(page, 'Items');
  });

  test('should show new item button', async ({ page }) => {
    const newButton = page.locator('button:has-text("新規"), button:has-text("+ 新規")');
    await expect(newButton).toBeVisible();
    await takeScreenshot(page, '10_items_new_button');
  });

  test('should open new item editor on click', async ({ page }) => {
    const newButton = page.locator('button:has-text("新規"), button:has-text("+ 新規")');
    await newButton.click();
    await page.waitForTimeout(300);

    // エディタパネルが表示される
    await expect(page.locator('.items-editor-panel, .item-editor')).toBeVisible();
    await takeScreenshot(page, '11_items_new_editor');
  });
});

// ====================================
// レシピエディタテスト
// ====================================
test.describe('Recipe Editor', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    const hasMainScreen = await page.locator('.app-header').isVisible().catch(() => false);
    if (!hasMainScreen) {
      test.skip();
    }
    await selectTab(page, 'Recipes');
  });

  test('should show recipe flow canvas', async ({ page }) => {
    // React Flowキャンバスが表示される
    await expect(page.locator('.react-flow, .rf-canvas, [data-testid="rf__wrapper"]')).toBeVisible();
    await takeScreenshot(page, '20_recipes_canvas');
  });
});

// ====================================
// フルテストシナリオ
// ====================================
test.describe('Full Test Scenario', () => {
  test('should complete full editor navigation', async ({ page }) => {
    await page.goto('/');

    // 初期画面
    await takeScreenshot(page, 'full_01_initial');

    // メイン画面が表示されるか確認
    const hasMainScreen = await page.locator('.app-header').isVisible().catch(() => false);
    if (!hasMainScreen) {
      // セットアップ画面の場合
      await takeScreenshot(page, 'full_02_setup_screen');
      console.log('Setup screen displayed - assets path not configured');
      return;
    }

    // 各タブを順番にテスト
    const tabs = ['Items', 'Recipes', 'Quests', 'Multiblock', 'Biomes', 'Sounds'];
    for (let i = 0; i < tabs.length; i++) {
      await selectTab(page, tabs[i]);
      await page.waitForTimeout(500);
      await takeScreenshot(page, `full_${String(i + 3).padStart(2, '0')}_${tabs[i].toLowerCase()}_tab`);
    }

    // Items タブに戻って新規アイテム作成
    await selectTab(page, 'Items');
    const newButton = page.locator('button:has-text("新規"), button:has-text("+ 新規")');
    if (await newButton.isVisible()) {
      await newButton.click();
      await page.waitForTimeout(300);
      await takeScreenshot(page, 'full_10_new_item_editor');
    }

    console.log('Full test scenario completed');
  });
});

// ====================================
// UIデザインパターンチェック
// ====================================
test.describe('UI Design Pattern Compliance', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    const hasMainScreen = await page.locator('.app-header').isVisible().catch(() => false);
    if (!hasMainScreen) {
      test.skip();
    }
  });

  test('U1: Information Hierarchy - tabs should be clearly visible', async ({ page }) => {
    // タブナビゲーションが見える
    const tabs = page.locator('.editor-tabs button');
    const tabCount = await tabs.count();
    expect(tabCount).toBeGreaterThan(0);
  });

  test('U2: Feedback - buttons should have hover state', async ({ page }) => {
    const button = page.locator('.editor-tabs button').first();
    await button.hover();
    await page.waitForTimeout(100);
    // ホバー状態でスタイルが変わることを視覚的に確認
    await takeScreenshot(page, 'design_u2_hover_state');
  });

  test('U3: Recovery - should have change folder button', async ({ page }) => {
    // Assets パス変更ボタンが存在
    const changeButton = page.locator('button:has-text("Change")');
    await expect(changeButton).toBeVisible();
  });

  test('U4: State Visibility - should show current assets path', async ({ page }) => {
    // 現在のアセットパスが表示される
    const pathDisplay = page.locator('.assets-path-display');
    await expect(pathDisplay).toBeVisible();
  });
});
