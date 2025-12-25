//! ブロックレイヤー決定
//!
//! 座標からブロックタイプを決定するロジック

/// ブロックレイヤー解決器
pub struct BlockLayerResolver;

impl BlockLayerResolver {
    /// 座標からブロックIDを決定
    ///
    /// # Arguments
    /// * `world_y` - ブロックのワールドY座標
    /// * `surface_height` - この(X,Z)位置の地表高さ
    /// * `bedrock_floor` - 岩盤の上限Y座標
    /// * `soil_depth` - 土壌の深さ（表面から）
    ///
    /// # Returns
    /// ブロックID文字列
    pub fn get_block(
        world_y: i32,
        surface_height: i32,
        bedrock_floor: i32,
        soil_depth: u32,
    ) -> &'static str {
        // 岩盤（最下層）
        if world_y <= bedrock_floor {
            return "bedrock";
        }

        // 地表より上 = 空気
        if world_y > surface_height {
            return "air";
        }

        // 地表 = 草ブロック
        if world_y == surface_height {
            return "grass";
        }

        // 地表下（土壌層内）= 土
        let depth_from_surface = surface_height - world_y;
        if depth_from_surface <= soil_depth as i32 {
            return "dirt";
        }

        // それ以外 = 石
        "stone"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SURFACE: i32 = 64;
    const BEDROCK_FLOOR: i32 = -60;
    const SOIL_DEPTH: u32 = 4;

    #[test]
    fn test_surface_is_grass() {
        assert_eq!(
            BlockLayerResolver::get_block(SURFACE, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
            "grass"
        );
    }

    #[test]
    fn test_above_surface_is_air() {
        assert_eq!(
            BlockLayerResolver::get_block(SURFACE + 1, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
            "air"
        );
        assert_eq!(
            BlockLayerResolver::get_block(SURFACE + 100, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
            "air"
        );
    }

    #[test]
    fn test_subsurface_is_dirt() {
        // 地表から1-4ブロック下は土
        for depth in 1..=SOIL_DEPTH as i32 {
            assert_eq!(
                BlockLayerResolver::get_block(SURFACE - depth, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
                "dirt",
                "Depth {} should be dirt",
                depth
            );
        }
    }

    #[test]
    fn test_deep_is_stone() {
        // 土壌層より深い = 石
        assert_eq!(
            BlockLayerResolver::get_block(
                SURFACE - SOIL_DEPTH as i32 - 1,
                SURFACE,
                BEDROCK_FLOOR,
                SOIL_DEPTH
            ),
            "stone"
        );
        assert_eq!(
            BlockLayerResolver::get_block(0, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
            "stone"
        );
    }

    #[test]
    fn test_bottom_is_bedrock() {
        assert_eq!(
            BlockLayerResolver::get_block(BEDROCK_FLOOR, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
            "bedrock"
        );
        assert_eq!(
            BlockLayerResolver::get_block(BEDROCK_FLOOR - 10, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
            "bedrock"
        );
    }

    #[test]
    fn test_layer_transition() {
        // 各層の境界をテスト
        let y_values = [
            (SURFACE + 10, "air"),
            (SURFACE, "grass"),
            (SURFACE - 1, "dirt"),
            (SURFACE - SOIL_DEPTH as i32, "dirt"),
            (SURFACE - SOIL_DEPTH as i32 - 1, "stone"),
            (BEDROCK_FLOOR + 1, "stone"),
            (BEDROCK_FLOOR, "bedrock"),
        ];

        for (y, expected) in y_values {
            assert_eq!(
                BlockLayerResolver::get_block(y, SURFACE, BEDROCK_FLOOR, SOIL_DEPTH),
                expected,
                "Y={} should be {}",
                y,
                expected
            );
        }
    }
}
