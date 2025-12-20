import { memo } from "react";
import { Handle, Position } from "@xyflow/react";
import type { ResultNodeData } from "../../types/recipe";
import "./nodes.css";

interface ResultNodeProps {
  data: ResultNodeData;
  selected?: boolean;
}

function ResultNode({ data, selected }: ResultNodeProps) {
  const typeIcon = data.itemType === "item" ? "📦" : "💧";
  const typeColor = data.itemType === "item" ? "#f472b6" : "#818cf8";

  const chancePercent = Math.round(data.chance * 100);

  return (
    <div className={`custom-node result-node ${selected ? "selected" : ""}`} style={{ borderColor: typeColor }}>
      <Handle
        type="target"
        position={Position.Left}
        id="input"
        className="handle handle-target"
      />
      <div className="node-header" style={{ backgroundColor: typeColor }}>
        <span className="node-icon">{typeIcon}</span>
        <span className="node-type">Result</span>
      </div>
      <div className="node-content">
        <div className="node-label">{data.label}</div>
        <div className="node-details">
          <span className="node-id">{data.itemId}</span>
          <span className="node-amount">x{data.amount}</span>
        </div>
        {data.chance < 1.0 && (
          <div className="node-chance">
            <span className="chance-label">Chance:</span>
            <span className="chance-value">{chancePercent}%</span>
          </div>
        )}
      </div>
    </div>
  );
}

export default memo(ResultNode);
