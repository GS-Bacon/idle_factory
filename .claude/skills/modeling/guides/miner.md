# 採掘機 (Miner) モデリングガイド

## 概要

| 項目 | 値 |
|------|-----|
| ID | `miner` |
| 日本語名 | 採掘機 |
| サイズ | 1ブロック (1m x 1m x 1m) |
| 機能 | 地面から自動で資源を採掘 |

## デザインコンセプト

- **設置型のドリル装置**
- 下向きにドリルが伸びて地面を掘る
- 本体上部にモーター、前面に出力口
- 脚で安定して設置

## 構造

```
      [Motor]        ← 上部: モーター/エンジン
    ┌─────────┐
    │  Body   │      ← 本体: メインボックス
    │ [Light] │→[OUT]← 前面: インジケーター + 排出口
    └────┬────┘
    /    │    \      ← 脚: 4本（ピラミッド型）
   /     │     \
  ─     [Shaft]  ─   ← シャフト: 本体からドリルへ
        │
    [Drill Head]     ← ドリルヘッド: 八角形/螺旋
```

## パーツ詳細

| パーツ | 形状 | 推奨サイズ | 色 |
|--------|------|-----------|-----|
| Body | ボックス | 0.25 x 0.12 x 0.25 | 暗灰色 #4a4a4a |
| Motor | 八角柱 | 直径0.1 x 高さ0.06 | 黒 #333333 |
| Outlet | ボックス | 0.08 x 0.06 x 0.04 | 灰色 #666666 |
| Shaft | ボックス | 0.06 x 0.25 x 0.06 | 灰色 #666666 |
| Drill | 八角錐（5段） | 根元0.08 | 明灰色 #888888 |
| Legs | 細長ボックス x4 | 太さ0.03 | 暗灰色 #555555 |
| Indicator | 小ボックス | 0.025 | 緑 #00ff00 |

## I/Oポート

| ポート | 位置 | 方向 |
|--------|------|------|
| 出力 | 前面 (Front) | アイテム出力 |

## HTMLプレビュー

```
UIプレビュー/mining_drill_preview.html
```

## 確定デザイン設定 (JSON)

**最終版** - この設定でGLBエクスポート済み

```json
{
  "bodyWidth": 0.26,
  "bodyHeight": 0.18,
  "bodyDepth": 0.25,
  "bodyColor": "#4a4a4a",
  "shaftLength": 0.15,
  "shaftWidth": 0.08,
  "shaftColor": "#666666",
  "drillStyle": "cone",
  "drillLength": 0.17,
  "drillWidth": 0.12,
  "drillColor": "#888888",
  "legCount": 4,
  "legSpread": 0.2,
  "legThickness": 0.03,
  "legColor": "#555555",
  "indicatorColor": "#00ff00",
  "outletSize": 0.13,
  "outletColor": "#666666"
}
```

## エクスポート済みファイル

| ファイル | サイズ |
|----------|--------|
| `assets/models/machines/miner.glb` | 20KB |

## 再生成方法

上記JSONをClaudeに渡すだけで自動再生成:

```
このパラメータでモデリングして、モデル名は「miner」

{上記JSON}
```

または:

```bash
echo '{上記JSON}' | python3 scripts/generate-blender-model.py - miner
```

## 検索キーワード

- `mining drill machine`
- `ore extractor industrial`
- `Factorio mining drill`
- `Satisfactory miner`
