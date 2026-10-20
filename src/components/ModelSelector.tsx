import { useState, useRef, useEffect } from 'react';
import { ChevronDown, Cpu, Check, Plus } from 'lucide-react';

interface ModelSelectorProps {
  currentModel: string;
  onSelectModel: (model: string) => Promise<void>;
  disabled?: boolean;
}

interface ModelOption {
  id: string;
  name: string;
  provider: string;
}

const COMMON_MODELS: ModelOption[] = [
  { id: 'gemini-3.8-flash', name: 'Gemini 3.8 Flash (Khuyên dùng)', provider: 'Google Gemini' },
  { id: 'gemini-2.5-flash', name: 'Gemini 2.5 Flash (15 RPM Free)', provider: 'Google Gemini' },
  { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro', provider: 'Google Gemini' },
  { id: 'gpt-4o-mini', name: 'GPT-4o Mini', provider: 'OpenAI' },
  { id: 'gpt-4o', name: 'GPT-4o', provider: 'OpenAI' },
  { id: 'claude-sonnet-4-20250514', name: 'Claude Sonnet 4', provider: 'Anthropic' },
  { id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet', provider: 'Anthropic' },
  { id: 'llama-3.3-70b-versatile', name: 'Llama 3.3 70B', provider: 'Groq' },
  { id: 'qwen2.5-coder:7b', name: 'Qwen 2.5 Coder 7B (Ollama Cục bộ)', provider: 'Ollama' },
  { id: 'llama3.2', name: 'Llama 3.2 (Cục bộ)', provider: 'Ollama' },
];

export const ModelSelector = ({ currentModel, onSelectModel, disabled = false }: ModelSelectorProps) => {
  const [isOpen, setIsOpen] = useState(false);
  const [customModel, setCustomModel] = useState('');
  const [isChanging, setIsChanging] = useState(false);
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const handleSelect = async (model: string) => {
    if (!model.trim() || disabled || isChanging) return;
    setIsChanging(true);
    try {
      await onSelectModel(model.trim());
      setIsOpen(false);
      setCustomModel('');
    } finally {
      setIsChanging(false);
    }
  };

  return (
    <div className="relative inline-block" ref={dropdownRef}>
      <button
        onClick={() => !disabled && setIsOpen(!isOpen)}
        disabled={disabled || isChanging}
        title="Đổi mô hình LLM chạy tác tử này"
        className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#121622] hover:bg-[#161c2c] border border-white/[0.08] text-[11px] font-mono text-fg-muted transition-colors disabled:opacity-50 cursor-pointer"
      >
        <Cpu className="w-3 h-3 text-primary" />
        <span className="truncate max-w-[120px] sm:max-w-[150px] font-medium">{currentModel}</span>
        <ChevronDown className="w-3 h-3 text-fg-muted" />
      </button>

      {isOpen && (
        <div className="absolute left-0 mt-1.5 w-68 bg-[#0e111a] border border-white/[0.09] rounded-xl shadow-2xl shadow-black/70 z-50 p-2 animate-fade-in backdrop-blur-xl">
          <div className="text-[10px] font-semibold text-fg-muted px-2 py-1 uppercase tracking-wider">
            Chọn mô hình cho Agent
          </div>
          <div className="max-h-60 overflow-y-auto space-y-0.5 my-1 divide-y divide-white/[0.04]">
            {COMMON_MODELS.map((item) => {
              const active = item.id === currentModel;
              return (
                <button
                  key={item.id}
                  onClick={() => handleSelect(item.id)}
                  className={`w-full text-left px-2.5 py-1.5 rounded-lg text-xs flex items-center justify-between transition-colors cursor-pointer ${
                    active ? 'bg-primary-strong/20 text-primary-soft font-medium border border-primary/30' : 'text-fg-muted hover:bg-white/[0.06] border border-transparent'
                  }`}
                >
                  <div className="truncate">
                    <div className="truncate">{item.name}</div>
                    <div className="text-[10px] text-fg-muted font-mono">{item.provider}</div>
                  </div>
                  {active && <Check className="w-3.5 h-3.5 text-primary shrink-0" />}
                </button>
              );
            })}
          </div>

          <div className="pt-2 border-t border-white/[0.06] flex items-center gap-1.5">
            <input
              type="text"
              placeholder="Nhập tên model khác..."
              value={customModel}
              onChange={(e) => setCustomModel(e.target.value)}
              onKeyDown={(e) => e.key === 'Enter' && handleSelect(customModel)}
              className="flex-1 bg-[#090b10] border border-white/[0.08] rounded-lg px-2.5 py-1 text-[11px] text-fg placeholder:text-fg-subtle focus:outline-hidden font-mono focus:border-primary/60"
            />
            <button
              onClick={() => handleSelect(customModel)}
              disabled={!customModel.trim()}
              className="p-1.5 bg-primary hover:bg-primary-strong disabled:opacity-40 text-on-primary rounded-lg text-xs transition-colors cursor-pointer"
            >
              <Plus className="w-3.5 h-3.5" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
};
