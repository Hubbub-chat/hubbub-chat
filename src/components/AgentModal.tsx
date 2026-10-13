import { useState } from 'react';
import { X, Bot, Sparkles, Check, Wrench, Loader2 } from 'lucide-react';
import type { Agent } from '../types';

interface AgentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSaveAgent: (agent: Agent) => Promise<void>;
  existingAgent?: Agent | null;
}

const AVAILABLE_TOOLS = [
  { id: 'web_search', label: 'Tìm kiếm Web (Google / Tavily)' },
  { id: 'web_fetch', label: 'Tải & Đọc nội dung trang Web' },
  { id: 'fs_read', label: 'Đọc tệp tin cục bộ trong Workspace' },
  { id: 'fs_list', label: 'Liệt kê danh mục tệp tin Workspace' },
  { id: 'report_write', label: 'Soạn thảo & Lưu báo cáo Markdown' },
  { id: 'report_read', label: 'Đọc báo cáo đã lưu' },
  { id: 'report_list', label: 'Danh sách báo cáo' },
];

const POPULAR_MODELS = [
  'gemini-3.8-flash',
  'gemini-1.5-pro',
  'gpt-4o-mini',
  'gpt-4o',
  'claude-sonnet-4-20250514',
  'claude-3-5-sonnet',
  'llama-3.3-70b-versatile',
  'qwen2.5-coder:7b',
  'llama3.2',
];

