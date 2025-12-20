// TypeScript type tests for Factory Data Architect
// These tests verify that types are correctly defined and exportable

import {
  AnimationType,
  AssetConfig,
  LocalizationEntry,
  LocalizationData,
  ItemCategory,
  ItemData,
  MachineItemData,
  createDefaultAssetConfig,
  createDefaultLocalizationData,
  createDefaultItemData,
  createDefaultMachineData,
} from "./index";

// Type assertion tests
describe("AnimationType", () => {
  test("None type is valid", () => {
    const animation: AnimationType = { type: "None" };
    expect(animation.type).toBe("None");
  });

  test("Rotational type is valid", () => {
    const animation: AnimationType = {
      type: "Rotational",
      params: { axis: [0, 1, 0], speed: 90 },
    };
    expect(animation.type).toBe("Rotational");
    expect(animation.params.axis).toEqual([0, 1, 0]);
  });

  test("Linear type is valid", () => {
    const animation: AnimationType = {
      type: "Linear",
      params: { direction: [1, 0, 0], distance: 2, speed: 1 },
    };
    expect(animation.type).toBe("Linear");
  });

  test("Skeletal type is valid", () => {
    const animation: AnimationType = {
      type: "Skeletal",
      params: { animation_path: "anims/idle.glb", looping: true },
    };
    expect(animation.type).toBe("Skeletal");
  });
});

describe("createDefaultAssetConfig", () => {
  test("returns valid AssetConfig with null paths", () => {
    const config = createDefaultAssetConfig();
    expect(config.icon_path).toBeNull();
    expect(config.model_path).toBeNull();
    expect(config.animation.type).toBe("None");
  });
});

describe("createDefaultLocalizationData", () => {
  test("returns valid LocalizationData with empty strings", () => {
    const data = createDefaultLocalizationData();
    expect(data.ja.name).toBe("");
    expect(data.ja.description).toBe("");
    expect(data.en.name).toBe("");
    expect(data.en.description).toBe("");
  });
});

describe("createDefaultItemData", () => {
  test("creates item with correct id", () => {
    const item = createDefaultItemData("iron_ore");
    expect(item.id).toBe("iron_ore");
  });

  test("creates item with correct i18n_key", () => {
    const item = createDefaultItemData("copper_ingot");
    expect(item.i18n_key).toBe("item.copper_ingot");
  });

  test("creates item with default category 'item'", () => {
    const item = createDefaultItemData("test");
    expect(item.category).toBe("item");
  });

  test("creates item with default asset config", () => {
    const item = createDefaultItemData("test");
    expect(item.asset.icon_path).toBeNull();
    expect(item.asset.model_path).toBeNull();
    expect(item.asset.animation.type).toBe("None");
  });

  test("creates item with empty properties", () => {
    const item = createDefaultItemData("test");
    expect(item.properties).toEqual({});
  });
});

describe("createDefaultMachineData", () => {
  test("creates machine with correct id", () => {
    const machine = createDefaultMachineData("assembler");
    expect(machine.id).toBe("assembler");
  });

  test("creates machine with correct i18n_key", () => {
    const machine = createDefaultMachineData("mixer");
    expect(machine.i18n_key).toBe("machine.mixer");
  });

  test("creates machine with category 'machine'", () => {
    const machine = createDefaultMachineData("test");
    expect(machine.category).toBe("machine");
  });

  test("creates machine with default slots", () => {
    const machine = createDefaultMachineData("test");
    expect(machine.inputSlots).toBe(1);
    expect(machine.outputSlots).toBe(1);
    expect(machine.fluidInputSlots).toBe(0);
    expect(machine.fluidOutputSlots).toBe(0);
  });

  test("creates machine with default processing values", () => {
    const machine = createDefaultMachineData("test");
    expect(machine.baseTorqueConsumption).toBe(8);
    expect(machine.baseProcessingSpeed).toBe(1.0);
    expect(machine.vibrationLevel).toBe(1);
  });
});

