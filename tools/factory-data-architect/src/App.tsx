import { useState, useEffect, useCallback, Component, ErrorInfo, ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ItemEditor } from "./components/ItemEditor";
import RecipeEditor from "./components/RecipeEditor";
import QuestEditor from "./components/QuestEditor";
import MultiblockEditor from "./components/MultiblockEditor";
import BiomeEditor from "./components/BiomeEditor";
import SoundEditor from "./components/SoundEditor";
import "./App.css";

// Error Boundary for crash logging
interface ErrorBoundaryState {
  hasError: boolean;
  error: Error | null;
  errorInfo: ErrorInfo | null;
}

class ErrorBoundary extends Component<{ children: ReactNode }, ErrorBoundaryState> {
  constructor(props: { children: ReactNode }) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError(error: Error): Partial<ErrorBoundaryState> {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    this.setState({ errorInfo });
    // Log error to console for debugging
    console.error("=== CRASH LOG ===");
    console.error("Error:", error.message);
    console.error("Stack:", error.stack);
    console.error("Component Stack:", errorInfo.componentStack);
    console.error("=================");
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="error-boundary">
          <h2>エラーが発生しました</h2>
          <p>アプリケーションでエラーが発生しました。</p>
          <details>
            <summary>エラー詳細</summary>
            <pre>{this.state.error?.message}</pre>
            <pre>{this.state.error?.stack}</pre>
            {this.state.errorInfo && (
              <pre>{this.state.errorInfo.componentStack}</pre>
            )}
          </details>
          <button onClick={() => window.location.reload()}>
            再読み込み
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

// Default assets path
const DEFAULT_ASSETS_PATH = "C:/Users/bacon/OneDrive/ドキュメント/github/IdealFactoryGame/my-bevy-project/assets";

type EditorTab = "items" | "recipes" | "quests" | "multiblock" | "biome" | "sounds";

// Saved item type
interface SavedItem {
  id: string;
  i18n_key: string;
  subcategory?: string;
}

// Asset catalog type from Rust
interface CatalogEntry {
  id: string;
  name: string;
  icon_path: string | null;
}

interface AssetCatalog {
  items: CatalogEntry[];
  fluids: CatalogEntry[];
  machines: CatalogEntry[];
  tags: string[];
}

// Items tab with category routing
interface ItemsTabProps {
  assetsPath: string | null;
}

function ItemsTab({ assetsPath }: ItemsTabProps) {
  const [items, setItems] = useState<SavedItem[]>([]);
  const [selectedItemId, setSelectedItemId] = useState<string | null>(null);
  const [showEditor, setShowEditor] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Load existing items on mount
  useEffect(() => {
    const loadItems = async () => {
      if (!assetsPath) {
        setIsLoading(false);
        return;
      }
      try {
        const catalog = await invoke<AssetCatalog>("get_assets_catalog");
        const loadedItems: SavedItem[] = catalog.items.map((entry) => ({
          id: entry.id,
          i18n_key: `item.${entry.id}`, // Default, will be updated when item is loaded
        }));
        setItems(loadedItems);
      } catch (err) {
        console.error("Failed to load items:", err);
      } finally {
        setIsLoading(false);
      }
    };
    loadItems();
  }, [assetsPath]);

  const handleNewItem = useCallback(() => {
    setSelectedItemId(null);
    setShowEditor(true);
  }, []);

  const handleSelectItem = useCallback((itemId: string) => {
    setSelectedItemId(itemId);
    setShowEditor(true);
  }, []);

  const handleSaveItem = useCallback((item: { id: string; i18n_key: string; subcategory?: string }) => {
    setItems((prev) => {
      const existing = prev.find((i) => i.id === item.id);
      if (existing) {
        return prev.map((i) => (i.id === item.id ? { id: item.id, i18n_key: item.i18n_key, subcategory: item.subcategory } : i));
      }
      return [...prev, { id: item.id, i18n_key: item.i18n_key, subcategory: item.subcategory }];
    });
  }, []);

  const handleDeleteItem = useCallback(async (itemId: string) => {
    if (confirm(`アイテム "${itemId}" を削除しますか？\n※ファイルも削除されます`)) {
      try {
        await invoke("delete_item_data", { itemId });
        setItems((prev) => prev.filter((i) => i.id !== itemId));
        if (selectedItemId === itemId) {
          setSelectedItemId(null);
          setShowEditor(false);
        }
      } catch (err) {
        alert(`削除エラー: ${err}`);
      }
    }
  }, [selectedItemId]);

  // Get existing item IDs for validation
  const existingItemIds = items.map((i) => i.id);

  // Get existing subcategories for autocomplete
  const existingSubcategories = [...new Set(items.map((i) => i.subcategory).filter((s): s is string => !!s))];

  if (isLoading) {
    return (
      <div className="items-tab-layout">
        <div className="items-list-panel">
          <div className="loading-state">読み込み中...</div>
        </div>
      </div>
    );
  }

  return (
    <div className="items-tab-layout">
      {/* Left: Item List */}
      <div className="items-list-panel">
        <div className="items-list-header">
          <h3>アイテム一覧</h3>
          <button onClick={handleNewItem} className="new-item-btn">+ 新規</button>
        </div>
        <div className="items-list">
          {items.length === 0 ? (
            <div className="empty-list">アイテムがありません</div>
          ) : (
            items.map((item) => (
              <div
                key={item.id}
                className={`item-list-entry ${selectedItemId === item.id ? "selected" : ""}`}
                onClick={() => handleSelectItem(item.id)}
              >
                <span className="item-icon">📦</span>
                <span className="item-name">{item.id}</span>
                <button
                  className="delete-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDeleteItem(item.id);
                  }}
                >
                  ×
                </button>
              </div>
            ))
          )}
        </div>
      </div>

      {/* Right: Editor */}
      <div className="items-editor-panel">
        {showEditor ? (
          <ErrorBoundary key={selectedItemId || "new"}>
            <ItemEditor
              key={selectedItemId || "new"}
              assetsPath={assetsPath}
              itemId={selectedItemId}
              existingItemIds={existingItemIds}
              existingSubcategories={existingSubcategories}
              onSave={(item) => handleSaveItem({ id: item.id, i18n_key: item.i18n_key, subcategory: item.subcategory })}
            />
          </ErrorBoundary>
        ) : (
          <div className="no-selection">
            <p>左のリストからアイテムを選択するか、</p>
            <p>「+ 新規」ボタンで新しいアイテムを作成してください</p>
          </div>
        )}
      </div>
    </div>
  );
}

