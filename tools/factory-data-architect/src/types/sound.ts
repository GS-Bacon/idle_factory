// Sound Types for Editor

export interface SoundData {
  id: string;
  filePath: string;
  // Playback settings
  volume: number; // 0.0 - 1.0
  pitch: number; // 0.5 - 2.0, 1.0 = normal
  loop: boolean;
  // 3D sound settings
  is3D: boolean;
  minDistance: number;
  maxDistance: number;
  // Categories
  category: "machine" | "environment" | "ui" | "player";
}

export interface MachineSoundSet {
  machineId: string;
  idleSound: string | null;
  runningSound: string | null;
  startSound: string | null;
  stopSound: string | null;
}

export function createDefaultSound(id: string): SoundData {
  return {
    id,
    filePath: "",
    volume: 1.0,
    pitch: 1.0,
    loop: false,
    is3D: true,
    minDistance: 1,
    maxDistance: 50,
    category: "machine",
  };
}

export function createDefaultMachineSoundSet(machineId: string): MachineSoundSet {
  return {
    machineId,
    idleSound: null,
    runningSound: null,
    startSound: null,
    stopSound: null,
  };
}
