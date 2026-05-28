import { useTimerStore } from "../stores/timerStore";

export function Controls() {
  const { state, toggle, reset, skip, toggleSettings, windowWidth } =
    useTimerStore();

  if (windowWidth < 120) return null;

  const isRunning = state.status === "running";
  const isStopped = state.status === "stopped";

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        left: "50%",
        transform: "translate(-50%, -50%)",
        display: "flex",
        gap: 4,
        opacity: 0,
        transition: "opacity 0.2s",
        pointerEvents: "none",
      }}
      className="controls-overlay"
    >
      {!isStopped && (
        <button onClick={reset} style={buttonStyle} title="Reset (R)">
          ↺
        </button>
      )}
      <button
        onClick={toggle}
        style={{ ...buttonStyle, fontSize: 18 }}
        title="Start/Pause (Space)"
      >
        {isRunning ? "⏸" : "▶"}
      </button>
      {!isStopped && (
        <button onClick={skip} style={buttonStyle} title="Skip (S)">
          ⏭
        </button>
      )}
      <button
        onClick={toggleSettings}
        style={buttonStyle}
        title="Settings (⌘,)"
      >
        ⚙
      </button>
    </div>
  );
}

const buttonStyle: React.CSSProperties = {
  background: "rgba(255,255,255,0.15)",
  border: "none",
  borderRadius: "50%",
  width: 32,
  height: 32,
  fontSize: 14,
  color: "white",
  cursor: "pointer",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  pointerEvents: "auto",
  backdropFilter: "blur(4px)",
};
