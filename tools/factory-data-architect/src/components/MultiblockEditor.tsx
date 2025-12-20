import { useState, useCallback, useRef, useEffect } from "react";
import { Canvas, useThree, ThreeEvent } from "@react-three/fiber";
import { OrbitControls, Grid, Text } from "@react-three/drei";
import { invoke } from "@tauri-apps/api/core";
import * as THREE from "three";
import type {
  MultiblockMachineData,
  BlockPlacement,
  IOPort,
  Direction,
} from "../types/multiblock";
import {
  createDefaultMultiblockMachine,
  createDefaultIOPort,
  getFaceFromNormal,
} from "../types/multiblock";
import type { AssetCatalog } from "../types/recipe";
import "./MultiblockEditor.css";

// Block colors for different block types
const BLOCK_COLORS: Record<string, string> = {
  machine_casing: "#4a90d9",
  machine_frame: "#3a7bc8",
  glass: "#87ceeb",
  pipe: "#8b4513",
  wire: "#ffd700",
  default: "#808080",
};

// IO Port colors
const PORT_COLORS: Record<string, string> = {
  item_input: "#22c55e",
  item_output: "#ef4444",
  fluid_input: "#3b82f6",
  fluid_output: "#f97316",
  power: "#fbbf24",
  torque: "#a855f7",
};

// Single Block Component
interface BlockProps {
  position: [number, number, number];
  blockId: string;
  selected: boolean;
  onClick: (e: ThreeEvent<MouseEvent>) => void;
  onFaceClick?: (face: Direction) => void;
}

function Block({ position, blockId, selected, onClick, onFaceClick }: BlockProps) {
  const meshRef = useRef<THREE.Mesh>(null);
  const color = BLOCK_COLORS[blockId] || BLOCK_COLORS.default;

  const handleClick = (e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();

    if (onFaceClick && e.face) {
      const normal = e.face.normal;
      const face = getFaceFromNormal([normal.x, normal.y, normal.z]);
      onFaceClick(face);
    } else {
      onClick(e);
    }
  };

  return (
    <mesh
      ref={meshRef}
      position={position}
      onClick={handleClick}
    >
      <boxGeometry args={[0.95, 0.95, 0.95]} />
      <meshStandardMaterial
        color={color}
        transparent={true}
        opacity={selected ? 1 : 0.8}
      />
      {selected && (
        <lineSegments>
          <edgesGeometry args={[new THREE.BoxGeometry(0.96, 0.96, 0.96)]} />
          <lineBasicMaterial color="#ffffff" linewidth={2} />
        </lineSegments>
      )}
    </mesh>
  );
}

// IO Port Indicator Component
interface PortIndicatorProps {
  port: IOPort;
  onClick: () => void;
}

function PortIndicator({ port, onClick }: PortIndicatorProps) {
  const [x, y, z] = port.position;
  const color = PORT_COLORS[port.portType] || "#ffffff";

  // Calculate offset based on direction
  const offsets: Record<Direction, [number, number, number]> = {
    north: [0, 0, -0.5],
    south: [0, 0, 0.5],
    east: [0.5, 0, 0],
    west: [-0.5, 0, 0],
    up: [0, 0.5, 0],
    down: [0, -0.5, 0],
  };
  const offset = offsets[port.direction];

  return (
    <mesh
      position={[x + offset[0], y + offset[1], z + offset[2]]}
      onClick={(e) => { e.stopPropagation(); onClick(); }}
    >
      <sphereGeometry args={[0.15, 16, 16]} />
      <meshStandardMaterial color={color} emissive={color} emissiveIntensity={0.5} />
    </mesh>
  );
}

// Ghost Block for placement preview
interface GhostBlockProps {
  position: [number, number, number];
  visible: boolean;
}

function GhostBlock({ position, visible }: GhostBlockProps) {
  if (!visible) return null;
  return (
    <mesh position={position}>
      <boxGeometry args={[0.95, 0.95, 0.95]} />
      <meshStandardMaterial color="#ffffff" transparent opacity={0.3} />
    </mesh>
  );
}

// Grid Floor
function GridFloor({ size }: { size: number }) {
  return (
    <Grid
      args={[size, size]}
      cellSize={1}
      cellThickness={0.5}
      cellColor="#444"
      sectionSize={5}
      sectionThickness={1}
      sectionColor="#666"
      fadeDistance={50}
      position={[size / 2 - 0.5, -0.01, size / 2 - 0.5]}
    />
  );
}

