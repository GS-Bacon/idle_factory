import { useState, useCallback, useRef, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { SoundData, MachineSoundSet } from "../types/sound";
import { createDefaultSound, createDefaultMachineSoundSet } from "../types/sound";
import type { AssetCatalog } from "../types/recipe";
import "./SoundEditor.css";

// Audio Player Component
interface AudioPlayerProps {
  filePath: string | null;
  volume: number;
  pitch: number;
  loop: boolean;
  assetsPath: string | null;
}

function AudioPlayer({ filePath, volume, pitch, loop, assetsPath }: AudioPlayerProps) {
  const audioRef = useRef<HTMLAudioElement>(null);
  const [isPlaying, setIsPlaying] = useState(false);
  const [duration, setDuration] = useState(0);
  const [currentTime, setCurrentTime] = useState(0);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    audio.volume = volume;
    audio.playbackRate = pitch;
    audio.loop = loop;
  }, [volume, pitch, loop]);

  useEffect(() => {
    const audio = audioRef.current;
    if (!audio) return;

    const handleTimeUpdate = () => setCurrentTime(audio.currentTime);
    const handleLoadedMetadata = () => setDuration(audio.duration);
    const handleEnded = () => setIsPlaying(false);

    audio.addEventListener("timeupdate", handleTimeUpdate);
    audio.addEventListener("loadedmetadata", handleLoadedMetadata);
    audio.addEventListener("ended", handleEnded);

    return () => {
      audio.removeEventListener("timeupdate", handleTimeUpdate);
      audio.removeEventListener("loadedmetadata", handleLoadedMetadata);
      audio.removeEventListener("ended", handleEnded);
    };
  }, []);

  const togglePlay = useCallback(() => {
    const audio = audioRef.current;
    if (!audio) return;

    if (isPlaying) {
      audio.pause();
    } else {
      audio.play().catch(console.error);
    }
    setIsPlaying(!isPlaying);
  }, [isPlaying]);

  const stop = useCallback(() => {
    const audio = audioRef.current;
    if (!audio) return;

    audio.pause();
    audio.currentTime = 0;
    setIsPlaying(false);
  }, []);

  const formatTime = (time: number) => {
    const minutes = Math.floor(time / 60);
    const seconds = Math.floor(time % 60);
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
  };

  if (!filePath) {
    return (
      <div className="audio-player empty">
        <span>No audio file selected</span>
      </div>
    );
  }

  // Build full path
  const fullPath = assetsPath ? `${assetsPath}/${filePath}` : filePath;

  return (
    <div className="audio-player">
      <audio ref={audioRef} src={fullPath} />
      <div className="player-controls">
        <button onClick={togglePlay} className="play-btn">
          {isPlaying ? "⏸" : "▶"}
        </button>
        <button onClick={stop} className="stop-btn">
          ⏹
        </button>
        <div className="time-display">
          <span>{formatTime(currentTime)}</span>
          <span>/</span>
          <span>{formatTime(duration)}</span>
        </div>
        <div className="progress-bar">
          <div
            className="progress-fill"
            style={{ width: `${duration > 0 ? (currentTime / duration) * 100 : 0}%` }}
          />
        </div>
      </div>
      <div className="file-path">{filePath}</div>
    </div>
  );
}

// Single Sound Editor
interface SoundItemEditorProps {
  sound: SoundData;
  assetsPath: string | null;
  onChange: (sound: SoundData) => void;
  onDelete: () => void;
}

