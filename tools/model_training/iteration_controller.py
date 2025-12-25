"""
Iteration Controller
生成-評価-改善ループの制御

最大イテレーション数、早期終了、停滞検出などを管理。
"""

from datetime import datetime
from typing import Dict, Any, Optional, List, Tuple

# 設定
MAX_ITERATIONS = 5
EARLY_EXIT_THRESHOLD = 9.0  # このスコア以上で早期終了
IMPROVEMENT_THRESHOLD = 0.5  # 最低改善量
STAGNATION_CHECK_WINDOW = 3  # 停滞判定のウィンドウサイズ


class IterationController:
    """
    イテレーションループを制御するクラス。

    Usage:
        controller = IterationController(challenge)

        while True:
            # モデル生成
            model = generate_model(...)

            # 評価
            evaluation = evaluate_model(model, ...)
            controller.record_iteration(evaluation["total_score"], evaluation)

            # 継続判定
            should_continue, reason = controller.should_continue()
            if not should_continue:
                print(reason)
                break

            # 改善プロンプト生成
            feedback = generate_improvement_prompt(evaluation, challenge, controller.iteration_count)
            # ... 再生成
    """

    def __init__(self, challenge: Dict[str, Any]):
        """
        Args:
            challenge: 課題定義
        """
        self.challenge = challenge
        self.iterations: List[Dict[str, Any]] = []
        self.max_iterations = MAX_ITERATIONS
        self.success_threshold = challenge.get("success_threshold", 7.5)
        self.start_time = datetime.now()

    @property
    def iteration_count(self) -> int:
        """現在のイテレーション数"""
        return len(self.iterations)

    def should_continue(self) -> Tuple[bool, str]:
        """
        イテレーションを継続すべきか判定。

        Returns:
            (継続すべきか, 理由)
        """
        if not self.iterations:
            return True, "FIRST: 初回イテレーション"

        current = self.iterations[-1]
        score = current["score"]

        # 1. 成功条件
        if score >= self.success_threshold:
            return False, f"SUCCESS: スコア {score:.1f} >= 閾値 {self.success_threshold}"

        # 2. 早期終了（優秀なスコア）
        if score >= EARLY_EXIT_THRESHOLD:
            return False, f"EARLY_EXIT: 優秀なスコア {score:.1f}"

        # 3. 最大イテレーション到達
        if len(self.iterations) >= self.max_iterations:
            return False, f"MAX_ITERATIONS: {self.max_iterations}回到達"

        # 4. 停滞検出
        if len(self.iterations) >= STAGNATION_CHECK_WINDOW:
            recent_scores = [i["score"] for i in self.iterations[-STAGNATION_CHECK_WINDOW:]]
            score_range = max(recent_scores) - min(recent_scores)

            if score_range < IMPROVEMENT_THRESHOLD:
                return False, f"STAGNATION: 直近{STAGNATION_CHECK_WINDOW}回で改善なし (範囲: {score_range:.2f})"

        # 5. 継続
        return True, f"CONTINUE: スコア {score:.1f} < 閾値 {self.success_threshold}"

    def record_iteration(
        self,
        score: float,
        evaluation: Dict[str, Any],
        model_path: Optional[str] = None,
        generation_code: Optional[str] = None
    ) -> None:
        """
        イテレーション結果を記録。

        Args:
            score: 評価スコア
            evaluation: 完全な評価結果
            model_path: 生成されたモデルのパス（オプション）
            generation_code: 生成に使用したコード（オプション）
        """
        self.iterations.append({
            "iteration": len(self.iterations) + 1,
            "score": score,
            "evaluation": evaluation,
            "model_path": model_path,
            "generation_code": generation_code,
            "timestamp": datetime.now().isoformat(),
        })

    def get_best_result(self) -> Optional[Dict[str, Any]]:
        """最高スコアのイテレーション結果を取得"""
        if not self.iterations:
            return None
        return max(self.iterations, key=lambda x: x["score"])

    def get_latest_result(self) -> Optional[Dict[str, Any]]:
        """最新のイテレーション結果を取得"""
        if not self.iterations:
            return None
        return self.iterations[-1]

    def get_improvement(self) -> float:
        """初回からの改善量を取得"""
        if len(self.iterations) < 2:
            return 0.0
        return self.iterations[-1]["score"] - self.iterations[0]["score"]

    def get_summary(self) -> Dict[str, Any]:
        """イテレーションサマリーを生成"""
        if not self.iterations:
            return {"status": "not_started"}

        best = self.get_best_result()
        latest = self.get_latest_result()

        elapsed = (datetime.now() - self.start_time).total_seconds()

        return {
            "challenge_id": self.challenge.get("id", "unknown"),
            "challenge_name": self.challenge.get("name", "Unknown"),
            "status": "success" if best["score"] >= self.success_threshold else "incomplete",
            "total_iterations": len(self.iterations),
            "final_score": latest["score"],
            "best_score": best["score"],
            "best_iteration": best["iteration"],
            "success": best["score"] >= self.success_threshold,
            "success_threshold": self.success_threshold,
            "score_history": [i["score"] for i in self.iterations],
            "improvement": self.get_improvement(),
            "elapsed_seconds": elapsed,
        }

    def get_score_history_chart(self) -> str:
        """スコア履歴のASCIIチャートを生成"""
        if not self.iterations:
            return "No data"

        scores = [i["score"] for i in self.iterations]
        max_score = 10
        height = 5
        width = len(scores)

        chart_lines = []

        for row in range(height, -1, -1):
            line = ""
            threshold_at = int(self.success_threshold / max_score * height)

            for col, score in enumerate(scores):
                bar_height = int(score / max_score * height)

                if bar_height >= row:
                    if score >= self.success_threshold:
                        line += "█"
                    else:
                        line += "▓"
                elif row == threshold_at:
                    line += "─"
                else:
                    line += " "

            # 右側にスケール表示
            scale_value = row / height * max_score
            if row == height:
                line += f" {max_score:.0f}"
            elif row == threshold_at:
                line += f" {self.success_threshold:.1f} (threshold)"
            elif row == 0:
                line += " 0"
            else:
                line += ""

            chart_lines.append(line)

        # X軸
        chart_lines.append("─" * width + "─")
        chart_lines.append(" ".join(str(i + 1) for i in range(width)))

        return "\n".join(chart_lines)

    def to_dict(self) -> Dict[str, Any]:
        """シリアライズ用のdictに変換"""
        return {
            "challenge": self.challenge,
            "iterations": self.iterations,
            "summary": self.get_summary(),
        }

    @classmethod
    def from_dict(cls, data: Dict[str, Any]) -> "IterationController":
        """dictからインスタンスを復元"""
        controller = cls(data.get("challenge", {}))
        controller.iterations = data.get("iterations", [])
        return controller


def create_training_session(challenge: Dict[str, Any]) -> IterationController:
    """
    新しいトレーニングセッションを作成。

    Args:
        challenge: 課題定義

    Returns:
        IterationController インスタンス
    """
    return IterationController(challenge)
