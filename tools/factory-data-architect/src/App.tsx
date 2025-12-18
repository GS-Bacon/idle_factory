import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ItemEditor } from "./components/ItemEditor";
import RecipeEditor from "./components/RecipeEditor";
import QuestEditor from "./components/QuestEditor";
import MultiblockEditor from "./components/MultiblockEditor";
import BiomeEditor from "./components/BiomeEditor";
import SoundEditor from "./components/SoundEditor";
import "./App.css";

// Default assets path
const DEFAULT_ASSETS_PATH = "C:/Users/bacon/OneDrive/ドキュメント/github/IdealFactoryGame/my-bevy-project/assets";

type EditorTab = "items" | "recipes" | "quests" | "multiblock" | "biome" | "sounds";
type ItemCategory = "item" | "machine" | "multiblock";

// Item category selector component
interface ItemCategorySelectorProps {
  onSelect: (category: ItemCategory) => void;
}

function ItemCategorySelector({ onSelect }: ItemCategorySelectorProps) {
  return (
    <div className="category-selector">
      <h2>Create New Item</h2>
      <p>Select the type of item to create:</p>
      <div className="category-buttons">
        <button className="category-btn item" onClick={() => onSelect("item")}>
          <span className="icon">📦</span>
          <span className="label">Simple Item</span>
          <span className="desc">Basic item without special functionality</span>
        </button>
        <button className="category-btn machine" onClick={() => onSelect("machine")}>
          <span className="icon">⚙️</span>
          <span className="label">Machine</span>
          <span className="desc">Single-block machine with processing capability</span>
        </button>
        <button className="category-btn multiblock" onClick={() => onSelect("multiblock")}>
          <span className="icon">🏗️</span>
          <span className="label">Multiblock Machine</span>
          <span className="desc">Large machine spanning multiple blocks</span>
        </button>
      </div>
    </div>
  );
}

// Items tab with category routing
interface ItemsTabProps {
  assetsPath: string | null;
}

function ItemsTab({ assetsPath }: ItemsTabProps) {
  const [selectedCategory, setSelectedCategory] = useState<ItemCategory | null>(null);
  const [showSelector, setShowSelector] = useState(false);

  const handleNewItem = useCallback(() => {
    setShowSelector(true);
    setSelectedCategory(null);
  }, []);

  const handleCategorySelect = useCallback((category: ItemCategory) => {
    setSelectedCategory(category);
    setShowSelector(false);
  }, []);

  const handleBack = useCallback(() => {
    setSelectedCategory(null);
    setShowSelector(false);
  }, []);

  // Show category selector when creating new item
  if (showSelector) {
    return <ItemCategorySelector onSelect={handleCategorySelect} />;
  }

  // Route to appropriate editor based on category
  if (selectedCategory === "multiblock") {
    return (
      <div className="editor-with-back">
        <button onClick={handleBack} className="back-btn">← Back to Items</button>
        <MultiblockEditor />
      </div>
    );
  }

  // Default: show item editor with "New" button
  return (
    <div className="items-tab">
      <div className="items-toolbar">
        <button onClick={handleNewItem} className="new-item-btn">+ New Item</button>
      </div>
      <ItemEditor assetsPath={assetsPath} />
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
