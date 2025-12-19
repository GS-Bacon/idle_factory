// Re-export all types

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

// Item Category for routing to different editors
export type ItemCategory = "item" | "machine" | "multiblock";

// Item Data
export interface ItemData {
  id: string;
  i18n_key: string;
  asset: AssetConfig;
  properties: Record<string, unknown>;
  category: ItemCategory;
  subcategory?: string;
}

// Machine-specific data (extends item)
export interface MachineItemData extends ItemData {
  category: "machine";
  workTypes: string[];
  inputSlots: number;
  outputSlots: number;
  fluidInputSlots: number;
  fluidOutputSlots: number;
  baseTorqueConsumption: number;
  baseProcessingSpeed: number;
  vibrationLevel: number;
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

export function createDefaultItemData(id: string = ""): ItemData {
  return {
    id,
    i18n_key: id ? `item.${id}` : "",
    asset: createDefaultAssetConfig(),
    properties: {},
    category: "item",
    subcategory: "",
  };
}

export function createDefaultMachineData(id: string): MachineItemData {
  return {
    id,
    i18n_key: `machine.${id}`,
    asset: createDefaultAssetConfig(),
    properties: {},
    category: "machine",
    workTypes: [],
    inputSlots: 1,
    outputSlots: 1,
    fluidInputSlots: 0,
    fluidOutputSlots: 0,
    baseTorqueConsumption: 8,
    baseProcessingSpeed: 1.0,
    vibrationLevel: 1,
  };
}

// Re-export from other type files
export * from "./quest";
export * from "./multiblock";
export * from "./biome";
export * from "./sound";
