# 組立機 (Assembler) モデリングガイド

## 概要

| 項目 | 値 |
|------|-----|
| ID | `assembler` |
| 日本語名 | 組立機 |
| サイズ | 1ブロック (1m x 1m x 1m) |
| 機能 | 素材を組み合わせて製品を製造 |

## デザインコンセプト

- **自動組立装置**
- ロボットアーム風の機構
- 複数の入力口（主素材・副素材）
- 作業台/プラットフォーム
- テクノロジー感

## 構造

```
      [Arm]
       ╱╲           ← アーム: 中央上部（動く）
    ┌──┴──┐
副→ │     │ ←副     ← 側面: 副素材入力
    │[Plat]│        ← プラットフォーム: 作業台
    │form │
主→ │     │         ← 後面: 主素材入力
    └──┬──┘
       ↓            ← 前面: 製品出力
```

## パーツ詳細

| パーツ | 形状 | 推奨サイズ | 色 |
|--------|------|-----------|-----|
| Body | ボックス | 0.7 x 0.5 x 0.7 | 白灰色 #6a6a7a |
| Arm Base | 円柱 | 直径0.1 x 高さ0.15 | 金属色 #555555 |
| Arm | 細ボックス | 0.03 x 0.2 x 0.03 | オレンジ #ff8800 |
| Platform | 薄いボックス | 0.4 x 0.02 x 0.4 | 暗色 #333333 |
| Input Ports | 小ボックス x3 | 0.1 x 0.1 | 緑 #00aa00 |
| Output Port | 小ボックス | 0.12 x 0.1 | 赤 #aa0000 |
| Display | 薄いボックス | 0.15 x 0.1 | 青光 #0088ff |

## I/Oポート

| ポート | 位置 | 方向 |
|--------|------|------|
| 入力(主素材) | 後面 (Back) | 主素材入力 |
| 入力(副素材) | 左側 (Left) | 副素材入力 |
| 入力(副素材) | 右側 (Right) | 副素材入力 |
| 出力 | 前面 (Front) | 製品出力 |

## 状態表現

| 状態 | 表現 |
|------|------|
| 停止 | アーム静止、ディスプレイ暗い |
| 稼働中 | アーム動作、ディスプレイ点灯 |

## 参考デザイン設定 (JSON)

```json
{
  "bodyWidth": 0.7,
  "bodyHeight": 0.5,
  "bodyDepth": 0.7,
  "bodyColor": "#6a6a7a",
  "armBaseRadius": 0.05,
  "armBaseHeight": 0.15,
  "armColor": "#ff8800",
  "platformSize": 0.4,
  "platformColor": "#333333",
  "inputPortColor": "#00aa00",
  "outputPortColor": "#aa0000",
  "displayColor": "#0088ff"
}
```

## 検索キーワード

- `assembly machine robot arm`
- `factory assembler industrial`
- `Factorio assembler`
- `automated assembly line`
