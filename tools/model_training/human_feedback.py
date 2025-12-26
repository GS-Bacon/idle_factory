"""
Human Feedback System for 3D Model Training

人間の評価を記録し、将来の生成に活用するシステム。

Usage:
    from tools.model_training.human_feedback import HumanFeedbackDB, ModelGeneration

    db = HumanFeedbackDB()

    # 生成を記録
    gen = ModelGeneration(
        model_name="pipe_straight",
        category="machine",
        parameters={"pipe_r": 0.12, "bolt_position": "inner"},
    )
    gen_id = db.record_generation(gen)

    # 人間評価を追加
    db.add_feedback(gen_id, {
        "shape": 4,
        "style": 5,
        "detail": 4,
        "color": 4,
        "overall": 4,
        "comments": "ボルトは内側に統一",
        "issues": ["bolt_position"],
        "fixes_applied": ["両方内側に変更"]
    })

    # 成功パターンを取得
    patterns = db.get_successful_patterns("machine")
"""

import json
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, List, Optional
from dataclasses import dataclass, asdict

FEEDBACK_DB_PATH = Path(__file__).parent.parent / "model_training_data" / "human_feedback.json"


@dataclass
class ModelGeneration:
    """モデル生成の記録"""
    model_name: str
    category: str  # item, machine, structure
    parameters: Dict[str, Any]
    script_path: Optional[str] = None
    screenshot_path: Optional[str] = None
    export_path: Optional[str] = None
    timestamp: str = ""

    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now().isoformat()


@dataclass
class HumanFeedback:
    """人間による評価"""
    shape: int  # 1-5: 形状のらしさ
    style: int  # 1-5: ローポリ感
    detail: int  # 1-5: ディテールバランス
    color: int  # 1-5: マテリアル/色
    overall: int  # 1-5: 総合評価
    comments: str = ""
    issues: List[str] = None  # 問題点
    fixes_applied: List[str] = None  # 適用した修正
    timestamp: str = ""

    def __post_init__(self):
        if not self.timestamp:
            self.timestamp = datetime.now().isoformat()
        if self.issues is None:
            self.issues = []
        if self.fixes_applied is None:
            self.fixes_applied = []

    @property
    def average_score(self) -> float:
        return (self.shape + self.style + self.detail + self.color + self.overall) / 5


