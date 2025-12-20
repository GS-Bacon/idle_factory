import { memo } from "react";
import { Handle, Position } from "@xyflow/react";
import type { MachineNodeData } from "../../types/recipe";
import "./nodes.css";

interface MachineNodeProps {
  data: MachineNodeData;
  selected?: boolean;
}

function MachineNode({ data, selected }: MachineNodeProps) {
  // Generate input handles
  const inputHandles = Array.from({ length: data.inputSlots }, (_, i) => (
    <Handle
      key={`input-${i}`}
      type="target"
      position={Position.Left}
      id={`input-${i}`}
      className="handle handle-target"
      style={{ top: `${((i + 1) / (data.inputSlots + 1)) * 100}%` }}
    />
  ));

  // Generate output handles
  const outputHandles = Array.from({ length: data.outputSlots }, (_, i) => (
    <Handle
      key={`output-${i}`}
      type="source"
      position={Position.Right}
      id={`output-${i}`}
      className="handle handle-source"
      style={{ top: `${((i + 1) / (data.outputSlots + 1)) * 100}%` }}
    />
  ));

  const machineTypeStr = typeof data.machineType === "string"
    ? data.machineType
    : data.machineType.Custom;

  return (
    <div className={`custom-node machine-node ${selected ? "selected" : ""}`}>
      <div className="node-header machine-header">
        <span className="node-icon">⚙️</span>
        <span className="node-type">{machineTypeStr}</span>
      </div>
      <div className="node-content">
        <div className="node-label">{data.label}</div>
        <div className="node-slots">
          <span className="slot-info">In: {data.inputSlots}</span>
          <span className="slot-info">Out: {data.outputSlots}</span>
        </div>
      </div>
      {inputHandles}
      {outputHandles}
    </div>
  );
}

export default memo(MachineNode);
