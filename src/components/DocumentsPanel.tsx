import { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
  FileText,
  Search,
  X,
  RefreshCw,
  ExternalLink,
  ArrowLeft,
  FolderOpen,
  Sparkles,
} from 'lucide-react';
import { MarkdownContent } from './MarkdownContent';
import { useWorkspaceDocuments } from '../hooks/useWorkspaceDocuments';

interface DocumentsPanelProps {
  /** Run ids belonging to the active conversation, used for the "this chat" filter. */
  conversationRunIds: Set<string>;
  refreshKey: number;
  onClose: () => void;
  onOpenInWorkspace: (path: string) => void;
}

type Scope = 'conversation' | 'all';

const SCOPES: [Scope, string][] = [
  ['conversation', 'Cuộc trò chuyện này'],
  ['all', 'Tất cả'],
];

const reportFilename = (path: string) => path.replace(/^reports[\\/]/, '');

export const DocumentsPanel = ({
  conversationRunIds,
  refreshKey,
  onClose,
  onOpenInWorkspace,
}: DocumentsPanelProps) => {
  const [scope, setScope] = useState<Scope>('conversation');
  const [query, setQuery] = useState('');
  const [previewPath, setPreviewPath] = useState<string | null>(null);
  const [previewContent, setPreviewContent] = useState('');

  const { documents, isLoading, reload } = useWorkspaceDocuments(query, refreshKey);

  const visibleDocs = useMemo(
    () =>
      scope === 'all'
        ? documents
        : documents.filter((d) => d.run_id && conversationRunIds.has(d.run_id)),
    [documents, scope, conversationRunIds]
  );

  // Re-read the open preview when documents change (e.g. an agent rewrote it).
  useEffect(() => {
    if (!previewPath) return;
    let cancelled = false;
    invoke<string>('read_report', { filename: reportFilename(previewPath) })
      .then((content) => !cancelled && setPreviewContent(content))
      .catch((err) => !cancelled && setPreviewContent('# Không thể đọc tệp\n\n' + String(err)));
    return () => {
      cancelled = true;
    };
  }, [previewPath, refreshKey]);

  const previewDoc = documents.find((d) => d.path === previewPath);

  return (
    <>
    <div
      className="xl:hidden fixed inset-0 z-30 bg-black/50"
      onClick={onClose}
      aria-hidden="true"
    />
    <aside className="w-[360px] xl:w-80 max-w-[92vw] h-full shrink-0 flex flex-col bg-[#0a0c12] border-l border-white/[0.06] max-xl:fixed max-xl:right-0 max-xl:top-0 max-xl:z-40 max-xl:shadow-2xl max-xl:shadow-black/60 animate-slide-in-right">
      <header className="h-14 sm:h-16 px-3 flex items-center justify-between border-b border-white/[0.06] shrink-0">
        {previewPath ? (
          <button
            onClick={() => setPreviewPath(null)}
            className="flex items-center gap-1.5 text-xs font-medium text-primary hover:text-primary-soft cursor-pointer"
          >
            <ArrowLeft className="w-4 h-4" />
            Tài liệu
          </button>
        ) : (
          <div className="flex items-center gap-2 text-sm font-semibold text-fg">
            <FolderOpen className="w-4 h-4 text-primary" />
            Tài liệu
            <span className="text-[10px] font-mono text-fg-subtle">{visibleDocs.length}</span>
          </div>
        )}
        <div className="flex items-center gap-1">
          {!previewPath && (
            <button
              onClick={reload}
              title="Làm mới"
              className="p-1.5 rounded-lg text-fg-muted hover:text-fg hover:bg-white/[0.06] cursor-pointer"
            >
              <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? 'animate-spin' : ''}`} />
            </button>
          )}
          <button
            onClick={onClose}
            title="Đóng khu vực tài liệu"
            className="p-1.5 rounded-lg text-fg-muted hover:text-fg hover:bg-white/[0.06] cursor-pointer"
          >
            <X className="w-4 h-4" />
          </button>
        </div>
      </header>

      {previewPath ? (
        <div className="flex-1 min-h-0 flex flex-col">
          <div className="px-3 py-2 border-b border-white/[0.06] flex items-center justify-between gap-2">
            <span className="truncate text-xs font-medium text-fg">
              {previewDoc?.title || previewPath}
            </span>
            <button
              onClick={() => onOpenInWorkspace(previewPath)}
              className="shrink-0 flex items-center gap-1 px-2 py-1 rounded-lg bg-primary text-on-primary text-[11px] font-semibold hover:bg-primary-strong cursor-pointer"
            >
              <ExternalLink className="w-3 h-3" />
              Mở & chỉnh sửa
            </button>
          </div>
          <div className="flex-1 overflow-y-auto p-4 select-text text-sm">
            <MarkdownContent content={previewContent} />
          </div>
        </div>
      ) : (
        <>
          <div className="p-3 space-y-2.5 shrink-0">
            <div className="relative flex items-center">
              <Search className="w-3.5 h-3.5 text-fg-muted absolute left-2.5 pointer-events-none" />
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Tìm trong tài liệu..."
                className="w-full bg-[#10131c] border border-white/[0.07] focus:border-primary/60 focus:ring-1 focus:ring-primary/20 rounded-xl pl-8 pr-3 py-1.5 text-xs text-fg placeholder:text-fg-subtle focus:outline-hidden"
              />
            </div>
            <div className="flex p-0.5 bg-[#11141e] rounded-lg border border-white/[0.07] text-[11px]">
              {SCOPES.map(([id, label]) => (
                <button
                  key={id}
                  onClick={() => setScope(id)}
                  className={`flex-1 py-1 rounded-md font-medium transition-colors cursor-pointer ${
                    scope === id ? 'bg-primary text-on-primary' : 'text-fg-muted hover:text-fg'
                  }`}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>

          <div className="flex-1 overflow-y-auto px-2 pb-3 space-y-1 min-h-0">
            {visibleDocs.length === 0 ? (
              <div className="p-6 text-center text-xs text-fg-muted space-y-2">
                <Sparkles className="w-7 h-7 text-primary/70 mx-auto" />
                <p>
                  {scope === 'conversation'
                    ? 'Cuộc trò chuyện này chưa tạo tài liệu nào.'
                    : 'Workspace chưa có tài liệu.'}
                </p>
                <p className="text-[11px] text-fg-subtle">
                  Nhờ tác tử viết báo cáo, tài liệu sẽ xuất hiện ở đây ngay khi hoàn tất.
                </p>
              </div>
            ) : (
              visibleDocs.map((doc) => (
                <button
                  key={doc.id}
                  onClick={() => setPreviewPath(doc.path)}
                  className="w-full text-left p-2.5 rounded-xl text-xs border border-transparent hover:border-white/[0.08] hover:bg-white/[0.04] flex items-start gap-2.5 cursor-pointer group transition-colors"
                >
                  <FileText className="w-4 h-4 mt-0.5 shrink-0 text-primary" />
                  <div className="min-w-0 flex-1">
                    <div className="truncate font-medium text-fg group-hover:text-primary-soft">
                      {doc.title || doc.path}
                    </div>
                    <div className="text-[10px] text-fg-subtle font-mono mt-0.5 flex gap-2">
                      <span>{(doc.size_bytes / 1024).toFixed(1)} KB</span>
                      <span>•</span>
                      <span>{new Date(doc.updated_at).toLocaleDateString('vi-VN')}</span>
                    </div>
                  </div>
                </button>
              ))
            )}
          </div>
        </>
      )}
    </aside>
    </>
  );
};
