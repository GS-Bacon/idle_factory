//! Mod依存関係解決
//!
//! トポロジカルソートによるロード順序決定と循環依存検出

use std::collections::{HashMap, HashSet, VecDeque};

/// 依存関係エラー
#[derive(Debug, Clone)]
pub enum DependencyError {
    /// 循環依存が検出された
    CircularDependency(Vec<String>),
    /// 必要なModが見つからない
    MissingDependency { mod_id: String, required: String },
    /// バージョンが一致しない
    VersionMismatch {
        mod_id: String,
        required: String,
        found: String,
    },
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyError::CircularDependency(cycle) => {
                write!(f, "Circular dependency detected: {}", cycle.join(" -> "))
            }
            DependencyError::MissingDependency { mod_id, required } => {
                write!(
                    f,
                    "Mod '{}' requires '{}' which is not installed",
                    mod_id, required
                )
            }
            DependencyError::VersionMismatch {
                mod_id,
                required,
                found,
            } => {
                write!(
                    f,
                    "Mod '{}' requires version '{}' but found '{}'",
                    mod_id, required, found
                )
            }
        }
    }
}

impl std::error::Error for DependencyError {}

/// Mod情報（依存解決用）
#[derive(Debug, Clone)]
pub struct ModDependencyInfo {
    pub id: String,
    pub version: String,
    pub dependencies: HashMap<String, String>, // mod_id -> required_version
}

/// 依存関係リゾルバ
pub struct DependencyResolver {
    mods: HashMap<String, ModDependencyInfo>,
}

impl DependencyResolver {
    /// 新しいリゾルバを作成
    pub fn new() -> Self {
        Self {
            mods: HashMap::new(),
        }
    }

    /// Modを登録
    pub fn add_mod(&mut self, info: ModDependencyInfo) {
        self.mods.insert(info.id.clone(), info);
    }

    /// 全Modを登録（一括）
    pub fn add_mods(&mut self, mods: impl IntoIterator<Item = ModDependencyInfo>) {
        for info in mods {
            self.add_mod(info);
        }
    }

    /// 循環依存をチェック
    pub fn check_circular(&self) -> Result<(), DependencyError> {
        // DFSで循環検出
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for mod_id in self.mods.keys() {
            if !visited.contains(mod_id) {
                if let Some(cycle) =
                    self.dfs_check_cycle(mod_id, &mut visited, &mut rec_stack, &mut path)
                {
                    return Err(DependencyError::CircularDependency(cycle));
                }
            }
        }
        Ok(())
    }

