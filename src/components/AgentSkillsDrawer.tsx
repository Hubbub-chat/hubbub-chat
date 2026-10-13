import { useState, useEffect } from 'react';
import {
  X,
  Bot,
  Sparkles,
  Globe,
  FileCode,
  FolderTree,
  FileEdit,
  BookOpen,
  ShieldCheck,
  Zap,
  Check,
  Save,
  RotateCcw,
} from 'lucide-react';
import type { Agent } from '../types';

interface AgentSkillsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  agent: Agent | null;
  onSaveAgent: (agent: Agent) => Promise<void>;
}

export const SKILLS_METADATA = [
  {
    id: 'web_search',
    name: 'Tìm kiếm Web Thời gian thực',
    description: 'Truy vấn thông tin cập nhật trên Internet qua Google & Tavily',
    icon: Globe,
    color: 'text-cyan-400 bg-cyan-400/10 border-cyan-400/20',
  },
  {
    id: 'web_fetch',
    name: 'Đọc Nội dung Trang Web',
    description: 'Tải và trích xuất Markdown từ các bài viết, tài liệu tham khảo URL',
    icon: Globe,
    color: 'text-blue-400 bg-blue-400/10 border-blue-400/20',
  },
  {
    id: 'fs_read',
    name: 'Đọc Tệp tin Workspace',
    description: 'Đọc nội dung tệp tin mã nguồn, ghi chú và tài liệu cục bộ',
    icon: FileCode,
    color: 'text-emerald-400 bg-emerald-400/10 border-emerald-400/20',
  },
  {
    id: 'fs_list',
    name: 'Duyệt Danh mục Thư mục',
    description: 'Khám phá cấu trúc tệp và cây thư mục của dự án',
    icon: FolderTree,
    color: 'text-amber-400 bg-amber-400/10 border-amber-400/20',
  },
  {
    id: 'report_write',
    name: 'Soạn thảo Báo cáo Markdown',
    description: 'Tạo và lưu trữ báo cáo hoàn chỉnh vào thư mục reports/',
    icon: FileEdit,
    color: 'text-indigo-400 bg-indigo-400/10 border-indigo-400/20',
  },
  {
    id: 'report_read',
    name: 'Đọc Báo cáo Workspace',
    description: 'Tra cứu và đọc lại các báo cáo tổng hợp đã xuất bản',
    icon: BookOpen,
    color: 'text-purple-400 bg-purple-400/10 border-purple-400/20',
  },
];

const POPULAR_MODELS = [
  'gemini-3.8-flash',
  'gemini-1.5-pro',
  'claude-sonnet-4-20250514',
  'claude-3-5-sonnet',
  'gpt-4o-mini',
  'gpt-4o',
  'llama-3.3-70b-versatile',
  'qwen2.5-coder:7b',
  'llama3.2',
];

