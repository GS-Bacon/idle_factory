import { defineConfig, devices } from '@playwright/test';

/**
 * Factory Data Architect E2E Test Configuration
 *
 * テストは以下のいずれかで実行:
 * - `npm run e2e` - ヘッドレスモード
 * - `npm run e2e:headed` - ブラウザ表示モード
 * - `DISPLAY=:10 npm run e2e:headed` - xRDP環境
 */
export default defineConfig({
  testDir: './e2e',
  fullyParallel: false, // Tauriアプリは並列実行しない
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Tauriアプリは1ワーカーのみ
  reporter: [
    ['html', { outputFolder: 'e2e-report' }],
    ['json', { outputFile: 'e2e-results.json' }],
    ['list']
  ],
  outputDir: 'e2e-screenshots',
  use: {
    baseURL: 'http://localhost:1420',
    trace: 'on-first-retry',
    screenshot: 'on',
    video: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Vite開発サーバーを自動起動
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:1420',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});
