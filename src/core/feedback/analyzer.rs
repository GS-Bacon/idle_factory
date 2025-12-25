//! 分析ロジック

use super::types::*;

/// コアコンセプト違反チェック
pub fn check_core_concept_violation(proposal: &str, config: &EvaluationConfig) -> Option<String> {
    let proposal_lower = proposal.to_lowercase();
    for keyword in &config.core_concept_violations {
        if proposal_lower.contains(keyword) {
            return Some(format!(
                "提案が「{}」を含んでおり、コアコンセプトに違反する可能性があります",
                keyword
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_concept_violation() {
        let config = EvaluationConfig::default();

        // 違反あり（英語キーワード）
        let result = check_core_concept_violation("Add time_limit quest", &config);
        assert!(result.is_some());

        let result = check_core_concept_violation("Add enemy spawner", &config);
        assert!(result.is_some());

        let result = check_core_concept_violation("Combat system", &config);
        assert!(result.is_some());

        // 違反なし
        let result = check_core_concept_violation("UIヒント追加", &config);
        assert!(result.is_none());

        let result = check_core_concept_violation("Add tooltip for inventory", &config);
        assert!(result.is_none());
    }
}
