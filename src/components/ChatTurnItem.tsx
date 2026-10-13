import { useState } from 'react';
import {
  Sparkles,
  FileText,
  Copy,
  Check,
  ChevronDown,
  ChevronRight,
  Wrench,
  CheckCircle2,
  AlertCircle,
  Loader2,
} from 'lucide-react';
import type { ChatTurn, Agent } from '../types';
import { MarkdownContent } from './MarkdownContent';

interface ChatTurnItemProps {
  turn: ChatTurn;
  agent: Agent | null;
  onSaveAsReport: (text: string) => void;
  isStreaming?: boolean;
}

export const ChatTurnItem = ({
  turn,
  agent,
  onSaveAsReport,
  isStreaming = false,
}: ChatTurnItemProps) => {
  const [copied, setCopied] = useState(false);
  const [expandedTools, setExpandedTools] = useState<Record<string, boolean>>({});

  const toggleToolExpand = (toolId: string) => {
    setExpandedTools((prev) => ({ ...prev, [toolId]: !prev[toolId] }));
  };

  const handleCopy = (text: string) => {
    navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  // Extract user text
  const userText = turn.userMessage?.parts
    .filter((p) => p.type === 'Text')
    .map((p) => String(p.content ?? ''))
    .join('\n') || '';

  return (
    <div className="w-full my-4 space-y-4">
      {/* 1. User Message (Box message on the right) */}
      {turn.userMessage && (
        <div className="flex justify-end my-3 group">
          <div className="bg-[#141824] hover:bg-[#171b29] text-zinc-100 border border-white/[0.08] rounded-2xl rounded-tr-xs px-4 py-2.5 max-w-[90%] sm:max-w-2xl shadow-lg shadow-black/20 transition-colors relative inner-top-glow">
            <div className="whitespace-pre-wrap select-text text-xs leading-relaxed font-normal">
              {userText}
            </div>
            <div className="flex items-center justify-end gap-2 mt-1 select-none">
              <span className="text-[10px] text-zinc-400 font-mono">
                {new Date(turn.userMessage.created_at).toLocaleTimeString('vi-VN', {
                  hour: '2-digit',
                  minute: '2-digit',
                })}
              </span>
              <button
                onClick={() => handleCopy(userText)}
                title="Sao chép câu hỏi"
                className="opacity-0 group-hover:opacity-100 text-zinc-400 hover:text-zinc-200 transition-opacity p-0.5 rounded cursor-pointer"
              >
                <Copy className="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 2. Unified Assistant Turn (Single Cohesive Block) */}
      {(turn.finalText || turn.toolSteps.length > 0 || isStreaming) && (
        <div className="w-full flex gap-3 sm:gap-3.5 my-4">
          {/* Avatar */}
          <div className="w-7 h-7 sm:w-8 sm:h-8 rounded-xl shrink-0 flex items-center justify-center text-xs font-semibold select-none bg-gradient-to-tr from-indigo-500/15 to-cyan-500/15 border border-indigo-500/25 text-indigo-400 shadow-xs mt-0.5">
            <Sparkles className={`w-3.5 h-3.5 sm:w-4 sm:h-4 text-cyan-400 ${isStreaming ? 'animate-pulse' : ''}`} />
          </div>

          <div className="flex-1 min-w-0">
            {/* Header: Agent Name, Model, Time */}
            <div className="flex items-center gap-2 mb-2 select-none flex-wrap">
              <span className="text-xs font-semibold text-zinc-100">
                {agent?.name || 'AI Assistant'}
              </span>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-white/[0.05] border border-white/[0.07] text-zinc-400 font-mono">
                {agent?.model || 'gemini-3.8-flash'}
              </span>
              <span className="text-[10px] text-zinc-400 font-mono">
                {new Date(turn.createdAt).toLocaleTimeString('vi-VN', {
                  hour: '2-digit',
                  minute: '2-digit',
                })}
              </span>
              {isStreaming && (
                <span className="inline-flex items-center gap-1.5 text-[10px] text-indigo-400 ml-1">
                  <span className="w-1.5 h-1.5 rounded-full bg-cyan-400 animate-ping" />
                  Đang phản hồi...
                </span>
              )}
            </div>

            {/* Collapsible Tool Call Steps Accordion */}
            {turn.toolSteps.length > 0 && (
              <div className="my-2 space-y-1.5 max-w-2xl">
                {turn.toolSteps.map((tool) => {
                  const isExpanded = !!expandedTools[tool.id];
                  return (
                    <div
                      key={tool.id}
                      className="border border-white/[0.07] bg-[#0f121b]/80 rounded-xl overflow-hidden text-xs transition-colors"
                    >
                      <button
                        onClick={() => toggleToolExpand(tool.id)}
                        className="w-full px-3 py-2 flex items-center justify-between hover:bg-white/[0.04] transition-colors text-left cursor-pointer"
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          {tool.status === 'running' ? (
                            <Loader2 className="w-3.5 h-3.5 text-cyan-400 animate-spin shrink-0" />
                          ) : tool.status === 'completed' ? (
                            <CheckCircle2 className="w-3.5 h-3.5 text-emerald-400 shrink-0" />
                          ) : (
                            <AlertCircle className="w-3.5 h-3.5 text-red-400 shrink-0" />
                          )}
                          <Wrench className="w-3 h-3 text-indigo-400 shrink-0" />
                          <span className="font-mono text-[11px] text-zinc-300 font-medium truncate">
                            {tool.toolName}
                          </span>
                        </div>
                        <div className="flex items-center gap-1.5 text-zinc-400 text-[10px] shrink-0">
                          <span>{tool.status === 'running' ? 'Đang chạy' : tool.status === 'completed' ? 'Hoàn thành' : 'Thất bại'}</span>
                          {isExpanded ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
                        </div>
                      </button>

                      {isExpanded && (
                        <div className="px-3 pb-2.5 pt-1 border-t border-white/5 bg-[#0a0c12]/50 text-[11px] font-mono space-y-1.5 select-text">
                          {tool.args !== undefined && (
                            <div>
                              <span className="text-zinc-400 text-[10px] block">Tham số (Input):</span>
                              <pre className="p-2 bg-zinc-950/80 rounded-lg text-zinc-300 overflow-x-auto text-[10px]">
                                {typeof tool.args === 'string' ? tool.args : JSON.stringify(tool.args, null, 2)}
                              </pre>
                            </div>
                          )}
                          {tool.result !== undefined && (
                            <div>
                              <span className="text-zinc-400 text-[10px] block">Kết quả (Output):</span>
                              <pre className="p-2 bg-zinc-950/80 rounded-lg text-zinc-300 overflow-x-auto text-[10px] max-h-40">
                                {tool.result}
                              </pre>
                            </div>
                          )}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            )}

            {/* Markdown Text Content */}
            {turn.finalText && (
              <div className="select-text text-xs leading-relaxed text-zinc-100">
                <MarkdownContent content={turn.finalText} />
                {isStreaming && (
                  <span className="inline-block w-1.5 h-3.5 bg-cyan-400 ml-0.5 align-middle animate-pulse" />
                )}
              </div>
            )}

            {/* Turn Action Bar */}
            {!isStreaming && turn.finalText && (
              <div className="mt-3 flex items-center gap-2 select-none">
                <button
                  onClick={() => handleCopy(turn.finalText)}
                  title="Sao chép toàn bộ phản hồi"
                  className="px-2.5 py-1 bg-white/[0.04] hover:bg-white/[0.08] text-[11px] text-zinc-400 hover:text-zinc-200 rounded-xl border border-white/[0.07] flex items-center gap-1.5 transition-colors cursor-pointer"
                >
                  {copied ? <Check className="w-3 h-3 text-emerald-400" /> : <Copy className="w-3 h-3" />}
                  <span>{copied ? 'Đã sao chép' : 'Sao chép'}</span>
                </button>

                <button
                  onClick={() => onSaveAsReport(turn.finalText)}
                  title="Lưu câu trả lời vào thư mục reports/ trong Workspace"
                  className="px-2.5 py-1 bg-white/[0.04] hover:bg-white/[0.08] text-[11px] text-zinc-400 hover:text-zinc-200 rounded-xl border border-white/[0.07] flex items-center gap-1.5 transition-colors cursor-pointer"
                >
                  <FileText className="w-3 h-3 text-indigo-400" />
                  <span>Lưu thành Báo cáo</span>
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
