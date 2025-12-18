import { useState, useCallback, useEffect } from "react";
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
  Handle,
  Position,
  NodeProps,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { invoke } from "@tauri-apps/api/core";
import type {
  QuestData,
  QuestNodeData,
  QuestRequirement,
  RewardType,
} from "../types/quest";
import {
  createDefaultQuest,
  createDefaultRequirement,
  createDefaultItemReward,
  createDefaultPortReward,
} from "../types/quest";
import type { AssetCatalog } from "../types/recipe";
import "./QuestEditor.css";

// Quest Node Component
function QuestNode({ data, selected }: NodeProps) {
  const questData = data as unknown as QuestNodeData;
  const quest = questData.quest;
  const isMain = quest.questType === "main";

  return (
    <div className={`quest-node ${isMain ? "main" : "sub"} ${selected ? "selected" : ""}`}>
      <Handle type="target" position={Position.Top} />
      <div className="quest-node-header">
        <span className="quest-type-badge">{isMain ? "MAIN" : "SUB"}</span>
        <span className="quest-phase">Phase {quest.phase}</span>
      </div>
      <div className="quest-node-body">
        <div className="quest-id">{quest.id}</div>
        <div className="quest-requirements">
          {quest.requirements.length} requirements
        </div>
        <div className="quest-rewards">
          {quest.rewards.length} rewards
        </div>
      </div>
      <Handle type="source" position={Position.Bottom} />
    </div>
  );
}

const nodeTypes = {
  quest: QuestNode,
};

// Inspector Panel for Quest
interface QuestInspectorProps {
  quest: QuestData | null;
  catalog: AssetCatalog;
  onUpdate: (quest: QuestData) => void;
}

