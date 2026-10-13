import { useState } from 'react';
import { CheckCircle2, XCircle } from 'lucide-react';
import type { AuditLog } from '../types';

interface AuditLogsTableProps {
  auditLogs: AuditLog[];
}

export const AuditLogsTable = ({ auditLogs }: AuditLogsTableProps) => {
  const [auditFilter, setAuditFilter] = useState<'all' | 'allow' | 'deny'>('all');

  const filteredLogs = auditLogs.filter((log) => {
    if (auditFilter === 'allow') return log.decision.toLowerCase() === 'allow';
    if (auditFilter === 'deny') return log.decision.toLowerCase() === 'deny';
    return true;
  });

  return (
    <div className="flex-1 flex flex-col overflow-hidden p-4 sm:p-6 max-w-5xl mx-auto w-full">
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-3 mb-4">
        <div>
          <h2 className="text-sm font-semibold text-zinc-100">Nhật ký kiểm toán bảo mật (Audit Logs)</h2>
          <p className="text-xs text-zinc-400">
            Ghi nhận mọi lượt thực thi công cụ (Tool execution) của các Agent, gồm cả hành động được cho phép và bị từ chối.
          </p>
        </div>

        <div className="flex items-center gap-1 p-1 bg-[#10131d] rounded-xl border border-white/[0.07] self-start sm:self-auto">
          <button
            onClick={() => setAuditFilter('all')}
            className={`px-2.5 py-1 text-xs rounded-lg font-medium transition-colors cursor-pointer ${
              auditFilter === 'all' ? 'bg-[#181d2c] text-zinc-100 shadow-xs' : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            Tất cả ({auditLogs.length})
          </button>
          <button
            onClick={() => setAuditFilter('allow')}
            className={`px-2.5 py-1 text-xs rounded-lg font-medium transition-colors cursor-pointer ${
              auditFilter === 'allow'
                ? 'bg-emerald-950/80 text-emerald-300 border border-emerald-800/60'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            Cho phép
          </button>
          <button
            onClick={() => setAuditFilter('deny')}
            className={`px-2.5 py-1 text-xs rounded-lg font-medium transition-colors cursor-pointer ${
              auditFilter === 'deny'
                ? 'bg-rose-950/80 text-rose-300 border border-rose-800/60'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            Từ chối
          </button>
        </div>
      </div>

      <div className="flex-1 bg-[#0c0e16] border border-white/[0.07] rounded-2xl overflow-hidden flex flex-col shadow-lg shadow-black/30">
        <div className="overflow-y-auto flex-1 divide-y divide-white/[0.05]">
          {filteredLogs.length === 0 ? (
            <div className="p-8 text-center text-xs text-zinc-400">Chưa có bản ghi kiểm toán nào phù hợp.</div>
          ) : (
            filteredLogs.map((log) => {
              const isDeny = log.decision.toLowerCase() === 'deny';
              return (
                <div key={log.id} className="p-3.5 hover:bg-zinc-850/40 transition-colors flex items-start gap-3">
                  {isDeny ? (
                    <XCircle className="w-4 h-4 text-rose-400 shrink-0 mt-0.5" />
                  ) : (
                    <CheckCircle2 className="w-4 h-4 text-emerald-400 shrink-0 mt-0.5" />
                  )}
                  <div className="flex-1 min-w-0 space-y-1">
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-xs font-semibold text-zinc-200">{log.tool_name}</span>
                        <span
                          className={`text-[10px] px-2 py-0.5 rounded-full font-medium ${
                            isDeny
                              ? 'bg-rose-950/80 text-rose-300 border border-rose-800/60'
                              : 'bg-emerald-950/80 text-emerald-300 border border-emerald-800/60'
                          }`}
                        >
                          {isDeny ? 'Bị từ chối' : 'Cho phép'}
                        </span>
                      </div>
                      <span className="font-mono text-[10px] text-zinc-400">
                        {new Date(log.timestamp).toLocaleString('vi-VN')}
                      </span>
                    </div>

                    <div className="font-mono text-[11px] text-zinc-400 bg-zinc-950/60 p-2 rounded border border-zinc-850 truncate">
                      <span className="text-zinc-400">args: </span>
                      {log.args_digest}
                    </div>

                    {log.result_digest && (
                      <div className="text-[11px] text-zinc-400 flex items-center gap-1.5 pl-1">
                        <span className="text-zinc-400">Kết quả:</span>
                        <span className="text-zinc-300 truncate">{log.result_digest}</span>
                      </div>
                    )}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
