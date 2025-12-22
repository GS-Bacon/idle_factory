# MOD開発API ベストプラクティス

**作成日**: 2025-12-22
**目的**: MODクリエイターにとって使いやすく、安全なMOD APIを設計するための指針

---

## 1. Factorioのデータライフサイクルモデル

### 1.1 3段階のロードフェーズ

```
1. Settings Stage（設定）
   ├── settings.lua
   └── 目的: MOD設定の定義

2. Data Stage（プロトタイプ）
   ├── data.lua
   ├── data-updates.lua
   └── data-final-fixes.lua
   └── 目的: アイテム、レシピ、エンティティの定義

3. Runtime Stage（実行時）
   ├── control.lua
   └── 目的: ゲームプレイ中のロジック
```

### 1.2 Data Stageの詳細

```lua
-- data.lua: 基本プロトタイプ定義
data:extend({
    {
        type = "item",
        name = "my-item",
        icon = "__my-mod__/graphics/my-item.png",
        icon_size = 64,
        stack_size = 100,
    }
})

-- data-updates.lua: 他MODのデータを変更
-- この時点で全MODのdata.luaが実行済み
local iron_plate = data.raw["item"]["iron-plate"]
iron_plate.stack_size = 200

-- data-final-fixes.lua: 最終調整
-- 全MODのdata-updates.luaが実行済み
```

### 1.3 MODロード順序

```
ロード順序の決定要因:
  1. 依存関係の深さ（依存が少ないMODが先）
  2. MOD内部名のアルファベット順

依存関係の種類:
  - dependencies: 必須
  - optional_dependencies: 任意
  - incompatible: 非互換
```

---

## 2. API設計原則

### 2.1 明確な境界

```rust
// 良い例: MODがアクセスできる範囲を明確に
pub mod mod_api {
    // 読み取り可能
    pub fn get_item(id: &str) -> Option<&ItemData>;
    pub fn get_recipe(id: &str) -> Option<&RecipeData>;

    // 追加可能
    pub fn register_item(item: ItemDefinition) -> Result<ItemId, Error>;
    pub fn register_recipe(recipe: RecipeDefinition) -> Result<RecipeId, Error>;

    // 変更可能（制限付き）
    pub fn modify_item(id: &str, modifier: ItemModifier) -> Result<(), Error>;

    // 内部APIは非公開
    // fn internal_system_call() { }  // 公開しない
}
```

### 2.2 バージョニング

```rust
// セマンティックバージョニング
pub const API_VERSION: ApiVersion = ApiVersion {
    major: 1,  // 破壊的変更
    minor: 5,  // 機能追加
    patch: 2,  // バグ修正
};

// 後方互換性の維持
fn load_mod(mod_info: &ModInfo) -> Result<(), Error> {
    if mod_info.required_api_version.major != API_VERSION.major {
        return Err(Error::IncompatibleApiVersion);
    }

    // マイナーバージョンは新しくてもOK
    if mod_info.required_api_version.minor > API_VERSION.minor {
        warn!("MOD requires newer API features");
    }

    Ok(())
}
```

### 2.3 サンドボックス化

```rust
// MODの権限制御
struct ModPermissions {
    can_read_save: bool,
    can_write_save: bool,
    can_access_network: bool,
    can_execute_commands: bool,
}

// デフォルトは最小権限
impl Default for ModPermissions {
    fn default() -> Self {
        Self {
            can_read_save: true,
            can_write_save: false,
            can_access_network: false,
            can_execute_commands: false,
        }
    }
}
```

---

## 3. データ定義システム

### 3.1 宣言的定義

```json
// item_definition.json
{
    "type": "item",
    "id": "advanced-circuit",
    "name": {
        "en": "Advanced Circuit",
        "ja": "高度な回路"
    },
    "icon": "graphics/advanced-circuit.png",
    "stack_size": 100,
    "recipe": {
        "ingredients": [
            { "item": "electronic-circuit", "amount": 2 },
            { "item": "copper-cable", "amount": 4 }
        ],
        "result_count": 1,
        "crafting_time": 6.0
    }
}
```

### 3.2 スキーマバリデーション

```rust
// JSON Schema でMODデータを検証
fn validate_mod_data(data: &serde_json::Value) -> Result<(), ValidationErrors> {
    let schema = load_schema("item_schema.json")?;
    let compiled = JSONSchema::compile(&schema)?;

    if let Err(errors) = compiled.validate(data) {
        return Err(ValidationErrors::from_iter(errors));
    }

    Ok(())
}
```

### 3.3 ホットリロード

```rust
// 開発中にMODを再読み込み
fn reload_mod(mod_id: &str) -> Result<(), Error> {
    // 1. 既存のMODデータをアンロード
    unload_mod_data(mod_id)?;

    // 2. 新しいデータを読み込み
    let new_data = load_mod_data(mod_id)?;

    // 3. バリデーション
    validate_mod_data(&new_data)?;

    // 4. 適用
    apply_mod_data(mod_id, new_data)?;

    info!("MOD {} reloaded successfully", mod_id);
    Ok(())
}
```

---

## 4. イベントシステム

### 4.1 イベント登録

```rust
// MODがゲームイベントにフック
pub trait ModEventHandler {
    fn on_item_crafted(&self, event: &ItemCraftedEvent);
    fn on_machine_placed(&self, event: &MachinePlacedEvent);
    fn on_research_completed(&self, event: &ResearchCompletedEvent);
}

// イベント登録
fn register_mod_events(mod_id: ModId, handler: Box<dyn ModEventHandler>) {
    EVENT_REGISTRY.lock().register(mod_id, handler);
}
```

### 4.2 イベント優先度

