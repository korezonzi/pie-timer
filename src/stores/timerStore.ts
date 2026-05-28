import { create } from "zustand";
import type { TimerState, Preset } from "../types/timer";
import { DEFAULT_PRESETS } from "../types/timer";
import * as bridge from "../lib/tauri-bridge";

interface TimerStore {
  state: TimerState;
  activePreset: Preset;
  showSettings: boolean;
  windowWidth: number;

  // Actions
  syncState: () => Promise<void>;
  updateState: (state: TimerState) => void;
  toggle: () => Promise<void>;
  reset: () => Promise<void>;
  skip: () => Promise<void>;
  changePreset: (preset: Preset) => Promise<void>;
  setSessionsGoal: (goal: number) => Promise<void>;
  toggleSettings: () => void;
  setWindowWidth: (width: number) => void;
}

export const useTimerStore = create<TimerStore>((set) => ({
  state: {
    phase: "idle",
    status: "stopped",
    totalDurationSec: 25 * 60,
    remainingSec: 25 * 60,
    completedSessions: 0,
    currentSessionIndex: 1,
    totalFocusTimeSec: 0,
    sessionsGoal: 8,
  },
  activePreset: DEFAULT_PRESETS[0],
  showSettings: false,
  windowWidth: 200,

  syncState: async () => {
    const state = await bridge.getTimerState();
    set({ state });
  },

  updateState: (state: TimerState) => {
    set({ state });
  },

  toggle: async () => {
    const state = await bridge.toggleTimer();
    set({ state });
  },

  reset: async () => {
    const state = await bridge.resetTimer();
    set({ state });
  },

  skip: async () => {
    const state = await bridge.skipSession();
    set({ state });
  },

  changePreset: async (preset: Preset) => {
    const state = await bridge.setPreset(preset);
    set({ state, activePreset: preset });
  },

  setSessionsGoal: async (goal: number) => {
    const state = await bridge.setSessionsGoal(goal);
    set({ state });
  },

  toggleSettings: () => {
    set((s) => ({ showSettings: !s.showSettings }));
  },

  setWindowWidth: (width: number) => {
    set({ windowWidth: width });
  },
}));
