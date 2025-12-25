"""
Learning Database
成功パターンと失敗パターンを記録し、将来の生成に活用

JSONファイルベースの軽量データベース。
"""

import json
from datetime import datetime
from pathlib import Path
from typing import Dict, Any, Optional, List

# デフォルトのデータベースパス
DEFAULT_DB_PATH = Path(__file__).parent.parent / "model_training_data" / "learning.json"
RESULTS_DIR = Path(__file__).parent.parent / "model_training_data" / "challenge_results"


class LearningDB:
    """
    モデリングの学習データを管理するクラス。

    Usage:
        db = LearningDB()

        # セッション結果を記録
        db.record_session(controller.to_dict())

        # 成功パターンを検索
        patterns = db.get_successful_patterns("tool_handle")

        # 統計を取得
        stats = db.get_statistics()
    """

    def __init__(self, db_path: Optional[Path] = None):
        """
        Args:
            db_path: データベースファイルのパス（オプション）
        """
        self.db_path = db_path or DEFAULT_DB_PATH
        self.data = self._load()

    def _load(self) -> Dict[str, Any]:
        """データベースをロード"""
        if self.db_path.exists():
            with open(self.db_path, "r", encoding="utf-8") as f:
                return json.load(f)
        return self._create_empty_db()

    def _save(self) -> None:
        """データベースを保存"""
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        with open(self.db_path, "w", encoding="utf-8") as f:
            json.dump(self.data, f, indent=2, ensure_ascii=False)

    def _create_empty_db(self) -> Dict[str, Any]:
        """空のデータベースを作成"""
        return {
            "version": "1.0.0",
            "last_updated": datetime.now().isoformat(),
            "successful_patterns": {},
            "common_failures": {},
            "challenge_history": {},
            "statistics": {
                "total_attempts": 0,
                "successful_attempts": 0,
                "average_iterations": 0.0,
                "most_failed_criterion": None,
            },
        }

    def record_session(self, session_data: Dict[str, Any]) -> None:
        """
        トレーニングセッションの結果を記録。

        Args:
            session_data: IterationController.to_dict()の結果
        """
        summary = session_data.get("summary", {})
        challenge_id = summary.get("challenge_id", "unknown")
        success = summary.get("success", False)
        iterations = session_data.get("iterations", [])

        # 課題履歴を更新
        if challenge_id not in self.data["challenge_history"]:
            self.data["challenge_history"][challenge_id] = {
                "attempts": 0,
                "successes": 0,
                "best_score": 0,
                "total_iterations": 0,
                "learned_patterns": [],
            }

        history = self.data["challenge_history"][challenge_id]
        history["attempts"] += 1
        history["total_iterations"] += len(iterations)
        history["last_attempt"] = datetime.now().isoformat()

        if success:
            history["successes"] += 1

        if summary.get("best_score", 0) > history.get("best_score", 0):
            history["best_score"] = summary["best_score"]

        # 成功パターンを抽出
        if success and iterations:
            self._extract_success_patterns(challenge_id, iterations[-1])

        # 失敗パターンを抽出
        if not success and iterations:
            self._extract_failure_patterns(iterations)

        # 統計を更新
        self._update_statistics(success, len(iterations))

        # 保存
        self.data["last_updated"] = datetime.now().isoformat()
        self._save()

        # 詳細結果を別ファイルに保存
        self._save_session_details(session_data)

    def _extract_success_patterns(self, challenge_id: str, final_iteration: Dict) -> None:
        """成功イテレーションからパターンを抽出"""
        evaluation = final_iteration.get("evaluation", {})
        scores = evaluation.get("scores", {})

        # 高スコアの基準からパターンを抽出
        for criterion, score in scores.items():
            if score >= 9.0:
                pattern_key = f"{challenge_id}_{criterion}"

                if pattern_key not in self.data["successful_patterns"]:
                    self.data["successful_patterns"][pattern_key] = {
                        "description": f"{challenge_id}の{criterion}で高スコア",
                        "success_count": 0,
                        "example_score": score,
                    }

                self.data["successful_patterns"][pattern_key]["success_count"] += 1

    def _extract_failure_patterns(self, iterations: List[Dict]) -> None:
        """失敗イテレーションからパターンを抽出"""
        # 全イテレーションで低スコアだった基準を特定
        criterion_scores = {}

        for iteration in iterations:
            evaluation = iteration.get("evaluation", {})
            scores = evaluation.get("scores", {})

            for criterion, score in scores.items():
                if criterion not in criterion_scores:
                    criterion_scores[criterion] = []
                criterion_scores[criterion].append(score)

        # 平均スコアが低い基準を失敗パターンとして記録
        for criterion, scores in criterion_scores.items():
            avg_score = sum(scores) / len(scores)

            if avg_score < 5.0:
                if criterion not in self.data["common_failures"]:
                    self.data["common_failures"][criterion] = {
                        "description": f"{criterion}の低スコアが頻発",
                        "frequency": 0,
                        "average_score": avg_score,
                    }

                self.data["common_failures"][criterion]["frequency"] += 1
                # 移動平均で更新
                current_avg = self.data["common_failures"][criterion]["average_score"]
                self.data["common_failures"][criterion]["average_score"] = (current_avg + avg_score) / 2

    def _update_statistics(self, success: bool, iteration_count: int) -> None:
        """統計を更新"""
        stats = self.data["statistics"]

        stats["total_attempts"] += 1
        if success:
            stats["successful_attempts"] += 1

        # 平均イテレーション数を更新（移動平均）
        n = stats["total_attempts"]
        current_avg = stats["average_iterations"]
        stats["average_iterations"] = ((n - 1) * current_avg + iteration_count) / n

        # 最も失敗の多い基準を更新
        if self.data["common_failures"]:
            most_failed = max(
                self.data["common_failures"].items(),
                key=lambda x: x[1]["frequency"]
            )
            stats["most_failed_criterion"] = most_failed[0]

    def _save_session_details(self, session_data: Dict[str, Any]) -> None:
        """セッション詳細を個別ファイルに保存"""
        RESULTS_DIR.mkdir(parents=True, exist_ok=True)

        summary = session_data.get("summary", {})
        challenge_id = summary.get("challenge_id", "unknown")
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")

        filename = f"{challenge_id}_{timestamp}.json"
        filepath = RESULTS_DIR / filename

        with open(filepath, "w", encoding="utf-8") as f:
            json.dump(session_data, f, indent=2, ensure_ascii=False)

    def get_successful_patterns(self, pattern_filter: Optional[str] = None) -> Dict[str, Any]:
        """
        成功パターンを取得。

        Args:
            pattern_filter: パターンキーのフィルタ（部分一致）

        Returns:
            成功パターンのdict
        """
        patterns = self.data.get("successful_patterns", {})

        if pattern_filter:
            patterns = {
                k: v for k, v in patterns.items()
                if pattern_filter.lower() in k.lower()
            }

        return patterns

    def get_common_failures(self) -> Dict[str, Any]:
        """よくある失敗パターンを取得"""
        return self.data.get("common_failures", {})

    def get_challenge_history(self, challenge_id: Optional[str] = None) -> Dict[str, Any]:
        """
        課題履歴を取得。

        Args:
            challenge_id: 特定の課題ID（オプション）

        Returns:
            課題履歴のdict
        """
        history = self.data.get("challenge_history", {})

        if challenge_id:
            return history.get(challenge_id, {})

        return history

    def get_statistics(self) -> Dict[str, Any]:
        """統計情報を取得"""
        stats = self.data.get("statistics", {})

        # 成功率を計算
        total = stats.get("total_attempts", 0)
        successful = stats.get("successful_attempts", 0)
        stats["success_rate"] = successful / total if total > 0 else 0.0

        return stats

    def get_recommendations_for_challenge(self, challenge_id: str) -> Dict[str, Any]:
        """
        課題に対する推奨事項を取得。

        Args:
            challenge_id: 課題ID

        Returns:
            推奨事項のdict
        """
        recommendations = {
            "patterns_to_use": [],
            "failures_to_avoid": [],
            "estimated_iterations": None,
            "success_probability": None,
        }

        # 関連する成功パターン
        patterns = self.get_successful_patterns(challenge_id)
        for pattern_key, pattern_data in patterns.items():
            if pattern_data.get("success_count", 0) >= 2:
                recommendations["patterns_to_use"].append({
                    "pattern": pattern_key,
                    "description": pattern_data.get("description", ""),
                    "success_count": pattern_data["success_count"],
                })

        # 避けるべき失敗パターン
        failures = self.get_common_failures()
        for criterion, failure_data in failures.items():
            if failure_data.get("frequency", 0) >= 2:
                recommendations["failures_to_avoid"].append({
                    "criterion": criterion,
                    "frequency": failure_data["frequency"],
                    "average_score": failure_data.get("average_score", 0),
                })

        # 推定イテレーション数
        history = self.get_challenge_history(challenge_id)
        if history and history.get("attempts", 0) > 0:
            avg_iter = history["total_iterations"] / history["attempts"]
            recommendations["estimated_iterations"] = round(avg_iter, 1)

            # 成功確率
            if history["attempts"] >= 3:
                recommendations["success_probability"] = history["successes"] / history["attempts"]

        return recommendations

    def generate_report(self) -> str:
        """学習データベースのレポートを生成"""
        stats = self.get_statistics()
        history = self.get_challenge_history()
        patterns = self.get_successful_patterns()
        failures = self.get_common_failures()

        lines = [
            "# モデリング学習データベース レポート",
            "",
            f"最終更新: {self.data.get('last_updated', 'N/A')}",
            "",
            "## 統計",
            "",
            f"- 総試行回数: {stats.get('total_attempts', 0)}",
            f"- 成功回数: {stats.get('successful_attempts', 0)}",
            f"- 成功率: {stats.get('success_rate', 0):.1%}",
            f"- 平均イテレーション数: {stats.get('average_iterations', 0):.1f}",
            f"- 最も失敗の多い基準: {stats.get('most_failed_criterion', 'N/A')}",
            "",
            "## 課題別履歴",
            "",
        ]

        for challenge_id, data in history.items():
            success_rate = data["successes"] / data["attempts"] if data["attempts"] > 0 else 0
            lines.append(f"### {challenge_id}")
            lines.append(f"- 試行: {data['attempts']} | 成功: {data['successes']} ({success_rate:.0%})")
            lines.append(f"- 最高スコア: {data.get('best_score', 0):.1f}")
            lines.append("")

        if patterns:
            lines.append("## 成功パターン")
            lines.append("")
            for pattern_key, data in sorted(patterns.items(), key=lambda x: -x[1].get("success_count", 0))[:5]:
                lines.append(f"- **{pattern_key}**: {data.get('success_count', 0)}回成功")

        if failures:
            lines.append("")
            lines.append("## よくある失敗")
            lines.append("")
            for criterion, data in sorted(failures.items(), key=lambda x: -x[1].get("frequency", 0))[:5]:
                lines.append(f"- **{criterion}**: {data.get('frequency', 0)}回 (平均スコア: {data.get('average_score', 0):.1f})")

        return "\n".join(lines)


# シングルトンインスタンス
_db_instance: Optional[LearningDB] = None


def get_learning_db() -> LearningDB:
    """グローバルなLearningDBインスタンスを取得"""
    global _db_instance
    if _db_instance is None:
        _db_instance = LearningDB()
    return _db_instance
