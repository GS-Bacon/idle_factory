# コンベア仕様メモ（2026-01-01 確定）

## 座標系

```
画面上の配置:
     0   1   2   3   4   5   6  (X軸/横)
   ┌───┬───┬───┬───┬───┬───┬───┐
 A │   │   │   │   │   │   │   │  ← 上がNorth(-Z)
   ├───┼───┼───┼───┼───┼───┼───┤
 B │   │   │   │   │   │   │   │
   ├───┼───┼───┼───┼───┼───┼───┤
 C │   │   │   │   │   │   │   │
   ├───┼───┼───┼───┼───┼───┼───┤
 D │   │   │   │   │   │   │   │
   ├───┼───┼───┼───┼───┼───┼───┤
 E │   │   │   │   │   │   │   │
   ├───┼───┼───┼───┼───┼───┼───┤
 F │   │   │   │   │   │   │   │  ← 下がSouth(+Z)
   └───┴───┴───┴───┴───┴───┴───┘
   ← West(-X)            East(+X) →
```

## Direction定義

| Direction | ベクトル | 画面上 |
|-----------|---------|--------|
| North | (0, 0, -1) | ↑ 上へ |
| South | (0, 0, +1) | ↓ 下へ |
| East | (+1, 0, 0) | → 右へ |
| West | (-1, 0, 0) | ← 左へ |

## left() / right() 定義（進行方向を見て）

| Direction | left() | right() |
|-----------|--------|---------|
| North(↑) | West(←) | East(→) |
| East(→) | North(↑) | South(↓) |
| South(↓) | East(→) | West(←) |
| West(←) | South(↓) | North(↑) |

## ConveyorShape判定ロジック

1. 自分の「左側」の位置にあるコンベアが自分へフィードしていれば → **CornerLeft**
2. 自分の「右側」の位置にあるコンベアが自分へフィードしていれば → **CornerRight**
3. 両方からフィードされていれば → **TJunction**
4. どちらからもなければ → **Straight**

## 検証済みパターン

### パターン1: East向きコンベアに南から合流 ✓
```
     3   4
   ┌───┬───┐
 E │ → │ → │  (3,E)=CornerRight, (4,E)=Straight
   ├───┼───┤
 F │ ↑ │   │  (3,F)=Straight
   └───┴───┘

判定:
- (3,E) East向き
- East.right() = South = 下方向 = (3,F)
- (3,F)↑は(3,E)へフィードしている
- → (3,E)は CornerRight ✓
```

## 3Dモデル仕様

### モデルのデフォルト向き: North(-Z)

| モデル | 入力側（North向き時） |
|--------|---------------------|
| corner_left.glb | -X側（左側）から入力 |
| corner_right.glb | +X側（右側）から入力 |

### 回転適用

```rust
Direction::North => Quat::from_rotation_y(0.0)      // 0°
Direction::East  => Quat::from_rotation_y(-PI/2.0)  // -90°（時計回り）
Direction::South => Quat::from_rotation_y(PI)       // 180°
Direction::West  => Quat::from_rotation_y(PI/2.0)   // 90°（反時計回り）
```

## ビジュアライザー

### ファイル
`tools/conveyor_visualizer.html`

### 起動方法
```bash
cd /home/bacon/idle_factory
python3 -m http.server 8081 --bind 0.0.0.0
# または
simple-http-server -p 8081 .
```

### アクセスURL
http://100.84.170.32:8081/tools/conveyor_visualizer.html

### 使い方
1. 方向ボタン（↑North, ↓South, →East, ←West）で設置方向を選択
2. グリッドをクリックでコンベア設置（既存セルはクリックで回転）
3. 右クリックで削除
4. 色で形状を判別:
   - 青系: Straight
   - 黄系: CornerLeft (L)
   - 水色系: CornerRight (R)
   - 紫系: TJunction (T)

### ロジック
ゲームコード（Rust）と同じロジックをJavaScriptで実装:
- `feedsInto(ox, oz, tx, tz)`: 位置(ox,oz)のコンベアが(tx,tz)へ出力するか
- `calculateShape()`: 左右からのフィードを確認して形状決定
- `LEFT`/`RIGHT`マップ: Direction.left()/right()と同等

このビジュアライザーのロジックは正しいことを確認済み（2026-01-01）

## 形状判定ロジック（2026-01-01 最終版）

### 基本ルール

```
入力可能な方向: 4方向すべて（後ろ、前、左、右）
出力可能な方向: 入力がない方向（後ろ以外）
  - 入力がある方向には出力しない（衝突防止）
  - 後ろには出力しない（基本ルール）
```

### 入力判定

```javascript
hasBackInput  = 後ろのコンベアが自分に出力しているか
hasFrontInput = 前のコンベアが自分に出力しているか
hasLeftInput  = 左のコンベアが自分に出力しているか
hasRightInput = 右のコンベアが自分に出力しているか
```

### 出力判定

```javascript
canGoFront = !hasFrontInput && 前にコンベアあり && 受け入れ可能
canGoLeft  = !hasLeftInput  && 左にコンベアあり && 受け入れ可能
canGoRight = !hasRightInput && 右にコンベアあり && 受け入れ可能
```

### 形状分類

| 入力数 | 条件 | 形状 | 説明 |
|--------|------|------|------|
| 2+ | 出力1 | TJunction | 合流 |
| 2+ | 出力2+ | Cross | 交差 |
| 1 | 実際の接続先3 | Cross | 分散 |
| 1 | 実際の接続先2 | Splitter | 分岐 |
| 1 | 後ろ入力、前方接続なし、右接続あり | CornerRight | 右曲がり |
| 1 | 後ろ入力、前方接続なし、左接続あり | CornerLeft | 左曲がり |
| 1 | 左から入力 | CornerLeft | 左から前へ |
| 1 | 右から入力 | CornerRight | 右から前へ |
| 0 | - | Straight | デフォルト |

### 重要なポイント

1. **前方からの入力も受け付ける**: 向かい合うコンベアからの入力OK
2. **入力がある方向には出力しない**: 衝突防止
3. **接続先がない場合はStraight**: 孤立したコンベアは前方出力

## 現在の状態

- [x] ビジュアライザーのロジック確認済み
- [x] ロジック整理済み（入力数×出力数で分類）
- [x] Blenderスクリプト修正済み (corner_left.py, corner_right.py)
- [x] GLBモデル再生成済み
- [ ] ゲーム内での動作確認
