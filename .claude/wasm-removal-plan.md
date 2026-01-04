# WASM廃止計画

## 背景

- WASMはプレイ確認を簡単にする目的だったが、アップデータ導入で手元確認が可能になった
- WASMサポートは不要になったため廃止

## 削除対象一覧

| カテゴリ | ファイル/設定 | 状態 |
|----------|---------------|------|
| **スクリプト** | `scripts/*wasm*.sh` (5ファイル) | ✅ 削除済み |
| **ディレクトリ** | `web/` (~47MB) | [ ] |
| **GitHub CI** | `.github/workflows/ci.yml` のwasmジョブ (100-123行) | [ ] |
| **Cargo.toml** | WASM依存 (68-80行) | [ ] |
| | `profile.wasm-release` (109-116行) | [ ] |
| **.cargo/config.toml** | WASM設定 (6-9行) | [ ] |
| **ソースコード** | `#[cfg(target_arch = "wasm32")]` 分岐 | [ ] |

## ソースコード内WASM分岐（削除対象）

| ファイル | 行 | 内容 |
|----------|-----|------|
| `main.rs` | 13, 72-73, 107-130 | WASM初期化、パニックフック、プラグイン |
| `constants.rs` | 25-26 | WASM用VIEW_DISTANCE |
| `logging.rs` | 85-89, 121-125, 253+ | WASM用ログ設定 |
| `world/mod.rs` | 41-44 | PendingChunkのWASM分岐 |

## Cargo.toml削除対象

```toml
# 68-80行: WASM依存
[target.'cfg(target_arch = "wasm32")'.dependencies.bevy]
[target.'cfg(target_arch = "wasm32")'.dependencies]
web-sys, getrandom, console_error_panic_hook, tracing-wasm, web-time

# 109-116行: WASMプロファイル
[profile.wasm-release]
```

## .cargo/config.toml削除対象

```toml
# 6-9行
[target.wasm32-unknown-unknown]
rustflags = ["--cfg=getrandom_backend=\"wasm_js\""]
runner = "wasm-bindgen-test-runner"
```

## ci.yml削除対象

```yaml
# 100-123行: wasmジョブ全体
wasm:
  name: WASM Build
  ...
```

## 実行手順

1. [ ] `rm -rf web/`
2. [ ] `ci.yml` からwasmジョブ削除 (100-123行)
3. [ ] `Cargo.toml` からWASM設定削除 (68-80行, 109-116行)
4. [ ] `.cargo/config.toml` からWASM設定削除 (6-9行)
5. [ ] ソースコードからWASM分岐削除
   - [ ] `main.rs`
   - [ ] `constants.rs`
   - [ ] `logging.rs`
   - [ ] `world/mod.rs`
6. [ ] `cargo build && cargo test` で確認
7. [ ] コミット＆プッシュ

## 効果

- **コード削減**: WASM分岐コード (~100行)
- **依存削減**: 5パッケージ (web-sys, getrandom, console_error_panic_hook, tracing-wasm, web-time)
- **CI時間短縮**: WASMビルドジョブ削除
- **ディスク**: ~47MB削減 (web/)
- **メンテナンス**: WASM互換性を考慮する必要がなくなる
