import React, { useEffect, useRef, useState } from 'react';
import { Editor, rootCtx, defaultValueCtx } from '@milkdown/core';
import { gfm } from '@milkdown/preset-gfm';
import { nord } from '@milkdown/theme-nord';
import { listener, listenerCtx } from '@milkdown/plugin-listener';

interface MilkdownEditorProps {
  initialContent?: string;
  onChange?: (markdown: string) => void;
}

export const MilkdownEditor: React.FC<MilkdownEditorProps> = ({
  initialContent = '# Tiêu đề Báo Cáo Nghiên Cứu\n\nĐây là trình soạn thảo **Milkdown WYSIWYG** trên nền **ProseMirror**.\n\n### Kiểm tra gõ tiếng Việt:\n- Hãy gõ: *Trường Sa, Hoàng Sa là của Việt Nam.*\n- Thử sửa dấu: *tiếng Việt*, *nghiên cứu*, *ứng dụng*.\n\n| Tính năng | Trạng thái | Đánh giá |\n| :--- | :---: | :--- |\n| GFM Tables | ✅ | Hỗ trợ chuẩn |\n| Code Blocks | ✅ | Hỗ trợ syntax |\n| Tiếng Việt | 🧪 | Đang kiểm thử |\n\n```rust\nfn main() {\n    println!("Xin chào từ Hubbub!");\n}\n```\n',
  onChange,
}) => {
  const editorRef = useRef<HTMLDivElement>(null);
  const [isReady, setIsReady] = useState(false);
  const [content, setContent] = useState(initialContent);
  const onChangeRef = useRef(onChange);
  useEffect(() => {
    onChangeRef.current = onChange;
  }, [onChange]);
  const initialContentRef = useRef(initialContent);

  useEffect(() => {
    if (!editorRef.current) return;

    let destroyed = false;
    let editorInstance: Editor | null = null;

    Editor.make()
      .config((ctx) => {
        ctx.set(rootCtx, editorRef.current);
        ctx.set(defaultValueCtx, initialContentRef.current);
        ctx.get(listenerCtx).markdownUpdated((_ctx, markdown) => {
          if (!destroyed) {
            setContent(markdown);
            onChangeRef.current?.(markdown);
          }
        });
      })
      .config(nord)
      .use(gfm)
      .use(listener)
      .create()
      .then((editor) => {
        if (!destroyed) {
          editorInstance = editor;
          setIsReady(true);
        } else {
          editor.destroy();
        }
      })
      .catch((err) => {
        console.error('Failed to init Milkdown editor:', err);
      });

    return () => {
      destroyed = true;
      if (editorInstance) {
        (editorInstance as Editor).destroy();
      }
    };
  }, []);

  return (
    <div className="flex flex-col h-full bg-zinc-900 border border-zinc-800 rounded-lg overflow-hidden">
      <div className="flex items-center justify-between px-4 py-2 bg-zinc-950 border-b border-zinc-800 text-xs text-zinc-400">
        <div className="flex items-center gap-2">
          <span className={`w-2 h-2 rounded-full ${isReady ? 'bg-emerald-500' : 'bg-amber-500 animate-pulse'}`} />
          <span>Milkdown Editor (ProseMirror Engine) {isReady ? '— Đã sẵn sàng' : '— Đang nạp...'}</span>
        </div>
        <div className="text-zinc-500">
          Độ dài: {content.length} ký tự
        </div>
      </div>
      <div
        ref={editorRef}
        className="flex-1 p-6 overflow-y-auto milkdown-container focus:outline-none text-zinc-200"
      />
    </div>
  );
};
