export type Phase = "idle" | "work" | "break" | "longBreak";
export type Status = "running" | "paused" | "stopped";

export interface TimerState {
  phase: Phase;
  status: Status;
  totalDurationSec: number;
  remainingSec: number;
  completedSessions: number;
  currentSessionIndex: number;
  totalFocusTimeSec: number;
  sessionsGoal: number;
}

export interface Preset {
  id: string;
  name: string;
  workDurationSec: number;
  breakDurationSec: number;
  longBreakDurationSec: number;
  sessionsBeforeLongBreak: number;
}

export const DEFAULT_PRESETS: Preset[] = [
  {
    id: "pomodoro",
    name: "Pomodoro",
    workDurationSec: 25 * 60,
    breakDurationSec: 5 * 60,
    longBreakDurationSec: 15 * 60,
    sessionsBeforeLongBreak: 4,
  },
  {
    id: "52-17",
    name: "52-17",
    workDurationSec: 52 * 60,
    breakDurationSec: 17 * 60,
    longBreakDurationSec: 17 * 60,
    sessionsBeforeLongBreak: 4,
  },
  {
    id: "deep-work-90",
    name: "Deep Work 90min",
    workDurationSec: 90 * 60,
    breakDurationSec: 20 * 60,
    longBreakDurationSec: 30 * 60,
    sessionsBeforeLongBreak: 2,
  },
];
