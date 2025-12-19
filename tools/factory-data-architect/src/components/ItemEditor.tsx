import { useState, useCallback, useEffect, DragEvent } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  ItemData,
  ItemCategory,
  AnimationType,
  LocalizationData,
  createDefaultItemData,
  createDefaultLocalizationData,
} from "../types";

// カテゴリ定義
const ITEM_CATEGORIES: { value: ItemCategory; label: string; icon: string }[] = [
  { value: "item", label: "アイテム", icon: "📦" },
  { value: "machine", label: "機械", icon: "⚙️" },
  { value: "multiblock", label: "マルチブロック", icon: "🏗️" },
];

// カテゴリに基づいてi18n_keyを生成
function generateI18nKey(id: string, category: ItemCategory): string {
  return `${category}.${id}`;
}

import "./ItemEditor.css";

// Helper type for animation params (excluding None)
type AnimationWithParams = Exclude<AnimationType, { type: "None" }>;
type AnyAnimationParams = AnimationWithParams["params"];

interface ItemEditorProps {
  assetsPath: string | null;
  itemId?: string | null;
  onSave?: (item: ItemData, localization: LocalizationData) => void;
}

export function ItemEditor({ assetsPath, itemId, onSave }: ItemEditorProps) {
  const [item, setItem] = useState<ItemData>(createDefaultItemData(itemId || "new_item"));
  const [localization, setLocalization] = useState<LocalizationData>(
    createDefaultLocalizationData()
  );
  const [isDraggingIcon, setIsDraggingIcon] = useState(false);
  const [isDraggingModel, setIsDraggingModel] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Load item data when itemId changes
  useEffect(() => {
    if (itemId) {
      setIsLoading(true);
      setLoadError(null);

      // Try to load item data from file
      const loadItem = async () => {
        try {
          const itemData = await invoke<ItemData>("load_item_data", {
            path: `${assetsPath}/data/items/${itemId}.ron`,
          });
          setItem(itemData);

          // Also load localization
          try {
            const loc = await invoke<LocalizationData>("load_localization", {
              i18nKey: itemData.i18n_key,
            });
            setLocalization(loc);
          } catch {
            // Localization may not exist yet
            setLocalization(createDefaultLocalizationData());
          }
        } catch (err) {
          console.error("Failed to load item:", err);
          setLoadError(`アイテム読み込みエラー: ${err}`);
          // Create new item with the given ID
          setItem(createDefaultItemData(itemId));
          setLocalization(createDefaultLocalizationData());
        } finally {
          setIsLoading(false);
        }
      };

      loadItem();
    } else {
      // New item
      setItem(createDefaultItemData("new_item"));
      setLocalization(createDefaultLocalizationData());
      setLoadError(null);
    }
  }, [itemId, assetsPath]);

  // File selection handler
  const handleSelectFile = useCallback(
    async (
      field: "icon_path" | "model_path",
      filters: { name: string; extensions: string[] }[]
    ) => {
      const selected = await open({
        multiple: false,
        filters,
        defaultPath: assetsPath ?? undefined,
      });

      if (selected && typeof selected === "string") {
        // Convert to relative path if possible
        let relativePath = selected;
        if (assetsPath) {
          try {
            relativePath = await invoke<string>("to_relative_path", {
              absolutePath: selected,
            });
          } catch {
            // Keep absolute path if conversion fails
          }
        }

        setItem((prev) => ({
          ...prev,
          asset: { ...prev.asset, [field]: relativePath },
        }));
      }
    },
    [assetsPath]
  );

  // Drag & Drop handlers
  const handleDragOver = useCallback((e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
  }, []);

  const handleDrop = useCallback(
    async (e: DragEvent, field: "icon_path" | "model_path") => {
      e.preventDefault();
      e.stopPropagation();

      if (field === "icon_path") setIsDraggingIcon(false);
      if (field === "model_path") setIsDraggingModel(false);

      const files = e.dataTransfer.files;
      if (files.length > 0) {
        const filePath = (files[0] as File & { path?: string }).path;
        if (filePath) {
          let relativePath = filePath;
          if (assetsPath) {
            try {
              relativePath = await invoke<string>("to_relative_path", {
                absolutePath: filePath,
              });
            } catch {
              // Keep absolute path if conversion fails
            }
          }

          setItem((prev) => ({
            ...prev,
            asset: { ...prev.asset, [field]: relativePath },
          }));
        }
      }
    },
    [assetsPath]
  );

  // Animation type change handler
  const handleAnimationTypeChange = useCallback(
    (type: AnimationType["type"]) => {
      let animation: AnimationType;
      switch (type) {
        case "Rotational":
          animation = {
            type: "Rotational",
            params: { axis: [0, 1, 0], speed: 90 },
          };
          break;
        case "Linear":
          animation = {
            type: "Linear",
            params: { direction: [0, 1, 0], distance: 1, speed: 1 },
          };
          break;
        case "Skeletal":
          animation = {
            type: "Skeletal",
            params: { animation_path: "", looping: true },
          };
          break;
        default:
          animation = { type: "None" };
      }

      setItem((prev) => ({
        ...prev,
        asset: { ...prev.asset, animation },
      }));
    },
    []
  );

  // Update animation params
  const updateAnimationParams = useCallback(
    (params: Partial<AnyAnimationParams>) => {
      setItem((prev) => {
        const animation = prev.asset.animation;
        if (animation.type === "None") return prev;
        return {
          ...prev,
          asset: {
            ...prev.asset,
            animation: {
              ...animation,
              params: { ...animation.params, ...params },
            } as AnimationType,
          },
        };
      });
    },
    []
  );

  // Localization update handlers
  const updateLocalization = useCallback(
    (lang: "ja" | "en", field: "name" | "description", value: string) => {
      setLocalization((prev) => ({
        ...prev,
        [lang]: { ...prev[lang], [field]: value },
      }));
    },
    []
  );

  // Save handler
  const handleSave = useCallback(async () => {
    try {
      // Save item data to file
      await invoke("save_item_data", {
        item,
        path: `${assetsPath}/data/items/${item.id}.ron`,
      });

      // Save localization to locale files
      await invoke("save_localization", {
        i18nKey: item.i18n_key,
        localization,
      });

      onSave?.(item, localization);
      alert("保存しました");
    } catch (error) {
      alert(`保存エラー: ${error}`);
    }
  }, [item, localization, onSave, assetsPath]);

  // Get preview URL for icon
  const getIconPreviewUrl = useCallback((iconPath: string | null): string => {
    if (!iconPath) return "";
    // For Tauri, we need to use the asset protocol or convertFileSrc
    // For simplicity, just show the path
    return iconPath;
  }, []);

  if (isLoading) {
    return (
      <div className="item-editor">
        <h2>アイテムエディタ</h2>
        <div className="loading-state">読み込み中...</div>
      </div>
    );
  }

  return (
    <div className="item-editor">
      <h2>アイテムエディタ</h2>

      {loadError && (
        <div className="load-error">
          <p>{loadError}</p>
          <p>新規アイテムとして編集します。</p>
        </div>
      )}

      {/* Basic Info */}
      <section className="editor-section">
        <h3>基本情報</h3>
        <div className="form-row">
          <label>
            ID:
            <input
              type="text"
              value={item.id}
              onChange={(e) => {
                const id = e.target.value;
                setItem((prev) => ({
                  ...prev,
                  id,
                  i18n_key: generateI18nKey(id, prev.category),
                }));
              }}
            />
          </label>
        </div>
        <div className="form-row">
          <label>カテゴリ:</label>
          <div className="category-selector-inline">
            {ITEM_CATEGORIES.map((cat) => (
              <button
                key={cat.value}
                type="button"
                className={`category-btn-small ${item.category === cat.value ? "selected" : ""}`}
                onClick={() =>
                  setItem((prev) => ({
                    ...prev,
                    category: cat.value,
                    i18n_key: generateI18nKey(prev.id, cat.value),
                  }))
                }
              >
                <span className="cat-icon">{cat.icon}</span>
                <span className="cat-label">{cat.label}</span>
              </button>
            ))}
          </div>
        </div>
        <div className="form-row">
          <label>
            i18n Key:
            <input type="text" value={item.i18n_key} readOnly className="readonly-field" />
          </label>
          <span className="auto-generated-hint">（自動生成）</span>
        </div>
      </section>

      {/* Asset Configuration */}
      <section className="editor-section">
        <h3>ビジュアル & アニメーション設定</h3>

        {/* Icon Path */}
        <div className="form-row file-input-row">
          <label>Icon Path:</label>
          <div
            className={`drop-zone ${isDraggingIcon ? "dragging" : ""}`}
            onDragOver={handleDragOver}
            onDragEnter={() => setIsDraggingIcon(true)}
            onDragLeave={() => setIsDraggingIcon(false)}
            onDrop={(e) => handleDrop(e, "icon_path")}
          >
            <input
              type="text"
              value={item.asset.icon_path ?? ""}
              onChange={(e) =>
                setItem((prev) => ({
                  ...prev,
                  asset: { ...prev.asset, icon_path: e.target.value || null },
                }))
              }
              placeholder="ドラッグ&ドロップまたは選択..."
            />
            <button
              type="button"
              onClick={() =>
                handleSelectFile("icon_path", [
                  { name: "Images", extensions: ["png", "jpg", "jpeg", "webp"] },
                ])
              }
            >
              選択
            </button>
          </div>
          {item.asset.icon_path && (
            <div className="preview-container">
              <img
                src={getIconPreviewUrl(item.asset.icon_path)}
                alt="Icon Preview"
                className="icon-preview"
                onError={(e) => {
                  (e.target as HTMLImageElement).style.display = "none";
                }}
              />
              <span className="path-display">{item.asset.icon_path}</span>
            </div>
          )}
        </div>

        {/* Model Path */}
        <div className="form-row file-input-row">
          <label>Model Path:</label>
          <div
            className={`drop-zone ${isDraggingModel ? "dragging" : ""}`}
            onDragOver={handleDragOver}
            onDragEnter={() => setIsDraggingModel(true)}
            onDragLeave={() => setIsDraggingModel(false)}
            onDrop={(e) => handleDrop(e, "model_path")}
          >
            <input
              type="text"
              value={item.asset.model_path ?? ""}
              onChange={(e) =>
                setItem((prev) => ({
                  ...prev,
                  asset: { ...prev.asset, model_path: e.target.value || null },
                }))
              }
              placeholder="ドラッグ&ドロップまたは選択..."
            />
            <button
              type="button"
              onClick={() =>
                handleSelectFile("model_path", [
                  { name: "3D Models", extensions: ["glb", "gltf", "vox", "obj"] },
                ])
              }
            >
              選択
            </button>
          </div>
          {item.asset.model_path && (
            <div className="preview-container">
              <span className="path-display">{item.asset.model_path}</span>
            </div>
          )}
        </div>

        {/* Animation Config */}
        <div className="form-row">
          <label>Animation Type:</label>
          <select
            value={item.asset.animation.type}
            onChange={(e) =>
              handleAnimationTypeChange(e.target.value as AnimationType["type"])
            }
          >
            <option value="None">None</option>
            <option value="Rotational">Rotational (回転)</option>
            <option value="Linear">Linear (往復)</option>
            <option value="Skeletal">Skeletal (スケルタル)</option>
          </select>
        </div>

        {/* Animation Parameters */}
        {item.asset.animation.type === "Rotational" && (
          <div className="animation-params">
            <div className="form-row">
              <label>回転軸 (X, Y, Z):</label>
              <div className="vector-input">
                <input
                  type="number"
                  step="0.1"
                  value={item.asset.animation.params.axis[0]}
                  onChange={(e) =>
                    updateAnimationParams({
                      axis: [
                        parseFloat(e.target.value),
                        item.asset.animation.type === "Rotational"
                          ? item.asset.animation.params.axis[1]
                          : 0,
                        item.asset.animation.type === "Rotational"
                          ? item.asset.animation.params.axis[2]
                          : 0,
                      ],
                    })
                  }
                />
                <input
                  type="number"
                  step="0.1"
                  value={item.asset.animation.params.axis[1]}
                  onChange={(e) =>
                    updateAnimationParams({
                      axis: [
                        item.asset.animation.type === "Rotational"
                          ? item.asset.animation.params.axis[0]
                          : 0,
                        parseFloat(e.target.value),
                        item.asset.animation.type === "Rotational"
                          ? item.asset.animation.params.axis[2]
                          : 0,
                      ],
                    })
                  }
                />
                <input
                  type="number"
                  step="0.1"
                  value={item.asset.animation.params.axis[2]}
                  onChange={(e) =>
                    updateAnimationParams({
                      axis: [
                        item.asset.animation.type === "Rotational"
                          ? item.asset.animation.params.axis[0]
                          : 0,
                        item.asset.animation.type === "Rotational"
                          ? item.asset.animation.params.axis[1]
                          : 0,
                        parseFloat(e.target.value),
                      ],
                    })
                  }
                />
              </div>
            </div>
            <div className="form-row">
              <label>回転速度 (deg/s):</label>
              <input
                type="number"
                value={item.asset.animation.params.speed}
                onChange={(e) =>
                  updateAnimationParams({ speed: parseFloat(e.target.value) })
                }
              />
            </div>
          </div>
        )}

        {item.asset.animation.type === "Linear" && (
          <div className="animation-params">
            <div className="form-row">
              <label>移動方向 (X, Y, Z):</label>
              <div className="vector-input">
                <input
                  type="number"
                  step="0.1"
                  value={item.asset.animation.params.direction[0]}
                  onChange={(e) =>
                    updateAnimationParams({
                      direction: [
                        parseFloat(e.target.value),
                        item.asset.animation.type === "Linear"
                          ? item.asset.animation.params.direction[1]
                          : 0,
                        item.asset.animation.type === "Linear"
                          ? item.asset.animation.params.direction[2]
                          : 0,
                      ],
                    })
                  }
                />
                <input
                  type="number"
                  step="0.1"
                  value={item.asset.animation.params.direction[1]}
                  onChange={(e) =>
                    updateAnimationParams({
                      direction: [
                        item.asset.animation.type === "Linear"
                          ? item.asset.animation.params.direction[0]
                          : 0,
                        parseFloat(e.target.value),
                        item.asset.animation.type === "Linear"
                          ? item.asset.animation.params.direction[2]
                          : 0,
                      ],
                    })
                  }
                />
                <input
                  type="number"
                  step="0.1"
                  value={item.asset.animation.params.direction[2]}
                  onChange={(e) =>
                    updateAnimationParams({
                      direction: [
                        item.asset.animation.type === "Linear"
                          ? item.asset.animation.params.direction[0]
                          : 0,
                        item.asset.animation.type === "Linear"
                          ? item.asset.animation.params.direction[1]
                          : 0,
                        parseFloat(e.target.value),
                      ],
                    })
                  }
                />
              </div>
            </div>
            <div className="form-row">
              <label>移動距離:</label>
              <input
                type="number"
                step="0.1"
                value={item.asset.animation.params.distance}
                onChange={(e) =>
                  updateAnimationParams({
                    distance: parseFloat(e.target.value),
                  })
                }
              />
            </div>
            <div className="form-row">
              <label>移動速度 (units/s):</label>
              <input
                type="number"
                step="0.1"
                value={item.asset.animation.params.speed}
                onChange={(e) =>
                  updateAnimationParams({ speed: parseFloat(e.target.value) })
                }
              />
            </div>
          </div>
        )}

        {item.asset.animation.type === "Skeletal" && (
          <div className="animation-params">
            <div className="form-row">
              <label>Animation File:</label>
              <input
                type="text"
                value={item.asset.animation.params.animation_path}
                onChange={(e) =>
                  updateAnimationParams({ animation_path: e.target.value })
                }
                placeholder="path/to/animation.glb"
              />
            </div>
            <div className="form-row">
              <label>
                <input
                  type="checkbox"
                  checked={item.asset.animation.params.looping}
                  onChange={(e) =>
                    updateAnimationParams({ looping: e.target.checked })
                  }
                />
                ループ再生
              </label>
            </div>
          </div>
        )}
      </section>

      {/* Localization */}
      <section className="editor-section">
        <h3>ローカライズ</h3>

        <div className="localization-grid">
          {/* Japanese */}
          <div className="localization-lang">
            <h4>🇯🇵 Japanese</h4>
            <div className="form-row">
              <label>Name:</label>
              <input
                type="text"
                value={localization.ja.name}
                onChange={(e) => updateLocalization("ja", "name", e.target.value)}
                placeholder="日本語名"
              />
            </div>
            <div className="form-row">
              <label>Description:</label>
              <textarea
                value={localization.ja.description}
                onChange={(e) =>
                  updateLocalization("ja", "description", e.target.value)
                }
                placeholder="日本語説明"
                rows={3}
              />
            </div>
          </div>

          {/* English */}
          <div className="localization-lang">
            <h4>🇺🇸 English</h4>
            <div className="form-row">
              <label>Name:</label>
              <input
                type="text"
                value={localization.en.name}
                onChange={(e) => updateLocalization("en", "name", e.target.value)}
                placeholder="English Name"
              />
            </div>
            <div className="form-row">
              <label>Description:</label>
              <textarea
                value={localization.en.description}
                onChange={(e) =>
                  updateLocalization("en", "description", e.target.value)
                }
                placeholder="English Description"
                rows={3}
              />
            </div>
          </div>
        </div>
      </section>

      {/* Actions */}
      <section className="editor-actions">
        <button type="button" onClick={handleSave} className="save-button">
          保存
        </button>
      </section>
    </div>
  );
}