// Rotation Preview Component
interface RotationPreviewProps {
  blocks: BlockPlacement[];
  size: [number, number, number];
  rotation: number; // 0, 90, 180, 270
}

function RotationPreview({ blocks, size, rotation }: RotationPreviewProps) {
  const rotatedBlocks = blocks.map((block) => {
    let x = block.x;
    let z = block.z;
    const cx = (size[0] - 1) / 2;
    const cz = (size[2] - 1) / 2;

    // Rotate around center
    const rad = (rotation * Math.PI) / 180;
    const cos = Math.cos(rad);
    const sin = Math.sin(rad);
    const nx = Math.round((x - cx) * cos - (z - cz) * sin + cx);
    const nz = Math.round((x - cx) * sin + (z - cz) * cos + cz);

    return { ...block, x: nx, z: nz };
  });

  return (
    <group>
      {rotatedBlocks.map((block, i) => (
        <mesh key={i} position={[block.x, block.y, block.z]}>
          <boxGeometry args={[0.9, 0.9, 0.9]} />
          <meshStandardMaterial
            color={BLOCK_COLORS[block.blockId] || BLOCK_COLORS.default}
            transparent
            opacity={0.6}
          />
        </mesh>
      ))}
    </group>
  );
}

// Camera Controller
function CameraController({ size }: { size: number }) {
  const { camera } = useThree();

  useEffect(() => {
    camera.position.set(size * 1.5, size, size * 1.5);
    camera.lookAt(size / 2, 0, size / 2);
  }, [camera, size]);

  return null;
}

// Block Palette Component
interface BlockPaletteProps {
  selectedBlock: string;
  onSelectBlock: (blockId: string) => void;
  searchTerm: string;
  onSearchChange: (term: string) => void;
  catalog: AssetCatalog;
}

function BlockPalette({ selectedBlock, onSelectBlock, searchTerm, onSearchChange, catalog }: BlockPaletteProps) {
  const defaultBlocks = [
    { id: "machine_casing", name: "Machine Casing" },
    { id: "machine_frame", name: "Machine Frame" },
    { id: "glass", name: "Glass" },
    { id: "pipe", name: "Pipe" },
    { id: "wire", name: "Wire" },
  ];

  const allBlocks = [...defaultBlocks, ...catalog.items.map((item) => ({ id: item.id, name: item.name }))];
  const filteredBlocks = allBlocks.filter((block) =>
    block.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    block.id.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="block-palette">
      <div className="palette-header">
        <h3>Block Palette</h3>
        <input
          type="text"
          placeholder="Search blocks..."
          value={searchTerm}
          onChange={(e) => onSearchChange(e.target.value)}
        />
      </div>
      <div className="palette-list">
        {filteredBlocks.map((block) => (
          <div
            key={block.id}
            className={`palette-item ${selectedBlock === block.id ? "selected" : ""}`}
            onClick={() => onSelectBlock(block.id)}
          >
            <div
              className="block-preview"
              style={{ backgroundColor: BLOCK_COLORS[block.id] || BLOCK_COLORS.default }}
            />
            <span>{block.name}</span>
          </div>
        ))}
      </div>
    </div>
  );
}

// Inspector Panel
interface MultiblockInspectorProps {
  machine: MultiblockMachineData;
  selectedPort: IOPort | null;
  onUpdate: (machine: MultiblockMachineData) => void;
  onPortUpdate: (port: IOPort) => void;
  onPortRemove: () => void;
}