export const AgentSkillsDrawer = ({
  isOpen,
  onClose,
  agent,
  onSaveAgent,
}: AgentSkillsDrawerProps) => {
  const [name, setName] = useState('');
  const [model, setModel] = useState('');
  const [systemPrompt, setSystemPrompt] = useState('');
  const [selectedTools, setSelectedTools] = useState<string[]>([]);
  const [isSaving, setIsSaving] = useState(false);
  const [toastMsg, setToastMsg] = useState<string | null>(null);

  useEffect(() => {
    if (agent) {
      setName(agent.name);
      setModel(agent.model);
      setSystemPrompt(agent.system_prompt);
      setSelectedTools(agent.tools.builtin || []);
    }
  }, [agent]);

  if (!isOpen || !agent) return null;

  const toggleTool = (toolId: string) => {
    setSelectedTools((prev) =>
      prev.includes(toolId) ? prev.filter((t) => t !== toolId) : [...prev, toolId]
    );
  };

  const handleSave = async () => {
    setIsSaving(true);
    try {
      const updatedAgent: Agent = {
        ...agent,
        name: name.trim() || agent.name,
        model: model.trim() || agent.model,
        system_prompt: systemPrompt.trim() || agent.system_prompt,
        tools: {
          ...agent.tools,
          builtin: selectedTools,
        },
      };
      await onSaveAgent(updatedAgent);
      setToastMsg('Đã lưu cấu hình Tác tử!');
      setTimeout(() => setToastMsg(null), 2500);
    } catch (err) {
      alert(`Lỗi lưu cấu hình: ${String(err)}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex justify-end bg-black/60 backdrop-blur-xs transition-opacity animate-fade-in">
      {/* Background click to dismiss */}
      <div className="flex-1" onClick={onClose} />

      {/* Drawer Panel */}
      <aside className="w-[440px] max-w-[92vw] h-full bg-[#0e111a] border-l border-white/[0.08] flex flex-col shadow-2xl shadow-black/80 animate-slide-in-right select-none z-10 inner-top-glow">
        {/* Header */}
        <div className="p-4 border-b border-white/[0.07] flex items-center justify-between bg-[#121622]/80">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-indigo-500/15 border border-indigo-500/30 flex items-center justify-center text-indigo-400 shadow-xs">
              <Bot className="w-5 h-5" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-zinc-100 flex items-center gap-2">
                <span>Cấu hình & Kỹ năng Tác tử</span>
              </h2>
              <p className="text-[11px] text-zinc-400 font-mono">
                ID: {agent.id}
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] rounded-xl transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content Body */}
        <div className="flex-1 overflow-y-auto p-4 sm:p-5 space-y-5 text-xs">
          {toastMsg && (
            <div className="p-2.5 bg-emerald-500/10 border border-emerald-500/30 text-emerald-300 rounded-xl flex items-center gap-2 text-[11px] animate-fade-in">
              <Check className="w-4 h-4 text-emerald-400 shrink-0" />
              <span>{toastMsg}</span>
            </div>
          )}

          {/* Name & Model */}
          <div className="space-y-3 p-3.5 bg-[#121622]/60 border border-white/[0.06] rounded-xl">
            <div>
              <label className="block text-[11px] font-medium text-zinc-300 mb-1">
                Tên Tác tử (Vai trò nhiệm vụ)
              </label>
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                className="w-full bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/80 rounded-xl px-3 py-2 text-zinc-100 placeholder:text-zinc-500 focus:outline-hidden text-xs"
              />
            </div>

            <div>
              <label className="block text-[11px] font-medium text-zinc-300 mb-1">
                Mô hình AI vận hành (Model)
              </label>
              <div className="flex gap-2">
                <select
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  className="flex-1 bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/80 rounded-xl px-3 py-2 text-zinc-100 focus:outline-hidden font-mono text-[11px]"
                >
                  {POPULAR_MODELS.map((m) => (
                    <option key={m} value={m}>
                      {m}
                    </option>
                  ))}
                </select>
                <input
                  type="text"
                  placeholder="Custom model..."
                  value={model}
                  onChange={(e) => setModel(e.target.value)}
                  className="w-32 bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/80 rounded-xl px-2.5 py-1.5 text-zinc-100 placeholder:text-zinc-500 focus:outline-hidden font-mono text-[11px]"
                />
              </div>
            </div>
          </div>

          {/* Skills / Tools Selector */}
          <div>
            <div className="flex items-center justify-between mb-2">
              <label className="text-[11px] font-semibold text-zinc-200 flex items-center gap-1.5">
                <Sparkles className="w-3.5 h-3.5 text-cyan-400" />
                <span>Bộ Kỹ năng & Công cụ cho phép ({selectedTools.length})</span>
              </label>
            </div>

            <div className="space-y-2">
              {SKILLS_METADATA.map((skill) => {
                const Icon = skill.icon;
                const isEnabled = selectedTools.includes(skill.id);
                return (
                  <div
                    key={skill.id}
                    onClick={() => toggleTool(skill.id)}
                    className={`p-3 rounded-xl border transition-all cursor-pointer flex items-start gap-3 select-none ${
                      isEnabled
                        ? 'bg-indigo-950/20 border-indigo-500/40 hover:border-indigo-500/60'
                        : 'bg-[#121622]/40 border-white/5 hover:border-white/10 opacity-75 hover:opacity-100'
                    }`}
                  >
                    <div
                      className={`w-8 h-8 rounded-lg border flex items-center justify-center shrink-0 mt-0.5 transition-colors ${
                        isEnabled
                          ? skill.color
                          : 'bg-zinc-900 border-zinc-800 text-zinc-500'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                    </div>

                    <div className="flex-1 min-w-0">
                      <div className="flex items-center justify-between">
                        <span className={`font-medium text-xs ${isEnabled ? 'text-zinc-100' : 'text-zinc-400'}`}>
                          {skill.name}
                        </span>
                        <div
                          className={`w-4 h-4 rounded border flex items-center justify-center transition-colors ${
                            isEnabled
                              ? 'bg-indigo-600 border-indigo-500 text-white'
                              : 'border-zinc-700 bg-zinc-900'
                          }`}
                        >
                          {isEnabled && <Check className="w-3 h-3 stroke-[3]" />}
                        </div>
                      </div>
                      <p className="text-[11px] text-zinc-400 mt-0.5 leading-relaxed">
                        {skill.description}
                      </p>
                      <span className="inline-block mt-1 font-mono text-[10px] text-zinc-500">
                        tool: {skill.id}
                      </span>
                    </div>
                  </div>
                );
              })}
            </div>
          </div>

          {/* System Prompt Instruction */}
          <div>
            <label className="block text-[11px] font-semibold text-zinc-200 mb-1.5 flex items-center gap-1.5">
              <Zap className="w-3.5 h-3.5 text-amber-400" />
              <span>Chỉ dẫn Chuyên môn & Quy tắc (System Prompt)</span>
            </label>
            <textarea
              rows={6}
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              placeholder="Quy tắc trả lời, phong cách tư duy và phạm vi chuyên sâu của tác tử này..."
              className="w-full bg-[#0a0c12] border border-white/10 focus:border-indigo-500/80 rounded-xl p-3 text-zinc-100 placeholder:text-zinc-600 focus:outline-hidden leading-relaxed text-xs font-sans resize-y"
            />
          </div>

          {/* Security & Sandbox Policy Badges */}
          <div className="p-3.5 bg-[#141824]/40 border border-white/5 rounded-xl space-y-2">
            <span className="text-[11px] font-semibold text-zinc-300 flex items-center gap-1.5">
              <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
              <span>Chính sách Bảo mật & Giới hạn Ngân sách</span>
            </span>
            <div className="grid grid-cols-2 gap-2 text-[11px] text-zinc-400 pt-1">
              <div className="p-2 bg-[#0a0c12] rounded-lg border border-white/5">
                <span className="text-zinc-500 block text-[10px]">Tối đa số bước:</span>
                <span className="font-semibold text-zinc-200">{agent.budget.max_steps} steps</span>
              </div>
              <div className="p-2 bg-[#0a0c12] rounded-lg border border-white/5">
                <span className="text-zinc-500 block text-[10px]">Thời gian tối đa:</span>
                <span className="font-semibold text-zinc-200">{agent.budget.timeout_s}s</span>
              </div>
              <div className="p-2 bg-[#0a0c12] rounded-lg border border-white/5 col-span-2">
                <span className="text-zinc-500 block text-[10px]">Giới hạn Token & Chi phí:</span>
                <span className="font-semibold text-zinc-200 font-mono">
                  {agent.budget.max_tokens.toLocaleString()} tokens • ${agent.budget.max_cost_usd}
                </span>
              </div>
            </div>
          </div>
        </div>

        {/* Footer Actions */}
        <div className="p-3.5 sm:p-4 border-t border-white/[0.07] bg-[#121622]/90 flex items-center justify-between gap-3 shrink-0">
          <button
            onClick={() => {
              setName(agent.name);
              setModel(agent.model);
              setSystemPrompt(agent.system_prompt);
              setSelectedTools(agent.tools.builtin || []);
            }}
            title="Khôi phục trạng thái ban đầu"
            className="px-3 py-2 bg-white/[0.05] hover:bg-white/[0.09] text-zinc-300 rounded-xl text-xs font-medium flex items-center gap-1.5 transition-colors cursor-pointer border border-white/[0.06]"
          >
            <RotateCcw className="w-3.5 h-3.5" />
            <span>Hoàn tác</span>
          </button>

          <div className="flex items-center gap-2">
            <button
              onClick={onClose}
              className="px-3.5 py-2 text-zinc-400 hover:text-zinc-200 text-xs font-medium transition-colors cursor-pointer"
            >
              Đóng
            </button>
            <button
              onClick={handleSave}
              disabled={isSaving}
              className="px-4 py-2 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 disabled:opacity-40 text-white rounded-xl text-xs font-semibold shadow-md shadow-indigo-600/25 flex items-center gap-1.5 transition-all cursor-pointer inner-top-glow"
            >
              <Save className="w-3.5 h-3.5" />
              <span>{isSaving ? 'Đang lưu...' : 'Lưu Tác tử'}</span>
            </button>
          </div>
        </div>
      </aside>
    </div>
  );
};
