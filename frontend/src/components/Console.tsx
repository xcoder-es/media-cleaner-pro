import { Terminal } from 'lucide-react';

interface LogMessage {
  timestamp: string;
  level: string;
  stage: string;
  message: string;
}

export default function Console({ logs }: { logs: LogMessage[] }) {
  const levelColors: Record<string, string> = {
    INFO: 'text-mc-primary',
    WARN: 'text-mc-warning',
    ERROR: 'text-mc-error',
  };

  return (
    <div className="glass rounded-2xl border border-mc-border overflow-hidden">
      <div className="flex items-center gap-2 px-4 py-3 border-b border-mc-border bg-mc-bgCard">
        <Terminal className="w-4 h-4 text-mc-text-dim" />
        <span className="text-xs font-semibold text-mc-text-muted uppercase tracking-wider">System Log</span>
        <span className="ml-auto text-xs text-mc-text-dim font-mono">{logs.length} entries</span>
      </div>
      <div className="h-[500px] overflow-y-auto p-4 font-mono text-xs space-y-1.5">
        {logs.length === 0 ? (
          <div className="text-mc-text-dim text-center py-12">No log entries yet...</div>
        ) : (
          logs.slice(-500).map((log, i) => (
            <div key={i} className="flex gap-3 hover:bg-mc-bgElevated/50 rounded px-2 py-1 transition-colors">
              <span className="text-mc-text-dim flex-shrink-0">{new Date(log.timestamp).toLocaleTimeString()}</span>
              <span className={`flex-shrink-0 w-12 font-bold ${levelColors[log.level] || 'text-mc-text-muted'}`}>{log.level}</span>
              <span className="text-mc-accent flex-shrink-0 w-32 truncate">[{log.stage}]</span>
              <span className="text-mc-text-muted break-all">{log.message}</span>
            </div>
          ))
        )}
      </div>
    </div>
  );
}