function MultiblockInspector({
  machine,
  selectedPort,
  onUpdate,
  onPortUpdate,
  onPortRemove,
}: MultiblockInspectorProps) {
  return (
    <div className="multiblock-inspector">
      <div className="inspector-header">
        <h3>Inspector</h3>
      </div>
      <div className="inspector-content">
        <div className="inspector-section">
          <h4>Machine Info</h4>
          <div className="form-group">
            <label>ID</label>
            <input
              type="text"
              value={machine.id}
              onChange={(e) => onUpdate({ ...machine, id: e.target.value })}
            />
          </div>
          <div className="form-group">
            <label>Size (W x H x D)</label>
            <div className="size-inputs">
              <input
                type="number"
                min="1"
                max="10"
                value={machine.size[0]}
                onChange={(e) =>
                  onUpdate({ ...machine, size: [parseInt(e.target.value) || 1, machine.size[1], machine.size[2]] })
                }
              />
              <input
                type="number"
                min="1"
                max="10"
                value={machine.size[1]}
                onChange={(e) =>
                  onUpdate({ ...machine, size: [machine.size[0], parseInt(e.target.value) || 1, machine.size[2]] })
                }
              />
              <input
                type="number"
                min="1"
                max="10"
                value={machine.size[2]}
                onChange={(e) =>
                  onUpdate({ ...machine, size: [machine.size[0], machine.size[1], parseInt(e.target.value) || 1] })
                }
              />
            </div>
          </div>
        </div>

        <div className="inspector-section">
          <h4>Parameters</h4>
          <div className="form-group">
            <label>Torque Consumption</label>
            <input
              type="number"
              min="0"
              value={machine.baseTorqueConsumption}
              onChange={(e) =>
                onUpdate({ ...machine, baseTorqueConsumption: parseInt(e.target.value) || 0 })
              }
            />
          </div>
          <div className="form-group">
            <label>Processing Speed</label>
            <input
              type="number"
              min="0"
              step="0.1"
              value={machine.baseProcessingSpeed}
              onChange={(e) =>
                onUpdate({ ...machine, baseProcessingSpeed: parseFloat(e.target.value) || 1.0 })
              }
            />
          </div>
          <div className="form-group">
            <label>Vibration Level</label>
            <input
              type="number"
              min="0"
              max="10"
              value={machine.vibrationLevel}
              onChange={(e) =>
                onUpdate({ ...machine, vibrationLevel: parseInt(e.target.value) || 0 })
              }
            />
          </div>
          <div className="form-group">
            <label>Heat Output</label>
            <input
              type="number"
              min="0"
              value={machine.heatOutput}
              onChange={(e) =>
                onUpdate({ ...machine, heatOutput: parseInt(e.target.value) || 0 })
              }
            />
          </div>
        </div>

        {selectedPort && (
          <div className="inspector-section">
            <h4>IO Port</h4>
            <div className="form-group">
              <label>Type</label>
              <select
                value={selectedPort.portType}
                onChange={(e) =>
                  onPortUpdate({ ...selectedPort, portType: e.target.value as IOPort["portType"] })
                }
              >
                <option value="item_input">Item Input</option>
                <option value="item_output">Item Output</option>
                <option value="fluid_input">Fluid Input</option>
                <option value="fluid_output">Fluid Output</option>
                <option value="power">Power</option>
                <option value="torque">Torque</option>
              </select>
            </div>
            <div className="form-group">
              <label>Direction</label>
              <select
                value={selectedPort.direction}
                onChange={(e) =>
                  onPortUpdate({ ...selectedPort, direction: e.target.value as Direction })
                }
              >
                <option value="north">North</option>
                <option value="south">South</option>
                <option value="east">East</option>
                <option value="west">West</option>
                <option value="up">Up</option>
                <option value="down">Down</option>
              </select>
            </div>
            <button onClick={onPortRemove} className="remove-btn">
              Remove Port
            </button>
          </div>
        )}

        <div className="inspector-section">
          <h4>Statistics</h4>
          <div className="stat-row">
            <span>Blocks:</span>
            <span>{machine.blocks.length}</span>
          </div>
          <div className="stat-row">
            <span>IO Ports:</span>
            <span>{machine.ioPorts.length}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// Main 3D Scene
interface EditorSceneProps {
  machine: MultiblockMachineData;
  selectedBlock: string;
  mode: "place" | "delete" | "port";
  onPlaceBlock: (x: number, y: number, z: number) => void;
  onDeleteBlock: (x: number, y: number, z: number) => void;
  onAddPort: (x: number, y: number, z: number, face: Direction) => void;
  selectedPortIndex: number | null;
  onSelectPort: (index: number | null) => void;
}

function EditorScene({
  machine,
  selectedBlock: _selectedBlock,
  mode,
  onPlaceBlock,
  onDeleteBlock,
  onAddPort,
  selectedPortIndex,
  onSelectPort,
}: EditorSceneProps) {
  const [ghostPosition, setGhostPosition] = useState<[number, number, number] | null>(null);

  const handleBackgroundClick = useCallback(
    (_e: ThreeEvent<MouseEvent>) => {
      if (mode === "place" && ghostPosition) {
        onPlaceBlock(ghostPosition[0], ghostPosition[1], ghostPosition[2]);
      }
    },
    [mode, ghostPosition, onPlaceBlock]
  );

  const handlePointerMove = useCallback(
    (e: ThreeEvent<PointerEvent>) => {
      if (mode !== "place") {
        setGhostPosition(null);
        return;
      }

      const point = e.point;
      const x = Math.floor(point.x + 0.5);
      const y = Math.max(0, Math.floor(point.y + 0.5));
      const z = Math.floor(point.z + 0.5);

      if (x >= 0 && x < machine.size[0] && z >= 0 && z < machine.size[2] && y < machine.size[1]) {
        setGhostPosition([x, y, z]);
      } else {
        setGhostPosition(null);
      }
    },
    [mode, machine.size]
  );

  return (
    <>
      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 10, 10]} intensity={1} />
      <CameraController size={Math.max(...machine.size)} />
      <OrbitControls makeDefault />

      {/* Grid Floor */}
      <GridFloor size={machine.size[0]} />

      {/* Axis Labels */}
      <Text position={[machine.size[0] + 0.5, 0, 0]} fontSize={0.5} color="#ff0000">X</Text>
      <Text position={[0, machine.size[1] + 0.5, 0]} fontSize={0.5} color="#00ff00">Y</Text>
      <Text position={[0, 0, machine.size[2] + 0.5]} fontSize={0.5} color="#0000ff">Z</Text>

      {/* Placed Blocks */}
      {machine.blocks.map((block) => (
        <Block
          key={`${block.x}-${block.y}-${block.z}`}
          position={[block.x, block.y, block.z]}
          blockId={block.blockId}
          selected={false}
          onClick={() => {
            if (mode === "delete") {
              onDeleteBlock(block.x, block.y, block.z);
            }
          }}
          onFaceClick={mode === "port" ? (face) => onAddPort(block.x, block.y, block.z, face) : undefined}
        />
      ))}

      {/* IO Ports */}
      {machine.ioPorts.map((port, index) => (
        <PortIndicator
          key={index}
          port={port}
          onClick={() => onSelectPort(selectedPortIndex === index ? null : index)}
        />
      ))}

      {/* Ghost Block */}
      {ghostPosition && (
        <GhostBlock position={ghostPosition} visible={mode === "place"} />
      )}

      {/* Click area for placing blocks */}
      <mesh
        position={[machine.size[0] / 2 - 0.5, 0, machine.size[2] / 2 - 0.5]}
        rotation={[-Math.PI / 2, 0, 0]}
        onClick={handleBackgroundClick}
        onPointerMove={handlePointerMove}
        visible={false}
      >
        <planeGeometry args={[machine.size[0], machine.size[2]]} />
        <meshBasicMaterial transparent opacity={0} />
      </mesh>
    </>
  );
}

