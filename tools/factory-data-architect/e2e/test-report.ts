/**
 * E2Eテストレポート生成ユーティリティ
 *
 * Playwrightテスト結果をゲームE2Eテストと同じ形式で出力
 */
import * as fs from 'fs';
import * as path from 'path';

interface TestResult {
  name: string;
  passed: boolean;
  message: string;
  duration?: number;
}

interface TestReport {
  title: string;
  timestamp: string;
  results: TestResult[];
  passed: number;
  failed: number;
  total: number;
}

/**
 * テスト結果をファイルに保存
 */
export function saveTestReport(report: TestReport, outputPath: string): void {
  let content = `# ${report.title}\n`;
  content += `Generated: ${report.timestamp}\n\n`;
  content += `## Summary\n`;
  content += `- Total: ${report.total}\n`;
  content += `- Passed: ${report.passed} ✅\n`;
  content += `- Failed: ${report.failed} ❌\n\n`;

  if (report.failed > 0) {
    content += `## Failed Tests\n`;
    for (const result of report.results.filter((r) => !r.passed)) {
      content += `### ❌ ${result.name}\n`;
      content += `- Message: ${result.message}\n\n`;
    }
  }

  content += `## All Results\n`;
  for (const result of report.results) {
    const icon = result.passed ? '✅' : '❌';
    const duration = result.duration ? ` (${result.duration}ms)` : '';
    content += `- ${icon} ${result.name}: ${result.message}${duration}\n`;
  }

  fs.writeFileSync(outputPath, content, 'utf-8');
}

/**
 * Playwright結果JSONからレポートを生成
 */
export function generateReportFromPlaywright(jsonPath: string, outputPath: string): void {
  if (!fs.existsSync(jsonPath)) {
    console.error(`JSON file not found: ${jsonPath}`);
    return;
  }

  const json = JSON.parse(fs.readFileSync(jsonPath, 'utf-8'));
  const results: TestResult[] = [];

  for (const suite of json.suites || []) {
    for (const spec of suite.specs || []) {
      for (const test of spec.tests || []) {
        const passed = test.status === 'expected' || test.status === 'passed';
        results.push({
          name: `${suite.title} > ${spec.title}`,
          passed,
          message: passed ? 'Test passed' : test.error?.message || 'Test failed',
          duration: test.results?.[0]?.duration,
        });
      }
    }
  }

  const report: TestReport = {
    title: 'Editor E2E Test Report',
    timestamp: new Date().toISOString().replace('T', ' ').slice(0, 19),
    results,
    passed: results.filter((r) => r.passed).length,
    failed: results.filter((r) => !r.passed).length,
    total: results.length,
  };

  saveTestReport(report, outputPath);
}

// CLI実行時
if (process.argv[2]) {
  const jsonPath = process.argv[2];
  const outputPath = process.argv[3] || 'e2e-screenshots/test_report.txt';
  generateReportFromPlaywright(jsonPath, outputPath);
}