function App() {
  const [assetsPath, setAssetsPath] = useState<string | null>(null);
  const [isSettingUp, setIsSettingUp] = useState(true);
  const [activeTab, setActiveTab] = useState<EditorTab>("items");

  // Load saved assets path on startup, or use default
  useEffect(() => {
    const initAssetsPath = async () => {
      // Check existing settings first
      const existingPath = await invoke<string | null>("get_assets_path").catch(() => null);
      if (existingPath) {
        setAssetsPath(existingPath);
        setIsSettingUp(false);
        return;
      }

      // Set default path
      try {
        await invoke("set_assets_path", { path: DEFAULT_ASSETS_PATH });
        setAssetsPath(DEFAULT_ASSETS_PATH);
        setIsSettingUp(false);
      } catch {
        // If default path is invalid, prompt manual selection
        setIsSettingUp(true);
      }
    };

    initAssetsPath();
  }, []);

  // Select assets folder
  const handleSelectAssetsFolder = useCallback(async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Assets Folder",
    });

    if (selected && typeof selected === "string") {
      try {
        await invoke("set_assets_path", { path: selected });
        setAssetsPath(selected);
        setIsSettingUp(false);
      } catch (error) {
        alert(`Error: ${error}`);
      }
    }
  }, []);

  // Setup screen
  if (isSettingUp && !assetsPath) {
    return (
      <main className="container setup-screen">
        <h1>Factory Data Architect</h1>
        <p>Please select your assets folder.</p>
        <p className="hint">
          This is the game's assets/ folder where icons, models, and localization files are stored.
        </p>
        <button onClick={handleSelectAssetsFolder} className="setup-button">
          Select Folder
        </button>
      </main>
    );
  }

  return (
    <main className="container">
      <header className="app-header">
        <h1>Factory Data Architect</h1>
        <nav className="editor-tabs">
          <button
            className={activeTab === "items" ? "active" : ""}
            onClick={() => setActiveTab("items")}
          >
            📦 Items
          </button>
          <button
            className={activeTab === "recipes" ? "active" : ""}
            onClick={() => setActiveTab("recipes")}
          >
            ⚙️ Recipes
          </button>
          <button
            className={activeTab === "quests" ? "active" : ""}
            onClick={() => setActiveTab("quests")}
          >
            📜 Quests
          </button>
          <button
            className={activeTab === "multiblock" ? "active" : ""}
            onClick={() => setActiveTab("multiblock")}
          >
            🏗️ Multiblock
          </button>
          <button
            className={activeTab === "biome" ? "active" : ""}
            onClick={() => setActiveTab("biome")}
          >
            🌍 Biomes
          </button>
          <button
            className={activeTab === "sounds" ? "active" : ""}
            onClick={() => setActiveTab("sounds")}
          >
            🔊 Sounds
          </button>
        </nav>
        <div className="assets-path-display">
          <span>Assets: {assetsPath}</span>
          <button onClick={handleSelectAssetsFolder} className="change-path-button">
            Change
          </button>
        </div>
      </header>

      <div className="editor-content">
        {activeTab === "items" && <ItemsTab assetsPath={assetsPath} />}
        {activeTab === "recipes" && <RecipeEditor />}
        {activeTab === "quests" && <QuestEditor />}
        {activeTab === "multiblock" && <MultiblockEditor />}
        {activeTab === "biome" && <BiomeEditor />}
        {activeTab === "sounds" && <SoundEditor assetsPath={assetsPath} />}
      </div>
    </main>
  );
}

export default App;
