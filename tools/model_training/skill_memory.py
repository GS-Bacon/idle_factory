"""
Skill Memory - 汎用モデリングスキル記憶システム

課題固有のパターンではなく、汎用的なモデリングスキルを学習・蓄積。
新しい課題に過去の学習を自動適用する。

構造:
- principles: 普遍的な原則（例: "八角形は円の代替"）
- techniques: 具体的な技法（例: "ハンドルはoct_prism + grip_rings"）
- mistakes: 避けるべきミス（例: "cylinderは使わない"）
- code_patterns: 成功したコードパターン
"""

import json
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List, Optional

SKILL_MEMORY_PATH = Path(__file__).parent.parent / "model_training_data" / "skill_memory.json"


class SkillMemory:
    """
    汎用モデリングスキルの記憶システム。

    Usage:
        memory = SkillMemory()

        # 成功から学習
        memory.learn_from_success(evaluation, challenge, code)

        # 失敗から学習
        memory.learn_from_failure(evaluation, challenge, feedback)

        # 新しい課題に適用するプロンプトを生成
        prompt = memory.generate_guidance_prompt(new_challenge)
    """

    def __init__(self, path: Optional[Path] = None):
        self.path = path or SKILL_MEMORY_PATH
        self.data = self._load()

    def _load(self) -> Dict[str, Any]:
        if self.path.exists():
            with open(self.path, "r", encoding="utf-8") as f:
                return json.load(f)
        return self._create_initial_memory()

    def _save(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2, ensure_ascii=False)

    def _create_initial_memory(self) -> Dict[str, Any]:
        """初期スキル記憶（基本原則をシード）"""
        return {
            "version": "1.0.0",
            "last_updated": datetime.now().isoformat(),
            "skill_level": 1,  # 1-10のスキルレベル
            "total_successes": 0,
            "total_attempts": 0,

            # 普遍的な原則（カテゴリ非依存）
            "principles": [
                {
                    "id": "P001",
                    "rule": "円形はoctagon/octagonal_prismで代替する",
                    "reason": "ローポリスタイルの一貫性",
                    "confidence": 1.0,
                    "source": "seed",
                },
                {
                    "id": "P002",
                    "rule": "マテリアルはMATERIALSプリセットのみ使用",
                    "reason": "Astroneer風テクスチャレススタイル",
                    "confidence": 1.0,
                    "source": "seed",
                },
                {
                    "id": "P003",
                    "rule": "パーツ結合後にapply_edge_darkening(0.85)を適用",
                    "reason": "立体感の強調",
                    "confidence": 1.0,
                    "source": "seed",
                },
                {
                    "id": "P004",
                    "rule": "finalize_model()でカテゴリに応じた原点設定",
                    "reason": "ゲーム内配置の正確性",
                    "confidence": 1.0,
                    "source": "seed",
                },
            ],

            # カテゴリ別技法
            "techniques": {
                "tool": [
                    {
                        "id": "T001",
                        "name": "tool_handle",
                        "description": "ツールハンドルの標準構成",
                        "pattern": {
                            "base": "octagonal_prism",
                            "grip_rings": 3,
                            "grip_ratio": 1.15,
                            "cap_at_bottom": True,
                        },
                        "code_snippet": "handle = create_octagonal_prism(0.012, 0.15, (0,0,0), 'Handle')",
                        "success_count": 2,
                        "source": "seed",
                    },
                    {
                        "id": "T002",
                        "name": "tool_head_connection",
                        "description": "ヘッドとハンドルの接続",
                        "pattern": {
                            "use_collar": True,
                            "collar_material": "iron",
                            "overlap": 0.003,
                        },
                        "code_snippet": "collar = create_octagonal_prism(0.015, 0.02, (0,0,handle_top), 'Collar')",
                        "success_count": 2,
                        "source": "seed",
                    },
                ],
                "machine": [
                    {
                        "id": "T010",
                        "name": "machine_frame",
                        "description": "機械のベースフレーム構成",
                        "pattern": {
                            "base": "chamfered_cube",
                            "fill_ratio": 0.85,
                            "material": "dark_steel",
                        },
                        "code_snippet": "frame = create_machine_frame(0.9, 0.9, 0.3, 'dark_steel')",
                        "success_count": 3,
                        "source": "seed",
                    },
                    {
                        "id": "T011",
                        "name": "corner_bolts",
                        "description": "四隅のボルト装飾",
                        "pattern": {
                            "count": 4,
                            "offset": 0.4,
                            "material": "brass",
                        },
                        "code_snippet": "bolts = create_corner_bolts(0.9, 0.9, z_pos, 0.04, 'brass')",
                        "success_count": 3,
                        "source": "seed",
                    },
                ],
            },

            # 避けるべきミス
            "mistakes": [
                {
                    "id": "M001",
                    "mistake": "bpy.ops.mesh.primitive_cylinder_addの使用",
                    "fix": "create_octagonal_prism()を使用",
                    "frequency": 0,
                    "severity": "high",
                },
                {
                    "id": "M002",
                    "mistake": "カスタムRGBカラーの直接指定",
                    "fix": "apply_preset_material()を使用",
                    "frequency": 0,
                    "severity": "medium",
                },
                {
                    "id": "M003",
                    "mistake": "パーツ間に隙間がある",
                    "fix": "0.003-0.005のオーバーラップを確保",
                    "frequency": 0,
                    "severity": "medium",
                },
            ],

            # 成功したコードパターン（抽象化済み）
            "code_patterns": {},
        }

    def learn_from_success(
        self,
        evaluation: Dict[str, Any],
        challenge: Dict[str, Any],
        code: Optional[str] = None
    ) -> List[str]:
        """
        成功から汎用パターンを学習。

        Args:
            evaluation: 評価結果
            challenge: 課題定義
            code: 生成に使用したコード（オプション）

        Returns:
            学習した項目のリスト
        """
        learned = []
        category = challenge.get("category", "tool")
        scores = evaluation.get("scores", {})

        self.data["total_successes"] += 1
        self.data["total_attempts"] += 1

        # 高スコア基準から技法を抽出
        for criterion, score in scores.items():
            if score >= 9.0:
                # この基準で優秀 → 関連技法の信頼度UP
                self._boost_technique_confidence(category, criterion)
                learned.append(f"Boosted confidence for {criterion} techniques")

        # コードパターンの抽出（簡易版）
        if code:
            pattern_id = f"{challenge.get('id', 'unknown')}_{datetime.now().strftime('%Y%m%d')}"
            self.data["code_patterns"][pattern_id] = {
                "category": category,
                "challenge": challenge.get("id"),
                "score": evaluation.get("total_score", 0),
                "timestamp": datetime.now().isoformat(),
                "code_hash": hash(code) % 10000,  # 簡易識別用
            }
            learned.append(f"Stored code pattern: {pattern_id}")

        # スキルレベル更新
        self._update_skill_level()

        self.data["last_updated"] = datetime.now().isoformat()
        self._save()

        return learned

    def learn_from_failure(
        self,
        evaluation: Dict[str, Any],
        challenge: Dict[str, Any],
        feedback: str
    ) -> List[str]:
        """
        失敗から避けるべきパターンを学習。

        Args:
            evaluation: 評価結果
            challenge: 課題定義
            feedback: 生成された改善フィードバック

        Returns:
            学習した項目のリスト
        """
        learned = []
        scores = evaluation.get("scores", {})
        details = evaluation.get("details", {})

        self.data["total_attempts"] += 1

        # 低スコア基準からミスパターンを抽出
        for criterion, score in scores.items():
            if score < 5.0:
                detail = details.get(criterion, {})
                violations = detail.get("violations", [])

                for violation in violations[:2]:  # 上位2つのみ
                    self._record_mistake(criterion, violation)
                    learned.append(f"Recorded mistake in {criterion}")

        self.data["last_updated"] = datetime.now().isoformat()
        self._save()

        return learned

    def _boost_technique_confidence(self, category: str, criterion: str) -> None:
        """成功した技法の信頼度を上げる"""
        techniques = self.data["techniques"].get(category, [])
        for tech in techniques:
            # 関連する技法を見つけて信頼度UP
            if criterion in tech.get("related_criteria", [criterion]):
                tech["success_count"] = tech.get("success_count", 0) + 1

    def _record_mistake(self, criterion: str, violation: Any) -> None:
        """ミスパターンを記録"""
        # 既存のミスか確認
        for mistake in self.data["mistakes"]:
            if criterion in mistake.get("mistake", ""):
                mistake["frequency"] = mistake.get("frequency", 0) + 1
                return

        # 新しいミスを追加
        violation_str = str(violation) if not isinstance(violation, str) else violation
        self.data["mistakes"].append({
            "id": f"M{len(self.data['mistakes']) + 1:03d}",
            "mistake": f"{criterion}: {violation_str[:50]}",
            "fix": "See feedback for details",
            "frequency": 1,
            "severity": "medium",
            "learned_at": datetime.now().isoformat(),
        })

    def _update_skill_level(self) -> None:
        """スキルレベルを更新"""
        total = self.data["total_attempts"]
        successes = self.data["total_successes"]

        if total == 0:
            return

        success_rate = successes / total

        # 成功率と試行回数に基づくレベル計算
        base_level = min(10, 1 + int(success_rate * 5) + min(4, total // 10))
        self.data["skill_level"] = base_level

    def generate_guidance_prompt(self, challenge: Dict[str, Any]) -> str:
        """
        新しい課題に対するガイダンスプロンプトを生成。

        過去の学習を活かした具体的なアドバイスを含む。

        Args:
            challenge: 新しい課題定義

        Returns:
            ガイダンスプロンプト
        """
        category = challenge.get("category", "tool")

        lines = [
            f"# モデリングガイダンス（スキルLv.{self.data['skill_level']}）",
            "",
            "## 基本原則（必ず守る）",
            "",
        ]

        # 原則
        for p in self.data["principles"]:
            lines.append(f"- **{p['rule']}** ({p['reason']})")

        lines.append("")
        lines.append(f"## {category.upper()}カテゴリの推奨技法")
        lines.append("")

        # カテゴリ別技法
        techniques = self.data["techniques"].get(category, [])
        for tech in sorted(techniques, key=lambda x: -x.get("success_count", 0))[:5]:
            lines.append(f"### {tech['name']}")
            lines.append(f"{tech['description']}")
            if tech.get("code_snippet"):
                lines.append(f"```python")
                lines.append(tech["code_snippet"])
                lines.append(f"```")
            lines.append("")

        # 避けるべきミス
        frequent_mistakes = [m for m in self.data["mistakes"] if m.get("frequency", 0) > 0]
        if frequent_mistakes:
            lines.append("## 避けるべきミス")
            lines.append("")
            for m in sorted(frequent_mistakes, key=lambda x: -x.get("frequency", 0))[:3]:
                lines.append(f"- ❌ {m['mistake']} → ✅ {m['fix']}")
            lines.append("")

        # 統計
        lines.append("---")
        lines.append(f"*成功率: {self.data['total_successes']}/{self.data['total_attempts']} | ")
        lines.append(f"スキルLv: {self.data['skill_level']}/10*")

        return "\n".join(lines)

    def get_skill_level(self) -> int:
        """現在のスキルレベルを取得"""
        return self.data.get("skill_level", 1)

    def get_statistics(self) -> Dict[str, Any]:
        """統計情報を取得"""
        return {
            "skill_level": self.data.get("skill_level", 1),
            "total_successes": self.data.get("total_successes", 0),
            "total_attempts": self.data.get("total_attempts", 0),
            "success_rate": (
                self.data["total_successes"] / self.data["total_attempts"]
                if self.data["total_attempts"] > 0 else 0
            ),
            "principles_count": len(self.data.get("principles", [])),
            "techniques_count": sum(
                len(v) for v in self.data.get("techniques", {}).values()
            ),
            "mistakes_count": len(self.data.get("mistakes", [])),
        }


# シングルトン
_skill_memory: Optional[SkillMemory] = None


def get_skill_memory() -> SkillMemory:
    """グローバルなSkillMemoryインスタンスを取得"""
    global _skill_memory
    if _skill_memory is None:
        _skill_memory = SkillMemory()
    return _skill_memory
