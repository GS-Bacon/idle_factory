# Factory Data Architect アーキテクチャレビュー

## 調査日: 2025-12-22

---

## エグゼクティブサマリー

エディタとゲーム間に**重大なデータフォーマット不整合**が存在し、現状ではエディタで作成したデータをゲームで直接使用できない。また、型定義の重複（DRY違反）やSOLID原則からの逸脱が散見される。

---

## 重大な問題 (Critical)

### 1. データフォーマットの不整合

| コンポーネント | 保存形式 | 読込形式 |
|---------------|----------|----------|
| エディタ | RON | - |
| ゲーム (Items) | - | YAML |
| ゲーム (Recipes) | - | YAML |
| ゲーム (Quests) | - | YAML |

**影響**: エディタで作成したデータをゲームで使用できない

**ファイル**:
- `tools/factory-data-architect/src-tauri/src/lib.rs:88` - RONで保存
- `src/gameplay/inventory.rs:320` - YAMLで読込
- `src/gameplay/quest.rs:339` - YAMLで読込
- `src/gameplay/machines/recipe_system.rs:227` - YAMLで読込

### 2. レシピ型の非互換性

**エディタ側 (`recipe.rs`)**:
```rust
pub struct RecipeDef {
    pub machine_type: MachineType,  // Assembler, Mixer, Press...
    pub ingredients: Vec<Ingredient>,
    pub results: Vec<Product>,
    pub process_time: f32,
    pub stress_impact: f32,
}
```

**ゲーム側 (`recipe_system.rs`)**:
```rust
pub struct Recipe {
    pub work_type: WorkType,  // Pressing, Crushing, Cutting...
    pub inputs: Vec<ItemIO>,
    pub outputs: Vec<ItemIO>,
    pub input_fluid: Option<FluidIO>,
    pub craft_time: f32,
}
```

**問題点**:
- `machine_type` vs `work_type`: 異なるenum
- `ingredients` vs `inputs`: 異なる構造体
- `process_time` vs `craft_time`: 同じ意味で異なるフィールド名
- エディタの `stress_impact` がゲームに存在しない

### 3. アイテムプロパティの型不整合

**エディタ (`models.rs`)**:
```rust
pub properties: HashMap<String, serde_json::Value>
```

**ゲーム (`inventory.rs`)**:
```rust
pub custom_properties: HashMap<String, String>
```

**影響**: 数値型やブール型のプロパティがStringに変換される際にデータ損失

---

## 設計上の問題 (DRY/SOLID違反)

### 4. 型定義の三重重複 (DRY違反)

同じ概念が3箇所で定義:

| 概念 | エディタRust | エディタTS | ゲームRust |
|------|-------------|-----------|-----------|
| アイテム | `models::ItemData` | `types/index.ts::ItemData` | `inventory::ItemData` |
| レシピ | `recipe::RecipeDef` | `types/recipe.ts::RecipeDef` | `recipe_system::Recipe` |
| カタログ | `recipe::AssetCatalog` | `types/recipe.ts::AssetCatalog` | - |

### 5. 単一責任原則違反 (SRP)

`lib.rs` (396行) に以下が混在:
- アイテム操作 (create, update, save, load, delete)
- ローカライズ操作 (save, load, update)
- レシピ操作 (save, load, list)
- カタログ操作 (get_assets_catalog)
- パス操作 (set/get_assets_path, to_relative_path)

**推奨**: 機能ごとにモジュール分割

### 6. i18n_key生成ロジックの重複

以下の箇所で同じロジックが重複:
- `tools/.../src-tauri/src/models.rs:97` - `format!("item.{}", id)`
- `src/gameplay/inventory.rs` - ハードコード
- `src/gameplay/quest.rs:129` - `format!("quest.{}", &id)`

---

## パフォーマンス問題

### 7. ファイルシステムスキャンのキャッシュなし

`get_assets_catalog` (`lib.rs:148-182`):
- 呼び出しごとにディレクトリをスキャン
- 大量のファイルがある場合にボトルネック

**推奨**: キャッシュ機構の追加、ファイル変更監視

### 8. 非効率なデータ読み込み

`ItemsTab` (`App.tsx:101-119`):
- マウント時に全アイテムをロード
- 個別アイテム選択時に再度ファイル読み込み

---

## セキュリティ・安定性

### 9. ハードコードされたパス

`App.tsx:64`:
```typescript
const DEFAULT_ASSETS_PATH = "C:/Users/bacon/OneDrive/ドキュメント/github/IdealFactoryGame/my-bevy-project/assets";
```

**問題**: 他の開発者/環境で動作しない

### 10. unwrap()の過剰使用

`lib.rs` 内で `unwrap()` が複数箇所:
- `state.assets_path.lock().unwrap()` (行29, 35, 51, 62, 72, 81, 100, 112, 123, 131, 149)
- Mutexロック失敗時にパニック

**推奨**: `?` 演算子または適切なエラーハンドリング

---

## 未使用コード

### 11. dead_code警告の抑制

以下のアイテムに `#[allow(dead_code)]` が付与:
- `models.rs:48` - `ItemVisuals`
- `recipe.rs:86` - `RecipeDef::new`
- `recipe.rs:103` - `AssetNode`

**推奨**: 使用されていない場合は削除、将来使用予定なら TODO コメント

---

## 修正優先度

### 高優先度 (機能影響)
1. **データフォーマット統一**: RON→YAML変換、または共通フォーマット採用
2. **レシピ型の統一**: エディタとゲームで互換性のある型定義
3. **プロパティ型の統一**: JSON Value → String変換ロジック

### 中優先度 (保守性)
4. **型定義の共有**: 共通crateの作成
5. **lib.rsの分割**: 機能ごとにモジュール化
6. **i18n_key生成の統一**: 共通関数化

### 低優先度 (最適化)
7. **カタログキャッシュ**: ファイル変更監視
8. **ハードコードパス削除**: 設定ファイル化
9. **unwrap削除**: 適切なエラーハンドリング

---

## 推奨アクション

### 短期 (今すぐ)
1. エディタにYAMLエクスポート機能を追加
2. ゲーム側でRONファイルも読めるようにフォールバック追加

### 中期
1. 共通型定義crate (`factory-data-types`) の作成
2. TypeScript型はRust型から自動生成 (ts-rs等)
3. lib.rsをモジュール分割

### 長期
1. データフォーマットをYAMLまたはRONに統一
2. エディタ-ゲーム間のデータ同期機能
3. ホットリロード対応

---

## 参考: ファイル一覧

### エディタ
- `tools/factory-data-architect/src-tauri/src/lib.rs` - Tauriコマンド
- `tools/factory-data-architect/src-tauri/src/models.rs` - アイテム型
- `tools/factory-data-architect/src-tauri/src/recipe.rs` - レシピ型
- `tools/factory-data-architect/src/types/index.ts` - TS型定義
- `tools/factory-data-architect/src/types/recipe.ts` - TSレシピ型

### ゲーム
- `src/gameplay/inventory.rs` - アイテム/インベントリ
- `src/gameplay/quest.rs` - クエスト
- `src/gameplay/machines/recipe_system.rs` - レシピ
