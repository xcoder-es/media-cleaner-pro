import { Cpu, HardDrive, Clock, Image } from 'lucide-react';

export default function StatsPanel() {
  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      {[
        { icon: Image, label: 'Images Found', value: '—', color: 'text-mc-primary' },
        { icon: Clock, label: 'Elapsed', value: '—', color: 'text-mc-accent' },
        { icon: Cpu, label: 'CPU Usage', value: '—', color: 'text-mc-warning' },
        { icon: HardDrive, label: 'Memory', value: '—', color: 'text-mc-success' },
      ].map((stat, i) => (
        <div key={i} className="glass rounded-xl p-4 border border-mc-border">
          <div className="flex items-center gap-2 mb-2">
            <stat.icon className={`w-4 h-4 ${stat.color}`} />
            <span className="text-xs text-mc-text-muted">{stat.label}</span>
          </div>
          <div className="text-xl font-bold font-mono text-mc-text">{stat.value}</div>
        </div>
      ))}
    </div>
  );
}