    fn dfs_check_cycle(
        &self,
        mod_id: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(mod_id.to_string());
        rec_stack.insert(mod_id.to_string());
        path.push(mod_id.to_string());

        if let Some(info) = self.mods.get(mod_id) {
            for dep_id in info.dependencies.keys() {
                if !visited.contains(dep_id) {
                    if let Some(cycle) = self.dfs_check_cycle(dep_id, visited, rec_stack, path) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep_id) {
                    // 循環発見
                    let cycle_start = path.iter().position(|x| x == dep_id).unwrap();
                    let mut cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycle.push(dep_id.to_string());
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(mod_id);
        None
    }

    /// 依存関係を検証（存在チェック）
    pub fn validate(&self) -> Result<(), DependencyError> {
        for (mod_id, info) in &self.mods {
            for dep_id in info.dependencies.keys() {
                if !self.mods.contains_key(dep_id) {
                    return Err(DependencyError::MissingDependency {
                        mod_id: mod_id.clone(),
                        required: dep_id.clone(),
                    });
                }
                // TODO: バージョンチェック（semver）
            }
        }
        Ok(())
    }

    /// ロード順序を解決（トポロジカルソート）
    /// 依存されているModが先にロードされる順序を返す
    pub fn resolve(&self) -> Result<Vec<String>, DependencyError> {
        // まず検証
        self.validate()?;
        self.check_circular()?;

        // Kahn's algorithm でトポロジカルソート
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut graph: HashMap<&str, Vec<&str>> = HashMap::new();

        // グラフ初期化
        for mod_id in self.mods.keys() {
            in_degree.insert(mod_id, 0);
            graph.insert(mod_id, Vec::new());
        }

        // エッジを追加（依存関係）
        for (mod_id, info) in &self.mods {
            for dep_id in info.dependencies.keys() {
                if let Some(edges) = graph.get_mut(dep_id.as_str()) {
                    edges.push(mod_id.as_str());
                }
                *in_degree.get_mut(mod_id.as_str()).unwrap() += 1;
            }
        }

        // in_degree が 0 のノードをキューに追加
        let mut queue: VecDeque<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();

        while let Some(mod_id) = queue.pop_front() {
            result.push(mod_id.to_string());

            if let Some(dependents) = graph.get(mod_id) {
                for &dependent in dependents {
                    let deg = in_degree.get_mut(dependent).unwrap();
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dependent);
                    }
                }
            }
        }

        // 全ノードが処理されたか確認（循環がなければ全て処理される）
        if result.len() != self.mods.len() {
            // ここには到達しないはず（check_circularで検出済み）
            return Err(DependencyError::CircularDependency(vec![
                "unknown".to_string()
            ]));
        }

        Ok(result)
    }
}

impl Default for DependencyResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mod(id: &str, deps: &[(&str, &str)]) -> ModDependencyInfo {
        ModDependencyInfo {
            id: id.to_string(),
            version: "1.0.0".to_string(),
            dependencies: deps
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn test_simple_resolution() {
        let mut resolver = DependencyResolver::new();
        resolver.add_mod(make_mod("base", &[]));
        resolver.add_mod(make_mod("mod_a", &[("base", ">=1.0")]));
        resolver.add_mod(make_mod("mod_b", &[("base", ">=1.0"), ("mod_a", ">=1.0")]));

        let order = resolver.resolve().unwrap();

        // base が最初
        assert_eq!(order[0], "base");
        // mod_a は mod_b より前
        let pos_a = order.iter().position(|x| x == "mod_a").unwrap();
        let pos_b = order.iter().position(|x| x == "mod_b").unwrap();
        assert!(pos_a < pos_b);
    }

    #[test]
    fn test_circular_dependency() {
        let mut resolver = DependencyResolver::new();
        resolver.add_mod(make_mod("mod_a", &[("mod_b", ">=1.0")]));
        resolver.add_mod(make_mod("mod_b", &[("mod_a", ">=1.0")]));

        let result = resolver.check_circular();
        assert!(result.is_err());

        if let Err(DependencyError::CircularDependency(cycle)) = result {
            assert!(cycle.contains(&"mod_a".to_string()));
            assert!(cycle.contains(&"mod_b".to_string()));
        }
    }

    #[test]
    fn test_missing_dependency() {
        let mut resolver = DependencyResolver::new();
        resolver.add_mod(make_mod("mod_a", &[("nonexistent", ">=1.0")]));

        let result = resolver.validate();
        assert!(result.is_err());

        if let Err(DependencyError::MissingDependency { mod_id, required }) = result {
            assert_eq!(mod_id, "mod_a");
            assert_eq!(required, "nonexistent");
        }
    }

    #[test]
    fn test_complex_dependencies() {
        let mut resolver = DependencyResolver::new();
        // 複雑な依存関係グラフ
        //     base
        //    /    \
        //   A      B
        //    \    /
        //      C
        //      |
        //      D
        resolver.add_mod(make_mod("base", &[]));
        resolver.add_mod(make_mod("A", &[("base", ">=1.0")]));
        resolver.add_mod(make_mod("B", &[("base", ">=1.0")]));
        resolver.add_mod(make_mod("C", &[("A", ">=1.0"), ("B", ">=1.0")]));
        resolver.add_mod(make_mod("D", &[("C", ">=1.0")]));

        let order = resolver.resolve().unwrap();
        assert_eq!(order.len(), 5);

        // 順序チェック
        let pos = |id: &str| order.iter().position(|x| x == id).unwrap();
        assert!(pos("base") < pos("A"));
        assert!(pos("base") < pos("B"));
        assert!(pos("A") < pos("C"));
        assert!(pos("B") < pos("C"));
        assert!(pos("C") < pos("D"));
    }

    #[test]
    fn test_no_dependencies() {
        let mut resolver = DependencyResolver::new();
        resolver.add_mod(make_mod("standalone", &[]));

        let order = resolver.resolve().unwrap();
        assert_eq!(order, vec!["standalone"]);
    }
}
