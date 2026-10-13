import {
  Bot,
  Plus,
  Sparkles,
  Code2,
  Search,
  PenTool,
  GraduationCap,
  BookOpen,
  Sliders,
  MessageSquare,
  Globe,
  FileCode,
  FileEdit,
} from 'lucide-react';
import type { Agent } from '../types';

interface AgentsHubViewProps {
  agents: Agent[];
  onSelectAgentForChat: (agentId: string) => void;
  onOpenAgentDrawer: (agent: Agent) => void;
  onOpenCreateAgent: () => void;
}

export const AgentsHubView = ({
  agents,
  onSelectAgentForChat,
  onOpenAgentDrawer,
  onOpenCreateAgent,
}: AgentsHubViewProps) => {
  const getAgentIcon = (id: string) => {
    switch (id) {
      case 'analyst':
        return <Sparkles className="w-5 h-5 text-indigo-400" />;
      case 'developer':
        return <Code2 className="w-5 h-5 text-cyan-400" />;
      case 'researcher':
        return <Search className="w-5 h-5 text-blue-400" />;
      case 'writer':
        return <PenTool className="w-5 h-5 text-purple-400" />;
      case 'tutor':
        return <GraduationCap className="w-5 h-5 text-amber-400" />;
      case 'librarian':
        return <BookOpen className="w-5 h-5 text-emerald-400" />;
      default:
        return <Bot className="w-5 h-5 text-zinc-400" />;
    }
  };

  const getToolBadge = (toolId: string) => {
    switch (toolId) {
      case 'web_search':
        return { label: 'Web Search', icon: Globe, color: 'text-cyan-400 bg-cyan-950/40 border-cyan-500/20' };
      case 'web_fetch':
        return { label: 'Web Fetch', icon: Globe, color: 'text-blue-400 bg-blue-950/40 border-blue-500/20' };
      case 'fs_read':
      case 'fs_list':
        return { label: 'Files', icon: FileCode, color: 'text-emerald-400 bg-emerald-950/40 border-emerald-500/20' };
      case 'report_write':
      case 'report_read':
        return { label: 'Reports', icon: FileEdit, color: 'text-indigo-400 bg-indigo-950/40 border-indigo-500/20' };
      default:
        return { label: toolId, icon: Sparkles, color: 'text-zinc-400 bg-zinc-900 border-zinc-700' };
    }
  };

  return (
    <div className="flex-1 flex flex-col h-full bg-[#08090d] overflow-hidden select-none">
      {/* Top Header */}
      <header className="py-3 px-4 sm:px-6 border-b border-white/[0.06] flex flex-col sm:flex-row sm:items-center justify-between gap-3 shrink-0 bg-[#0a0c12]/85 backdrop-blur-md">
        <div>
          <h2 className="text-sm sm:text-base font-bold text-zinc-100 flex items-center gap-2">
            <span>Trung tâm Tác tử & Kỹ năng (Agents & Skills Hub)</span>
          </h2>
          <p className="text-[11px] text-zinc-400">
            Mỗi Tác tử sở hữu vai trò chuyên biệt, bộ công cụ riêng và chính sách bảo mật kiểm soát chặt chẽ
          </p>
        </div>

        <button
          onClick={onOpenCreateAgent}
          className="self-start sm:self-auto px-4 py-2 bg-gradient-to-r from-indigo-600 via-indigo-500 to-cyan-500 hover:brightness-110 active:scale-[0.98] text-white rounded-xl text-xs font-semibold shadow-md shadow-indigo-600/25 flex items-center gap-1.5 transition-all cursor-pointer inner-top-glow"
        >
          <Plus className="w-4 h-4" />
          <span>Tạo Tác tử Mới</span>
        </button>
      </header>

      {/* Grid of Agent Cards */}
      <div className="flex-1 overflow-y-auto p-4 sm:p-6">
        <div className="max-w-7xl mx-auto grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4 gap-4">
          {agents.map((agent) => {
            return (
              <div
                key={agent.id}
                className="cursor-card rounded-2xl p-5 flex flex-col justify-between transition-all group"
              >
                <div>
                  {/* Card Header: Icon + Name + Model */}
                  <div className="flex items-start justify-between mb-3">
                    <div className="flex items-center gap-3">
                      <div className="w-10 h-10 rounded-xl bg-white/[0.05] border border-white/[0.08] flex items-center justify-center group-hover:scale-105 transition-transform shadow-xs">
                        {getAgentIcon(agent.id)}
                      </div>
                      <div>
                        <h3 className="text-sm font-bold text-zinc-100 group-hover:text-indigo-300 transition-colors">
                          {agent.name}
                        </h3>
                        <span className="text-[10px] text-zinc-400 font-mono">
                          ID: {agent.id}
                        </span>
                      </div>
                    </div>

                    <span className="text-[10px] px-2 py-0.5 rounded-full bg-white/[0.05] border border-white/[0.07] text-zinc-400 font-mono">
                      {agent.model}
                    </span>
                  </div>

                  {/* System Prompt Excerpt */}
                  <p className="text-xs text-zinc-400 line-clamp-3 mb-4 leading-relaxed font-sans">
                    {agent.system_prompt.replace(/^(You are|Bạn là)\s*/i, '')}
                  </p>

                  {/* Skills / Tools Badges */}
                  <div className="mb-4">
                    <span className="text-[10px] font-semibold text-zinc-400 uppercase tracking-wider block mb-1.5">
                      Công cụ & Kỹ năng ({agent.tools.builtin.length})
                    </span>
                    <div className="flex flex-wrap gap-1.5">
                      {agent.tools.builtin.map((toolId) => {
                        const badge = getToolBadge(toolId);
                        const Icon = badge.icon;
                        return (
                          <span
                            key={toolId}
                            className={`text-[10px] px-2 py-0.5 rounded-lg border flex items-center gap-1 font-medium ${badge.color}`}
                          >
                            <Icon className="w-2.5 h-2.5" />
                            <span>{badge.label}</span>
                          </span>
                        );
                      })}
                    </div>
                  </div>
                </div>

                {/* Card Footer Actions */}
                <div className="pt-3 border-t border-white/[0.06] flex items-center justify-between gap-2">
                  <button
                    onClick={() => onOpenAgentDrawer(agent)}
                    className="flex-1 py-1.5 px-3 bg-white/[0.04] hover:bg-white/[0.08] text-zinc-300 hover:text-zinc-100 rounded-xl text-xs font-medium flex items-center justify-center gap-1.5 border border-white/[0.06] transition-colors cursor-pointer"
                  >
                    <Sliders className="w-3.5 h-3.5 text-zinc-400" />
                    <span>Cấu hình & Skills</span>
                  </button>

                  <button
                    onClick={() => onSelectAgentForChat(agent.id)}
                    className="py-1.5 px-3.5 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 text-white rounded-xl text-xs font-medium flex items-center justify-center gap-1.5 transition-colors shadow-xs cursor-pointer inner-top-glow"
                  >
                    <MessageSquare className="w-3.5 h-3.5" />
                    <span>Chat</span>
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
