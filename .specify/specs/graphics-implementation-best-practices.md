# ゲームグラフィック/レンダリング実装ベストプラクティス

**作成日**: 2025-12-22
**目的**: 安定したフレームレートと良好な視覚体験を提供するための実装指針

---

## 1. パフォーマンス最適化の基本原則

### 1.1 目標フレームレート

| ターゲット | フレーム時間 | 用途 |
|-----------|-------------|------|
| 60 FPS | 16.67ms | 標準ゲームプレイ |
| 30 FPS | 33.33ms | 複雑なシーン/低スペック |
| 120+ FPS | <8.33ms | 競技/VR |

### 1.2 フレーム時間予算配分（60FPS目標）

```
総予算: 16.67ms
├── CPU
│   ├── ゲームロジック: 4-6ms
│   ├── 物理演算: 2-3ms
│   └── カリング/準備: 2-3ms
└── GPU
    ├── シャドウ: 2-3ms
    ├── メインパス: 4-6ms
    ├── ポストプロセス: 1-2ms
    └── UI: 1ms
```

---

## 2. カリング技術

### 2.1 フラスタムカリング

> カメラの視錐台外のオブジェクトを描画から除外

```rust
// Bevyでは自動で有効
// AABBに基づいて自動カリング
commands.spawn((
    Mesh3d(mesh),
    Aabb::from_min_max(min, max),  // バウンディングボックス設定
));
```

**効果**: 大規模シーンで60FPS維持に必須

### 2.2 オクルージョンカリング

> 他のオブジェクトに隠れたオブジェクトを描画から除外

| 手法 | CPU負荷 | 効果 |
|------|--------|------|
| ソフトウェアラスタライズ | 中 | 高 |
| GPU Occlusion Query | 低 | 中（1フレーム遅延） |
| Hierarchical Z-Buffer | 低 | 高 |

### 2.3 バックフェースカリング

> カメラに背を向けた面を描画しない

```rust
// 標準で有効、透明オブジェクトでは無効化が必要な場合も
```

---

## 3. バッチングとインスタンシング

### 3.1 Bevyの自動バッチング（0.12+）

> 同じメッシュ・マテリアルを使用するエンティティを自動的にまとめて描画

```rust
// 良い例: 同じハンドルを共有
let shared_mesh = meshes.add(Cuboid::default());
let shared_material = materials.add(Color::srgb(0.8, 0.2, 0.2));

for _ in 0..1000 {
    commands.spawn((
        Mesh3d(shared_mesh.clone()),
        MeshMaterial3d(shared_material.clone()),
        Transform::from_xyz(x, y, z),
    ));
}
// → 1回のインスタンス描画で1000個レンダリング

// 悪い例: 毎回新しいハンドル
for _ in 0..1000 {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),  // 毎回新規
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
        Transform::from_xyz(x, y, z),
    ));
}
// → 1000回の個別描画（激重）
```

**パフォーマンス向上**: 最大3倍のフレームレート向上（公式ベンチマーク）

### 3.2 バッチングの条件

| 要素 | バッチ可能条件 |
|------|---------------|
| メッシュ | 同じHandle<Mesh> |
| マテリアル | 同じHandle<Material> |
| シェーダー | 同じパイプライン |

### 3.3 テクスチャアトラス

```rust
// 複数テクスチャを1枚にまとめる
// → マテリアル切り替えを削減
TextureAtlas {
    layout: atlas_layout,
    index: sprite_index,
}
```

---

## 4. LOD（Level of Detail）

### 4.1 距離ベースLOD

```rust
#[derive(Component)]
enum LodLevel {
    Full,      // 0-50m: フルディテール
    Medium,    // 50-150m: 50%ポリゴン
    Low,       // 150-300m: 10%ポリゴン
    Billboard, // 300m+: 2Dスプライト
}

fn update_lod(
    camera: Query<&Transform, With<Camera>>,
    mut objects: Query<(&Transform, &mut LodLevel)>,
) {
    let cam_pos = camera.single().translation;
    for (transform, mut lod) in &mut objects {
        let distance = transform.translation.distance(cam_pos);
        *lod = match distance {
            d if d < 50.0 => LodLevel::Full,
            d if d < 150.0 => LodLevel::Medium,
            d if d < 300.0 => LodLevel::Low,
            _ => LodLevel::Billboard,
        };
    }
}
```

### 4.2 推奨LODレベル数

| オブジェクト種類 | LODレベル数 |
|----------------|------------|
| 背景建物 | 3-4 |
| キャラクター | 2-3 |
| 小物 | 2 |
| 地形 | 4+ |

---

## 5. ボクセル/工場ゲーム特有の最適化

### 5.1 チャンクシステム

```rust
const CHUNK_SIZE: i32 = 32;  // 32x32x32が一般的

#[derive(Component)]
struct Chunk {
    position: IVec3,
    voxels: [[[VoxelType; 32]; 32]; 32],
    mesh_dirty: bool,
}

// チャンク単位でメッシュ更新
fn update_dirty_chunks(
    mut chunks: Query<&mut Chunk>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for mut chunk in &mut chunks {
        if chunk.mesh_dirty {
            let mesh = generate_chunk_mesh(&chunk);
            // メッシュ更新
            chunk.mesh_dirty = false;
        }
    }
}
```

### 5.2 グリーディメッシング

> 隣接する同じ種類のボクセル面を結合して大きなクワッドに

```
Before: 100個のボクセル = 600面（最大）
After:  グリーディメッシング = 6-20面程度

パフォーマンス向上: 10-50倍の頂点削減
```

**トレードオフ**:
- CPU負荷増加（メッシュ生成時）
- 実装複雑度
- 異なるテクスチャが多いと効果減少

