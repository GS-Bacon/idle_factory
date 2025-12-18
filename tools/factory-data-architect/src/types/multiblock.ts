// Multiblock Machine Types for 3D Grid Editor

export type Direction = "north" | "south" | "east" | "west" | "up" | "down";

export interface BlockPlacement {
  x: number;
  y: number;
  z: number;
  blockId: string;
}

export interface IOPort {
  position: [number, number, number];
  direction: Direction;
  portType: "item_input" | "item_output" | "fluid_input" | "fluid_output" | "power" | "torque";
}

export interface MultiblockMachineData {
  id: string;
  i18nKey: string;
  size: [number, number, number]; // width, height, depth
  blocks: BlockPlacement[];
  ioPorts: IOPort[];
  masterOffset: [number, number, number]; // Position of the master block
  // Machine parameters
  baseTorqueConsumption: number;
  baseProcessingSpeed: number;
  vibrationLevel: number;
  heatOutput: number;
}

export function createDefaultMultiblockMachine(id: string): MultiblockMachineData {
  return {
    id,
    i18nKey: `machine.${id}`,
    size: [3, 3, 3],
    blocks: [],
    ioPorts: [],
    masterOffset: [1, 0, 1],
    baseTorqueConsumption: 16,
    baseProcessingSpeed: 1.0,
    vibrationLevel: 1,
    heatOutput: 0,
  };
}

export function createDefaultIOPort(): IOPort {
  return {
    position: [0, 0, 0],
    direction: "north",
    portType: "item_input",
  };
}

// Helper to get face position from click
export function getFaceFromNormal(normal: [number, number, number]): Direction {
  const [x, y, z] = normal;
  if (y > 0.5) return "up";
  if (y < -0.5) return "down";
  if (x > 0.5) return "east";
  if (x < -0.5) return "west";
  if (z > 0.5) return "south";
  return "north";
}
