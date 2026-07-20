import { useState, useEffect, useCallback } from 'react';
import { Play, Pause, Square, FolderOpen, FolderOutput, Settings, Zap, Activity } from 'lucide-react';
import StageCard from './StageCard';
import ProgressBar from './ProgressBar';
import Console from './Console';
import StatsPanel from './StatsPanel';

const API_URL = import.meta.env.PUBLIC_API_URL || 'http://127.0.0.1:8081';

interface StageInfo {
  name: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
  progress: number;
  processed: number;
  total: number;
  started_at: string | null;
  completed_at: string | null;
  error: string | null;
}

interface ProcessingStats {
  current_file: string | null;
  current_dhash: string | null;
  unique_count: number;
  duplicate_count: number;
  error_count: number;
  speed: number;
  eta_seconds: number;
  memory_mb: number;
  cpu_percent: number;
}

interface AppState {
  stages: StageInfo[];
  stats: ProcessingStats;
  is_running: boolean;
  is_paused: boolean;
}

export default function Dashboard() {
  const [state, setState] = useState<AppState>({
    stages: [],
    stats: {
      current_file: null,
      current_dhash: null,
      unique_count: 0,
      duplicate_count: 0,
      error_count: 0,
      speed: 0,
      eta_seconds: 0,
      memory_mb: 0,
      cpu_percent: 0,
    },
    is_running: false,
    is_paused: false,
  });

  const [sourceDir, setSourceDir] = useState('');
  const [destDir, setDestDir] = useState('');
  const [threshold, setThreshold] = useState(4);
  const [logs, setLogs] = useState<any[]>([]);
  const [showSettings, setShowSettings] = useState(false);
  const [activeTab, setActiveTab] = useState<'pipeline' | 'console'>('pipeline');

  // Poll status
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`${API_URL}/api/status`);
        if (res.ok) {
          const data = await res.json();
          setState(data);
        }
      } catch (e) {
        // Server not ready yet
      }
    }, 1000);
    return () => clearInterval(interval);
  }, []);

  // Poll logs
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const res = await fetch(`${API_URL}/api/logs`);
        if (res.ok) {
          const data = await res.json();
          setLogs(data);
        }
      } catch (e) {}
    }, 2000);
    return () => clearInterval(interval);
  }, []);

  const startJob = useCallback(async () => {
    await fetch(`${API_URL}/api/start`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        source_dir: sourceDir || '/data/source',
        dest_dir: destDir || '/data/output',
        hamming_threshold: threshold,
      }),
    });
  }, [sourceDir, destDir, threshold]);

  const controlJob = useCallback(async (action: string) => {
    await fetch(`${API_URL}/api/control`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ action }),
    });
  }, []);

  const overallProgress = state.stages.length > 0
    ? state.stages.reduce((acc, s) => acc + s.progress, 0) / state.stages.length
    : 0;

  const currentStage = state.stages.find(s => s.status === 'running');

  return (
    <div className="min-h-screen bg-mc-bg text-mc-text">
      {/* Header */}
      <header className="sticky top-0 z-50 glass border-b border-mc-border">
        <div className="max-w-7xl mx-auto px-6 py-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-4">
              <div className="w-10 h-10 rounded-xl bg-gradient-to-br from-mc-primary to-mc-accent flex items-center justify-center shadow-lg shadow-mc-primary/20">
                <Zap className="w-5 h-5 text-white" />
              </div>
              <div>
                <h1 className="text-xl font-bold tracking-tight">
                  <span className="text-gradient">MediaCleaner</span>
                  <span className="text-mc-text ml-2 text-sm font-medium opacity-60">Pro</span>
                </h1>
                <p className="text-xs text-mc-text-dim">by Carlos Pinto &lt;capintobe@gmail.com&gt;</p>
              </div>
            </div>

            <div className="flex items-center gap-3">
              <div className="flex items-center gap-2 px-3 py-1.5 rounded-lg bg-mc-bgElevated border border-mc-border">
                <Activity className={`w-4 h-4 ${state.is_running ? 'text-mc-primary animate-pulse' : 'text-mc-text-dim'}`} />
                <span className="text-xs font-medium text-mc-text-muted">
                  {state.is_running ? (state.is_paused ? 'Paused' : 'Running') : 'Idle'}
                </span>
              </div>
              <button
                onClick={() => setShowSettings(!showSettings)}
                className="p-2 rounded-lg hover:bg-mc-bgElevated transition-colors"
              >
                <Settings className="w-5 h-5 text-mc-text-muted" />
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Settings Panel */}
      {showSettings && (
        <div className="border-b border-mc-border bg-mc-bgCard animate-slide-up">
          <div className="max-w-7xl mx-auto px-6 py-6">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="space-y-2">
                <label className="text-xs font-medium text-mc-text-muted uppercase tracking-wider">Source Directory</label>
                <div className="flex gap-2">
                  <div className="flex-1 flex items-center gap-2 px-3 py-2 rounded-lg bg-mc-bgElevated border border-mc-border focus-within:border-mc-primary transition-colors">
                    <FolderOpen className="w-4 h-4 text-mc-text-dim" />
                    <input
                      type="text"
                      value={sourceDir}
                      onChange={(e) => setSourceDir(e.target.value)}
                      placeholder="/data/source"
                      className="flex-1 bg-transparent text-sm outline-none placeholder:text-mc-text-dim"
                    />
                  </div>
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-medium text-mc-text-muted uppercase tracking-wider">Destination Directory</label>
                <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-mc-bgElevated border border-mc-border focus-within:border-mc-primary transition-colors">
                  <FolderOutput className="w-4 h-4 text-mc-text-dim" />
                  <input
                    type="text"
                    value={destDir}
                    onChange={(e) => setDestDir(e.target.value)}
                    placeholder="/data/output"
                    className="flex-1 bg-transparent text-sm outline-none placeholder:text-mc-text-dim"
                  />
                </div>
              </div>

              <div className="space-y-2">
                <label className="text-xs font-medium text-mc-text-muted uppercase tracking-wider">Hamming Threshold</label>
                <div className="flex items-center gap-3 px-3 py-2 rounded-lg bg-mc-bgElevated border border-mc-border">
                  <input
                    type="range"
                    min="0"
                    max="16"
                    value={threshold}
                    onChange={(e) => setThreshold(Number(e.target.value))}
                    className="flex-1 accent-mc-primary"
                  />
                  <span className="text-sm font-mono font-medium text-mc-primary w-6">{threshold}</span>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="max-w-7xl mx-auto px-6 py-8">
        {/* Control Bar */}
        <div className="mb-8 flex items-center justify-between">
          <div className="flex items-center gap-3">
            {!state.is_running ? (
              <>
                <button
                  onClick={startJob}
                  className="flex items-center gap-2 px-6 py-3 rounded-xl bg-gradient-to-r from-mc-primary to-emerald-500 text-mc-bg font-semibold shadow-lg shadow-mc-primary/25 hover:shadow-mc-primary/40 hover:scale-[1.02] active:scale-[0.98] transition-all"
                >
                  <Play className="w-5 h-5 fill-current" />
                  Start Processing
                </button>
              </>
            ) : (
              <>
                {state.is_paused ? (
                  <button
                    onClick={() => controlJob('resume')}
                    className="flex items-center gap-2 px-6 py-3 rounded-xl bg-mc-primary text-mc-bg font-semibold shadow-lg shadow-mc-primary/25 hover:scale-[1.02] active:scale-[0.98] transition-all"
                  >
                    <Play className="w-5 h-5 fill-current" />
                    Resume
                  </button>
                ) : (
                  <button
                    onClick={() => controlJob('pause')}
                    className="flex items-center gap-2 px-6 py-3 rounded-xl bg-mc-bgElevated border border-mc-border text-mc-text font-semibold hover:border-mc-warning hover:text-mc-warning transition-all"
                  >
                    <Pause className="w-5 h-5" />
                    Pause
                  </button>
                )}
                <button
                  onClick={() => controlJob('cancel')}
                  className="flex items-center gap-2 px-6 py-3 rounded-xl bg-mc-bgElevated border border-mc-border text-mc-text font-semibold hover:border-mc-error hover:text-mc-error transition-all"
                >
                  <Square className="w-5 h-5 fill-current" />
                  Cancel
                </button>
              </>
            )}
          </div>

          <div className="flex items-center gap-2 text-sm text-mc-text-muted">
            <span className="w-2 h-2 rounded-full bg-mc-primary animate-pulse" />
            Backend: {state.stages.length > 0 ? 'Connected' : 'Connecting...'}
          </div>
        </div>

        {/* Overall Progress */}
        {state.is_running && (
          <div className="mb-8 glass rounded-2xl p-6 glow-primary">
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-lg font-semibold text-mc-text">Overall Progress</h2>
                {currentStage && (
                  <p className="text-sm text-mc-text-muted mt-1">
                    Current: <span className="text-mc-primary font-medium">{currentStage.name}</span>
                    {state.stats.current_file && (
                      <span className="ml-2 text-mc-text-dim">— {state.stats.current_file}</span>
                    )}
                  </p>
                )}
              </div>
              <div className="text-right">
                <div className="text-2xl font-bold text-gradient">{overallProgress.toFixed(1)}%</div>
                <div className="text-xs text-mc-text-dim">
                  ETA: {formatDuration(state.stats.eta_seconds)}
                </div>
              </div>
            </div>
            <ProgressBar progress={overallProgress} animated />

            <div className="mt-4 grid grid-cols-2 md:grid-cols-4 gap-4">
              <StatBox label="Speed" value={`${state.stats.speed.toFixed(1)} img/s`} color="primary" />
              <StatBox label="Unique" value={state.stats.unique_count.toLocaleString()} color="success" />
              <StatBox label="Duplicates" value={state.stats.duplicate_count.toLocaleString()} color="warning" />
              <StatBox label="Errors" value={state.stats.error_count.toLocaleString()} color="error" />
            </div>
          </div>
        )}

        {/* Tabs */}
        <div className="flex gap-1 mb-6 p-1 rounded-xl bg-mc-bgElevated border border-mc-border w-fit">
          <button
            onClick={() => setActiveTab('pipeline')}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
              activeTab === 'pipeline'
                ? 'bg-mc-bgCard text-mc-primary shadow-sm'
                : 'text-mc-text-muted hover:text-mc-text'
            }`}
          >
            Pipeline
          </button>
          <button
            onClick={() => setActiveTab('console')}
            className={`px-4 py-2 rounded-lg text-sm font-medium transition-all ${
              activeTab === 'console'
                ? 'bg-mc-bgCard text-mc-primary shadow-sm'
                : 'text-mc-text-muted hover:text-mc-text'
            }`}
          >
            Console
          </button>
        </div>

        {/* Content */}
        {activeTab === 'pipeline' ? (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {state.stages.map((stage, idx) => (
              <StageCard
                key={idx}
                index={idx + 1}
                stage={stage}
                isActive={stage.status === 'running'}
              />
            ))}
          </div>
        ) : (
          <Console logs={logs} />
        )}

        {/* Footer */}
        <footer className="mt-12 pt-6 border-t border-mc-border text-center">
          <p className="text-xs text-mc-text-dim">
            MediaCleaner Pro v2.0.0 — Built with Rust + Astro — Carlos Pinto &lt;capintobe@gmail.com&gt;
          </p>
        </footer>
      </div>
    </div>
  );
}

function StatBox({ label, value, color }: { label: string; value: string; color: string }) {
  const colorMap: Record<string, string> = {
    primary: 'text-mc-primary bg-mc-primaryDim',
    success: 'text-mc-success bg-emerald-500/10',
    warning: 'text-mc-warning bg-amber-500/10',
    error: 'text-mc-error bg-red-500/10',
  };

  return (
    <div className={`rounded-xl p-3 ${colorMap[color] || colorMap.primary}`}>
      <div className="text-xs font-medium opacity-70 mb-1">{label}</div>
      <div className="text-lg font-bold font-mono">{value}</div>
    </div>
  );
}

function formatDuration(seconds: number): string {
  if (!seconds || seconds === 0) return '--:--:--';
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;
  return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}:${s.toString().padStart(2, '0')}`;
}
