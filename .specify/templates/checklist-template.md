# [機能名] チェックリスト

## ⛔ 禁止パターン確認（最優先）

<!-- 最初にこれを確認。1つでも失敗したら実装やり直し -->

- [ ] `grep -r "禁止パターン1" src/` が0件
- [ ] `grep -r "禁止パターン2" src/` が0件

---

## ビルド・テスト

- [ ] `cargo build` 通過
- [ ] `cargo test` 通過
- [ ] `cargo clippy` 警告0件

---

## 機能要件

<!-- spec.mdのFR-xxxと対応 -->

- [ ] FR-001: [要件名]
- [ ] FR-002: [要件名]

---

## コード品質

- [ ] `// TODO`, `// FIXME` がない
- [ ] `unwrap()` がない（または適切な理由がある）
- [ ] テストカバレッジ追加済み

---

## 統合確認

- [ ] 新Systemが `Plugin::build()` に登録されている
- [ ] 新PluginがGameに登録されている
- [ ] 新イベントが `app.add_event::<>()` されている

---

## ドキュメント更新

- [ ] `.claude/implementation-plan.md` のタスク状態更新
- [ ] 必要に応じて `.claude/architecture.md` 更新
- [ ] 必要に応じて `AGENTS.md` 更新

---

## 最終確認コマンド

```bash
# 全部まとめて実行
cargo build && cargo test && cargo clippy && \
grep -r "禁止パターン" src/ && echo "❌ FAIL" || echo "✅ ALL PASS"
```

---

*最終更新: YYYY-MM-DD*
