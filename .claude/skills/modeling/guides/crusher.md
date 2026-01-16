# 粉砕機 (Crusher) モデリングガイド

## 概要

| 項目 | 値 |
|------|-----|
| ID | `crusher` |
| 日本語名 | 粉砕機 |
| サイズ | 1ブロック (1m x 1m x 1m) |
| 機能 | 鉱石を粉末に粉砕（2倍に増える） |

## デザインコンセプト

- **工業用粉砕装置**
- 投入ホッパー（上部）
- 粉砕ローラー/ジョー（内部）
- モーター（側面）
- 排出口（前面下部）

## 構造

```
    ╱──────╲
   ╱ Hopper ╲     ← ホッパー: 投入口（上部）
  ┌──────────┐
  │ ◎    ◎  │     ← ローラー: 粉砕部（内部）
  │ [Body]  │
  │  Motor→ │     ← モーター: 側面
  └────┬────┘
       ↓          ← 排出口: 前面下部
```

## パーツ詳細

| パーツ | 形状 | 推奨サイズ | 色 |
|--------|------|-----------|-----|
| Body | ボックス | 0.7 x 0.6 x 0.7 | 暗青灰色 #3a4a5a |
| Hopper | 逆四角錐台 | 上0.5 下0.3 高0.2 | 金属色 #555555 |
| Motor | 円柱 | 直径0.15 x 奥行0.1 | 黒 #2a2a2a |
| Outlet | ボックス（凹み） | 0.2 x 0.15 | 暗灰色 #333333 |
| Frame | ボックス | 0.8 x 0.1 x 0.8 | 金属色 #4a4a4a |
| Indicator | 小ライト | 0.03 | 青 #0066ff |

## I/Oポート

| ポート | 位置 | 方向 |
|--------|------|------|
| 入力 | 後面 (Back) | 鉱石入力 |
| 出力 | 前面 (Front) | 粉末出力 |

## 状態表現

| 状態 | 表現 |
|------|------|
| 停止 | インジケーター青暗い |
| 稼働中 | インジケーター青く点灯、振動エフェクト |

## 参考デザイン設定 (JSON)

```json
{
  "bodyWidth": 0.7,
  "bodyHeight": 0.6,
  "bodyDepth": 0.7,
  "bodyColor": "#3a4a5a",
  "hopperTopWidth": 0.5,
  "hopperBottomWidth": 0.3,
  "hopperHeight": 0.2,
  "hopperColor": "#555555",
  "motorRadius": 0.075,
  "motorDepth": 0.1,
  "motorColor": "#2a2a2a",
  "frameHeight": 0.1,
  "frameColor": "#4a4a4a",
  "indicatorColor": "#0066ff"
}
```

## 検索キーワード

- `industrial crusher machine`
- `jaw crusher 3D`
- `ore crusher factory`
- `rock crusher low poly`