describe("ItemCategory", () => {
  test("item category is valid", () => {
    const category: ItemCategory = "item";
    expect(category).toBe("item");
  });

  test("machine category is valid", () => {
    const category: ItemCategory = "machine";
    expect(category).toBe("machine");
  });

  test("multiblock category is valid", () => {
    const category: ItemCategory = "multiblock";
    expect(category).toBe("multiblock");
  });
});

describe("ItemData type compatibility", () => {
  test("ItemData can have all categories", () => {
    const itemItem: ItemData = {
      id: "test1",
      i18n_key: "item.test1",
      asset: createDefaultAssetConfig(),
      properties: {},
      category: "item",
    };

    const machineItem: ItemData = {
      id: "test2",
      i18n_key: "machine.test2",
      asset: createDefaultAssetConfig(),
      properties: {},
      category: "machine",
    };

    const multiblockItem: ItemData = {
      id: "test3",
      i18n_key: "multiblock.test3",
      asset: createDefaultAssetConfig(),
      properties: {},
      category: "multiblock",
    };

    expect(itemItem.category).toBe("item");
    expect(machineItem.category).toBe("machine");
    expect(multiblockItem.category).toBe("multiblock");
  });

  test("ItemData can have custom properties", () => {
    const item: ItemData = {
      id: "special",
      i18n_key: "item.special",
      asset: createDefaultAssetConfig(),
      properties: {
        durability: 100,
        stackable: true,
        rarity: "rare",
      },
      category: "item",
    };

    expect(item.properties.durability).toBe(100);
    expect(item.properties.stackable).toBe(true);
    expect(item.properties.rarity).toBe("rare");
  });
});

describe("LocalizationData", () => {
  test("can create complete localization", () => {
    const loc: LocalizationData = {
      ja: { name: "鉄鉱石", description: "生の鉄鉱石" },
      en: { name: "Iron Ore", description: "Raw iron ore" },
    };

    expect(loc.ja.name).toBe("鉄鉱石");
    expect(loc.en.name).toBe("Iron Ore");
  });
});

describe("LocalizationEntry", () => {
  test("can create localization entry", () => {
    const entry: LocalizationEntry = { name: "Test", description: "Test description" };
    expect(entry.name).toBe("Test");
    expect(entry.description).toBe("Test description");
  });
});

describe("MachineItemData", () => {
  test("extends ItemData with machine-specific fields", () => {
    const machine: MachineItemData = {
      id: "assembler",
      i18n_key: "machine.assembler",
      asset: createDefaultAssetConfig(),
      properties: {},
      category: "machine",
      workTypes: ["assembling", "crafting"],
      inputSlots: 3,
      outputSlots: 2,
      fluidInputSlots: 1,
      fluidOutputSlots: 0,
      baseTorqueConsumption: 16,
      baseProcessingSpeed: 1.5,
      vibrationLevel: 2,
    };

    expect(machine.category).toBe("machine");
    expect(machine.workTypes).toContain("assembling");
    expect(machine.inputSlots).toBe(3);
  });
});

describe("AssetConfig", () => {
  test("can have all animation types", () => {
    const configNone: AssetConfig = {
      icon_path: null,
      model_path: null,
      animation: { type: "None" },
    };

    const configRotational: AssetConfig = {
      icon_path: "icons/gear.png",
      model_path: "models/gear.glb",
      animation: { type: "Rotational", params: { axis: [0, 1, 0], speed: 90 } },
    };

    expect(configNone.animation.type).toBe("None");
    expect(configRotational.animation.type).toBe("Rotational");
  });
});

// i18n_key generation function test
describe("i18n_key generation pattern", () => {
  test("item category generates item.{id}", () => {
    const item = createDefaultItemData("iron_ore");
    expect(item.i18n_key).toMatch(/^item\./);
  });

  test("machine category generates machine.{id}", () => {
    const machine = createDefaultMachineData("assembler");
    expect(machine.i18n_key).toMatch(/^machine\./);
  });
});
