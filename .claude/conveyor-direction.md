# コンベア方向定義

## 座標系

```
       -Z (North)
         ^
         |
-X (West) --+-- +X (East)
         |
         v
       +Z (South)
```

Y軸は上向き。Minecraftと同じ座標系。

## Direction列挙型

```rust
pub enum Direction {
    North,  // -Z方向
    East,   // +X方向
    South,  // +Z方向
    West,   // -X方向
}
```

## 「左」「右」の定義

進行方向に対して時計回りに90度 = 右
進行方向に対して反時計回りに90度 = 左

```
      North
        ^
        |
  West <--+--> East (右)
   (左)   |
        v
      South
```

### Direction::left() / right() の動作

| 向き | left() | right() |
|------|--------|---------|
| North | West | East |
| East | North | South |
| South | East | West |
| West | South | North |

## コンベア形状

### CornerLeft（左曲がり）

入力: 後ろから
出力: 左へ

```
    ^
    |
----+
入力 → 出力
```

例: North向きCornerLeftの場合
- 入力: South (0, 0, +1) からアイテムが来る
- 出力: West (-1, 0, 0) へアイテムが行く

### CornerRight（右曲がり）

入力: 後ろから
出力: 右へ

```
    ^
    |
    +----
入力 → 出力
```

例: North向きCornerRightの場合
- 入力: South (0, 0, +1) からアイテムが来る
- 出力: East (+1, 0, 0) へアイテムが行く

## 3Dモデルの対応

Blenderでモデルを作成する際:
- モデルは-Z方向（North）を向いているものとして作成
- CornerLeft: アイテムが後ろから来て左（-X）へ出る
- CornerRight: アイテムが後ろから来て右（+X）へ出る

## テストケース

```rust
// North向き: left=West, right=East
assert_eq!(Direction::North.left(), Direction::West);
assert_eq!(Direction::North.right(), Direction::East);

// East向き: left=North, right=South  
assert_eq!(Direction::East.left(), Direction::North);
assert_eq!(Direction::East.right(), Direction::South);
```

## 実装ファイル

- `src/components/machines.rs` - Direction列挙型、left()/right()実装
- `src/main.rs` - テストケース
- `tests/e2e_test.rs` - L字コンベアテスト
