// Quest Types for Editor

export type RewardType =
  | { type: "PortUnlock"; count: number }
  | { type: "Item"; itemId: string; amount: number };

export interface QuestRequirement {
  itemId: string;
  amount: number;
  itemType: "item" | "fluid" | "power" | "torque";
}

export interface QuestData {
  id: string;
  i18nKey: string;
  questType: "main" | "sub";
  phase: number;
  requirements: QuestRequirement[];
  rewards: RewardType[];
  prerequisites: string[]; // Quest IDs that must be completed first
}

export interface QuestNodeData extends Record<string, unknown> {
  quest: QuestData;
  label: string;
}

export function createDefaultQuest(id: string, questType: "main" | "sub" = "main"): QuestData {
  return {
    id,
    i18nKey: `quest.${id}`,
    questType,
    phase: 1,
    requirements: [],
    rewards: [],
    prerequisites: [],
  };
}

export function createDefaultRequirement(): QuestRequirement {
  return {
    itemId: "",
    amount: 1,
    itemType: "item",
  };
}

export function createDefaultItemReward(): RewardType {
  return { type: "Item", itemId: "", amount: 1 };
}

export function createDefaultPortReward(): RewardType {
  return { type: "PortUnlock", count: 1 };
}
