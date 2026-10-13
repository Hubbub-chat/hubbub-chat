import { useState, useRef, useEffect, useMemo, type KeyboardEvent, type ChangeEvent } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Message, Agent, ToolLog, ToolStep, ChatTurn } from '../types';
import {
  Send,
  Square,
  Bot,
  Sparkles,
  AlertTriangle,
  CheckCircle2,
  Sliders,
  Gauge,
  Trash2,
  PanelLeft,
} from 'lucide-react';
import { ModelSelector } from './ModelSelector';
import { ChatTurnItem } from './ChatTurnItem';
import { groupMessagesIntoTurns } from '../utils/turnGrouper';

interface ChatViewProps {
  conversationId: string;
  conversationTitle: string;
  messages: Message[];
  agent: Agent | null;
  onSendMessage: (prompt: string) => Promise<void>;
  onCancelRun: () => void;
  onSelectModel: (model: string) => Promise<void>;
  isStreaming: boolean;
  streamingText: string;
  activeTools: ToolLog[];
  errorMsg: string | null;
  onOpenSettings: () => void;
  onOpenAgentDrawer: () => void;
  onOpenQuotaModal: () => void;
  isSidebarOpen?: boolean;
  onToggleSidebar?: () => void;
}

export const ChatView = ({
  conversationId: _conversationId,
  conversationTitle,
  messages,
  agent,
  onSendMessage,
  onCancelRun,
  onSelectModel,
  isStreaming,
  streamingText,
  activeTools,
  errorMsg,
  onOpenSettings,
  onOpenAgentDrawer,
  onOpenQuotaModal,
  isSidebarOpen = true,
  onToggleSidebar,
}: ChatViewProps) => {
  const [input, setInput] = useState('');
  const [isComposing, setIsComposing] = useState(false);
  const [reportToast, setReportToast] = useState<string | null>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const activeTurnRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const isSendingLocalRef = useRef(false);

  // Group database messages into consolidated conversational turns
  const completedTurns = useMemo(() => {
    return groupMessagesIntoTurns(messages);
  }, [messages]);

  // Handle saving text to reports/
  const handleSaveAsReport = async (text: string) => {
    if (!text.trim()) return;
    const firstLine = text.trim().split('\n')[0].replace(/^#+\s*/, '').slice(0, 40) || 'Báo cáo từ AI';
    try {
      await invoke('write_report', {
        title: firstLine,
        content: text,
        filename: null,
      });
      setReportToast(`Đã lưu "${firstLine}" vào reports/`);
      setTimeout(() => setReportToast(null), 3000);
    } catch (err) {
      setReportToast('Lỗi khi lưu: ' + String(err));
      setTimeout(() => setReportToast(null), 3000);
    }
  };

  // Scroll to bottom on conversation switch
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'auto' });
  }, [_conversationId]);

  // Focus textarea when conversation changes
  useEffect(() => {
    textareaRef.current?.focus();
  }, [conversationTitle]);

  const handleSend = () => {
    if (!input.trim() || isStreaming || isSendingLocalRef.current) return;
    const prompt = input.trim();
    setInput('');
    if (textareaRef.current) {
      textareaRef.current.style.height = 'auto';
    }
    isSendingLocalRef.current = true;
    onSendMessage(prompt).finally(() => {
      isSendingLocalRef.current = false;
    });
    setTimeout(() => {
      activeTurnRef.current?.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }, 60);
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey && !isComposing) {
      e.preventDefault();
      handleSend();
    }
  };

  const handleTextareaInput = (e: ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    e.target.style.height = 'auto';
    e.target.style.height = `${Math.min(e.target.scrollHeight, 180)}px`;
  };

  // Convert active streaming tools into ToolStep format
  const activeToolSteps: ToolStep[] = activeTools.map((t) => ({
    id: t.id,
    toolName: t.name,
    args: t.preview,
    result: t.summary,
    status: t.status,
  }));

  // Build streaming turn if active
  const streamingTurn: ChatTurn | null = isStreaming
    ? {
        id: `streaming-${Date.now()}`,
        assistantMessages: [],
        toolSteps: activeToolSteps,
        finalText: streamingText,
        createdAt: new Date().toISOString(),
      }
    : null;

  return (
    <div className="flex-1 flex flex-col h-full bg-[#08090d] overflow-hidden relative select-none">
      {/* Toast Notification */}
      {reportToast && (
        <div className="fixed top-4 right-4 z-50 bg-[#141824] border border-indigo-500/50 text-indigo-200 text-xs px-3.5 py-2 rounded-xl shadow-2xl flex items-center gap-2 animate-fade-in">
          <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0" />
          <span>{reportToast}</span>
        </div>
      )}

      {/* Top Header */}
      <header className="h-14 sm:h-16 border-b border-white/[0.06] px-3 sm:px-6 flex items-center justify-between shrink-0 bg-[#0a0c12]/85 backdrop-blur-md z-20">
        <div className="flex items-center gap-2 sm:gap-3 min-w-0">
          {onToggleSidebar && (
            <button
              onClick={onToggleSidebar}
              title={isSidebarOpen ? 'Thu gọn thanh bên' : 'Mở rộng thanh bên'}
              className="p-1.5 sm:p-2 rounded-xl text-zinc-400 hover:text-zinc-100 hover:bg-white/[0.06] transition-colors cursor-pointer shrink-0"
            >
              <PanelLeft className="w-4 h-4" />
            </button>
          )}

          <div className="w-8 h-8 sm:w-9 sm:h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shadow-xs shrink-0">
            <Bot className="w-4 h-4 sm:w-5 sm:h-5" />
          </div>
          <div className="min-w-0">
            <h2 className="text-xs sm:text-sm font-semibold text-zinc-100 truncate max-w-[160px] sm:max-w-xs md:max-w-md">
              {conversationTitle || 'Cuộc trò chuyện'}
            </h2>
            <div className="flex items-center gap-1.5 sm:gap-2 text-[10px] sm:text-[11px] text-zinc-400 truncate">
              <span className="text-indigo-400 font-medium truncate">{agent?.name || 'AI Assistant'}</span>
              <span>•</span>
              <ModelSelector
                currentModel={agent?.model || 'gemini-3.8-flash'}
                onSelectModel={onSelectModel}
                disabled={isStreaming}
              />
            </div>
          </div>
        </div>

        {/* Right Header Actions: Skills Badges, Quota Pill, Inspector Button */}
        <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
          {/* Active Skills Badges (Click to inspect) */}
          {agent && (
            <button
              onClick={onOpenAgentDrawer}
              title="Nhấp để xem và cấu hình kỹ năng của Tác tử này"
              className="hidden md:flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/[0.05] hover:bg-white/[0.09] border border-white/[0.08] text-[11px] text-zinc-300 transition-colors cursor-pointer"
            >
              <Sparkles className="w-3 h-3 text-cyan-400" />
              <span>{agent.tools.builtin.length} Kỹ năng kích hoạt</span>
            </button>
          )}

          {/* Mini Quota Pill */}
          <button
            onClick={onOpenQuotaModal}
            title="Theo dõi Hạn mức & Sử dụng Model"
            className="flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-white/[0.05] hover:bg-white/[0.09] border border-white/[0.08] text-[11px] text-zinc-300 transition-colors cursor-pointer"
          >
            <Gauge className="w-3.5 h-3.5 text-cyan-400" />
            <span className="hidden sm:inline">Hạn mức</span>
          </button>

          {/* Agent Inspector Button */}
          <button
            onClick={onOpenAgentDrawer}
            title="Cấu hình Tác tử & Bộ kỹ năng"
            className="p-1.5 sm:p-2 rounded-xl bg-white/[0.05] hover:bg-white/[0.09] text-zinc-300 hover:text-white border border-white/[0.08] transition-colors cursor-pointer"
          >
            <Sliders className="w-4 h-4" />
          </button>
        </div>
      </header>

      {/* Messages Scroll Area */}
      <div className="flex-1 overflow-y-auto px-3 sm:px-5 py-4">
        <div className="max-w-4xl mx-auto w-full">
          {completedTurns.length === 0 && !isStreaming ? (
            /* Empty State */
            <div className="min-h-[55vh] flex flex-col items-center justify-center text-center p-4 sm:p-8 select-none">
              <div className="w-14 h-14 rounded-2xl bg-gradient-to-tr from-indigo-600/20 to-cyan-500/20 border border-indigo-500/30 flex items-center justify-center text-indigo-400 mb-4 shadow-xl shadow-indigo-500/10">
                <Sparkles className="w-7 h-7 text-cyan-400" />
              </div>
              <h3 className="text-base font-bold text-zinc-100 mb-1.5">
                Bắt đầu trò chuyện cùng {agent?.name || 'Hubbub AI'}
              </h3>
              <p className="text-xs text-zinc-400 max-w-md mb-6 leading-relaxed">
                {agent?.system_prompt.slice(0, 180)}...
              </p>

              {/* Suggestion Chips */}
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-2.5 max-w-lg w-full">
                <button
                  onClick={() => setInput('Giải thích trực quan về cơ chế mượn (Borrowing & Ownership) trong Rust')}
                  className="p-3 bg-[#11141e] hover:bg-[#151926] border border-white/[0.07] hover:border-indigo-500/30 rounded-xl text-xs text-zinc-300 transition-all text-left shadow-xs cursor-pointer group"
                >
                  <span className="font-medium text-zinc-200 group-hover:text-indigo-300 block mb-0.5">
                    💡 Cơ chế Ownership trong Rust
                  </span>
                  <span className="text-[11px] text-zinc-400 block line-clamp-1">
                    Hiểu sâu về bộ nhớ an toàn và borrow checker
                  </span>
                </button>
                <button
                  onClick={() => setInput('Tìm kiếm thông tin mới nhất và tổng hợp báo cáo về Tauri 2')}
                  className="p-3 bg-[#11141e] hover:bg-[#151926] border border-white/[0.07] hover:border-indigo-500/30 rounded-xl text-xs text-zinc-300 transition-all text-left shadow-xs cursor-pointer group"
                >
                  <span className="font-medium text-zinc-200 group-hover:text-cyan-300 block mb-0.5">
                    🌐 Tổng hợp báo cáo về Tauri 2
                  </span>
                  <span className="text-[11px] text-zinc-400 block line-clamp-1">
                    Thu thập tính năng mới và xuất báo cáo markdown
                  </span>
                </button>
              </div>
            </div>
          ) : (
            <>
              {/* Completed Turns */}
              {completedTurns.map((turn, idx) => (
                <div key={turn.id} ref={idx === completedTurns.length - 1 ? activeTurnRef : undefined}>
                  <ChatTurnItem
                    turn={turn}
                    agent={agent}
                    onSaveAsReport={handleSaveAsReport}
                    isStreaming={false}
                  />
                </div>
              ))}

              {/* Streaming Turn (Live Response) */}
              {streamingTurn && (
                <div ref={activeTurnRef}>
                  <ChatTurnItem
                    turn={streamingTurn}
                    agent={agent}
                    onSaveAsReport={handleSaveAsReport}
                    isStreaming={true}
                  />
                </div>
              )}
            </>
          )}

          <div ref={messagesEndRef} />
        </div>
      </div>

      {/* Error Banner */}
      {errorMsg && (
        <div className="mx-3 sm:mx-6 mb-3 p-3 bg-rose-500/10 border border-rose-500/30 rounded-xl flex items-center justify-between text-xs text-rose-300 animate-fade-in">
          <div className="flex items-center gap-2.5 min-w-0">
            <AlertTriangle className="w-4 h-4 shrink-0 text-rose-400" />
            <span className="leading-relaxed break-words">{errorMsg}</span>
          </div>
          <button
            onClick={onOpenSettings}
            className="px-3 py-1 bg-rose-500/20 hover:bg-rose-500/30 text-rose-200 border border-rose-500/30 rounded-lg text-[11px] font-medium transition-colors shrink-0 ml-3 cursor-pointer"
          >
            Cài đặt API Key
          </button>
        </div>
      )}

      {/* Modern Cursor-style Composer */}
      <footer className="p-2 sm:p-4 bg-gradient-to-t from-[#08090d] via-[#08090d]/95 to-transparent shrink-0">
        <div className="max-w-4xl mx-auto w-full">
          <div className="bg-[#10131d]/95 hover:bg-[#121622] border border-white/[0.08] focus-within:border-indigo-500/60 focus-within:ring-2 focus-within:ring-indigo-500/15 rounded-2xl transition-all shadow-2xl shadow-black/60 p-2 sm:p-2.5 flex flex-col gap-1.5 backdrop-blur-xl">
            {/* Top Composer Chip: Active Agent & Skills */}
            <div className="flex items-center justify-between px-2 pt-1 text-[11px] text-zinc-400 select-none">
              <div className="flex items-center gap-2">
                <span className="flex items-center gap-1.5 text-indigo-400 font-medium">
                  <Sparkles className="w-3 h-3 text-cyan-400" />
                  {agent?.name || 'Tác tử'}
                </span>
                <span className="text-zinc-600">•</span>
                <span className="text-zinc-400 font-mono text-[10px]">
                  {agent?.model}
                </span>
              </div>
              {input.length > 0 && (
                <button
                  onClick={() => setInput('')}
                  title="Xóa nội dung nhập"
                  className="text-zinc-400 hover:text-zinc-200 transition-colors p-0.5 cursor-pointer"
                >
                  <Trash2 className="w-3 h-3" />
                </button>
              )}
            </div>

            {/* Input Row */}
            <div className="flex items-end gap-2 px-1">
              <textarea
                ref={textareaRef}
                rows={1}
                value={input}
                onChange={handleTextareaInput}
                onKeyDown={handleKeyDown}
                onCompositionStart={() => setIsComposing(true)}
                onCompositionEnd={() => setIsComposing(false)}
                placeholder={`Nhắn tin với ${agent?.name || 'Hubbub'}... (Enter để gửi, Shift+Enter xuống dòng)`}
                className="flex-1 bg-transparent border-0 resize-none px-1 py-1.5 text-xs text-zinc-100 placeholder:text-zinc-400 focus:outline-hidden min-h-[38px] max-h-[180px] leading-relaxed"
              />

              {isStreaming ? (
                <button
                  onClick={onCancelRun}
                  title="Dừng sinh phản hồi"
                  className="p-2.5 bg-rose-600 hover:bg-rose-500 text-white rounded-xl text-xs font-semibold transition-all flex items-center justify-center shrink-0 shadow-md shadow-rose-600/25 active:scale-95 cursor-pointer"
                >
                  <Square className="w-4 h-4 fill-current" />
                </button>
              ) : (
                <button
                  onClick={handleSend}
                  disabled={!input.trim()}
                  title="Gửi tin nhắn (Enter)"
                  className="p-2.5 bg-gradient-to-tr from-indigo-600 via-indigo-500 to-indigo-600 hover:brightness-110 disabled:opacity-30 text-white rounded-xl text-xs font-semibold transition-all flex items-center justify-center shrink-0 shadow-md shadow-indigo-600/25 active:scale-95 cursor-pointer disabled:cursor-not-allowed inner-top-glow"
                >
                  <Send className="w-4 h-4" />
                </button>
              )}
            </div>
          </div>
        </div>
      </footer>
    </div>
  );
};
