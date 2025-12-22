//! TypeScript type generation
//!
//! Run `cargo test --features typescript` to generate TypeScript definitions
//! to the `bindings/` directory.

#[cfg(all(test, feature = "typescript"))]
mod ts_export {
    use ts_rs::TS;

    /// Export all types to TypeScript
    #[test]
    fn export_typescript_types() {
        use crate::item::*;
        use crate::recipe::*;
        use crate::quest::*;

        // Create bindings directory
        let bindings_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("bindings");
        std::fs::create_dir_all(&bindings_dir).expect("Failed to create bindings directory");

        // Export item types
        ItemCategory::export_all_to(&bindings_dir).expect("Failed to export ItemCategory");
        AnimationType::export_all_to(&bindings_dir).expect("Failed to export AnimationType");
        AssetConfig::export_all_to(&bindings_dir).expect("Failed to export AssetConfig");
        LocalizationEntry::export_all_to(&bindings_dir).expect("Failed to export LocalizationEntry");
        ItemData::export_all_to(&bindings_dir).expect("Failed to export ItemData");
        GameItemData::export_all_to(&bindings_dir).expect("Failed to export GameItemData");

        // Export recipe types
        WorkType::export_all_to(&bindings_dir).expect("Failed to export WorkType");
        MachineType::export_all_to(&bindings_dir).expect("Failed to export MachineType");
        IngredientType::export_all_to(&bindings_dir).expect("Failed to export IngredientType");
        Ingredient::export_all_to(&bindings_dir).expect("Failed to export Ingredient");
        ProductType::export_all_to(&bindings_dir).expect("Failed to export ProductType");
        Product::export_all_to(&bindings_dir).expect("Failed to export Product");
        RecipeDef::export_all_to(&bindings_dir).expect("Failed to export RecipeDef");
        ItemIO::export_all_to(&bindings_dir).expect("Failed to export ItemIO");
        FluidIO::export_all_to(&bindings_dir).expect("Failed to export FluidIO");
        GameRecipe::export_all_to(&bindings_dir).expect("Failed to export GameRecipe");

        // Export quest types
        QuestType::export_all_to(&bindings_dir).expect("Failed to export QuestType");
        RequirementType::export_all_to(&bindings_dir).expect("Failed to export RequirementType");
        QuestRequirement::export_all_to(&bindings_dir).expect("Failed to export QuestRequirement");
        RewardType::export_all_to(&bindings_dir).expect("Failed to export RewardType");
        QuestData::export_all_to(&bindings_dir).expect("Failed to export QuestData");

        println!("TypeScript types exported to: {:?}", bindings_dir);
    }
}
