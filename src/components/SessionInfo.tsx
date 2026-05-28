import { useTimerStore } from "../stores/timerStore";
import { formatFocusTime } from "../lib/format";
import { useEffect, useState } from "react";

export function SessionInfo() {
  const { state, windowWidth } = useTimerStore();
  const [currentTime, setCurrentTime] = useState(formatCurrentTime());

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(formatCurrentTime());
    }, 10000);
    return () => clearInterval(interval);
  }, []);

  if (windowWidth < 140) return null;

  return (
    <div
      style={{
        position: "absolute",
        bottom: 4,
        left: 0,
        right: 0,
        display: "flex",
        justifyContent: "center",
        gap: 8,
        fontSize: 10,
        color: "rgba(255,255,255,0.6)",
        fontFamily: "system-ui, -apple-system, sans-serif",
        pointerEvents: "none",
        userSelect: "none",
      }}
    >
      <span>
        {state.currentSessionIndex}/{state.sessionsGoal}
      </span>
      <span>|</span>
      <span>{formatFocusTime(state.totalFocusTimeSec)}</span>
      <span>|</span>
      <span>{currentTime}</span>
    </div>
  );
}

function formatCurrentTime(): string {
  const now = new Date();
  return `${now.getHours()}:${String(now.getMinutes()).padStart(2, "0")}`;
}
