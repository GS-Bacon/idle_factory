// Recipe Editor Types

// Ingredient type (Item or Tag)
export type IngredientType =
  | { type: "Item"; value: string }
  | { type: "Tag"; value: string };

// Ingredient
export interface Ingredient {
  ingredient_type: IngredientType;
  amount: number;
}

// Product type (Item or Fluid)
export type ProductType =
  | { type: "Item"; value: string }
  | { type: "Fluid"; value: string };

// Product (result)
export interface Product {
  product_type: ProductType;
  amount: number;
  chance: number; // 0.0 - 1.0
}

// Machine types
export type MachineType =
  | "Assembler"
  | "Mixer"
  | "Press"
  | "Furnace"
  | "Crusher"
  | "Centrifuge"
  | "ChemicalReactor"
  | "Packager"
  | { Custom: string };

// Recipe definition (matches Rust RecipeDef)
export interface RecipeDef {
  id: string;
  machine_type: MachineType;
  ingredients: Ingredient[];
  results: Product[];
  process_time: number;
  stress_impact: number;
  i18n_key: string;
}

// Catalog entry for palette
export interface CatalogEntry {
  id: string;
  name: string;
  icon_path: string | null;
}

// Machine catalog entry
export interface MachineCatalogEntry {
  id: string;
  name: string;
  machine_type: MachineType;
  input_slots: number;
  output_slots: number;
}

// Asset catalog (for palette) - matches Rust backend
export interface AssetCatalog {
  items: CatalogEntry[];
  fluids: CatalogEntry[];
  machines: MachineCatalogEntry[];
  tags: string[];
}

// ============ React Flow Node Types ============

// Base node data
export interface BaseNodeData {
  label: string;
}

// Source Node (材料)
export interface SourceNodeData extends BaseNodeData {
  nodeType: "source";
  itemType: "item" | "fluid" | "tag";
  itemId: string;
  amount: number;
}

// Machine Node (加工機械)
export interface MachineNodeData extends BaseNodeData {
  nodeType: "machine";
  machineType: MachineType;
  machineId?: string;
  inputSlots: number;
  outputSlots: number;
}

// Result Node (成果物)
export interface ResultNodeData extends BaseNodeData {
  nodeType: "result";
  itemType: "item" | "fluid";
  itemId: string;
  amount: number;
  chance: number;
}

// Union type for all node data
export type RecipeNodeData = SourceNodeData | MachineNodeData | ResultNodeData;

// Node types for React Flow
export const NODE_TYPES = {
  source: "source",
  machine: "machine",
  result: "result",
} as const;

// Palette item for drag & drop
export interface PaletteItem {
  type: "source" | "machine" | "result";
  itemType?: "item" | "fluid" | "tag";
  id?: string;
  machineType?: MachineType;
}

// Helper functions
export function createDefaultRecipe(id: string): RecipeDef {
  return {
    id,
    machine_type: "Assembler",
    ingredients: [],
    results: [],
    process_time: 1.0,
    stress_impact: 0.0,
    i18n_key: `recipe.${id}`,
  };
}

export function createSourceNodeData(
  itemType: "item" | "fluid" | "tag",
  itemId: string,
  name: string
): SourceNodeData {
  return {
    nodeType: "source",
    label: name,
    itemType,
    itemId,
    amount: 1,
  };
}

export function createMachineNodeData(
  machineType: MachineType,
  machineId: string,
  name: string,
  inputSlots: number,
  outputSlots: number
): MachineNodeData {
  return {
    nodeType: "machine",
    label: name,
    machineType,
    machineId,
    inputSlots,
    outputSlots,
  };
}

export function createResultNodeData(
  itemType: "item" | "fluid",
  itemId: string,
  name: string
): ResultNodeData {
  return {
    nodeType: "result",
    label: name,
    itemType,
    itemId,
    amount: 1,
    chance: 1.0,
  };
}