export const AgentModal = ({ isOpen, onClose, onSaveAgent, existingAgent }: AgentModalProps) => {
  const [name, setName] = useState(existingAgent?.name || '');
  const [model, setModel] = useState(existingAgent?.model || 'gemini-3.8-flash');
  const [systemPrompt, setSystemPrompt] = useState(
    existingAgent?.system_prompt ||
      'Bạn là một tác tử AI chuyên biệt. Luôn cung cấp phản hồi chi tiết, có cấu trúc rõ ràng và chính xác theo nhiệm vụ được giao.'
  );
  const [selectedTools, setSelectedTools] = useState<string[]>(
    existingAgent?.tools.builtin || ['web_search', 'report_write']
  );
  const [isSaving, setIsSaving] = useState(false);

  if (!isOpen) return null;

  const toggleTool = (toolId: string) => {
    setSelectedTools((prev) =>
      prev.includes(toolId) ? prev.filter((t) => t !== toolId) : [...prev, toolId]
    );
  };

  const handleSave = async () => {
    if (!name.trim()) {
      alert('Vui lòng nhập tên cho Agent');
      return;
    }

    setIsSaving(true);
    try {
      const slug =
        existingAgent?.id ||
        name
          .toLowerCase()
          .replace(/[^a-z0-9]/g, '_')
          .slice(0, 24) ||
        `agent_${Date.now()}`;

      const newAgent: Agent = {
        id: slug,
        name: name.trim(),
        model: model.trim(),
        system_prompt: systemPrompt.trim(),
        tools: {
          builtin: selectedTools,
          mcp: [],
        },
        budget: existingAgent?.budget || {
          max_steps: 25,
          max_tokens: 200_000,
          max_cost_usd: 0.5,
          timeout_s: 300,
        },
      };

      await onSaveAgent(newAgent);
      onClose();
    } catch (err) {
      alert(`Lỗi lưu Agent: ${String(err)}`);
    } finally {
      setIsSaving(false);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-3 sm:p-4 animate-fade-in select-none">
      <div className="bg-[#0e111a] border border-white/[0.09] rounded-2xl w-full max-w-xl max-h-[85vh] flex flex-col shadow-2xl shadow-black/80 overflow-hidden inner-top-glow">
        {/* Header */}
        <div className="flex items-center justify-between p-4 sm:p-5 border-b border-white/[0.07] bg-[#121622]/80">
          <div className="flex items-center gap-2.5">
            <div className="w-8 h-8 rounded-xl bg-indigo-500/10 border border-indigo-500/20 flex items-center justify-center text-indigo-400 shadow-xs">
              <Bot className="w-4 h-4" />
            </div>
            <div>
              <h2 className="text-sm font-semibold text-zinc-100">
                {existingAgent ? 'Chỉnh sửa Tác tử (Agent)' : 'Tạo Tác tử Mới theo Chức năng'}
              </h2>
              <p className="text-[11px] text-zinc-400">
                Một mô hình có thể dùng để tạo nhiều Agent cho từng nhiệm vụ riêng biệt
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="text-zinc-400 hover:text-zinc-200 p-1.5 rounded-xl hover:bg-white/[0.06] transition-colors cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Form Body */}
        <div className="p-4 sm:p-5 overflow-y-auto space-y-4 text-xs">
          {/* Agent Name */}
          <div>
            <label className="block font-medium text-zinc-200 mb-1">Tên Tác tử (Chức năng nhiệm vụ)</label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Ví dụ: Chuyên gia DevOps, Dịch thuật Đa ngữ, Reviewer Mã Nguồn..."
              className="w-full bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/60 rounded-xl px-3 py-2 text-zinc-100 placeholder:text-zinc-500 focus:outline-hidden"
            />
          </div>

          {/* Model Selection */}
          <div>
            <label className="block font-medium text-zinc-200 mb-1">Mô hình AI vận hành (Model)</label>
            <div className="flex flex-col sm:flex-row gap-2">
              <select
                value={model}
                onChange={(e) => setModel(e.target.value)}
                className="flex-1 bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/60 rounded-xl px-3 py-2 text-zinc-100 focus:outline-hidden font-mono text-xs"
              >
                {POPULAR_MODELS.map((m) => (
                  <option key={m} value={m}>
                    {m}
                  </option>
                ))}
              </select>
              <input
                type="text"
                placeholder="Hoặc tự gõ tên model..."
                value={model}
                onChange={(e) => setModel(e.target.value)}
                className="w-full sm:w-48 bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/60 rounded-xl px-3 py-2 text-zinc-100 placeholder:text-zinc-500 focus:outline-hidden font-mono text-xs"
              />
            </div>
          </div>

          {/* System Prompt */}
          <div>
            <label className="block font-medium text-zinc-200 mb-1">
              Chỉ dẫn Nhiệm vụ & Vai trò (System Prompt)
            </label>
            <textarea
              rows={4}
              value={systemPrompt}
              onChange={(e) => setSystemPrompt(e.target.value)}
              placeholder="Mô tả chi tiết nhiệm vụ, phong cách trả lời và kiến thức chuyên sâu của tác tử này..."
              className="w-full bg-[#080a10] border border-white/[0.08] focus:border-indigo-500/60 rounded-xl px-3 py-2 text-zinc-100 placeholder:text-zinc-500 focus:outline-hidden leading-relaxed resize-y"
            />
          </div>

          {/* Tools Selection */}
          <div>
            <label className="block font-medium text-zinc-200 mb-1.5 flex items-center gap-1.5">
              <Wrench className="w-3.5 h-3.5 text-indigo-400" />
              Công cụ cho phép tác tử sử dụng
            </label>
            <div className="grid grid-cols-1 gap-1.5 p-2 bg-[#080a10] border border-white/[0.06] rounded-xl">
              {AVAILABLE_TOOLS.map((tool) => {
                const checked = selectedTools.includes(tool.id);
                return (
                  <label
                    key={tool.id}
                    onClick={() => toggleTool(tool.id)}
                    className="flex items-center gap-2 p-1.5 hover:bg-white/[0.04] rounded-lg cursor-pointer select-none transition-colors"
                  >
                    <div
                      className={`w-4 h-4 rounded-md border flex items-center justify-center transition-colors ${
                        checked ? 'bg-indigo-600 border-indigo-500 text-white' : 'border-white/[0.12] bg-[#121622]'
                      }`}
                    >
                      {checked && <Check className="w-3 h-3 stroke-[3]" />}
                    </div>
                    <span className="text-zinc-300 font-mono text-[11px] font-semibold">{tool.id}</span>
                    <span className="text-zinc-400 text-[11px] truncate">— {tool.label}</span>
                  </label>
                );
              })}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="p-3.5 sm:p-4 border-t border-white/[0.07] bg-[#121622]/90 flex items-center justify-end gap-2.5">
          <button
            onClick={onClose}
            className="px-3.5 py-1.5 bg-white/[0.05] hover:bg-white/[0.08] text-zinc-300 rounded-xl text-xs font-medium border border-white/[0.06] transition-colors cursor-pointer"
          >
            Hủy
          </button>
          <button
            onClick={handleSave}
            disabled={isSaving || !name.trim()}
            className="px-4 py-1.5 bg-gradient-to-r from-indigo-600 to-indigo-500 hover:from-indigo-500 hover:to-indigo-400 disabled:opacity-40 text-white rounded-xl text-xs font-medium transition-colors flex items-center gap-1.5 cursor-pointer inner-top-glow shadow-xs"
          >
            {isSaving ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : <Sparkles className="w-3.5 h-3.5" />}
            Lưu Tác tử
          </button>
        </div>
      </div>
    </div>
  );
};