### 5.3 面カリング

```rust
// 隣接ボクセルがある場合、間の面を生成しない
fn should_generate_face(chunk: &Chunk, pos: IVec3, dir: IVec3) -> bool {
    let neighbor_pos = pos + dir;
    if is_out_of_bounds(neighbor_pos) {
        return true;  // チャンク境界は生成
    }
    chunk.get_voxel(neighbor_pos).is_transparent()
}
```

---

## 6. シェーダーコンパイル対策

### 6.1 問題

> 初回描画時にシェーダーをコンパイルするとスタッターが発生

**2024年の問題ゲーム例**:
- Silent Hill 2 Remake
- STALKER 2
- Black Myth: Wukong

### 6.2 対策

```rust
// 起動時にシェーダーをプリコンパイル
fn precompile_shaders(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 使用する全マテリアルバリエーションを事前ロード
    for variant in MaterialVariant::all() {
        let _ = materials.add(variant.to_material());
    }
}
```

### 6.3 ベストプラクティス

| 対策 | 効果 |
|------|------|
| 起動時プリコンパイル | スタッター防止 |
| ローディング中にコンパイル | プレイ中の遅延回避 |
| シェーダーバリエーション最小化 | コンパイル時間短縮 |
| PSOキャッシュ保存 | 再起動時の高速化 |

---

## 7. パーティクル/VFX最適化

### 7.1 オーバードロー問題

> 透明パーティクルの重なりで同じピクセルを何度も描画

**目標**: オーバードロー値 < 2.0（理想は1.0）

### 7.2 対策

```rust
// パーティクル最適化設定
struct ParticleSettings {
    max_particles: u32,      // 上限設定（500-2000）
    particle_size: f32,      // 最小限のサイズ
    lifetime: f32,           // 短めのライフタイム
    use_additive_blend: bool, // 加算合成（ソート不要）
}
```

### 7.3 予算配分

| 品質 | 最大パーティクル数 | フレーム時間予算 |
|------|-------------------|-----------------|
| 低 | 500 | 1ms |
| 中 | 1000 | 1.5ms |
| 高 | 2000 | 2ms |

---

## 8. UI/HUD最適化

### 8.1 再描画の最小化

```rust
// 変更時のみ更新
fn update_health_ui(
    health: Res<PlayerHealth>,
    mut text: Query<&mut Text, With<HealthText>>,
) {
    if health.is_changed() {  // 変更検出
        for mut t in &mut text {
            t.0 = format!("HP: {}", health.current);
        }
    }
}
```

### 8.2 UIレイヤー分離

```
レイヤー構成:
├── Layer 0: 3Dワールド
├── Layer 1: ワールドスペースUI（機械ラベル等）
├── Layer 2: HUD（常時表示）
└── Layer 3: メニュー/ダイアログ
```

---

## 9. グラフィック設定UI

### 9.1 必須設定項目

```rust
struct GraphicsSettings {
    // 解像度
    resolution: (u32, u32),
    fullscreen_mode: FullscreenMode,

    // 品質プリセット
    quality_preset: QualityPreset,  // Low/Medium/High/Ultra

    // 個別設定
    shadow_quality: ShadowQuality,
    texture_quality: TextureQuality,
    view_distance: ViewDistance,
    vsync: bool,
    fps_limit: Option<u32>,

    // アクセシビリティ
    colorblind_mode: ColorblindMode,
    ui_scale: f32,
}
```

### 9.2 アクセシビリティ

| 機能 | 実装 |
|------|------|
| 色覚サポート | 3種類のカラーモード（Deuteranopia/Protanopia/Tritanopia） |
| 高コントラスト | UIコントラスト4.5:1以上 |
| UIスケール | 100%-200% |
| 字幕サイズ | 複数サイズ選択 |

---

## 10. 本プロジェクトへの適用

### 10.1 工場ゲーム特有の考慮点

| 課題 | 対策 |
|------|------|
| 大量の機械 | インスタンシング必須 |
| アイテム移動 | GPUインスタンシング |
| ボクセル地形 | チャンク+グリーディメッシング |
| 長時間プレイ | メモリリーク監視 |

### 10.2 推奨実装順

1. **チャンクシステム整備**
   - 32x32x32チャンク
   - 非同期メッシュ生成

2. **インスタンシング活用**
   - 機械の同種まとめ描画
   - アイテムエンティティ

3. **LODシステム**
   - 距離ベースLOD
   - チャンクLOD

4. **カリング強化**
   - オクルージョンカリング検討
   - 地下空間の最適化

### 10.3 パフォーマンス目標

| 項目 | 目標値 |
|------|--------|
| フレームレート | 60 FPS安定 |
| 描画コール | <500/フレーム |
| VRAM使用 | <2GB |
| フレーム時間変動 | <5ms |

---

## 参考文献

- [Bevy 0.12 Release Notes - Automatic Batching](https://bevy.org/news/bevy-0-12/)
- [Meshing in a Minecraft Game](https://0fps.net/2012/06/30/meshing-in-a-minecraft-game/)
- [Unreal Engine VFX Optimization Guide](https://docs.unrealengine.com/4.26/en-US/RenderingAndGraphics/ParticleSystems/Optimization)
- [Unity Overdraw Optimization](https://thegamedev.guru/unity-gpu-performance/overdraw-optimization/)
- [Game Engines and Shader Stuttering](https://www.unrealengine.com/en-US/tech-blog/game-engines-and-shader-stuttering-unreal-engines-solution-to-the-problem)
- [Xbox Accessibility Guidelines](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112)

---

*このレポートはゲームグラフィック実装のベストプラクティス調査に基づいています。*
