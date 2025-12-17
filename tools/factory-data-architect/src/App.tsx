import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { ItemEditor } from "./components/ItemEditor";
import RecipeEditor from "./components/RecipeEditor";
import "./App.css";

// デフォルトのアセットパス
const DEFAULT_ASSETS_PATH = "C:/Users/bacon/OneDrive/ドキュメント/github/IdealFactoryGame/my-bevy-project/assets";

type EditorTab = "items" | "recipes";

function App() {
  const [assetsPath, setAssetsPath] = useState<string | null>(null);
  const [isSettingUp, setIsSettingUp] = useState(true);
  const [activeTab, setActiveTab] = useState<EditorTab>("items");

  // Load saved assets path on startup, or use default
  useEffect(() => {
    const initAssetsPath = async () => {
      // まず既存の設定を確認
      const existingPath = await invoke<string | null>("get_assets_path").catch(() => null);
      if (existingPath) {
        setAssetsPath(existingPath);
        setIsSettingUp(false);
        return;
      }

      // デフォルトパスを設定
      try {
        await invoke("set_assets_path", { path: DEFAULT_ASSETS_PATH });
        setAssetsPath(DEFAULT_ASSETS_PATH);
        setIsSettingUp(false);
      } catch {
        // デフォルトパスが無効な場合は手動選択を促す
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
      title: "アセットフォルダを選択",
    });

    if (selected && typeof selected === "string") {
      try {
        await invoke("set_assets_path", { path: selected });
        setAssetsPath(selected);
        setIsSettingUp(false);
      } catch (error) {
        alert(`エラー: ${error}`);
      }
    }
  }, []);

  // Setup screen
  if (isSettingUp && !assetsPath) {
    return (
      <main className="container setup-screen">
        <h1>Factory Data Architect</h1>
        <p>アセットフォルダを選択してください。</p>
        <p className="hint">
          これはゲームの assets/ フォルダで、アイコン、モデル、ローカライズファイルが保存される場所です。
        </p>
        <button onClick={handleSelectAssetsFolder} className="setup-button">
          フォルダを選択
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
        </nav>
        <div className="assets-path-display">
          <span>Assets: {assetsPath}</span>
          <button onClick={handleSelectAssetsFolder} className="change-path-button">
            変更
          </button>
        </div>
      </header>

      <div className="editor-content">
        {activeTab === "items" && <ItemEditor assetsPath={assetsPath} />}
        {activeTab === "recipes" && <RecipeEditor />}
      </div>
    </main>
  );
}

export default App;
