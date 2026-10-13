import {
  MessageSquare,
  Bot,
  FileText,
  Gauge,
  Settings,
  Activity,
} from 'lucide-react';

export type MainNavView = 'chat' | 'agents' | 'workspace' | 'quota';

interface ActivityDockProps {
  activeView: MainNavView;
  onSelectView: (view: MainNavView) => void;
  onOpenSettings: () => void;
  onOpenBenchmark: () => void;
  showBenchmark: boolean;
  appVersion: string;
}

export const ActivityDock = ({
  activeView,
  onSelectView,
  onOpenSettings,
  onOpenBenchmark,
  showBenchmark,
  appVersion,
}: ActivityDockProps) => {
  const navItems: { id: MainNavView; label: string; icon: typeof MessageSquare }[] = [
    { id: 'chat', label: 'Trò chuyện', icon: MessageSquare },
    { id: 'agents', label: 'Tác tử & Kỹ năng', icon: Bot },
    { id: 'workspace', label: 'Báo cáo Workspace', icon: FileText },
    { id: 'quota', label: 'Hạn mức Model', icon: Gauge },
  ];

  return (
    <nav className="w-14 bg-[#07080c] border-r border-white/[0.06] flex flex-col items-center justify-between py-3.5 select-none shrink-0 z-30">
      {/* Top Branding Logo */}
      <div className="flex flex-col items-center gap-4 w-full">
        <div
          title={`Hubbub Multi-Agent v${appVersion}`}
          className="w-9 h-9 rounded-xl bg-gradient-to-tr from-indigo-600 via-indigo-500 to-cyan-400 p-0.5 flex items-center justify-center shadow-lg shadow-indigo-500/25 cursor-default transition-transform hover:scale-105"
        >
          <div className="w-full h-full bg-[#090a0f] rounded-[10px] flex items-center justify-center font-bold text-transparent bg-clip-text bg-gradient-to-tr from-indigo-300 via-white to-cyan-200 text-sm tracking-tight">
            H
          </div>
        </div>

        {/* Main Nav Items */}
        <div className="flex flex-col items-center gap-1.5 w-full px-2 mt-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            const isActive = activeView === item.id;
            return (
              <button
                key={item.id}
                onClick={() => onSelectView(item.id)}
                title={item.label}
                className={`relative w-10 h-10 rounded-xl flex items-center justify-center transition-all group ${
                  isActive
                    ? 'bg-indigo-600/15 text-indigo-300 border border-indigo-500/30 shadow-xs'
                    : 'text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.06] border border-transparent'
                }`}
              >
                {/* Active Indicator Bar on the left */}
                {isActive && (
                  <span className="absolute -left-2 w-1 h-5 rounded-r-full bg-gradient-to-b from-indigo-400 to-cyan-400 shadow-sm shadow-indigo-500/50" />
                )}
                <Icon className={`w-5 h-5 transition-transform group-hover:scale-105 ${isActive ? 'stroke-[2.2]' : 'stroke-[1.8]'}`} />
              </button>
            );
          })}
        </div>
      </div>

      {/* Bottom Actions: Settings & Benchmark */}
      <div className="flex flex-col items-center gap-2 w-full px-2">
        <button
          onClick={onOpenSettings}
          title="Cài đặt API Keys & Hệ thống"
          className="w-10 h-10 rounded-xl flex items-center justify-center text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.06] border border-transparent transition-all"
        >
          <Settings className="w-4.5 h-4.5 stroke-[1.8]" />
        </button>

        <button
          onClick={onOpenBenchmark}
          title={showBenchmark ? 'Quay lại' : 'Benchmark Gate 0'}
          className={`w-10 h-10 rounded-xl flex items-center justify-center transition-all ${
            showBenchmark
              ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] border border-transparent'
          }`}
        >
          <Activity className="w-4 h-4" />
        </button>

        <span className="text-[9px] font-mono text-zinc-400 pt-1 tracking-wider">
          v{appVersion.slice(0, 5)}
        </span>
      </div>
    </nav>
  );
};