```rust
enum EventPriority {
    First,    // 最初に実行
    Early,
    Normal,   // デフォルト
    Late,
    Last,     // 最後に実行
}

fn register_event<E: Event>(
    handler: impl Fn(&E),
    priority: EventPriority,
) {
    // 優先度順に実行
}
```

### 4.3 イベントキャンセル

```rust
struct ItemCraftedEvent {
    item: ItemId,
    amount: u32,
    player: PlayerId,
    cancelled: Cell<bool>,  // MODがキャンセル可能
}

impl ItemCraftedEvent {
    pub fn cancel(&self) {
        self.cancelled.set(true);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.get()
    }
}
```

---

## 5. マルチプレイ対応

### 5.1 MOD同期

```rust
// マルチプレイでのMOD一致確認
struct ModChecksum {
    mod_id: String,
    version: String,
    data_hash: u64,
}

fn verify_mod_compatibility(
    server_mods: &[ModChecksum],
    client_mods: &[ModChecksum],
) -> Result<(), ModMismatchError> {
    for server_mod in server_mods {
        let client_mod = client_mods.iter()
            .find(|m| m.mod_id == server_mod.mod_id);

        match client_mod {
            None => return Err(ModMismatchError::Missing(server_mod.mod_id.clone())),
            Some(cm) if cm.version != server_mod.version =>
                return Err(ModMismatchError::VersionMismatch),
            _ => {}
        }
    }
    Ok(())
}
```

### 5.2 決定的実行

```rust
// MODコードは決定的でなければならない
fn mod_guidelines() {
    // NG: ランダムをそのまま使用
    let random = rand::random::<u32>();

    // OK: ゲームのシードを使用
    let random = game_rng.gen::<u32>();

    // NG: 現在時刻に依存
    let now = std::time::Instant::now();

    // OK: ゲームティックを使用
    let tick = game_state.current_tick();
}
```

---

## 6. ドキュメントとツール

### 6.1 API ドキュメント

```rust
/// アイテムを登録します
///
/// # 引数
/// * `definition` - アイテムの定義
///
/// # 戻り値
/// 成功時は登録されたアイテムのID
///
/// # エラー
/// * `DuplicateId` - 同じIDのアイテムが既に存在
/// * `InvalidData` - 定義が不正
///
/// # 例
/// ```
/// let item = ItemDefinition {
///     id: "my-item".to_string(),
///     name: "My Item".to_string(),
///     stack_size: 100,
/// };
/// let id = register_item(item)?;
/// ```
pub fn register_item(definition: ItemDefinition) -> Result<ItemId, Error> {
    // ...
}
```

### 6.2 開発ツール

```
MOD開発者向けツール:
  1. MODテンプレート生成
  2. スキーマ検証ツール
  3. ホットリロード対応のデバッグモード
  4. MODパッケージャー
  5. 依存関係解析ツール
```

### 6.3 サンプルMOD

```
公式サンプル:
  - basic-item: 基本的なアイテム追加
  - custom-machine: カスタム機械
  - new-research: 研究ツリー拡張
  - ui-extension: UI拡張
  - multiplayer-safe: マルチプレイ対応MOD
```

---

## 7. 工場ゲーム特有の考慮

### 7.1 レシピ拡張

```rust
// 既存レシピの変更
pub fn modify_recipe(
    recipe_id: &str,
    modifier: impl FnOnce(&mut RecipeData),
) -> Result<(), Error> {
    let recipe = get_recipe_mut(recipe_id)?;
    modifier(recipe);
    recalculate_production_chains()?;  // 影響範囲を再計算
    Ok(())
}

// 使用例
modify_recipe("iron-ingot", |recipe| {
    recipe.crafting_time *= 0.5;  // 速度2倍
});
```

### 7.2 機械追加

```rust
// カスタム機械の定義
pub struct MachineDefinition {
    pub id: String,
    pub name: LocalizedString,
    pub category: MachineCategory,
    pub power_consumption: f32,
    pub crafting_speed: f32,
    pub allowed_recipes: Vec<String>,  // 空なら全て
    pub model: ModelPath,
}
```

### 7.3 リソース追加

```rust
// 新しい資源の追加
pub struct ResourceDefinition {
    pub id: String,
    pub name: LocalizedString,
    pub mining_time: f32,
    pub yields: Vec<(ItemId, f32)>,  // (アイテム, 確率)
    pub spawn_config: SpawnConfig,
}
```

---

## 8. チェックリスト

### API設計
- [ ] 公開APIと内部APIが分離されているか
- [ ] バージョニングがあるか
- [ ] 後方互換性を考慮しているか

### セキュリティ
- [ ] サンドボックス化されているか
- [ ] 権限システムがあるか
- [ ] 悪意あるMODへの対策があるか

### 開発者体験
- [ ] ドキュメントがあるか
- [ ] サンプルMODがあるか
- [ ] ホットリロードがあるか
- [ ] エラーメッセージが分かりやすいか

### マルチプレイ
- [ ] MOD同期の仕組みがあるか
- [ ] 決定的実行のガイドラインがあるか

---

## 参考文献

- [Factorio Data Lifecycle - Lua API](https://lua-api.factorio.com/latest/auxiliary/data-lifecycle.html)
- [Factorio Mod Structure - Lua API](https://lua-api.factorio.com/latest/auxiliary/mod-structure.html)
- [Factorio Modding Architecture - Wiki](https://wiki.factorio.com/User:Vadcx/Modding_Architecture)
- [Minecraft Modding Guide - Forge Docs](https://docs.minecraftforge.net/)

---

*このレポートはFactorioおよびMinecraftのMOD APIアーキテクチャ調査に基づいています。*
