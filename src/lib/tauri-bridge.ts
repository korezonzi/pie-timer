import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { TimerState, Preset } from "../types/timer";

export async function getTimerState(): Promise<TimerState> {
  return invoke<TimerState>("get_timer_state");
}

export async function startTimer(): Promise<TimerState> {
  return invoke<TimerState>("start_timer");
}

export async function pauseTimer(): Promise<TimerState> {
  return invoke<TimerState>("pause_timer");
}

export async function toggleTimer(): Promise<TimerState> {
  return invoke<TimerState>("toggle_timer");
}

export async function resetTimer(): Promise<TimerState> {
  return invoke<TimerState>("reset_timer");
}

export async function skipSession(): Promise<TimerState> {
  return invoke<TimerState>("skip_session");
}

export async function setPreset(preset: Preset): Promise<TimerState> {
  return invoke<TimerState>("set_preset", { preset });
}

export async function setSessionsGoal(goal: number): Promise<TimerState> {
  return invoke<TimerState>("set_sessions_goal", { goal });
}

export function onTimerTick(
  callback: (state: TimerState) => void,
): Promise<UnlistenFn> {
  return listen<TimerState>("timer:tick", (event) => {
    callback(event.payload);
  });
}

export function onTimerCompleted(
  callback: (phase: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("timer:completed", (event) => {
    callback(event.payload);
  });
}
