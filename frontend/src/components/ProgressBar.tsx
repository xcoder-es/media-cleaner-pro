export default function ProgressBar({ progress, animated = false }: { progress: number; animated?: boolean }) {
  return (
    <div className="h-3 rounded-full bg-mc-bgElevated overflow-hidden border border-mc-border">
      <div 
        className={`h-full rounded-full bg-gradient-to-r from-mc-primary via-emerald-400 to-mc-primary ${animated ? 'animate-shimmer' : ''} progress-glow`}
        style={{ 
          width: `${Math.min(100, Math.max(0, progress))}%`,
          backgroundSize: '200% 100%',
        }}
      />
    </div>
  );
}