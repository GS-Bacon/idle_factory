# 工作機械アニメーションガイド

このドキュメントでは、工作機械（Kinetic Machines）のアニメーションに必要なアセットファイルについて説明します。

## 概要

各工作機械は加工中にアニメーションを再生します。アニメーションは`MachineAnimation`コンポーネントで管理され、フレームベースで進行します。

## 必要なアセット

### 1. プレス機（Mechanical Press）

**アニメーション仕様:**
- フレーム数: 8
- フレーム間隔: 50ms
- 総アニメーション時間: 400ms

**必要ファイル:**
```
assets/textures/machines/mechanical_press/
├── frame_0.png   # 待機状態（上位置）
├── frame_1.png   # 下降開始
├── frame_2.png   # 下降中
├── frame_3.png   # プレス接触
├── frame_4.png   # プレス圧縮（最下点）
├── frame_5.png   # 上昇開始
├── frame_6.png   # 上昇中
└── frame_7.png   # 上昇完了
```

**3Dモデル（オプション）:**
```
assets/models/machines/mechanical_press.gltf
```
- ボーン: `press_head`（上下移動）
- アニメーション: `press_cycle`

---

### 2. 粉砕機（Crusher）

**アニメーション仕様:**
- フレーム数: 12
- フレーム間隔: 40ms
- 総アニメーション時間: 480ms

**必要ファイル:**
```
assets/textures/machines/crusher/
├── frame_0.png   # 待機状態
├── frame_1.png   # 回転開始
├── frame_2.png   # 回転中（30°）
├── frame_3.png   # 回転中（60°）
├── frame_4.png   # 回転中（90°）
├── frame_5.png   # 回転中（120°）
├── frame_6.png   # 回転中（150°）
├── frame_7.png   # 回転中（180°）
├── frame_8.png   # 回転中（210°）
├── frame_9.png   # 回転中（240°）
├── frame_10.png  # 回転中（270°）
└── frame_11.png  # 回転中（300°）
```

**3Dモデル（オプション）:**
```
assets/models/machines/crusher.gltf
```
- ボーン: `crusher_wheel`（Y軸回転）
- アニメーション: `crush_cycle`

---

### 3. 自動ノコギリ（Mechanical Saw）

**アニメーション仕様:**
- フレーム数: 6
- フレーム間隔: 30ms
- 総アニメーション時間: 180ms（高速回転）

**必要ファイル:**
```
assets/textures/machines/mechanical_saw/
├── frame_0.png   # 刃位置A
├── frame_1.png   # 刃位置B（回転ブラー）
├── frame_2.png   # 刃位置C
├── frame_3.png   # 刃位置D（回転ブラー）
├── frame_4.png   # 刃位置E
└── frame_5.png   # 刃位置F（回転ブラー）
```

**3Dモデル（オプション）:**
```
assets/models/machines/mechanical_saw.gltf
```
- ボーン: `saw_blade`（Z軸回転）
- アニメーション: `saw_spin`

---

### 4. ミキサー（Mixer）

**アニメーション仕様:**
- フレーム数: 16
- フレーム間隔: 60ms
- 総アニメーション時間: 960ms

**必要ファイル:**
```
assets/textures/machines/mixer/
├── frame_0.png    # 待機状態
├── frame_1.png    # 攪拌開始
├── frame_2.png    # 攪拌中（羽根位置1）
├── ...
└── frame_15.png   # 攪拌完了
```

**流体表示用テクスチャ:**
```
assets/textures/machines/mixer/
├── fluid_empty.png     # 空タンク
├── fluid_low.png       # 25%以下
├── fluid_medium.png    # 50%程度
├── fluid_high.png      # 75%程度
└── fluid_full.png      # 満杯
```

**3Dモデル（オプション）:**
```
assets/models/machines/mixer.gltf
```
- ボーン: `mixer_blade`（Y軸回転）、`fluid_surface`（スケール変更）
- アニメーション: `mix_cycle`

---

### 5. 伸線機（Wire Drawer）

**アニメーション仕様:**
- フレーム数: 10
- フレーム間隔: 50ms
- 総アニメーション時間: 500ms

