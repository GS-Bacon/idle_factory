#!/usr/bin/env node
/**
 * HTMLプレビューのThree.jsモデルをGLBとしてエクスポート
 *
 * 使い方:
 *   node scripts/export-html-model.js '{"bodyWidth": 0.26, ...}' output.glb
 */

const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

async function exportModel(configJson, outputPath) {
    const config = JSON.parse(configJson);

    const browser = await puppeteer.launch({
        headless: 'new',
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });

    const page = await browser.newPage();

    // コンソールをキャッチ（ページロード前に設定）
    page.on('console', msg => {
        console.log(`[Browser ${msg.type()}]:`, msg.text());
    });
    page.on('pageerror', err => {
        console.error('Page error:', err.message);
    });

    // HTMLファイルを読み込む
    const htmlPath = path.resolve(__dirname, '../UIプレビュー/mining_drill_preview.html');
    await page.goto(`file://${htmlPath}`, { waitUntil: 'networkidle0' });

    // 少し待ってからスクリプト実行
    await new Promise(r => setTimeout(r, 1000));

    // 設定を適用してモデルを再構築、GLBをエクスポート
    const glbData = await page.evaluate(async (config) => {
        console.log('drillGroup:', window.drillGroup);
        console.log('buildDrill:', typeof window.buildDrill);
        console.log('params:', window.params);

        // 設定を適用
        if (window.params) {
            Object.keys(config).forEach(key => {
                if (window.params.hasOwnProperty(key)) {
                    window.params[key] = config[key];
                }
            });
        }

        // モデル再構築
        if (typeof window.buildDrill === 'function') {
            window.buildDrill();
        }

        // 少し待つ
        await new Promise(r => setTimeout(r, 1000));

        console.log('drillGroup after rebuild:', window.drillGroup);

        if (!window.drillGroup) {
            throw new Error('drillGroup is null - JavaScript error in HTML?');
        }

        // GLTFExporterでエクスポート
        const { GLTFExporter } = await import('https://unpkg.com/three@0.160.0/examples/jsm/exporters/GLTFExporter.js');
        const exporter = new GLTFExporter();

        return new Promise((resolve, reject) => {
            exporter.parse(window.drillGroup, (result) => {
                // ArrayBufferをBase64に変換
                const bytes = new Uint8Array(result);
                let binary = '';
                for (let i = 0; i < bytes.byteLength; i++) {
                    binary += String.fromCharCode(bytes[i]);
                }
                resolve(btoa(binary));
            }, reject, { binary: true });
        });
    }, config);

    await browser.close();

    // Base64をデコードしてファイルに保存
    const buffer = Buffer.from(glbData, 'base64');
    fs.writeFileSync(outputPath, buffer);

    console.log(`Exported to ${outputPath} (${buffer.length} bytes)`);
}

// メイン
const args = process.argv.slice(2);
if (args.length < 2) {
    console.log('Usage: node export-html-model.js \'{"bodyWidth": 0.26, ...}\' output.glb');
    process.exit(1);
}

exportModel(args[0], args[1]).catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
