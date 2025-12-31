# ビルド設定

## プロファイル一覧

| プロファイル | 用途 | 時間 |
|-------------|------|------|
| dev | 開発 | **2秒** |
| dev-fast | 開発（最適化あり） | 2.5秒 |
| release-fast | リリーステスト | **10秒** |
| release | 最終リリース | 29秒 |

開発中は `cargo build`（devプロファイル）を使用。

## 設定ファイル

**.cargo/config.toml**:
```toml
[build]
jobs = 80
rustc-wrapper = "/home/bacon/.cargo/bin/sccache"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**Cargo.toml**:
```toml
[profile.dev]
split-debuginfo = "unpacked"

[profile.dev.package."*"]
opt-level = 3

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 16
strip = true

[profile.release-fast]
inherits = "release"
lto = false
opt-level = 2
```

## 必要ツール

```bash
sudo apt-get install -y mold
cargo install sccache --locked
```

## 最適化のポイント

- **split-debuginfo = "unpacked"**: デバッグ情報分離（4.3秒→2秒）
- **lto = false**: LTO無効で高速（29秒→10秒）
- **sccache**: コンパイル結果キャッシュ
- **mold**: 高速リンカー
- **jobs = 80**: 80コア並列

## WASMビルド

```bash
./deploy-wasm.sh          # 手動デプロイ
./watch-wasm.sh           # 自動リビルド（ファイル監視）
```

## 注意

- 複数プロファイルでtargetが肥大化（21GB超）
- 通常開発は1-2GB
- 容量不足時: `cargo clean`
