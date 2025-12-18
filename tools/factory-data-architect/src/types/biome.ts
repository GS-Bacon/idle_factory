// Biome Types for Editor

export interface ResourceDistribution {
  resourceId: string;
  probability: number; // 0.0 - 1.0
  concentration: number; // 1.0 = normal, 2.0 = double drops
}

export interface NoiseSettings {
  octaves: number;
  frequency: number;
  amplitude: number;
  persistence: number;
  lacunarity: number;
}

export interface BiomeData {
  id: string;
  i18nKey: string;
  // Terrain generation
  baseHeight: number;
  heightVariation: number;
  noiseSettings: NoiseSettings;
  // Resources
  resources: ResourceDistribution[];
  // Surface blocks
  surfaceBlock: string;
  subsurfaceBlock: string;
  stoneBlock: string;
  // Features
  hasWater: boolean;
  waterLevel: number;
  // Temperature
  baseTemperature: number;
}

export interface BiomePreviewSettings {
  seed: number;
  size: number;
  showTerrain: boolean;
  showResources: boolean;
  resourceFilters: string[]; // Which resources to show
}

export function createDefaultBiome(id: string): BiomeData {
  return {
    id,
    i18nKey: `biome.${id}`,
    baseHeight: 64,
    heightVariation: 16,
    noiseSettings: {
      octaves: 4,
      frequency: 0.01,
      amplitude: 1.0,
      persistence: 0.5,
      lacunarity: 2.0,
    },
    resources: [],
    surfaceBlock: "grass",
    subsurfaceBlock: "dirt",
    stoneBlock: "stone",
    hasWater: true,
    waterLevel: 62,
    baseTemperature: 20,
  };
}

export function createDefaultResource(): ResourceDistribution {
  return {
    resourceId: "",
    probability: 0.1,
    concentration: 1.0,
  };
}

export function createDefaultNoiseSettings(): NoiseSettings {
  return {
    octaves: 4,
    frequency: 0.01,
    amplitude: 1.0,
    persistence: 0.5,
    lacunarity: 2.0,
  };
}

export function createDefaultPreviewSettings(): BiomePreviewSettings {
  return {
    seed: 12345,
    size: 256,
    showTerrain: true,
    showResources: true,
    resourceFilters: [],
  };
}
