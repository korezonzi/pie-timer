import { load, type Store } from "@tauri-apps/plugin-store";

// 28:00 == 4 AM. Anything before this hour counts as the previous day, so the
// session progress only rolls over once the user is realistically asleep.
const LOGICAL_DAY_RESET_HOUR = 4;
const LAST_DAY_KEY = "lastActiveDay";

let store: Store | null = null;

async function getStore(): Promise<Store> {
  if (!store) {
    store = await load("settings.json");
  }
  return store;
}

// Logical day string (YYYY-MM-DD) using the 4 AM cutoff.
export function getLogicalDay(date: Date = new Date()): string {
  const d = new Date(date.getTime());
  d.setHours(d.getHours() - LOGICAL_DAY_RESET_HOUR);
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

async function loadLastActiveDay(): Promise<string | null> {
  const s = await getStore();
  return (await s.get<string>(LAST_DAY_KEY)) ?? null;
}

async function saveLastActiveDay(day: string): Promise<void> {
  const s = await getStore();
  await s.set(LAST_DAY_KEY, day);
}

// Reset session progress when the logical day rolls over (at 4 AM).
// Returns true if a reset was actually performed.
export async function checkDailyReset(
  onReset: () => Promise<void>,
): Promise<boolean> {
  try {
    const today = getLogicalDay();
    const last = await loadLastActiveDay();

    if (last === null) {
      await saveLastActiveDay(today);
      return false;
    }
    if (last !== today) {
      await onReset();
      await saveLastActiveDay(today);
      return true;
    }
    return false;
  } catch (err) {
    console.warn("Daily reset check failed:", err);
    return false;
  }
}
