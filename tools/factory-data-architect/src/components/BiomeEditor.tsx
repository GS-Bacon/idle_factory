import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type {
  BiomeData,
  ResourceDistribution,
  BiomePreviewSettings,
  NoiseSettings,
} from "../types/biome";
import {
  createDefaultBiome,
  createDefaultResource,
  createDefaultNoiseSettings,
  createDefaultPreviewSettings,
} from "../types/biome";
import type { AssetCatalog } from "../types/recipe";
import "./BiomeEditor.css";

// Noise generation for preview (simple implementation)
function generateNoise(
  x: number,
  y: number,
  settings: NoiseSettings,
  seed: number
): number {
  // Simple seeded random
  const seedRand = (x: number, y: number, s: number) => {
    const n = Math.sin(x * 12.9898 + y * 78.233 + s * 43758.5453) * 43758.5453;
    return n - Math.floor(n);
  };

  let value = 0;
  let amplitude = settings.amplitude;
  let frequency = settings.frequency;
  let maxValue = 0;

  for (let i = 0; i < settings.octaves; i++) {
    const nx = x * frequency;
    const ny = y * frequency;

    // Simple interpolated noise
    const x0 = Math.floor(nx);
    const y0 = Math.floor(ny);
    const x1 = x0 + 1;
    const y1 = y0 + 1;

    const sx = nx - x0;
    const sy = ny - y0;

    const n00 = seedRand(x0, y0, seed + i);
    const n10 = seedRand(x1, y0, seed + i);
    const n01 = seedRand(x0, y1, seed + i);
    const n11 = seedRand(x1, y1, seed + i);

    const nx0 = n00 * (1 - sx) + n10 * sx;
    const nx1 = n01 * (1 - sx) + n11 * sx;
    const n = nx0 * (1 - sy) + nx1 * sy;

    value += n * amplitude;
    maxValue += amplitude;

    amplitude *= settings.persistence;
    frequency *= settings.lacunarity;
  }

  return value / maxValue;
}

// Resource color mapping
const RESOURCE_COLORS: Record<string, string> = {
  iron_ore: "#8B4513",
  copper_ore: "#B87333",
  gold_ore: "#FFD700",
  coal: "#2F4F4F",
  oil: "#1a1a1a",
  uranium: "#32CD32",
};

// Preview Canvas Component
interface BiomePreviewProps {
  biome: BiomeData;
  settings: BiomePreviewSettings;
  onScroll: (dx: number, dy: number) => void;
}

