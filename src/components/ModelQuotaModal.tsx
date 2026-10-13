import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  X,
  Gauge,
  Zap,
  RefreshCw,
  Coins,
  Cpu,
  Layers,
} from 'lucide-react';
import type { ModelUsageStat } from '../types';

interface ModelQuotaModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const ModelQuotaModal = ({ isOpen, onClose }: ModelQuotaModalProps) => {
  const [stats, setStats] = useState<ModelUsageStat[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const fetchStats = useCallback(async () => {
    setIsLoading(true);
    try {
      const data = await invoke<ModelUsageStat[]>('get_model_usage_stats');
      setStats(data);
    } catch (err) {
      console.error('Failed to fetch model quota stats:', err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (isOpen) {
      fetchStats();
    }
  }, [isOpen, fetchStats]);

  if (!isOpen) return null;

  const totalTokens = stats.reduce((acc, s) => acc + s.total_tokens, 0);
  const totalRequestsToday = stats.reduce((acc, s) => acc + s.requests_today, 0);
  const totalCost = stats.reduce((acc, s) => acc + s.total_cost_usd, 0);

  const getProviderBadge = (provider: string) => {
    switch (provider) {
      case 'gemini':
        return { label: 'Google Gemini', color: 'bg-cyan-500/10 text-cyan-300 border-cyan-500/30' };
      case 'anthropic':
        return { label: 'Anthropic Claude', color: 'bg-amber-500/10 text-amber-300 border-amber-500/30' };
      case 'openai':
        return { label: 'OpenAI', color: 'bg-emerald-500/10 text-emerald-300 border-emerald-500/30' };
      case 'groq':
        return { label: 'Groq Cloud', color: 'bg-orange-500/10 text-orange-300 border-orange-500/30' };
      case 'ollama':
        return { label: 'Ollama Local', color: 'bg-purple-500/10 text-purple-300 border-purple-500/30' };
      default:
        return { label: provider, color: 'bg-zinc-800 text-zinc-300 border-zinc-700' };
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/75 backdrop-blur-xs p-3 sm:p-4 animate-fade-in select-none">
      <div className="bg-[#0e111a] border border-white/[0.09] rounded-2xl w-full max-w-xl md:max-w-3xl max-h-[85vh] flex flex-col shadow-2xl shadow-black/80 overflow-hidden inner-top-glow">
        {/* Header */}
        <div className="p-4 sm:p-4.5 border-b border-white/[0.07] bg-[#121622]/80 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-9 h-9 rounded-xl bg-gradient-to-tr from-cyan-600 to-indigo-600 p-0.5 shadow-md shadow-cyan-600/20 shrink-0">
              <div className="w-full h-full bg-[#0a0c12] rounded-[10px] flex items-center justify-center text-cyan-400">
                <Gauge className="w-5 h-5" />
              </div>
            </div>
            <div>
              <h2 className="text-sm sm:text-base font-semibold text-zinc-100 flex items-center gap-2">
                <span>Theo dõi Hạn mức & Mức sử dụng Model</span>
              </h2>
              <p className="text-[11px] text-zinc-400">
                Giám sát lưu lượng gọi API, tiến trình hạn mức ngày (RPD) và chi phí ước tính
              </p>
            </div>
          </div>
          <div className="flex items-center gap-1.5 sm:gap-2 shrink-0">
            <button
              onClick={fetchStats}
              disabled={isLoading}
              title="Cập nhật dữ liệu mới nhất"
              className="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] rounded-xl transition-colors cursor-pointer"
            >
              <RefreshCw className={`w-4 h-4 ${isLoading ? 'animate-spin' : ''}`} />
            </button>
            <button
              onClick={onClose}
              className="p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-white/[0.06] rounded-xl transition-colors cursor-pointer"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Global Overview Metrics */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-2.5 sm:gap-3 p-3.5 sm:p-4 bg-[#0a0c12]/60 border-b border-white/[0.06]">
          <div className="p-3 bg-[#131722] rounded-xl border border-white/[0.06] flex items-center gap-3 inner-top-glow">
            <div className="w-8 h-8 rounded-lg bg-indigo-500/10 text-indigo-400 flex items-center justify-center shrink-0">
              <Cpu className="w-4 h-4" />
            </div>
            <div>
              <span className="text-[10px] text-zinc-400 block font-medium">Tổng Tokens</span>
              <span className="text-sm font-bold text-zinc-100 font-mono">
                {totalTokens.toLocaleString()}
              </span>
            </div>
          </div>

          <div className="p-3 bg-[#131722] rounded-xl border border-white/[0.06] flex items-center gap-3 inner-top-glow">
            <div className="w-8 h-8 rounded-lg bg-cyan-500/10 text-cyan-400 flex items-center justify-center shrink-0">
              <Zap className="w-4 h-4" />
            </div>
            <div>
              <span className="text-[10px] text-zinc-400 block font-medium">Lượt gọi Hôm nay</span>
              <span className="text-sm font-bold text-zinc-100 font-mono">
                {totalRequestsToday} requests
              </span>
            </div>
          </div>

          <div className="p-3 bg-[#131722] rounded-xl border border-white/[0.06] flex items-center gap-3 inner-top-glow">
            <div className="w-8 h-8 rounded-lg bg-emerald-500/10 text-emerald-400 flex items-center justify-center shrink-0">
              <Coins className="w-4 h-4" />
            </div>
            <div>
              <span className="text-[10px] text-zinc-400 block font-medium">Chi phí ước tính</span>
              <span className="text-sm font-bold text-emerald-400 font-mono">
                ${totalCost.toFixed(4)}
              </span>
            </div>
          </div>
        </div>

        {/* Models List */}
        <div className="flex-1 overflow-y-auto p-5 space-y-3.5">
          {stats.map((stat) => {
            const providerInfo = getProviderBadge(stat.provider);
            const rpdPercent =
              stat.rpd_limit > 0
                ? Math.min(Math.round((stat.requests_today / stat.rpd_limit) * 100), 100)
                : 0;

            const isHighUsage = rpdPercent >= 80;

            return (
              <div
                key={stat.model}
                className="p-4 bg-[#121622]/50 hover:bg-[#121622]/80 border border-white/5 hover:border-white/10 rounded-xl transition-all"
              >
                <div className="flex items-center justify-between mb-3">
                  <div className="flex items-center gap-2.5">
                    <span className="text-xs font-bold text-zinc-100 font-mono">
                      {stat.model}
                    </span>
                    <span className={`text-[10px] px-2 py-0.5 rounded-full border font-medium ${providerInfo.color}`}>
                      {providerInfo.label}
                    </span>
                    {stat.is_free_tier && (
                      <span className="text-[9px] px-1.5 py-0.5 rounded bg-cyan-950/40 text-cyan-400 border border-cyan-500/20 font-medium">
                        Free Tier
                      </span>
                    )}
                  </div>

                  <div className="flex items-center gap-2 text-[11px]">
                    <span className="text-zinc-400 font-mono">
                      Tốc độ: <strong className="text-zinc-200">{stat.rpm_limit > 0 ? `${stat.rpm_limit} RPM` : 'Vô hạn'}</strong>
                    </span>
                    <span className="w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-xs shadow-emerald-400/50" />
                  </div>
                </div>

                {/* Progress Bar for Daily Requests Quota */}
                {stat.rpd_limit > 0 ? (
                  <div className="space-y-1.5 mb-3">
                    <div className="flex justify-between text-[11px]">
                      <span className="text-zinc-400">
                        Hạn mức ngày: <strong className="text-zinc-200 font-mono">{stat.requests_today}</strong> / {stat.rpd_limit.toLocaleString()} RPD
                      </span>
                      <span className={`font-mono font-medium ${isHighUsage ? 'text-amber-400' : 'text-zinc-400'}`}>
                        {rpdPercent}%
                      </span>
                    </div>
                    <div className="w-full h-2 bg-zinc-800/80 rounded-full overflow-hidden">
                      <div
                        className={`h-full rounded-full transition-all duration-500 ${
                          isHighUsage ? 'bg-gradient-to-r from-amber-500 to-rose-500' : 'bg-gradient-to-r from-indigo-500 to-cyan-400'
                        }`}
                        style={{ width: `${Math.max(rpdPercent, 1)}%` }}
                      />
                    </div>
                  </div>
                ) : (
                  <div className="text-[11px] text-zinc-400 mb-3 flex items-center gap-1.5">
                    <Layers className="w-3.5 h-3.5 text-zinc-500" />
                    <span>Mô hình chạy Offline / Local không giới hạn hạn mức API.</span>
                  </div>
                )}

                {/* Token Stats Breakdown */}
                <div className="grid grid-cols-4 gap-2 text-[10px] text-zinc-400 pt-2 border-t border-white/5 font-mono">
                  <div>
                    <span className="text-zinc-500 block">Prompt:</span>
                    <span className="text-zinc-300 font-medium">{stat.total_prompt_tokens.toLocaleString()}</span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Completion:</span>
                    <span className="text-zinc-300 font-medium">{stat.total_completion_tokens.toLocaleString()}</span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Hôm nay:</span>
                    <span className="text-zinc-300 font-medium">{stat.tokens_today.toLocaleString()}</span>
                  </div>
                  <div>
                    <span className="text-zinc-500 block">Tổng lượt chạy:</span>
                    <span className="text-zinc-300 font-medium">{stat.total_runs} runs</span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>

        {/* Footer */}
        <div className="p-3.5 sm:p-4 border-t border-white/[0.07] bg-[#121622]/90 flex items-center justify-between text-xs text-zinc-400">
          <span className="text-[11px] truncate max-w-sm">
            Hạn mức được tự động đối chiếu theo bảng định mức kỹ thuật của Google, Anthropic, OpenAI & Groq.
          </span>
          <button
            onClick={onClose}
            className="px-4 py-1.5 bg-white/[0.06] hover:bg-white/[0.1] text-zinc-200 rounded-xl text-xs font-medium transition-colors cursor-pointer"
          >
            Đóng
          </button>
        </div>
      </div>
    </div>
  );
};
