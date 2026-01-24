#!/usr/bin/env node
/**
 * コンベアプレビューHTMLからGLBをエクスポート
 *
 * 使い方:
 *   node scripts/export-conveyor-model.js [shape]
 *   shape: straight, corner_left, corner_right, t_junction, splitter (default: straight)
 */

const puppeteer = require('puppeteer');
const fs = require('fs');
const path = require('path');

async function exportConveyorModel(shape = 'straight') {
    const browser = await puppeteer.launch({
        headless: 'new',
        args: ['--no-sandbox', '--disable-setuid-sandbox']
    });

    const page = await browser.newPage();

    page.on('console', msg => {
        if (msg.type() === 'error') {
            console.log(`[Browser error]:`, msg.text());
        }
    });
    page.on('pageerror', err => {
        console.error('Page error:', err.message);
    });

    const htmlPath = path.resolve(__dirname, '../UIプレビュー/conveyor_preview.html');
    await page.goto(`file://${htmlPath}`, { waitUntil: 'networkidle0' });

    await new Promise(r => setTimeout(r, 2000));

    const glbData = await page.evaluate(async (shape) => {
        // 形状を設定
        if (window.params) {
            window.params.shape = shape;
        }

        // シェイプに応じてモデル再構築
        if (typeof window.setShape === 'function') {
            window.params.shape = shape;
            // buildModelを呼び出し
        }

        // buildModel関数を直接呼び出す（setShapeはボタン用なので）
        // グローバルスコープで実行される buildModel を取得
        const buildModel = window.buildModel;
        if (typeof buildModel === 'function') {
            buildModel();
        }

        await new Promise(r => setTimeout(r, 500));

        if (!window.modelGroup) {
            throw new Error('modelGroup is null - check JavaScript in HTML');
        }

        // GLTFExporterでエクスポート
        const { GLTFExporter } = await import('https://unpkg.com/three@0.160.0/examples/jsm/exporters/GLTFExporter.js');
        const exporter = new GLTFExporter();

        return new Promise((resolve, reject) => {
            exporter.parse(window.modelGroup, (result) => {
                const bytes = new Uint8Array(result);
                let binary = '';
                for (let i = 0; i < bytes.byteLength; i++) {
                    binary += String.fromCharCode(bytes[i]);
                }
                resolve(btoa(binary));
            }, reject, { binary: true });
        });
    }, shape);

    await browser.close();

    const outputPath = path.resolve(__dirname, `../assets/models/machines/conveyor/${shape}.glb`);
    const buffer = Buffer.from(glbData, 'base64');
    fs.writeFileSync(outputPath, buffer);

    console.log(`Exported to ${outputPath} (${buffer.length} bytes)`);
}

const shape = process.argv[2] || 'straight';
const validShapes = ['straight', 'corner_left', 'corner_right', 't_junction', 'splitter'];

if (!validShapes.includes(shape)) {
    console.log(`Invalid shape: ${shape}`);
    console.log(`Valid shapes: ${validShapes.join(', ')}`);
    process.exit(1);
}

exportConveyorModel(shape).catch(err => {
    console.error('Error:', err);
    process.exit(1);
});
