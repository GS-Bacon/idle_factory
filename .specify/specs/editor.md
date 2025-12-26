# エディタ連携仕様

## 概要

Factory Data Architect - ゲームデータを視覚的に編集するTauriベースのデスクトップアプリ。

## アーキテクチャ

```
┌─────────────────┐      ┌─────────────────┐
│  Factory Data   │      │     Game        │
│   Architect     │ ←──→ │  (Bevy/Rust)    │
│  (Tauri/React)  │ YAML │                 │
└─────────────────┘      └─────────────────┘
```

| 項目 | 技術 |
|------|------|
| エディタ | Tauri + React + TypeScript |
| データ形式 | YAML |
| 連携方式 | ファイル経由 |

## ゲームとの連携

### 方式

| 方式 | 説明 |
|------|------|
| ホットリロード | エディタで保存 → ゲームに即反映 |
| テストプレイ | エディタからゲーム起動 |
| エクスポート | MODパッケージとして出力 |

### ホットリロード

```
1. エディタでアイテム/レシピを編集
2. 保存（Ctrl+S）
3. ゲーム側でファイル変更を検知
4. 該当データをリロード
5. ゲーム内に即反映
```

**対象:** アイテム、レシピ、クエスト、テキスト
**非対象:** コード変更、モデル変更（再起動必要）

### テストプレイ

```
1. エディタで「テストプレイ」ボタン
2. 現在の編集データでゲーム起動
3. ゲームウィンドウが開く
4. テスト終了 → エディタに戻る
```

## 編集タブ

### Items（アイテム）

アイテムの定義を編集。

| 機能 | 説明 |
|------|------|
| リスト表示 | 全アイテム一覧、検索/フィルタ |
| プロパティ編集 | ID、名前、スタック数、カテゴリ |
| アイコンプレビュー | 3Dモデルからアイコン生成 |
| 重複チェック | ID重複を警告 |

```yaml
# item_definition.yaml
id: iron_ingot
name_key: item.iron_ingot  # i18n用
category: material
stack_size: 999
model: items/iron_ingot.gltf
```

### Recipes（レシピ）

レシピをノードベースで編集。

| 機能 | 説明 |
|------|------|
| ノードエディタ | React Flow使用 |
| 入出力接続 | ドラッグ&ドロップ |
| 機械指定 | どの機械で処理するか |
| 時間/電力設定 | 処理時間、消費電力 |

```yaml
# recipe_definition.yaml
id: iron_plate
machine: press
inputs:
  - item: iron_ingot
    count: 1
outputs:
  - item: iron_plate
    count: 1
time: 2.0  # 秒
power: 15  # kW
```

### Quests（クエスト）

クエストをツリー/リストで編集。

| 機能 | 説明 |
|------|------|
| ツリー表示 | 前提関係を可視化 |
| 目標設定 | 納品アイテム、数量 |
| 報酬設定 | 機械、アイテム、アンロック |
| 依存関係 | 前提クエスト設定 |

```yaml
# quest_definition.yaml
id: automation_dawn
name_key: quest.automation_dawn
type: main
prerequisites:
  - first_step
objectives:
  - type: deliver
    item: iron_ingot
    count: 100
rewards:
  - type: item
    item: miner
    count: 2
  - type: item
    item: smelter
    count: 2
```

### Machines（機械）

機械の定義を編集。

| 機能 | 説明 |
|------|------|
| 基本プロパティ | サイズ、消費電力、処理速度 |
| 入出力設定 | スロット数、面ごとの設定 |
| ティア設定 | アップグレード関係 |
| モジュールスロット | 拡張スロット数 |

```yaml
# machine_definition.yaml
id: smelter_t1
name_key: machine.smelter
size: [1, 1, 1]
tier: 1
power: 10  # kW
slots:
  input: 1
  fuel: 1
  output: 1
  module: 2
upgrades_to: smelter_t2
```

### Multiblock（マルチブロック）

マルチブロック構造を3Dで編集。

| 機能 | 説明 |
|------|------|
| 3Dグリッド | React Three Fiberで表示 |
| レイヤー表示 | 高さごとにスライス |
| ブロック配置 | クリックで配置/削除 |
| プレビュー | 完成形を3D表示 |

### Biomes（バイオーム）

バイオームと資源分布を編集。

| 機能 | 説明 |
|------|------|
| ノイズパラメータ | 周波数、振幅、オクターブ |
| 2Dプレビュー | Canvas 2Dでノイズ可視化 |
| 資源オーバーレイ | 資源バイオームの設定 |
| 出現バイオーム | どの自然バイオームに出るか |

### Sounds（サウンド）

サウンド設定を編集。

| 機能 | 説明 |
|------|------|
| カテゴリツリー | マスター > BGM/SE/Voice |
| プレビュー再生 | その場で音を確認 |
| バリエーション | 同一サウンドの複数ファイル |
| 空間設定 | 距離減衰、3D設定 |

### Localization（ローカライズ）

多言語対応を編集。

| 機能 | 説明 |
|------|------|
| キー一覧 | 全翻訳キーをリスト |
| 言語切替 | 編集する言語を選択 |
| 未翻訳表示 | 翻訳が足りない項目をハイライト |
| プレビュー | ゲーム内での表示をプレビュー |

```yaml
# ja.yaml
item:
  iron_ingot: 鉄インゴット
  copper_ingot: 銅インゴット
quest:
  automation_dawn: 自動化の夜明け
```

## UX原則

| 原則 | 実装 |
|------|------|
| 即時プレビュー | 編集結果をリアルタイム反映 |
| 非破壊編集 | Undo 100回以上 |
| 制約可視化 | 無効な設定を赤表示 |
| スマートデフォルト | よく使う値を初期設定 |
| 一括操作 | 複数選択して編集 |
| 参照整合性 | 削除時に参照元を警告 |

## MODエクスポート

編集したデータをMODパッケージとして出力。

```
my-mod/
├── manifest.yaml    # MODメタ情報
├── data/
│   ├── items.yaml
│   ├── recipes.yaml
│   ├── quests.yaml
│   └── machines.yaml
├── assets/
│   ├── models/
│   └── sounds/
├── scripts/         # Luaスクリプト
└── lang/
    ├── en.yaml
    └── ja.yaml
```

### manifest.yaml

```yaml
id: my-awesome-mod
name: My Awesome Mod
version: 1.0.0
author: username
description: Adds cool machines
dependencies:
  - base: ">=1.0.0"
```

## 設計思想

| ポイント | 理由 |
|----------|------|
| Tauriで外部アプリ | ゲームと分離、開発効率UP |
| YAMLデータ形式 | 可読性高い、マージしやすい |
| ホットリロード | 編集サイクル高速化 |
| MODエクスポート | コミュニティ貢献しやすい |
