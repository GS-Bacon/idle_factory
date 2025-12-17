// Animation Types
export type AnimationType =
  | { type: "None" }
  | { type: "Rotational"; params: { axis: [number, number, number]; speed: number } }
  | { type: "Linear"; params: { direction: [number, number, number]; distance: number; speed: number } }
  | { type: "Skeletal"; params: { animation_path: string; looping: boolean } };

// Asset Configuration
export interface AssetConfig {
  icon_path: string | null;
  model_path: string | null;
  animation: AnimationType;
}

// Localization Entry
export interface LocalizationEntry {
  name: string;
  description: string;
}

// Localization Data (for save/load)
export interface LocalizationData {
  ja: LocalizationEntry;
  en: LocalizationEntry;
}

// Item Data
export interface ItemData {
  id: string;
  i18n_key: string;
  asset: AssetConfig;
  properties: Record<string, unknown>;
}

// Create default values
export function createDefaultAssetConfig(): AssetConfig {
  return {
    icon_path: null,
    model_path: null,
    animation: { type: "None" },
  };
}

export function createDefaultLocalizationData(): LocalizationData {
  return {
    ja: { name: "", description: "" },
    en: { name: "", description: "" },
  };
}

export function createDefaultItemData(id: string): ItemData {
  return {
    id,
    i18n_key: `item.${id}`,
    asset: createDefaultAssetConfig(),
    properties: {},
  };
}
