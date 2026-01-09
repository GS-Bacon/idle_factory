# Resource Packs

このディレクトリにリソースパックを配置してください。

## 構造

```
resource_packs/
  my_pack/
    pack.toml          # 必須: パックメタデータ
    assets/
      textures/
        blocks/        # ブロックテクスチャ
          stone.png
        items/         # アイテムテクスチャ
      models/          # カスタムモデル (JSON)
      blockstates/     # ブロックステート定義 (JSON)
```

## pack.toml の形式

```toml
[pack]
id = "my_pack"
name = "My Resource Pack"
version = "1.0.0"
description = "カスタムテクスチャパック"
authors = ["Your Name"]
```

## 優先順位

1. リソースパック（後から読み込んだものが優先）
2. MOD
3. ベースゲーム

## サンプル

`example_pack/` を参照してください。