// Main Multiblock Editor Component
export default function MultiblockEditor() {
  const [machine, setMachine] = useState<MultiblockMachineData>(
    createDefaultMultiblockMachine("new_multiblock")
  );
  const [selectedBlock, setSelectedBlock] = useState("machine_casing");
  const [searchTerm, setSearchTerm] = useState("");
  const [mode, setMode] = useState<"place" | "delete" | "port">("place");
  const [selectedPortIndex, setSelectedPortIndex] = useState<number | null>(null);
  const [previewRotation, setPreviewRotation] = useState(0);
  const [showRotationPreview, setShowRotationPreview] = useState(false);
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

  const handlePlaceBlock = useCallback(
    (x: number, y: number, z: number) => {
      // Check if block already exists
      const exists = machine.blocks.some(
        (b) => b.x === x && b.y === y && b.z === z
      );
      if (!exists) {
        setMachine({
          ...machine,
          blocks: [...machine.blocks, { x, y, z, blockId: selectedBlock }],
        });
      }
    },
    [machine, selectedBlock]
  );

  const handleDeleteBlock = useCallback(
    (x: number, y: number, z: number) => {
      setMachine({
        ...machine,
        blocks: machine.blocks.filter(
          (b) => !(b.x === x && b.y === y && b.z === z)
        ),
        // Also remove any ports on this block
        ioPorts: machine.ioPorts.filter(
          (p) => !(p.position[0] === x && p.position[1] === y && p.position[2] === z)
        ),
      });
    },
    [machine]
  );

  const handleAddPort = useCallback(
    (x: number, y: number, z: number, face: Direction) => {
      const newPort = createDefaultIOPort();
      newPort.position = [x, y, z];
      newPort.direction = face;
      setMachine({
        ...machine,
        ioPorts: [...machine.ioPorts, newPort],
      });
      setSelectedPortIndex(machine.ioPorts.length);
    },
    [machine]
  );

  const handleUpdatePort = useCallback(
    (port: IOPort) => {
      if (selectedPortIndex !== null) {
        const newPorts = [...machine.ioPorts];
        newPorts[selectedPortIndex] = port;
        setMachine({ ...machine, ioPorts: newPorts });
      }
    },
    [machine, selectedPortIndex]
  );

  const handleRemovePort = useCallback(() => {
    if (selectedPortIndex !== null) {
      setMachine({
        ...machine,
        ioPorts: machine.ioPorts.filter((_, i) => i !== selectedPortIndex),
      });
      setSelectedPortIndex(null);
    }
  }, [machine, selectedPortIndex]);

  const handleSave = useCallback(async () => {
    try {
      await invoke("save_multiblock", { machine });
      alert("Multiblock saved successfully!");
    } catch (err) {
      console.error(err);
      alert("Save function not implemented yet");
    }
  }, [machine]);

  const handleClear = useCallback(() => {
    setMachine(createDefaultMultiblockMachine("new_multiblock"));
    setSelectedPortIndex(null);
  }, []);

  return (
    <div className="multiblock-editor">
      <BlockPalette
        selectedBlock={selectedBlock}
        onSelectBlock={setSelectedBlock}
        searchTerm={searchTerm}
        onSearchChange={setSearchTerm}
        catalog={catalog}
      />

      <div className="editor-main">
        <div className="editor-toolbar">
          <div className="tool-group">
            <button
              className={mode === "place" ? "active" : ""}
              onClick={() => setMode("place")}
              title="Place Block"
            >
              +
            </button>
            <button
              className={mode === "delete" ? "active" : ""}
              onClick={() => setMode("delete")}
              title="Delete Block"
            >
              -
            </button>
            <button
              className={mode === "port" ? "active" : ""}
              onClick={() => setMode("port")}
              title="Add IO Port"
            >
              IO
            </button>
          </div>
          <div className="tool-group">
            <button onClick={() => setShowRotationPreview(!showRotationPreview)}>
              {showRotationPreview ? "Hide Rotation" : "Show Rotation"}
            </button>
            {showRotationPreview && (
              <select
                value={previewRotation}
                onChange={(e) => setPreviewRotation(parseInt(e.target.value))}
              >
                <option value={0}>0</option>
                <option value={90}>90</option>
                <option value={180}>180</option>
                <option value={270}>270</option>
              </select>
            )}
          </div>
          <div className="tool-group">
            <button onClick={handleSave}>Save</button>
            <button onClick={handleClear}>Clear</button>
          </div>
        </div>

        <div className="canvas-container">
          <Canvas camera={{ fov: 50 }}>
            <EditorScene
              machine={machine}
              selectedBlock={selectedBlock}
              mode={mode}
              onPlaceBlock={handlePlaceBlock}
              onDeleteBlock={handleDeleteBlock}
              onAddPort={handleAddPort}
              selectedPortIndex={selectedPortIndex}
              onSelectPort={setSelectedPortIndex}
            />
            {showRotationPreview && previewRotation !== 0 && (
              <group position={[machine.size[0] + 2, 0, 0]}>
                <RotationPreview
                  blocks={machine.blocks}
                  size={machine.size}
                  rotation={previewRotation}
                />
              </group>
            )}
          </Canvas>
        </div>
      </div>

      <MultiblockInspector
        machine={machine}
        selectedPort={selectedPortIndex !== null ? machine.ioPorts[selectedPortIndex] : null}
        onUpdate={setMachine}
        onPortUpdate={handleUpdatePort}
        onPortRemove={handleRemovePort}
      />
    </div>
  );
}