function QuestInspector({ quest, catalog, onUpdate }: QuestInspectorProps) {
  if (!quest) {
    return (
      <div className="quest-inspector">
        <div className="inspector-header">
          <h3>Inspector</h3>
        </div>
        <div className="inspector-empty">
          Select a quest to edit
        </div>
      </div>
    );
  }

  const updateField = <K extends keyof QuestData>(field: K, value: QuestData[K]) => {
    onUpdate({ ...quest, [field]: value });
  };

  const addRequirement = () => {
    updateField("requirements", [...quest.requirements, createDefaultRequirement()]);
  };

  const updateRequirement = (index: number, req: QuestRequirement) => {
    const newReqs = [...quest.requirements];
    newReqs[index] = req;
    updateField("requirements", newReqs);
  };

  const removeRequirement = (index: number) => {
    updateField("requirements", quest.requirements.filter((_, i) => i !== index));
  };

  const addReward = (type: "item" | "port") => {
    const newReward = type === "item" ? createDefaultItemReward() : createDefaultPortReward();
    updateField("rewards", [...quest.rewards, newReward]);
  };

  const updateReward = (index: number, reward: RewardType) => {
    const newRewards = [...quest.rewards];
    newRewards[index] = reward;
    updateField("rewards", newRewards);
  };

  const removeReward = (index: number) => {
    updateField("rewards", quest.rewards.filter((_, i) => i !== index));
  };

  return (
    <div className="quest-inspector">
      <div className="inspector-header">
        <h3>Quest Inspector</h3>
      </div>
      <div className="inspector-content">
        <div className="inspector-section">
          <h4>Basic Info</h4>
          <div className="form-group">
            <label>Quest ID</label>
            <input
              type="text"
              value={quest.id}
              onChange={(e) => updateField("id", e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>i18n Key</label>
            <input
              type="text"
              value={quest.i18nKey}
              onChange={(e) => updateField("i18nKey", e.target.value)}
            />
          </div>
          <div className="form-group">
            <label>Phase</label>
            <input
              type="number"
              min="1"
              value={quest.phase}
              onChange={(e) => updateField("phase", parseInt(e.target.value) || 1)}
            />
          </div>
        </div>

        <div className="inspector-section">
          <h4>Requirements</h4>
          {quest.requirements.map((req, index) => (
            <div key={index} className="requirement-item">
              <select
                value={req.itemType}
                onChange={(e) => updateRequirement(index, { ...req, itemType: e.target.value as QuestRequirement["itemType"] })}
              >
                <option value="item">Item</option>
                <option value="fluid">Fluid</option>
                <option value="power">Power</option>
                <option value="torque">Torque</option>
              </select>
              <select
                value={req.itemId}
                onChange={(e) => updateRequirement(index, { ...req, itemId: e.target.value })}
              >
                <option value="">-- Select --</option>
                {(req.itemType === "item" ? catalog.items : catalog.fluids).map((entry) => (
                  <option key={entry.id} value={entry.id}>{entry.name}</option>
                ))}
              </select>
              <input
                type="number"
                min="1"
                value={req.amount}
                onChange={(e) => updateRequirement(index, { ...req, amount: parseInt(e.target.value) || 1 })}
              />
              <button onClick={() => removeRequirement(index)} className="remove-btn">X</button>
            </div>
          ))}
          <button onClick={addRequirement} className="add-btn">+ Add Requirement</button>
        </div>

        <div className="inspector-section">
          <h4>Rewards</h4>
          {quest.rewards.map((reward, index) => (
            <div key={index} className="reward-item">
              <select
                value={reward.type}
                onChange={(e) => {
                  const type = e.target.value as "Item" | "PortUnlock";
                  if (type === "Item") {
                    updateReward(index, { type: "Item", itemId: "", amount: 1 });
                  } else {
                    updateReward(index, { type: "PortUnlock", count: 1 });
                  }
                }}
              >
                <option value="Item">Item</option>
                <option value="PortUnlock">Port Unlock</option>
              </select>
              {reward.type === "Item" && (
                <>
                  <select
                    value={reward.itemId}
                    onChange={(e) => updateReward(index, { ...reward, itemId: e.target.value })}
                  >
                    <option value="">-- Select --</option>
                    {catalog.items.map((entry) => (
                      <option key={entry.id} value={entry.id}>{entry.name}</option>
                    ))}
                  </select>
                  <input
                    type="number"
                    min="1"
                    value={reward.amount}
                    onChange={(e) => updateReward(index, { ...reward, amount: parseInt(e.target.value) || 1 })}
                  />
                </>
              )}
              {reward.type === "PortUnlock" && (
                <input
                  type="number"
                  min="1"
                  value={reward.count}
                  onChange={(e) => updateReward(index, { ...reward, count: parseInt(e.target.value) || 1 })}
                  placeholder="Ports to unlock"
                />
              )}
              <button onClick={() => removeReward(index)} className="remove-btn">X</button>
            </div>
          ))}
          <div className="reward-add-buttons">
            <button onClick={() => addReward("item")} className="add-btn">+ Item</button>
            <button onClick={() => addReward("port")} className="add-btn">+ Port</button>
          </div>
        </div>
      </div>
    </div>
  );
}

// Phase Separator Component
interface PhaseSeparatorProps {
  phases: number[];
}

function PhaseOverlay({ phases }: PhaseSeparatorProps) {
  return (
    <div className="phase-overlay">
      {phases.map((phase) => (
        <div key={phase} className="phase-label" style={{ top: `${(phase - 1) * 300 + 50}px` }}>
          Phase {phase}
        </div>
      ))}
    </div>
  );
}

// Main Quest Editor Flow
function QuestEditorFlow({ questType }: { questType: "main" | "sub" }) {
  const [nodes, setNodes, onNodesChange] = useNodesState<Node>([]);
  const [edges, setEdges, onEdgesChange] = useEdgesState<Edge>([]);
  const [selectedQuest, setSelectedQuest] = useState<QuestData | null>(null);
  const [catalog, setCatalog] = useState<AssetCatalog>({
    items: [],
    fluids: [],
    machines: [],
    tags: [],
  });

  // Load catalog on mount
  useEffect(() => {
    invoke<AssetCatalog>("get_assets_catalog")
      .then(setCatalog)
      .catch(console.error);
  }, []);

  const onConnect = useCallback(
    (params: Connection) => {
      setEdges((eds) => addEdge(params, eds));
      // Update prerequisites when connecting
      if (params.source && params.target) {
        setNodes((nds) =>
          nds.map((node) => {
            if (node.id === params.target) {
              const data = node.data as unknown as QuestNodeData;
              const quest = data.quest;
              if (!quest.prerequisites.includes(params.source!)) {
                return {
                  ...node,
                  data: {
                    ...data,
                    quest: {
                      ...quest,
                      prerequisites: [...quest.prerequisites, params.source!],
                    },
                  },
                };
              }
            }
            return node;
          })
        );
      }
    },
    [setEdges, setNodes]
  );

  const onSelectionChange = useCallback(
    ({ nodes: selectedNodes }: { nodes: Node[] }) => {
      if (selectedNodes.length === 1) {
        const data = selectedNodes[0].data as unknown as QuestNodeData;
        setSelectedQuest(data.quest);
      } else {
        setSelectedQuest(null);
      }
    },
    []
  );

  const handleAddQuest = useCallback(() => {
    const id = `${questType}_quest_${Date.now()}`;
    const quest = createDefaultQuest(id, questType);
    const newNode: Node = {
      id,
      type: "quest",
      position: { x: 250, y: nodes.length * 150 + 50 },
      data: { quest, label: id } as QuestNodeData,
    };
    setNodes((nds) => [...nds, newNode]);
  }, [questType, nodes.length, setNodes]);

  const handleUpdateQuest = useCallback(
    (updatedQuest: QuestData) => {
      setSelectedQuest(updatedQuest);
      setNodes((nds) =>
        nds.map((node) => {
          if (node.id === updatedQuest.id) {
            return {
              ...node,
              id: updatedQuest.id,
              data: { quest: updatedQuest, label: updatedQuest.id },
            };
          }
          return node;
        })
      );
    },
    [setNodes]
  );

  const handleSave = useCallback(async () => {
    const quests = nodes.map((node) => {
      const data = node.data as unknown as QuestNodeData;
      return data.quest;
    });
    try {
      await invoke("save_quests", { quests, questType });
      alert("Quests saved successfully!");
    } catch (err) {
      console.error(err);
      alert("Save function not implemented yet");
    }
  }, [nodes, questType]);

  const handleClear = useCallback(() => {
    setNodes([]);
    setEdges([]);
    setSelectedQuest(null);
  }, [setNodes, setEdges]);

  // Calculate phases for overlay
  const phases = [...new Set(nodes.map((node) => {
    const data = node.data as unknown as QuestNodeData;
    return data.quest.phase;
  }))].sort((a, b) => a - b);

  return (
    <div className="quest-editor">
      <div className="canvas-panel">
        <div className="canvas-toolbar">
          <button onClick={handleAddQuest}>+ Add Quest</button>
          <button onClick={handleSave}>Save</button>
          <button onClick={handleClear}>Clear</button>
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
          <PhaseOverlay phases={phases} />
        </ReactFlow>
      </div>
      <QuestInspector
        quest={selectedQuest}
        catalog={catalog}
        onUpdate={handleUpdateQuest}
      />
    </div>
  );
}

// Main Quest Editor with Tabs
export default function QuestEditor() {
  const [activeTab, setActiveTab] = useState<"main" | "sub">("main");

  return (
    <div className="quest-editor-container">
      <div className="quest-tabs">
        <button
          className={activeTab === "main" ? "active" : ""}
          onClick={() => setActiveTab("main")}
        >
          Main Quests
        </button>
        <button
          className={activeTab === "sub" ? "active" : ""}
          onClick={() => setActiveTab("sub")}
        >
          Sub Quests
        </button>
      </div>
      <ReactFlowProvider>
        <QuestEditorFlow questType={activeTab} />
      </ReactFlowProvider>
    </div>
  );
}