function SoundItemEditor({ sound, assetsPath, onChange, onDelete }: SoundItemEditorProps) {
  const handleSelectFile = async () => {
    const selected = await open({
      multiple: false,
      filters: [{ name: "Audio", extensions: ["wav", "ogg", "mp3", "flac"] }],
      defaultPath: assetsPath ?? undefined,
    });

    if (selected && typeof selected === "string") {
      let relativePath = selected;
      if (assetsPath) {
        try {
          relativePath = await invoke<string>("to_relative_path", {
            absolutePath: selected,
          });
        } catch {
          // Keep absolute path
        }
      }
      onChange({ ...sound, filePath: relativePath });
    }
  };

  return (
    <div className="sound-item-editor">
      <div className="sound-header">
        <input
          type="text"
          value={sound.id}
          onChange={(e) => onChange({ ...sound, id: e.target.value })}
          placeholder="Sound ID"
        />
        <button onClick={onDelete} className="delete-btn">Delete</button>
      </div>

      <div className="sound-file">
        <button onClick={handleSelectFile}>Select File</button>
        <span className="file-name">{sound.filePath || "No file selected"}</span>
      </div>

      <AudioPlayer
        filePath={sound.filePath}
        volume={sound.volume}
        pitch={sound.pitch}
        loop={sound.loop}
        assetsPath={assetsPath}
      />

      <div className="sound-params">
        <div className="param-row">
          <label>
            Volume
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={sound.volume}
              onChange={(e) => onChange({ ...sound, volume: parseFloat(e.target.value) })}
            />
            <span>{Math.round(sound.volume * 100)}%</span>
          </label>
        </div>
        <div className="param-row">
          <label>
            Pitch
            <input
              type="range"
              min="0.5"
              max="2"
              step="0.05"
              value={sound.pitch}
              onChange={(e) => onChange({ ...sound, pitch: parseFloat(e.target.value) })}
            />
            <span>{sound.pitch.toFixed(2)}x</span>
          </label>
        </div>
        <div className="param-row checkbox">
          <label>
            <input
              type="checkbox"
              checked={sound.loop}
              onChange={(e) => onChange({ ...sound, loop: e.target.checked })}
            />
            Loop
          </label>
        </div>
        <div className="param-row checkbox">
          <label>
            <input
              type="checkbox"
              checked={sound.is3D}
              onChange={(e) => onChange({ ...sound, is3D: e.target.checked })}
            />
            3D Sound
          </label>
        </div>
        {sound.is3D && (
          <>
            <div className="param-row">
              <label>
                Min Distance
                <input
                  type="number"
                  min="0"
                  value={sound.minDistance}
                  onChange={(e) => onChange({ ...sound, minDistance: parseFloat(e.target.value) || 1 })}
                />
              </label>
            </div>
            <div className="param-row">
              <label>
                Max Distance
                <input
                  type="number"
                  min="0"
                  value={sound.maxDistance}
                  onChange={(e) => onChange({ ...sound, maxDistance: parseFloat(e.target.value) || 50 })}
                />
              </label>
            </div>
          </>
        )}
        <div className="param-row">
          <label>
            Category
            <select
              value={sound.category}
              onChange={(e) => onChange({ ...sound, category: e.target.value as SoundData["category"] })}
            >
              <option value="machine">Machine</option>
              <option value="environment">Environment</option>
              <option value="ui">UI</option>
              <option value="player">Player</option>
            </select>
          </label>
        </div>
      </div>
    </div>
  );
}

// Machine Sound Set Editor
interface MachineSoundSetEditorProps {
  soundSet: MachineSoundSet;
  sounds: SoundData[];
  onChange: (soundSet: MachineSoundSet) => void;
  onDelete: () => void;
}

function MachineSoundSetEditor({
  soundSet,
  sounds,
  onChange,
  onDelete,
}: MachineSoundSetEditorProps) {
  return (
    <div className="machine-sound-set">
      <div className="sound-set-header">
        <input
          type="text"
          value={soundSet.machineId}
          onChange={(e) => onChange({ ...soundSet, machineId: e.target.value })}
          placeholder="Machine ID"
        />
        <button onClick={onDelete} className="delete-btn">Delete</button>
      </div>
      <div className="sound-set-slots">
        {(["idleSound", "runningSound", "startSound", "stopSound"] as const).map((slot) => (
          <div key={slot} className="sound-slot">
            <label>{slot.replace("Sound", "")}</label>
            <select
              value={soundSet[slot] || ""}
              onChange={(e) => onChange({ ...soundSet, [slot]: e.target.value || null })}
            >
              <option value="">-- None --</option>
              {sounds.map((s) => (
                <option key={s.id} value={s.id}>{s.id}</option>
              ))}
            </select>
          </div>
        ))}
      </div>
    </div>
  );
}

