import { useState, useCallback, useRef, useEffect } from "react";
import {
  ReactFlow,
  Controls,
  Background,
  BackgroundVariant,
  addEdge,
  useNodesState,
  useEdgesState,
  Connection,
  Node,
  Edge,
  ReactFlowProvider,
  useReactFlow,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { invoke } from "@tauri-apps/api/core";
import SourceNode from "./nodes/SourceNode";
import MachineNode from "./nodes/MachineNode";
import ResultNode from "./nodes/ResultNode";
import type {
  RecipeNodeData,
  SourceNodeData,
  MachineNodeData,
  ResultNodeData,
  RecipeDef,
  MachineType,
  AssetCatalog,
  PaletteItem,
} from "../types/recipe";
import "./RecipeEditor.css";

// Node type mapping for React Flow
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes: any = {
  source: SourceNode,
  machine: MachineNode,
  result: ResultNode,
};

// Helper to get machine type string
function getMachineTypeString(mt: MachineType): string {
  return typeof mt === "string" ? mt : mt.Custom;
}

// Machine types are loaded from catalog - no hardcoded defaults

// Palette Panel Component
interface PalettePanelProps {
  catalog: AssetCatalog;
  onDragStart: (event: React.DragEvent, item: PaletteItem) => void;
}

function PalettePanel({ catalog, onDragStart }: PalettePanelProps) {
  const [activeTab, setActiveTab] = useState<"items" | "fluids" | "machines" | "tags">("items");

  return (
    <div className="palette-panel">
      <div className="palette-header">
        <h3>Palette</h3>
      </div>
      <div className="palette-tabs">
        <button
          className={activeTab === "items" ? "active" : ""}
          onClick={() => setActiveTab("items")}
        >
          Items
        </button>
        <button
          className={activeTab === "fluids" ? "active" : ""}
          onClick={() => setActiveTab("fluids")}
        >
          Fluids
        </button>
        <button
          className={activeTab === "machines" ? "active" : ""}
          onClick={() => setActiveTab("machines")}
        >
          Machines
        </button>
        <button
          className={activeTab === "tags" ? "active" : ""}
          onClick={() => setActiveTab("tags")}
        >
          Tags
        </button>
      </div>
      <div className="palette-content">
        {activeTab === "items" && (
          <div className="palette-list">
            {catalog.items.map((entry) => (
              <div
                key={entry.id}
                className="palette-item item"
                draggable
                onDragStart={(e) =>
                  onDragStart(e, { type: "source", itemType: "item", id: entry.id })
                }
              >
                <span className="icon">📦</span>
                <span className="label">{entry.name}</span>
              </div>
            ))}
          </div>
        )}
        {activeTab === "fluids" && (
          <div className="palette-list">
            {catalog.fluids.map((entry) => (
              <div
                key={entry.id}
                className="palette-item fluid"
                draggable
                onDragStart={(e) =>
                  onDragStart(e, { type: "source", itemType: "fluid", id: entry.id })
                }
              >
                <span className="icon">💧</span>
                <span className="label">{entry.name}</span>
              </div>
            ))}
          </div>
        )}
        {activeTab === "machines" && (
          <div className="palette-list">
            {catalog.machines.map((machine) => (
              <div
                key={machine.id}
                className="palette-item machine"
                draggable
                onDragStart={(e) =>
                  onDragStart(e, { type: "machine", machineType: { Custom: machine.id } })
                }
              >
                <span className="icon">⚙️</span>
                <span className="label">{machine.name}</span>
              </div>
            ))}
          </div>
        )}
        {activeTab === "tags" && (
          <div className="palette-list">
            {catalog.tags.map((tag) => (
              <div
                key={tag}
                className="palette-item tag"
                draggable
                onDragStart={(e) =>
                  onDragStart(e, { type: "source", itemType: "tag", id: tag })
                }
              >
                <span className="icon">🏷️</span>
                <span className="label">{tag}</span>
              </div>
            ))}
          </div>
        )}
      </div>
      <div className="palette-footer">
        <div
          className="palette-item result"
          draggable
          onDragStart={(e) =>
            onDragStart(e, { type: "result", itemType: "item", id: "" })
          }
        >
          <span className="icon">📤</span>
          <span className="label">Result Node</span>
        </div>
      </div>
    </div>
  );
}

// Inspector Panel Component
interface InspectorPanelProps {
  selectedNode: Node | null;
  recipeId: string;
  processTime: number;
  stressImpact: number;
  i18nKey: string;
  catalog: AssetCatalog;
  onNodeUpdate: (nodeId: string, data: Partial<RecipeNodeData>) => void;
  onRecipeUpdate: (field: string, value: string | number) => void;
}

function InspectorPanel({
  selectedNode,
  recipeId,
  processTime,
  stressImpact,
  i18nKey,
  catalog,
  onNodeUpdate,
  onRecipeUpdate,
}: InspectorPanelProps) {
  const nodeData = selectedNode?.data as RecipeNodeData | undefined;

  return (
    <div className="inspector-panel">
      <div className="inspector-header">
        <h3>Inspector</h3>
      </div>
      <div className="inspector-content">
        <div className="inspector-section">
          <h4>Recipe Properties</h4>
          <div className="form-group">
            <label>Recipe ID</label>
            <input
              type="text"
              value={recipeId}
              onChange={(e) => onRecipeUpdate("recipeId", e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Process Time (ticks)</label>
            <input
              type="number"
              value={processTime}
              onChange={(e) =>
                onRecipeUpdate("processTime", parseInt(e.target.value) || 0)
              }
            />
          </div>
          <div className="form-group">
            <label>Stress Impact</label>
            <input
              type="number"
              step="0.1"
              value={stressImpact}
              onChange={(e) =>
                onRecipeUpdate("stressImpact", parseFloat(e.target.value) || 0)
              }
            />
          </div>
          <div className="form-group">
            <label>i18n Key</label>
            <input
              type="text"
              value={i18nKey}
              onChange={(e) => onRecipeUpdate("i18nKey", e.target.value)}
            />
          </div>
        </div>

        {nodeData && selectedNode && (
          <div className="inspector-section">
            <h4>Node Properties</h4>
            {nodeData.nodeType === "source" && (
              <SourceNodeInspector
                data={nodeData as SourceNodeData}
                catalog={catalog}
                onUpdate={(data) => onNodeUpdate(selectedNode.id, data)}
              />
            )}
            {nodeData.nodeType === "machine" && (
              <MachineNodeInspector
                data={nodeData as MachineNodeData}
                catalog={catalog}
                onUpdate={(data) => onNodeUpdate(selectedNode.id, data)}
              />
            )}
            {nodeData.nodeType === "result" && (
              <ResultNodeInspector
                data={nodeData as ResultNodeData}
                catalog={catalog}
                onUpdate={(data) => onNodeUpdate(selectedNode.id, data)}
              />
            )}
          </div>
        )}
      </div>
    </div>
  );
}

// Source Node Inspector
interface SourceNodeInspectorProps {
  data: SourceNodeData;
  catalog: AssetCatalog;
  onUpdate: (data: Partial<SourceNodeData>) => void;
}

function SourceNodeInspector({ data, catalog, onUpdate }: SourceNodeInspectorProps) {
  // For tags, it's string[], for items/fluids it's CatalogEntry[]
  const isTag = data.itemType === "tag";
  const catalogEntries = data.itemType === "item" ? catalog.items : catalog.fluids;

  return (
    <>
      <div className="form-group">
        <label>Type</label>
        <select
          value={data.itemType}
          onChange={(e) =>
            onUpdate({
              itemType: e.target.value as "item" | "fluid" | "tag",
              itemId: "",
            })
          }
        >
          <option value="item">Item</option>
          <option value="fluid">Fluid</option>
          <option value="tag">Tag</option>
        </select>
      </div>
      <div className="form-group">
        <label>Item ID</label>
        <select value={data.itemId} onChange={(e) => onUpdate({ itemId: e.target.value })}>
          <option value="">-- Select --</option>
          {isTag
            ? catalog.tags.map((tag) => (
                <option key={tag} value={tag}>
                  {tag}
                </option>
              ))
            : catalogEntries.map((entry) => (
                <option key={entry.id} value={entry.id}>
                  {entry.name}
                </option>
              ))}
        </select>
      </div>
      <div className="form-group">
        <label>Amount</label>
        <input
          type="number"
          min="1"
          value={data.amount}
          onChange={(e) => onUpdate({ amount: parseInt(e.target.value) || 1 })}
        />
      </div>
    </>
  );
}

// Machine Node Inspector
interface MachineNodeInspectorProps {
  data: MachineNodeData;
  catalog: AssetCatalog;
  onUpdate: (data: Partial<MachineNodeData>) => void;
}

function MachineNodeInspector({ data, catalog, onUpdate }: MachineNodeInspectorProps) {
  const currentType = getMachineTypeString(data.machineType);

  return (
    <>
      <div className="form-group">
        <label>Machine Type</label>
        <select
          value={currentType}
          onChange={(e) => {
            const val = e.target.value;
            onUpdate({ machineType: { Custom: val } });
          }}
        >
          <option value="">-- Select --</option>
          {catalog.machines.map((machine) => (
            <option key={machine.id} value={machine.id}>
              {machine.name}
            </option>
          ))}
        </select>
      </div>
      <div className="form-group">
        <label>Input Slots</label>
        <input
          type="number"
          min="1"
          max="6"
          value={data.inputSlots}
          onChange={(e) => onUpdate({ inputSlots: parseInt(e.target.value) || 1 })}
        />
      </div>
      <div className="form-group">
        <label>Output Slots</label>
        <input
          type="number"
          min="1"
          max="6"
          value={data.outputSlots}
          onChange={(e) => onUpdate({ outputSlots: parseInt(e.target.value) || 1 })}
        />
      </div>
    </>
  );
}

// Result Node Inspector
interface ResultNodeInspectorProps {
  data: ResultNodeData;
  catalog: AssetCatalog;
  onUpdate: (data: Partial<ResultNodeData>) => void;
}

function ResultNodeInspector({ data, catalog, onUpdate }: ResultNodeInspectorProps) {
  const entries = data.itemType === "item" ? catalog.items : catalog.fluids;

  return (
    <>
      <div className="form-group">
        <label>Type</label>
        <select
          value={data.itemType}
          onChange={(e) =>
            onUpdate({ itemType: e.target.value as "item" | "fluid", itemId: "" })
          }
        >
          <option value="item">Item</option>
          <option value="fluid">Fluid</option>
        </select>
      </div>
      <div className="form-group">
        <label>Item ID</label>
        <select value={data.itemId} onChange={(e) => onUpdate({ itemId: e.target.value })}>
          <option value="">-- Select --</option>
          {entries.map((entry) => (
            <option key={entry.id} value={entry.id}>
              {entry.name}
            </option>
          ))}
        </select>
      </div>
      <div className="form-group">
        <label>Amount</label>
        <input
          type="number"
          min="1"
          value={data.amount}
          onChange={(e) => onUpdate({ amount: parseInt(e.target.value) || 1 })}
        />
      </div>
      <div className="form-group">
        <label>Chance (0.0 - 1.0)</label>
        <input
          type="number"
          min="0"
          max="1"
          step="0.1"
          value={data.chance}
          onChange={(e) => onUpdate({ chance: parseFloat(e.target.value) || 1.0 })}
        />
      </div>
    </>
  );
}

// Main Recipe Editor Flow Component
function RecipeEditorFlow() {
  const reactFlowWrapper = useRef<HTMLDivElement>(null);
  const { screenToFlowPosition } = useReactFlow();

  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [selectedNode, setSelectedNode] = useState<Node | null>(null);

  const [recipeId, setRecipeId] = useState("new_recipe");
  const [processTime, setProcessTime] = useState(60);
  const [stressImpact, setStressImpact] = useState(1.0);
  const [i18nKey, setI18nKey] = useState("recipe.new_recipe");

  const [catalog, setCatalog] = useState<AssetCatalog>({
    items: [],
    fluids: [],
    machines: [],
    tags: [],
  });

  const [draggedItem, setDraggedItem] = useState<PaletteItem | null>(null);

  // Load catalog on mount
  useEffect(() => {
    invoke<AssetCatalog>("get_assets_catalog")
      .then(setCatalog)
      .catch(console.error);
  }, []);

  const onConnect = useCallback(
    (params: Connection) => setEdges((eds) => addEdge(params, eds)),
    [setEdges]
  );

  const onSelectionChange = useCallback(
    ({ nodes: selectedNodes }: { nodes: Node[] }) => {
      if (selectedNodes.length === 1) {
        setSelectedNode(selectedNodes[0]);
      } else {
        setSelectedNode(null);
      }
    },
    []
  );

  const handleDragStart = useCallback(
    (_event: React.DragEvent, item: PaletteItem) => {
      setDraggedItem(item);
    },
    []
  );

  const handleDragOver = useCallback((event: React.DragEvent) => {
    event.preventDefault();
    event.dataTransfer.dropEffect = "move";
  }, []);

  const handleDrop = useCallback(
    (event: React.DragEvent) => {
      event.preventDefault();

      if (!draggedItem || !reactFlowWrapper.current) return;

      const position = screenToFlowPosition({
        x: event.clientX,
        y: event.clientY,
      });

      const newNodeId = `node_${Date.now()}`;
      let newNode: Node;

      if (draggedItem.type === "source") {
        newNode = {
          id: newNodeId,
          type: "source",
          position,
          data: {
            nodeType: "source",
            itemType: draggedItem.itemType || "item",
            itemId: draggedItem.id || "",
            amount: 1,
            label: draggedItem.id || "Source",
          },
        };
      } else if (draggedItem.type === "machine") {
        const mt = draggedItem.machineType || "Assembler";
        newNode = {
          id: newNodeId,
          type: "machine",
          position,
          data: {
            nodeType: "machine",
            machineType: mt,
            inputSlots: 2,
            outputSlots: 1,
            label: getMachineTypeString(mt),
          },
        };
      } else {
        newNode = {
          id: newNodeId,
          type: "result",
          position,
          data: {
            nodeType: "result",
            itemType: (draggedItem.itemType as "item" | "fluid") || "item",
            itemId: "",
            amount: 1,
            chance: 1.0,
            label: "Result",
          },
        };
      }

      setNodes((nds) => [...nds, newNode]);
      setDraggedItem(null);
    },
    [draggedItem, screenToFlowPosition, setNodes]
  );

  const handleNodeUpdate = useCallback(
    (nodeId: string, newData: Partial<RecipeNodeData>) => {
      setNodes((nds) =>
        nds.map((node) => {
          if (node.id === nodeId) {
            const currentData = node.data as unknown as RecipeNodeData;
            const updatedData = { ...currentData, ...newData };
            // Update label based on changes
            if ("itemId" in newData && newData.itemId) {
              updatedData.label = newData.itemId;
            }
            if ("machineType" in newData) {
              const mt = newData.machineType as MachineType;
              updatedData.label = getMachineTypeString(mt);
            }
            return { ...node, data: updatedData };
          }
          return node;
        })
      );
    },
    [setNodes]
  );

  const handleRecipeUpdate = useCallback((field: string, value: string | number) => {
    switch (field) {
      case "recipeId":
        setRecipeId(value as string);
        break;
      case "processTime":
        setProcessTime(value as number);
        break;
      case "stressImpact":
        setStressImpact(value as number);
        break;
      case "i18nKey":
        setI18nKey(value as string);
        break;
    }
  }, []);

  // Convert graph to RecipeDef
  const convertToRecipeDef = useCallback((): RecipeDef | null => {
    const machineNode = nodes.find((n) => (n.data as unknown as RecipeNodeData).nodeType === "machine");
    if (!machineNode) {
      alert("Recipe requires a machine node");
      return null;
    }

    const machineData = machineNode.data as unknown as MachineNodeData;

    // Find ingredients (source nodes connected to machine)
    const sourceNodes = nodes.filter((n) => (n.data as unknown as RecipeNodeData).nodeType === "source");
    const connectedSources = sourceNodes.filter((sourceNode) =>
      edges.some(
        (e: Edge) => e.source === sourceNode.id && e.target === machineNode.id
      )
    );
    const ingredients = connectedSources.map((sourceNode) => {
      const data = sourceNode.data as unknown as SourceNodeData;
      return {
        ingredient_type:
          data.itemType === "tag"
            ? { type: "Tag" as const, value: data.itemId }
            : { type: "Item" as const, value: data.itemId },
        amount: data.amount,
      };
    });

    // Find results (result nodes connected from machine)
    const resultNodes = nodes.filter((n) => (n.data as unknown as RecipeNodeData).nodeType === "result");
    const connectedResults = resultNodes.filter((resultNode) =>
      edges.some(
        (e: Edge) => e.source === machineNode.id && e.target === resultNode.id
      )
    );
    const results = connectedResults.map((resultNode) => {
      const data = resultNode.data as unknown as ResultNodeData;
      return {
        product_type:
          data.itemType === "fluid"
            ? { type: "Fluid" as const, value: data.itemId }
            : { type: "Item" as const, value: data.itemId },
        amount: data.amount,
        chance: data.chance,
      };
    });

    return {
      id: recipeId,
      machine_type: machineData.machineType,
      ingredients,
      results,
      process_time: processTime,
      stress_impact: stressImpact,
      i18n_key: i18nKey,
    };
  }, [nodes, edges, recipeId, processTime, stressImpact, i18nKey]);

  const handleSave = useCallback(async () => {
    const recipe = convertToRecipeDef();
    if (!recipe) return;

    try {
      await invoke("save_recipe", { recipe });
      alert("Recipe saved successfully!");
    } catch (err) {
      alert(`Failed to save recipe: ${err}`);
    }
  }, [convertToRecipeDef]);

  const handleExportJson = useCallback(() => {
    const recipe = convertToRecipeDef();
    if (!recipe) return;

    const json = JSON.stringify(recipe, null, 2);
    console.log("Exported Recipe:", json);
    navigator.clipboard.writeText(json);
    alert("Recipe JSON copied to clipboard!");
  }, [convertToRecipeDef]);

  const handleClear = useCallback(() => {
    setNodes([]);
    setEdges([]);
    setSelectedNode(null);
  }, [setNodes, setEdges]);

  return (
    <div className="recipe-editor">
      <PalettePanel catalog={catalog} onDragStart={handleDragStart} />
      <div
        className="canvas-panel"
        ref={reactFlowWrapper}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
      >
        <div className="canvas-toolbar">
          <button onClick={handleSave}>💾 Save</button>
          <button onClick={handleExportJson}>📋 Export JSON</button>
          <button onClick={handleClear}>🗑️ Clear</button>
        </div>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onConnect={onConnect}
          onSelectionChange={onSelectionChange}
          nodeTypes={nodeTypes}
          fitView
        >
          <Controls />
          <Background variant={BackgroundVariant.Dots} gap={12} size={1} />
        </ReactFlow>
      </div>
      <InspectorPanel
        selectedNode={selectedNode}
        recipeId={recipeId}
        processTime={processTime}
        stressImpact={stressImpact}
        i18nKey={i18nKey}
        catalog={catalog}
        onNodeUpdate={handleNodeUpdate}
        onRecipeUpdate={handleRecipeUpdate}
      />
    </div>
  );
}

// Wrapper with ReactFlowProvider
export default function RecipeEditor() {
  return (
    <ReactFlowProvider>
      <RecipeEditorFlow />
    </ReactFlowProvider>
  );
}