function BiomePreview({ biome, settings, onScroll }: BiomePreviewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [lastPos, setLastPos] = useState({ x: 0, y: 0 });

  // Render preview
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;

    // Create image data
    const imageData = ctx.createImageData(width, height);
    const data = imageData.data;

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const worldX = x + offset.x;
        const worldY = y + offset.y;

        // Generate terrain height
        const noise = generateNoise(worldX, worldY, biome.noiseSettings, settings.seed);
        const height = biome.baseHeight + noise * biome.heightVariation;

        // Base color from terrain
        let r = 0, g = 0, b = 0;

        if (settings.showTerrain) {
          if (height < biome.waterLevel) {
            // Water
            r = 64;
            g = 128;
            b = 200;
          } else if (height < biome.baseHeight) {
            // Beach/low ground
            r = 194;
            g = 178;
            b = 128;
          } else {
            // Normal ground - gradient based on height
            const t = Math.min(1, (height - biome.baseHeight) / biome.heightVariation);
            r = Math.floor(100 - t * 30);
            g = Math.floor(150 - t * 50);
            b = Math.floor(80 - t * 20);
          }
        } else {
          r = 40;
          g = 40;
          b = 40;
        }

        // Overlay resources
        if (settings.showResources) {
          for (const resource of biome.resources) {
            if (settings.resourceFilters.length > 0 && !settings.resourceFilters.includes(resource.resourceId)) {
              continue;
            }

            // Use resource probability and concentration for display
            const resourceNoise = generateNoise(
              worldX * 0.1,
              worldY * 0.1,
              createDefaultNoiseSettings(),
              settings.seed + resource.resourceId.length * 1000
            );

            if (resourceNoise < resource.probability) {
              const color = RESOURCE_COLORS[resource.resourceId] || "#ff00ff";
              const intensity = Math.min(1, resource.concentration * 0.5);

              // Parse hex color
              const cr = parseInt(color.slice(1, 3), 16);
              const cg = parseInt(color.slice(3, 5), 16);
              const cb = parseInt(color.slice(5, 7), 16);

              r = Math.floor(r * (1 - intensity) + cr * intensity);
              g = Math.floor(g * (1 - intensity) + cg * intensity);
              b = Math.floor(b * (1 - intensity) + cb * intensity);
            }
          }
        }

        const i = (y * width + x) * 4;
        data[i] = r;
        data[i + 1] = g;
        data[i + 2] = b;
        data[i + 3] = 255;
      }
    }

    ctx.putImageData(imageData, 0, 0);
  }, [biome, settings, offset]);

  // Mouse handlers for scrolling
  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setLastPos({ x: e.clientX, y: e.clientY });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!isDragging) return;

    const dx = e.clientX - lastPos.x;
    const dy = e.clientY - lastPos.y;

    setOffset((prev) => ({ x: prev.x - dx, y: prev.y - dy }));
    setLastPos({ x: e.clientX, y: e.clientY });
    onScroll(-dx, -dy);
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  return (
    <div className="biome-preview">
      <canvas
        ref={canvasRef}
        width={settings.size}
        height={settings.size}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        style={{ cursor: isDragging ? "grabbing" : "grab" }}
      />
      <div className="preview-info">
        <span>Offset: ({offset.x}, {offset.y})</span>
        <span>Seed: {settings.seed}</span>
      </div>
    </div>
  );
}

// Resource Editor
interface ResourceEditorProps {
  resources: ResourceDistribution[];
  catalog: AssetCatalog;
  onChange: (resources: ResourceDistribution[]) => void;
}

function ResourceEditor({ resources, catalog, onChange }: ResourceEditorProps) {
  const addResource = () => {
    onChange([...resources, createDefaultResource()]);
  };

  const updateResource = (index: number, resource: ResourceDistribution) => {
    const newResources = [...resources];
    newResources[index] = resource;
    onChange(newResources);
  };

  const removeResource = (index: number) => {
    onChange(resources.filter((_, i) => i !== index));
  };

  const availableResources = [
    { id: "iron_ore", name: "Iron Ore" },
    { id: "copper_ore", name: "Copper Ore" },
    { id: "gold_ore", name: "Gold Ore" },
    { id: "coal", name: "Coal" },
    { id: "oil", name: "Oil" },
    { id: "uranium", name: "Uranium" },
    ...catalog.items.map((item) => ({ id: item.id, name: item.name })),
  ];

  return (
    <div className="resource-editor">
      <h4>Resources</h4>
      {resources.map((resource, index) => (
        <div key={index} className="resource-item">
          <select
            value={resource.resourceId}
            onChange={(e) => updateResource(index, { ...resource, resourceId: e.target.value })}
          >
            <option value="">-- Select --</option>
            {availableResources.map((res) => (
              <option key={res.id} value={res.id}>{res.name}</option>
            ))}
          </select>
          <div className="resource-params">
            <label>
              Prob:
              <input
                type="number"
                min="0"
                max="1"
                step="0.05"
                value={resource.probability}
                onChange={(e) => updateResource(index, { ...resource, probability: parseFloat(e.target.value) || 0 })}
              />
            </label>
            <label>
              Conc:
              <input
                type="number"
                min="0"
                step="0.1"
                value={resource.concentration}
                onChange={(e) => updateResource(index, { ...resource, concentration: parseFloat(e.target.value) || 1 })}
              />
            </label>
          </div>
          <button onClick={() => removeResource(index)} className="remove-btn">X</button>
        </div>
      ))}
      <button onClick={addResource} className="add-btn">+ Add Resource</button>
    </div>
  );
}

