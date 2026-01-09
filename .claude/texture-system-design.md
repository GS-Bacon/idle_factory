# テクスチャシステム設計書

## 概要

Minecraft風の3層テクスチャシステムを実装する。

```
blockstates/*.json  →  models/*.json  →  textures/*.png
   (状態→モデル)        (形状定義)         (画像)
```

## 現状

| 項目 | 状態 |
|------|------|
| テクスチャファイル | 存在（16x16, blocks/items分離） |
| アトラス | 未使用（block_atlas_default.png存在） |
| メッシュ生成 | 単色Color使用 |
| blockstates | なし |
| models | なし |

## 既存ブロック一覧

| ID | 名前 | カテゴリ | placeable | テクスチャ |
|----|------|----------|-----------|------------|
| stone | Stone | Terrain | ✓ | blocks/stone.png ✓ |
| grass | Grass | Terrain | ✓ | blocks/grass_top.png, grass_side.png ✓ |
| iron_ore | Iron Ore | Ore | ✓ | blocks/iron_ore.png ✓ |
| copper_ore | Copper Ore | Ore | ✓ | blocks/copper_ore.png ✓ |
| coal | Coal | Ore | ✓ | blocks/coal_ore.png ✓ |
| iron_ingot | Iron Ingot | Processed | ✗ | items/iron_ingot.png ✓ |
| copper_ingot | Copper Ingot | Processed | ✗ | items/copper_ingot.png ✓ |
| iron_dust | Iron Dust | Processed | ✗ | 未作成 |
| copper_dust | Copper Dust | Processed | ✗ | 未作成 |
| miner | Miner | Machine | ✓ | items/miner.png ✓ |
| conveyor | Conveyor | Machine | ✓ | items/conveyor.png ✓ |
| furnace | Furnace | Machine | ✓ | items/furnace.png ✓ |
| crusher | Crusher | Machine | ✓ | items/crusher.png ✓ |
| assembler | Assembler | Machine | ✓ | items/assembler.png ✓ |
| platform | Platform | Machine | ✓ | 未作成 |
| stone_pickaxe | Stone Pickaxe | Tool | ✗ | 未作成 |

## 実装フェーズ

### T1: テクスチャアトラス基盤（優先度: 高）

**目的**: 単色→テクスチャ切り替え

```rust
// src/textures/atlas.rs
pub struct TextureAtlas {
    pub image: Handle<Image>,
    pub regions: HashMap<String, UVRect>,  // "stone" -> UVRect
    pub size: UVec2,  // 256x256など
}

pub struct UVRect {
    pub min: Vec2,  // UV座標 (0.0-1.0)
    pub max: Vec2,
}

// src/textures/registry.rs
pub struct TextureRegistry {
    pub block_atlas: TextureAtlas,
    pub item_atlas: TextureAtlas,
}
```

**作業**:
1. `src/textures/mod.rs` 新規作成
2. アトラス生成（起動時に個別PNGを結合）
3. メッシュ生成にUV座標を追加
4. シェーダー対応（テクスチャサンプリング）

### T2: 面別テクスチャ（優先度: 高）

**目的**: 草ブロックの上面/側面/下面を別テクスチャに

```rust
pub enum FaceTexture {
    All(String),              // 全面同じ: "stone"
    TopSideBottom {           // 上/側/下
        top: String,          // "grass_top"
        side: String,         // "grass_side"
        bottom: String,       // "dirt"
    },
    Directional {             // 向きあり（機械など）
        front: String,
        back: String,
        top: String,
        bottom: String,
        side: String,
    },
}
```

### T3: blockstates JSONシステム（優先度: 中）

**目的**: 状態→モデルマッピングをデータ駆動に

```
assets/blockstates/conveyor.json
assets/models/block/conveyor.json
```

```json
// assets/blockstates/conveyor.json
{
  "variants": {
    "facing=north": { "model": "block/conveyor", "y": 0 },
    "facing=east":  { "model": "block/conveyor", "y": 90 },
    "facing=south": { "model": "block/conveyor", "y": 180 },
    "facing=west":  { "model": "block/conveyor", "y": 270 }
  }
}
```

```json
// assets/models/block/conveyor.json
{
  "parent": "block/cube",
  "textures": {
    "top": "block/conveyor_top",
    "side": "block/conveyor_side",
    "bottom": "block/conveyor_bottom"
  }
}
```

```rust
// src/textures/blockstates.rs
pub struct BlockstateDefinition {
    pub variants: Option<HashMap<String, ModelVariant>>,
    pub multipart: Option<Vec<MultipartCase>>,
}

pub struct ModelVariant {
    pub model: String,
    pub x: Option<i32>,  // 0, 90, 180, 270
    pub y: Option<i32>,
    pub uvlock: Option<bool>,
    pub weight: Option<u32>,
}

pub struct MultipartCase {
    pub when: Option<HashMap<String, String>>,
    pub apply: ModelVariant,
}
```