class HumanFeedbackDB:
    """人間フィードバックデータベース"""

    def __init__(self, path: Optional[Path] = None):
        self.path = path or FEEDBACK_DB_PATH
        self.data = self._load()

    def _load(self) -> Dict[str, Any]:
        if self.path.exists():
            with open(self.path, "r", encoding="utf-8") as f:
                return json.load(f)
        return self._create_initial_db()

    def _save(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.path, "w", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2, ensure_ascii=False)

    def _create_initial_db(self) -> Dict[str, Any]:
        return {
            "version": "1.0.0",
            "last_updated": datetime.now().isoformat(),
            "generations": {},  # gen_id -> generation data
            "feedback": {},  # gen_id -> feedback data
            "learned_patterns": {
                "item": [],
                "machine": [],
                "structure": [],
            },
            "common_issues": [],  # よくある問題
            "statistics": {
                "total_generations": 0,
                "total_feedback": 0,
                "average_score": 0.0,
                "by_category": {},
            }
        }

    def record_generation(self, gen: ModelGeneration) -> str:
        """生成を記録してIDを返す"""
        gen_id = f"{gen.model_name}_{datetime.now().strftime('%Y%m%d_%H%M%S')}"

        self.data["generations"][gen_id] = asdict(gen)
        self.data["statistics"]["total_generations"] += 1
        self.data["last_updated"] = datetime.now().isoformat()

        # カテゴリ別統計
        cat = gen.category
        if cat not in self.data["statistics"]["by_category"]:
            self.data["statistics"]["by_category"][cat] = {
                "count": 0, "avg_score": 0.0, "scores": []
            }
        self.data["statistics"]["by_category"][cat]["count"] += 1

        self._save()
        return gen_id

    def add_feedback(self, gen_id: str, feedback: Dict[str, Any]) -> None:
        """人間評価を追加"""
        if gen_id not in self.data["generations"]:
            raise ValueError(f"Generation {gen_id} not found")

        fb = HumanFeedback(**feedback) if isinstance(feedback, dict) else feedback
        fb_dict = asdict(fb) if hasattr(fb, '__dataclass_fields__') else fb

        self.data["feedback"][gen_id] = fb_dict
        self.data["statistics"]["total_feedback"] += 1

        # 平均スコア更新
        avg = fb.average_score if hasattr(fb, 'average_score') else (
            (fb_dict["shape"] + fb_dict["style"] + fb_dict["detail"] +
             fb_dict["color"] + fb_dict["overall"]) / 5
        )

        # カテゴリ別スコア更新
        gen = self.data["generations"][gen_id]
        cat = gen["category"]
        cat_stats = self.data["statistics"]["by_category"].get(cat, {"scores": []})
        cat_stats["scores"].append(avg)
        cat_stats["avg_score"] = sum(cat_stats["scores"]) / len(cat_stats["scores"])
        self.data["statistics"]["by_category"][cat] = cat_stats

        # 全体平均更新
        all_scores = []
        for cat_data in self.data["statistics"]["by_category"].values():
            all_scores.extend(cat_data.get("scores", []))
        if all_scores:
            self.data["statistics"]["average_score"] = sum(all_scores) / len(all_scores)

        # 高評価パターンを学習
        if avg >= 4.0:
            self._learn_pattern(gen_id, gen, fb_dict)

        # 問題パターンを記録
        issues = fb_dict.get("issues", [])
        for issue in issues:
            self._record_issue(issue, gen, fb_dict)

        self.data["last_updated"] = datetime.now().isoformat()
        self._save()

    def _learn_pattern(self, gen_id: str, gen: Dict, feedback: Dict) -> None:
        """成功パターンを学習"""
        cat = gen["category"]
        pattern = {
            "model_name": gen["model_name"],
            "parameters": gen["parameters"],
            "score": feedback.get("overall", 0),
            "avg_score": (feedback["shape"] + feedback["style"] +
                         feedback["detail"] + feedback["color"] +
                         feedback["overall"]) / 5,
            "learned_from": gen_id,
            "timestamp": datetime.now().isoformat(),
            "notes": feedback.get("comments", ""),
        }

        # 同じモデル名の古いパターンを更新
        patterns = self.data["learned_patterns"][cat]
        updated = False
        for i, p in enumerate(patterns):
            if p["model_name"] == gen["model_name"]:
                if pattern["avg_score"] > p["avg_score"]:
                    patterns[i] = pattern
                    updated = True
                break

        if not updated:
            patterns.append(pattern)

    def _record_issue(self, issue: str, gen: Dict, feedback: Dict) -> None:
        """問題パターンを記録"""
        # 既存の問題か確認
        for existing in self.data["common_issues"]:
            if existing["issue"] == issue:
                existing["frequency"] += 1
                if feedback.get("fixes_applied"):
                    existing["fixes"].extend(feedback["fixes_applied"])
                    existing["fixes"] = list(set(existing["fixes"]))
                return

        # 新しい問題を追加
        self.data["common_issues"].append({
            "issue": issue,
            "frequency": 1,
            "category": gen["category"],
            "fixes": feedback.get("fixes_applied", []),
            "first_seen": datetime.now().isoformat(),
        })

    def get_successful_patterns(self, category: str) -> List[Dict]:
        """成功パターンを取得"""
        patterns = self.data["learned_patterns"].get(category, [])
        return sorted(patterns, key=lambda x: -x.get("avg_score", 0))

    def get_common_issues(self, category: Optional[str] = None) -> List[Dict]:
        """よくある問題を取得"""
        issues = self.data["common_issues"]
        if category:
            issues = [i for i in issues if i.get("category") == category]
        return sorted(issues, key=lambda x: -x.get("frequency", 0))

    def get_guidance_for_model(self, model_name: str, category: str) -> Dict[str, Any]:
        """モデル生成のガイダンスを取得"""
        guidance = {
            "successful_patterns": [],
            "issues_to_avoid": [],
            "recommended_parameters": {},
        }

        # 成功パターン
        patterns = self.get_successful_patterns(category)
        for p in patterns[:3]:
            guidance["successful_patterns"].append({
                "model": p["model_name"],
                "score": p["avg_score"],
                "parameters": p["parameters"],
            })

        # 避けるべき問題
        issues = self.get_common_issues(category)
        for i in issues[:5]:
            guidance["issues_to_avoid"].append({
                "issue": i["issue"],
                "frequency": i["frequency"],
                "fixes": i["fixes"],
            })

        # 同じモデルの過去パターン
        for p in patterns:
            if p["model_name"] == model_name:
                guidance["recommended_parameters"] = p["parameters"]
                break

        return guidance

    def get_statistics(self) -> Dict[str, Any]:
        """統計情報を取得"""
        return self.data["statistics"]

    def get_pending_reviews(self) -> List[Dict[str, Any]]:
        """評価待ちの生成を取得"""
        pending = []
        for gen_id, gen in self.data["generations"].items():
            if gen_id not in self.data["feedback"]:
                pending.append({
                    "gen_id": gen_id,
                    "model_name": gen["model_name"],
                    "category": gen["category"],
                    "screenshot_path": gen.get("screenshot_path"),
                    "export_path": gen.get("export_path"),
                    "timestamp": gen["timestamp"],
                })
        return sorted(pending, key=lambda x: x["timestamp"])

    def batch_review_report(self) -> str:
        """評価待ちモデルのレビューレポートを生成"""
        pending = self.get_pending_reviews()
        if not pending:
            return "評価待ちのモデルはありません。"

        lines = [
            "# 評価待ちモデル一覧",
            "",
            f"合計: {len(pending)}件",
            "",
        ]

        for i, item in enumerate(pending, 1):
            lines.append(f"## {i}. {item['model_name']} ({item['category']})")
            lines.append(f"- ID: `{item['gen_id']}`")
            if item.get("screenshot_path"):
                lines.append(f"- Screenshot: `{item['screenshot_path']}`")
            if item.get("export_path"):
                lines.append(f"- Model: `{item['export_path']}`")
            lines.append(f"- Generated: {item['timestamp'][:16]}")
            lines.append("")

        lines.append("---")
        lines.append("評価コマンド例:")
        lines.append("```python")
        lines.append(f'db.add_feedback("{pending[0]["gen_id"]}", {{"shape": 4, "style": 4, "detail": 4, "color": 4, "overall": 4, "comments": ""}})')
        lines.append("```")

        return "\n".join(lines)

    def generate_report(self) -> str:
        """レポートを生成"""
        stats = self.data["statistics"]

        lines = [
            "# Human Feedback Report",
            "",
            f"Last Updated: {self.data['last_updated']}",
            "",
            "## Statistics",
            f"- Total Generations: {stats['total_generations']}",
            f"- Total Feedback: {stats['total_feedback']}",
            f"- Average Score: {stats['average_score']:.2f}/5",
            "",
            "## By Category",
        ]

        for cat, data in stats.get("by_category", {}).items():
            lines.append(f"### {cat}")
            lines.append(f"- Count: {data.get('count', 0)}")
            lines.append(f"- Avg Score: {data.get('avg_score', 0):.2f}/5")
            lines.append("")

        lines.append("## Successful Patterns")
        for cat in ["item", "machine", "structure"]:
            patterns = self.get_successful_patterns(cat)
            if patterns:
                lines.append(f"### {cat}")
                for p in patterns[:3]:
                    lines.append(f"- {p['model_name']}: {p['avg_score']:.1f}/5")
                lines.append("")

        lines.append("## Common Issues")
        for issue in self.get_common_issues()[:5]:
            lines.append(f"- {issue['issue']} (x{issue['frequency']})")
            if issue["fixes"]:
                lines.append(f"  Fix: {', '.join(issue['fixes'][:2])}")

        return "\n".join(lines)


# シングルトン
_db_instance: Optional[HumanFeedbackDB] = None

def get_feedback_db() -> HumanFeedbackDB:
    """グローバルDBインスタンスを取得"""
    global _db_instance
    if _db_instance is None:
        _db_instance = HumanFeedbackDB()
    return _db_instance
