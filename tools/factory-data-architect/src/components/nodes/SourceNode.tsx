import { memo } from "react";
import { Handle, Position } from "@xyflow/react";
import type { SourceNodeData } from "../../types/recipe";
import "./nodes.css";

interface SourceNodeProps {
  data: SourceNodeData;
  selected?: boolean;
}

function SourceNode({ data, selected }: SourceNodeProps) {
  const typeIcon = data.itemType === "item" ? "📦" : data.itemType === "fluid" ? "💧" : "🏷️";
  const typeColor = data.itemType === "item" ? "#4ade80" : data.itemType === "fluid" ? "#60a5fa" : "#fbbf24";

  return (
    <div className={`custom-node source-node ${selected ? "selected" : ""}`} style={{ borderColor: typeColor }}>
      <div className="node-header" style={{ backgroundColor: typeColor }}>
        <span className="node-icon">{typeIcon}</span>
        <span className="node-type">Source</span>
      </div>
      <div className="node-content">
        <div className="node-label">{data.label}</div>
        <div className="node-details">
          <span className="node-id">{data.itemId}</span>
          <span className="node-amount">x{data.amount}</span>
        </div>
      </div>
      <Handle
        type="source"
        position={Position.Right}
        id="output"
        className="handle handle-source"
      />
    </div>
  );
}

export default memo(SourceNode);