### T4: リソースパック読み込み（優先度: 中）

**目的**: ユーザーがテクスチャを差し替え可能に

```
resource_packs/
  my_pack/
    pack.toml
    assets/
      textures/block/stone.png  # 上書き
```

```toml
# resource_packs/my_pack/pack.toml
[pack]
id = "my_pack"
name = "My Resource Pack"
version = "1.0.0"
description = "Custom textures"
```

```rust
// src/textures/resource_pack.rs
pub struct ResourcePackManager {
    packs: Vec<ResourcePack>,  // 優先度順
}

impl ResourcePackManager {
    pub fn resolve_texture(&self, path: &str) -> Option<PathBuf> {
        // 上から順に探す
        for pack in &self.packs {
            if let Some(p) = pack.find_texture(path) {
                return Some(p);
            }
        }
        // ベースゲームにフォールバック
        None
    }
}
```

### T5: MOD拡張ポイント（優先度: 低）

**目的**: MODがレンダリングをカスタマイズ可能に

```rust
// src/textures/resolver.rs
pub trait TextureResolver: Send + Sync + 'static {
    /// ブロックの特定の面のテクスチャを解決
    fn resolve(
        &self,
        block_id: ItemId,
        face: Face,
        block_state: &BlockState,
        neighbors: &NeighborInfo,
    ) -> TextureResult;
}

pub enum TextureResult {
    /// アトラス内のUV座標
    Atlas { region: UVRect },
    /// カスタムモデルに置換
    Model { handle: Handle<Scene> },
    /// このリゾルバでは処理しない（次へ）
    Pass,
}

// ベースゲーム実装
pub struct DefaultTextureResolver;

// MOD実装例: コネクテッドテクスチャ
pub struct ConnectedTextureResolver {
    patterns: HashMap<ItemId, CTMPattern>,
}
```

**WASM MOD API**:
```rust
// ホスト関数
#[host_fn]
fn register_texture_resolver(resolver_id: u32);

#[host_fn]
fn resolve_texture(block_id: u32, face: u8, neighbors: u8) -> u32;
```

## アトラス構成案

```
block_atlas.png (256x256, 16x16ブロック)
┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │
├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤
│st│gr│gr│di│sa│ir│cu│co│  │  │  │  │  │  │  │  │ Row 0: Terrain/Ore
│on│_t│_s│rt│nd│on│pp│al│  │  │  │  │  │  │  │  │
├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤
│mi│co│fu│cr│as│pl│  │  │  │  │  │  │  │  │  │  │ Row 1: Machines
│nr│nv│rn│us│mb│at│  │  │  │  │  │  │  │  │  │  │
├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤
│  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │  │ Row 2-15: 拡張用
└──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
```

## ファイル構成

```
src/
  textures/
    mod.rs              # モジュール公開
    atlas.rs            # TextureAtlas, UVRect
    registry.rs         # TextureRegistry
    blockstates.rs      # BlockstateDefinition
    models.rs           # ModelDefinition
    resolver.rs         # TextureResolver trait
    resource_pack.rs    # ResourcePackManager

assets/
  textures/
    blocks/             # 既存
    items/              # 既存
  blockstates/          # 新規
    stone.json
    grass.json
    conveyor.json
  models/
    block/              # 新規
      cube.json         # 基本立方体
      cube_all.json     # 全面同一テクスチャ
      cube_top.json     # 上面別
      stone.json
      grass.json
```

## 作業見積もり

| Phase | 内容 | 見積もり |
|-------|------|----------|
| T1 | アトラス基盤 + メッシュUV | 2-3時間 |
| T2 | 面別テクスチャ | 1-2時間 |
| T3 | blockstates JSON | 2-3時間 |
| T4 | リソースパック | 1-2時間 |
| T5 | MOD拡張 | 設計のみ（実装は後） |

**合計**: 6-10時間（T1-T4）

## CC0テクスチャソース

- [OpenGameArt 16x16 Block Texture Set](https://opengameart.org/content/16x16-block-texture-set) - CC0
- [Kenney Voxel Pack](https://www.kenney.nl/assets/voxel-pack) - CC0
- [ambientCG](https://ambientcg.com/) - CC0 PBRテクスチャ

## 次のステップ

1. OpenGameArtからCC0テクスチャをダウンロード
2. T1実装開始（アトラス基盤）
3. 既存ブロックにテクスチャを適用してテスト
