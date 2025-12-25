# .specify (AI compressed)

SDD(Spec-Driven-Dev)用設定

## structure

```
.specify/
├─memory/
│ ├─constitution.md   # 原則・標準
│ ├─changelog.md      # 開発履歴
│ ├─issues.md         # 課題管理
│ ├─patterns-compact.md # 52パターン
│ └─modeling-rules.md # 3Dモデリング
├─specs/
│ └─index-compact.md  # 全仕様集約
└─templates/          # テンプレート
```

## workflow

1. `/speckit.constitution` - 原則確認
2. `/speckit.specify` - 仕様定義→specs/feature.md
3. `/speckit.plan` - 実装計画→specs/feature-plan.md
4. `/speckit.tasks` - タスク分割→specs/feature-tasks.md
5. `/speckit.implement` - 実装

## project-guidelines

arch:ecs,data-driven(yaml),deterministic
test:70%+(core90%+),tdd
perf:60fps,instancing(1000+items),async-chunk