// Noise Settings Editor
interface NoiseEditorProps {
  settings: NoiseSettings;
  onChange: (settings: NoiseSettings) => void;
}

function NoiseEditor({ settings, onChange }: NoiseEditorProps) {
  return (
    <div className="noise-editor">
      <h4>Noise Settings</h4>
      <div className="form-row">
        <label>Octaves</label>
        <input
          type="number"
          min="1"
          max="8"
          value={settings.octaves}
          onChange={(e) => onChange({ ...settings, octaves: parseInt(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>Frequency</label>
        <input
          type="number"
          min="0.001"
          max="1"
          step="0.001"
          value={settings.frequency}
          onChange={(e) => onChange({ ...settings, frequency: parseFloat(e.target.value) || 0.01 })}
        />
      </div>
      <div className="form-row">
        <label>Amplitude</label>
        <input
          type="number"
          min="0"
          max="10"
          step="0.1"
          value={settings.amplitude}
          onChange={(e) => onChange({ ...settings, amplitude: parseFloat(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>Persistence</label>
        <input
          type="number"
          min="0"
          max="1"
          step="0.1"
          value={settings.persistence}
          onChange={(e) => onChange({ ...settings, persistence: parseFloat(e.target.value) || 0.5 })}
        />
      </div>
      <div className="form-row">
        <label>Lacunarity</label>
        <input
          type="number"
          min="1"
          max="4"
          step="0.1"
          value={settings.lacunarity}
          onChange={(e) => onChange({ ...settings, lacunarity: parseFloat(e.target.value) || 2 })}
        />
      </div>
    </div>
  );
}

// Main Biome Editor Component
export default function BiomeEditor() {
  const [biome, setBiome] = useState<BiomeData>(createDefaultBiome("new_biome"));
  const [previewSettings, setPreviewSettings] = useState<BiomePreviewSettings>(
    createDefaultPreviewSettings()
  );
  const [catalog, setCatalog] = useState<AssetCatalog>({
    items: [],
    fluids: [],
    machines: [],
    tags: [],
  });

  useEffect(() => {
    invoke<AssetCatalog>("get_assets_catalog")
      .then(setCatalog)
      .catch(console.error);
  }, []);

  const handleRandomSeed = useCallback(() => {
    setPreviewSettings((prev) => ({
      ...prev,
      seed: Math.floor(Math.random() * 1000000),
    }));
  }, []);

  const handleSave = useCallback(async () => {
    try {
      await invoke("save_biome", { biome });
      alert("Biome saved successfully!");
    } catch (err) {
      console.error(err);
      alert("Save function not implemented yet");
    }
  }, [biome]);

  const toggleResourceFilter = (resourceId: string) => {
    setPreviewSettings((prev) => {
      const filters = prev.resourceFilters.includes(resourceId)
        ? prev.resourceFilters.filter((id) => id !== resourceId)
        : [...prev.resourceFilters, resourceId];
      return { ...prev, resourceFilters: filters };
    });
  };

  return (
    <div className="biome-editor">
      <div className="editor-panel">
        <div className="panel-header">
          <h3>Biome Settings</h3>
        </div>
        <div className="panel-content">
          <div className="form-section">
            <h4>Basic Info</h4>
            <div className="form-row">
              <label>Biome ID</label>
              <input
                type="text"
                value={biome.id}
                onChange={(e) => setBiome({ ...biome, id: e.target.value })}
              />
            </div>
            <div className="form-row">
              <label>i18n Key</label>
              <input
                type="text"
                value={biome.i18nKey}
                onChange={(e) => setBiome({ ...biome, i18nKey: e.target.value })}
              />
            </div>
          </div>

          <div className="form-section">
            <h4>Terrain</h4>
            <div className="form-row">
              <label>Base Height</label>
              <input
                type="number"
                value={biome.baseHeight}
                onChange={(e) => setBiome({ ...biome, baseHeight: parseInt(e.target.value) || 64 })}
              />
            </div>
            <div className="form-row">
              <label>Height Variation</label>
              <input
                type="number"
                value={biome.heightVariation}
                onChange={(e) => setBiome({ ...biome, heightVariation: parseInt(e.target.value) || 16 })}
              />
            </div>
            <div className="form-row">
              <label>Water Level</label>
              <input
                type="number"
                value={biome.waterLevel}
                onChange={(e) => setBiome({ ...biome, waterLevel: parseInt(e.target.value) || 62 })}
              />
            </div>
            <div className="form-row checkbox">
              <label>
                <input
                  type="checkbox"
                  checked={biome.hasWater}
                  onChange={(e) => setBiome({ ...biome, hasWater: e.target.checked })}
                />
                Has Water
              </label>
            </div>
          </div>

          <div className="form-section">
            <h4>Blocks</h4>
            <div className="form-row">
              <label>Surface Block</label>
              <input
                type="text"
                value={biome.surfaceBlock}
                onChange={(e) => setBiome({ ...biome, surfaceBlock: e.target.value })}
              />
            </div>
            <div className="form-row">
              <label>Subsurface Block</label>
              <input
                type="text"
                value={biome.subsurfaceBlock}
                onChange={(e) => setBiome({ ...biome, subsurfaceBlock: e.target.value })}
              />
            </div>
            <div className="form-row">
              <label>Stone Block</label>
              <input
                type="text"
                value={biome.stoneBlock}
                onChange={(e) => setBiome({ ...biome, stoneBlock: e.target.value })}
              />
            </div>
          </div>

          <NoiseEditor
            settings={biome.noiseSettings}
            onChange={(noiseSettings) => setBiome({ ...biome, noiseSettings })}
          />

          <ResourceEditor
            resources={biome.resources}
            catalog={catalog}
            onChange={(resources) => setBiome({ ...biome, resources })}
          />
        </div>
        <div className="panel-footer">
          <button onClick={handleSave} className="save-btn">Save Biome</button>
        </div>
      </div>

      <div className="preview-panel">
        <div className="panel-header">
          <h3>Preview</h3>
          <div className="preview-controls">
            <label>
              <input
                type="checkbox"
                checked={previewSettings.showTerrain}
                onChange={(e) =>
                  setPreviewSettings({ ...previewSettings, showTerrain: e.target.checked })
                }
              />
              Terrain
            </label>
            <label>
              <input
                type="checkbox"
                checked={previewSettings.showResources}
                onChange={(e) =>
                  setPreviewSettings({ ...previewSettings, showResources: e.target.checked })
                }
              />
              Resources
            </label>
          </div>
        </div>

        <BiomePreview
          biome={biome}
          settings={previewSettings}
          onScroll={() => {}}
        />

        <div className="preview-settings">
          <div className="seed-control">
            <label>Seed:</label>
            <input
              type="number"
              value={previewSettings.seed}
              onChange={(e) =>
                setPreviewSettings({ ...previewSettings, seed: parseInt(e.target.value) || 0 })
              }
            />
            <button onClick={handleRandomSeed}>Random</button>
          </div>

          <div className="resource-filters">
            <h4>Resource Filters</h4>
            {biome.resources.map((resource) => (
              <label key={resource.resourceId}>
                <input
                  type="checkbox"
                  checked={
                    previewSettings.resourceFilters.length === 0 ||
                    previewSettings.resourceFilters.includes(resource.resourceId)
                  }
                  onChange={() => toggleResourceFilter(resource.resourceId)}
                />
                {resource.resourceId}
              </label>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