**必要ファイル:**
```
assets/textures/machines/wire_drawer/
├── frame_0.png    # 板挿入
├── frame_1.png    # 引き込み開始
├── frame_2.png    # 引き込み中
├── frame_3.png    # ダイス通過1
├── frame_4.png    # ダイス通過2
├── frame_5.png    # 細線化進行
├── frame_6.png    # 細線化進行
├── frame_7.png    # 排出開始
├── frame_8.png    # 排出中
└── frame_9.png    # 排出完了
```

**3Dモデル（オプション）:**
```
assets/models/machines/wire_drawer.gltf
```
- ボーン: `feed_roller`（Z軸回転）、`draw_die`（固定）
- アニメーション: `draw_cycle`

---

## テクスチャ仕様

### 推奨解像度
- UIアイコン: 64x64 または 128x128
- 3Dテクスチャ: 256x256 または 512x512

### フォーマット
- PNG（透過対応）
- または KTX2（Bevy推奨、圧縮対応）

### 命名規則
```
{machine_name}/frame_{number}.png
{machine_name}/icon.png           # インベントリアイコン
{machine_name}/preview.png        # ビルドプレビュー用
```

---

## 実装方法

### テクスチャアニメーション（2D）

```rust
// render.rsで実装
fn animate_machine_textures(
    mut query: Query<(&MachineAnimation, &mut Sprite)>,
    textures: Res<MachineTextures>,
) {
    for (anim, mut sprite) in &mut query {
        sprite.image = textures.get_frame(anim.frame);
    }
}
```

### ボーンアニメーション（3D）

```rust
// GLTFアニメーションを使用
fn animate_machine_models(
    mut query: Query<(&MachineAnimation, &mut AnimationPlayer)>,
) {
    for (anim, mut player) in &mut query {
        player.seek_to(anim.progress());
    }
}
```

---

## アセット作成ツール

推奨ツール:
- **Blender**: 3Dモデルとアニメーション
- **Aseprite**: ピクセルアートテクスチャ
- **Krita/GIMP**: 高解像度テクスチャ

Blenderエクスポート設定:
- フォーマット: glTF 2.0 (.gltf)
- アニメーション: 含める
- マテリアル: PBR

---

## 優先度

1. **高優先度**: アイコン（icon.png）- UIに必須
2. **中優先度**: プレビュー（preview.png）- ビルド時に表示
3. **低優先度**: アニメーションフレーム - なくても動作可能

アニメーションフレームがない場合、システムは静止画を表示します。

---

## サンプルYAML定義

`assets/data/machines/kinetic.yaml`:

```yaml
- id: mechanical_press
  name: "Mechanical Press"
  texture: "machines/mechanical_press/icon.png"
  model: "machines/mechanical_press.gltf"
  animation:
    frames: 8
    frame_duration: 0.05
  stress_impact: 8.0
  work_type: pressing

- id: crusher
  name: "Crusher"
  texture: "machines/crusher/icon.png"
  model: "machines/crusher.gltf"
  animation:
    frames: 12
    frame_duration: 0.04
  stress_impact: 12.0
  work_type: crushing

- id: mechanical_saw
  name: "Mechanical Saw"
  texture: "machines/mechanical_saw/icon.png"
  model: "machines/mechanical_saw.gltf"
  animation:
    frames: 6
    frame_duration: 0.03
  stress_impact: 4.0
  work_type: cutting

- id: mixer
  name: "Mixer"
  texture: "machines/mixer/icon.png"
  model: "machines/mixer.gltf"
  animation:
    frames: 16
    frame_duration: 0.06
  stress_impact: 16.0
  work_type: mixing
  has_fluid_tank: true

- id: wire_drawer
  name: "Wire Drawer"
  texture: "machines/wire_drawer/icon.png"
  model: "machines/wire_drawer.gltf"
  animation:
    frames: 10
    frame_duration: 0.05
  stress_impact: 6.0
  work_type: wire_drawing
```

---

*このガイドは開発中のため、仕様は変更される可能性があります。*
