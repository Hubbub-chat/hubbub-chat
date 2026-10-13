import { Loader2, CheckCircle2, XCircle } from 'lucide-react';
import type { ToolLog } from '../types';

interface ActiveToolsListProps {
  tools: ToolLog[];
}

export const ActiveToolsList = ({ tools }: ActiveToolsListProps) => {
  if (tools.length === 0) return null;

  return (
    <div className="space-y-1.5 mb-2.5 max-w-md">
      {tools.map((tool) => (
        <div
          key={tool.id}
          className="p-2.5 bg-[#0f121b] border border-white/[0.07] rounded-xl text-[11px] font-mono flex items-center justify-between shadow-xs"
        >
          <div className="flex items-center gap-2">
            {tool.status === 'running' && (
              <Loader2 className="w-3.5 h-3.5 animate-spin text-cyan-400 shrink-0" />
            )}
            {tool.status === 'completed' && (
              <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
            )}
            {tool.status === 'failed' && (
              <XCircle className="w-3.5 h-3.5 text-rose-400 shrink-0" />
            )}
            <span className="text-zinc-200 font-medium">{tool.name}</span>
          </div>
          <span className="text-zinc-400 truncate max-w-xs text-[10px]">
            {tool.summary || tool.preview}
          </span>
        </div>
      ))}
    </div>
  );
};