// Main Sound Editor
interface SoundEditorProps {
  assetsPath: string | null;
}

export default function SoundEditor({ assetsPath }: SoundEditorProps) {
  const [sounds, setSounds] = useState<SoundData[]>([]);
  const [machineSoundSets, setMachineSoundSets] = useState<MachineSoundSet[]>([]);
  const [activeTab, setActiveTab] = useState<"sounds" | "machines">("sounds");
  const [_catalog, setCatalog] = useState<AssetCatalog>({
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

  const addSound = useCallback(() => {
    const id = `sound_${Date.now()}`;
    setSounds([...sounds, createDefaultSound(id)]);
  }, [sounds]);

  const updateSound = useCallback((index: number, sound: SoundData) => {
    const newSounds = [...sounds];
    newSounds[index] = sound;
    setSounds(newSounds);
  }, [sounds]);

  const deleteSound = useCallback((index: number) => {
    setSounds(sounds.filter((_, i) => i !== index));
  }, [sounds]);

  const addMachineSoundSet = useCallback(() => {
    const id = `machine_${Date.now()}`;
    setMachineSoundSets([...machineSoundSets, createDefaultMachineSoundSet(id)]);
  }, [machineSoundSets]);

  const updateMachineSoundSet = useCallback((index: number, soundSet: MachineSoundSet) => {
    const newSets = [...machineSoundSets];
    newSets[index] = soundSet;
    setMachineSoundSets(newSets);
  }, [machineSoundSets]);

  const deleteMachineSoundSet = useCallback((index: number) => {
    setMachineSoundSets(machineSoundSets.filter((_, i) => i !== index));
  }, [machineSoundSets]);

  const handleSave = useCallback(async () => {
    try {
      await invoke("save_sounds", { sounds, machineSoundSets });
      alert("Sounds saved successfully!");
    } catch (err) {
      console.error(err);
      alert("Save function not implemented yet");
    }
  }, [sounds, machineSoundSets]);

  return (
    <div className="sound-editor">
      <div className="editor-header">
        <div className="tabs">
          <button
            className={activeTab === "sounds" ? "active" : ""}
            onClick={() => setActiveTab("sounds")}
          >
            Sound Files
          </button>
          <button
            className={activeTab === "machines" ? "active" : ""}
            onClick={() => setActiveTab("machines")}
          >
            Machine Sounds
          </button>
        </div>
        <button onClick={handleSave} className="save-btn">Save All</button>
      </div>

      <div className="editor-content">
        {activeTab === "sounds" && (
          <div className="sounds-list">
            <div className="list-header">
              <h3>Sound Files ({sounds.length})</h3>
              <button onClick={addSound} className="add-btn">+ Add Sound</button>
            </div>
            <div className="list-content">
              {sounds.map((sound, index) => (
                <SoundItemEditor
                  key={sound.id}
                  sound={sound}
                  assetsPath={assetsPath}
                  onChange={(s) => updateSound(index, s)}
                  onDelete={() => deleteSound(index)}
                />
              ))}
              {sounds.length === 0 && (
                <div className="empty-state">
                  No sounds defined. Click "+ Add Sound" to create one.
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === "machines" && (
          <div className="machine-sounds-list">
            <div className="list-header">
              <h3>Machine Sound Sets ({machineSoundSets.length})</h3>
              <button onClick={addMachineSoundSet} className="add-btn">+ Add Machine</button>
            </div>
            <div className="list-content">
              {machineSoundSets.map((soundSet, index) => (
                <MachineSoundSetEditor
                  key={soundSet.machineId}
                  soundSet={soundSet}
                  sounds={sounds}
                  onChange={(s) => updateMachineSoundSet(index, s)}
                  onDelete={() => deleteMachineSoundSet(index)}
                />
              ))}
              {machineSoundSets.length === 0 && (
                <div className="empty-state">
                  No machine sound sets defined. Click "+ Add Machine" to create one.
                </div>
              )}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
