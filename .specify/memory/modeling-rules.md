# 3Dモデリングルール

> **Note**: 詳細なリファレンスは `.specify/memory/modeling-compact.md` を参照

## スタイル
- **Minecraft / Unturned風ローポリ**: ブロック感＋ピクセルアート的な明確なシルエット
- **参考**:
  - [Kenney Conveyor Kit](https://kenney.nl/assets/conveyor-kit)
  - [Blockbench Minecraft Style Guide](https://www.blockbench.net/wiki/guides/minecraft-style-guide/)
- **シェーディング**: フラット、頂点カラーによるエッジ暗化

## ★★★ ツールデザインの黄金比（重要）★★★

### Minecraft/Unturned風ツールの比率

| 要素 | 比率 | 説明 |
|------|------|------|
| **ハンドル長** | 全長の60-70% | 持ち手が主役 |
| **ヘッド高さ** | 全長の30-40% | 金属部分は控えめ |
| **ヘッド幅** | ハンドル直径の4-6倍 | 横に広がる形 |
| **ハンドル太さ** | ヘッド高さの30-50% | しっかり握れる太さ |

```
【ピッケル参考プロポーション】
全長: 0.22 (約22cm)
├── ハンドル: 0.15 (長さ68%)  ← メイン
├── ヘッド: 0.06 (幅)
│   ├── 中央ブロック: 0.06 x 0.025 x 0.025
│   └── 先端（両側）: 0.04 x 0.02 x 0.015（テーパー）
└── ハンドル半径: 0.012

【斧の参考プロポーション】
全長: 0.22
├── ハンドル: 0.16 (長さ73%)
├── ヘッド: 0.08 x 0.06 x 0.02（片側に刃）
└── ハンドル半径: 0.012
```

### ツールの視認性向上ルール

1. **シルエットの明確化**
   - 離れて見ても何のツールか分かること
   - ピッケル = 両端が尖った横長ヘッド
   - 斧 = 片側に大きな刃
   - ハンマー = 四角いヘッド

2. **ヘッドの存在感**
   - ヘッドは「塊」として認識できる大きさ
   - 最小ヘッド幅: ハンドル直径の4倍以上
   - ヘッドの厚み: 薄すぎると見えない（最小0.02）

3. **ハンドルの握り込み表現**
   - グリップリング: 3-4本、等間隔
   - グリップ太さ: ハンドル+15%
   - 底部キャップ: ハンドル+20%、厚さ0.008

4. **カラーコントラスト**
   - ハンドル: wood（茶色）
   - ヘッド: iron/dark_steel（灰色/黒）
   - 明確に色を分けて認識性UP

## ★★★ 自己検証チェックリスト（スクショ前に必ず確認）★★★

モデル生成後、スクリーンショット確認の**前に**以下を検証:

### 1. 数値検証（Pythonで出力）
```python
# 必ず出力させる検証情報
print("=== MODEL VALIDATION ===")
print(f"Total height: {total_height:.4f} (expected: 0.18-0.25)")
print(f"Handle length: {handle_len:.4f} ({handle_len/total_height*100:.1f}% of total)")
print(f"Head width: {head_width:.4f}")
print(f"Head/handle ratio: {head_width/(handle_radius*2):.1f}x (expected: 4-6x)")
print(f"Parts connected: {are_parts_connected}")  # True/False
print(f"Triangle count: {tri_count}")
```

### 2. 比率チェック（自動判定）
```python
def validate_tool_proportions(handle_len, head_width, handle_radius, total_height):
    errors = []

    # ハンドル長は全長の60-70%
    handle_ratio = handle_len / total_height
    if not 0.60 <= handle_ratio <= 0.70:
        errors.append(f"Handle ratio {handle_ratio:.2f} not in 0.60-0.70")

    # ヘッド幅はハンドル直径の4-6倍
    head_handle_ratio = head_width / (handle_radius * 2)
    if not 4.0 <= head_handle_ratio <= 6.0:
        errors.append(f"Head/handle ratio {head_handle_ratio:.1f} not in 4-6x")

    # 全長チェック
    if not 0.18 <= total_height <= 0.25:
        errors.append(f"Total height {total_height:.3f} not in 0.18-0.25")

    if errors:
        print("❌ VALIDATION FAILED:")
        for e in errors:
            print(f"   - {e}")
        return False
    else:
        print("✅ Proportions OK")
        return True
```

### 3. 接続検証（座標出力）
```python
# 各パーツの境界を出力して接続確認
print(f"Handle top: {handle_top:.4f}")
print(f"Head center Z: {head_z:.4f}")
print(f"Head bottom: {head_z - head_h/2:.4f}")
print(f"Connection gap: {head_z - head_h/2 - handle_top:.4f} (should be <= 0)")
```

### 4. スクショ前の最終確認項目
- [ ] 全ての検証がパス（❌がない）
- [ ] 三角形数が予算内（ツール: 50-200）
- [ ] 接続ギャップが0以下（浮いていない）
- [ ] 比率が黄金比に従っている

## モデリング原則

### A. プリミティブ・キットバッシング戦略
**禁止**: 頂点移動などの複雑なメッシュ操作
**推奨**: 基本図形（Cube, Octagonal Prism）を配置して組み合わせる

```
❌ 頂点を動かしてモデリング
✅ 複数のプリミティブを適切な位置・角度・サイズで配置
```

### A-2. パーツ接続ルール（重要）
**禁止**: 浮いているパーツ、空中に分離したパーツ
**必須**: すべてのパーツは物理的に接触または重なり合うこと

```
❌ パーツ間に隙間がある（浮いている）
✅ パーツが接触または若干重なっている（0.001-0.005単位）

例：ハンマーヘッドとハンドル
❌ head_z = handle_top + 0.02  # 浮いている
✅ head_z = handle_top - head_height/2  # ヘッドの中心がハンドル上端に接続
```

**接続の計算方法**:
- 接続先の境界を計算（例: handle_top = handle_z + handle_length/2）
- 接続するパーツの中心を境界に配置
- または若干の重なり（overlap）を追加

### B. 座標系と階層構造
**必須**: すべてのパーツをルートEmpty（または統合後のオブジェクト）の子要素に
**原点**: 底面中心 (0, 0, 0)

```python
# 親Emptyを作成し、すべてのパーツをその子要素に
root = bpy.data.objects.new("MachineRoot", None)
bpy.context.collection.objects.link(root)
for part in parts:
    part.parent = root
```

### C. パーツの共通化（関数化）
`_base.py` の再利用可能パーツを活用:
- **ギア**: `create_gear(radius, thickness, teeth, hole_radius)`
- **シャフト**: `create_shaft(radius, length)`
- **パイプ**: `create_pipe(radius, length, wall)`
- **ボルト**: `create_bolt(size, length)`
- **ピストン**: `create_piston(rod_radius, rod_length, head_size)`
- **ローラー**: `create_roller(radius, length)`
- **コンベアベルト**: `create_conveyor_belt_segment(width, length, thickness)`
- **コンベアフレーム**: `create_conveyor_frame(width, length, height)`
- **サポート脚**: `create_support_leg(height, width)`

### D. カラーパレット・マテリアル
ローポリ感を出すため、単一テクスチャ or プリセットマテリアルを使用:

| プリセット | 用途 |
|-----------|------|
| `iron` | 鉄製パーツ、フレーム |
| `copper` | 配線、熱交換器 |
| `brass` | ギア、装飾 |
| `dark_steel` | 重機、産業機械 |
| `wood` | 初期段階、サポート構造 |
| `stone` | 基礎、炉 |

## Blender MCP連携

Blender MCPを使用したリアルタイム・フィードバック・ループ:

1. **コード生成** → Blenderで即座に実行
2. **結果確認** → エラーログ・オブジェクト一覧を取得
3. **自己修正** → 問題があれば修正して再実行

```
📝 スクリプト生成
   ↓
🔧 Blender MCP経由で実行
   ↓
👀 結果を視覚的に確認
   ↓
🔄 必要に応じて修正
```

### MCP連携ヘルパー関数
- `get_scene_info()`: シーン情報取得（オブジェクト一覧、三角形数）
- `validate_model(obj, category)`: モデルバリデーション
- `print_validation_report(obj, category)`: レポート出力

## 3Dモデル生成ワークフロー

「〇〇のモデルを作成して」という指示を受けたら:

1. **サブエージェント起動** (Task tool, subagent_type: general-purpose)
2. **プロンプト内容**:
```
tools/blender_scripts/_base.py を読み込み、以下のモデルのスクリプトを生成せよ。

【モデル】{ユーザー指定のモデル名}
【カテゴリ】{machine/item/structure}

【モデリング原則】
- プリミティブ・キットバッシング: 頂点移動禁止、基本図形の組み合わせのみ
- 階層構造: ルートEmptyから相対座標で配置
- 原点: 底面中心 (0, 0, 0)

【使用する関数】_base.pyから:
- プリミティブ: create_octagon, create_octagonal_prism, create_chamfered_cube, create_hexagon, create_trapezoid
- パーツ: create_gear, create_shaft, create_pipe, create_bolt, create_piston, create_roller
- 階層: create_root_empty, parent_to_root, join_all_meshes
- マテリアル: apply_preset_material(obj, "iron"/"copper"/"brass"/"dark_steel"/"wood"/"stone")
- アニメーション: create_rotation_animation, create_translation_animation
- 検証: validate_model, print_validation_report
- 仕上げ: finalize_model, export_gltf

【スクリプト構造】
exec(open("tools/blender_scripts/_base.py").read())
# ルートEmpty作成
# パーツ生成（プリミティブの組み合わせ）
# 階層構造設定
# マテリアル適用
# 結合
# アニメーション設定（必要時）
# バリデーション
# finalize_model + export_gltf

【Blender MCP使用時】
- 生成後、Blender MCPでプレビュー確認
- エラーがあれば修正して再実行

【出力】tools/blender_scripts/{model_name}.py
```

3. **サブエージェント完了後**: 結果をユーザーに報告

## ポリゴン予算

| カテゴリ | 推奨三角形数 | 最大 |
|----------|-------------|------|
| 手持ちアイテム | 50-200 | 500 |
| ドロップアイテム | 20-100 | 200 |
| 1ブロック機械 | 200-800 | 1500 |
| マルチブロック（小） | 500-2000 | 4000 |
| マルチブロック（大） | 2000-5000 | 10000 |

## スクリプト例

```python
exec(open("tools/blender_scripts/_base.py").read())

# シーンクリア
clear_scene()

# パーツ生成
parts = []

# ベースフレーム
frame = create_chamfered_cube(size=(1.0, 1.0, 0.3), location=(0, 0, 0.15), name="Frame")
parts.append(frame)

# ギア
gear = create_gear(radius=0.3, thickness=0.1, teeth=8, location=(0, 0, 0.4), name="Gear")
parts.append(gear)

# ボルト装飾
for i, pos in enumerate([(-0.4, -0.4), (0.4, -0.4), (-0.4, 0.4), (0.4, 0.4)]):
    bolt = create_bolt(size=0.05, length=0.08, location=(pos[0], pos[1], 0.31), name=f"Bolt_{i}")
    parts.append(bolt)

# 結合
machine = join_all_meshes(parts, name="MyMachine")

# マテリアル適用
apply_preset_material(machine, "iron")

# アニメーション（ギア回転）
# create_rotation_animation(gear, axis='Z', frames=30, rotations=1)

# バリデーション
print_validation_report(machine, category="machine")

# 仕上げ
finalize_model(machine, category="machine")

# エクスポート
export_gltf("assets/models/machines/my_machine.gltf")
```

## ★★★ ツール別デザインテンプレート ★★★

### ピッケル (Pickaxe)

**特徴**: 両端が尖った横長ヘッド、採掘用

```
    ◀━━━━━━━━━━━━━━━━▶  ← ヘッド（横に長い）
         ╱│     │╲         ← 先端は細くテーパー
        ╱ │     │ ╲
           │     │
           ┃     ┃  ← 中央の「塊」部分
           ┃     ┃
           ╱╲ ╱╲
           │││││  ← カラー（接続部）
           │ │ │
           │ │ │  ← ハンドル（長い！）
           │ │ │
           │ │ │
          ┌┴─┴┐    ← グリップ
          │   │
          └───┘    ← キャップ
```

**推奨値**:
```python
# ピッケルの推奨寸法
PICKAXE = {
    "total_height": 0.22,
    "handle_length": 0.15,      # 68% of total
    "handle_radius": 0.012,
    "head_width": 0.10,         # 4.2x handle diameter
    "head_depth": 0.025,
    "head_height": 0.03,
    "pick_tip_length": 0.04,    # 両端の尖った部分
    "pick_tip_taper": 0.6,      # 先端は60%細くなる
    "collar_radius": 0.016,
    "collar_height": 0.02,
}
```

### 斧 (Axe)

**特徴**: 片側に大きな刃、伐採用

```
          ┌─────┐
          │     │
          │ 刃  │←─ 大きな刃（片側のみ）
          │     │
          │     ╱
          │   ╱
          └─╱
          ╲╱ ← ヘッド本体
           │
           │  ← カラー
           │
           │  ← ハンドル
           │
           │
          [=]  ← グリップ
```

**推奨値**:
```python
AXEX = {
    "total_height": 0.22,
    "handle_length": 0.16,      # 73% of total
    "handle_radius": 0.012,
    "blade_width": 0.08,        # 刃の横幅
    "blade_height": 0.06,       # 刃の縦幅
    "blade_depth": 0.015,       # 刃の厚さ（薄い）
    "head_size": 0.03,          # 刃の付け根
}
```

### ハンマー (Hammer)

**特徴**: 四角いヘッド、叩く用

```
         ┌─────────┐
         │         │ ← 四角いヘッド
         │         │
         └────┬────┘
              │
              │  ← カラー
              │
              │  ← ハンドル
              │
             [=]  ← グリップ
```

**推奨値**:
```python
HAMMER = {
    "total_height": 0.20,
    "handle_length": 0.14,      # 70% of total
    "handle_radius": 0.011,
    "head_width": 0.06,
    "head_depth": 0.025,
    "head_height": 0.04,
}
```

### レンチ (Wrench)

**特徴**: 開いた口、ボルト回し用

```
         ╱   ╲
        │     │  ← 開いた口
        │ ╲ ╱ │
          │ │
          │ │  ← シャフト（細長い）
          │ │
          │ │
          ╲ ╱  ← グリップ端
```

### 共通ルール

1. **マテリアル分離**
   - ハンドル: `wood` マテリアル
   - ヘッド/金属部: `iron` または `dark_steel` マテリアル
   - 2つのオブジェクトに分けてマテリアル適用後、結合

2. **パーツ構成**
   ```
   ツール
   ├── ハンドル (octagonal_prism)
   │   ├── グリップリング x 3-4 (octagonal_prism, 太め)
   │   └── キャップ (octagonal_prism)
   ├── カラー (octagonal_prism, 接続部)
   └── ヘッド (chamfered_cube + 追加パーツ)
   ```

3. **スケール確認**
   - ゲーム内での見え方を意識
   - 1ブロック = 1.0ユニット
   - ツール = 約0.2ブロック高さ（手に持って見える大きさ）
