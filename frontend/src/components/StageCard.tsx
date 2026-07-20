import { CheckCircle2, Circle, XCircle, SkipForward, Loader2 } from 'lucide-react';

interface StageInfo {
  name: string;
  description: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
  progress: number;
  processed: number;
  total: number;
  error: string | null;
}

export default function StageCard({ index, stage, isActive }: { index: number; stage: StageInfo; isActive: boolean }) {
  const statusConfig = {
    pending: { icon: Circle, color: 'text-mc-text-dim', bg: 'bg-mc-bgElevated', border: 'border-mc-border' },
    running: { icon: Loader2, color: 'text-mc-primary', bg: 'bg-mc-primaryDim', border: 'border-mc-primary/50' },
    completed: { icon: CheckCircle2, color: 'text-mc-success', bg: 'bg-emerald-500/10', border: 'border-emerald-500/30' },
    failed: { icon: XCircle, color: 'text-mc-error', bg: 'bg-red-500/10', border: 'border-red-500/30' },
    skipped: { icon: SkipForward, color: 'text-mc-text-dim', bg: 'bg-mc-bgElevated', border: 'border-mc-border' },
  };

  const config = statusConfig[stage.status];
  const Icon = config.icon;

  return (
    <div className={`relative rounded-2xl border p-5 transition-all duration-300 ${config.bg} ${config.border} ${isActive ? 'ring-1 ring-mc-primary/30 shadow-lg shadow-mc-primary/5' : ''}`}>
      {isActive && (
        <div className="absolute inset-0 rounded-2xl bg-gradient-to-r from-mc-primary/5 to-transparent animate-pulse-slow pointer-events-none" />
      )}
      
      <div className="relative flex items-start gap-4">
        <div className={`flex-shrink-0 w-10 h-10 rounded-xl flex items-center justify-center ${stage.status === 'running' ? 'bg-mc-primary/20' : 'bg-mc-bgCard'}`}>
          <Icon className={`w-5 h-5 ${config.color} ${stage.status === 'running' ? 'animate-spin' : ''}`} />
        </div>
        
        <div className="flex-1 min-w-0">
          <div className="flex items-center justify-between mb-1">
            <h3 className="text-sm font-semibold text-mc-text">
              <span className="text-mc-text-dim mr-2">#{index}</span>
              {stage.name}
            </h3>
            <span className={`text-xs font-medium px-2 py-1 rounded-md ${config.color} bg-mc-bgCard`}>
              {stage.status}
            </span>
          </div>
          
          <p className="text-xs text-mc-text-muted mb-3 leading-relaxed">{stage.description}</p>
          
          {stage.total > 0 && (
            <div className="space-y-2">
              <div className="flex items-center justify-between text-xs">
                <span className="text-mc-text-dim">{stage.processed.toLocaleString()} / {stage.total.toLocaleString()}</span>
                <span className="font-mono font-medium text-mc-text">{stage.progress.toFixed(1)}%</span>
              </div>
              <div className="h-1.5 rounded-full bg-mc-bg overflow-hidden">
                <div 
                  className={`h-full rounded-full transition-all duration-500 ${stage.status === 'running' ? 'bg-gradient-to-r from-mc-primary to-emerald-400' : config.color.replace('text-', 'bg-')}`}
                  style={{ width: `${stage.progress}%` }}
                />
              </div>
            </div>
          )}
          
          {stage.error && (
            <div className="mt-3 p-2 rounded-lg bg-red-500/10 border border-red-500/20 text-xs text-mc-error">
              {stage.error}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}