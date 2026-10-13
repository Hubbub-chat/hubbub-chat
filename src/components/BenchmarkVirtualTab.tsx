import { useRef, useMemo } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';
import { Layers } from 'lucide-react';

interface BenchmarkMessage {
  id: number;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
}

export function BenchmarkVirtualTab() {
  const parentRef = useRef<HTMLDivElement>(null);
  const items: BenchmarkMessage[] = useMemo(() => {
    return Array.from({ length: 1000 }, (_, i) => ({
      id: i + 1,
      role: (i % 2 === 0 ? 'user' : 'assistant') as 'user' | 'assistant',
      content: `Tin nhắn mẫu #${i + 1}: Kiểm tra hiệu năng ảo hóa 1.000 tin nhắn với @tanstack/react-virtual.`,
      timestamp: '14:30',
    }));
  }, []);

  const rowVirtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 64,
    overscan: 5,
  });

  return (
    <div className="flex-1 p-8 flex flex-col overflow-hidden max-w-4xl mx-auto w-full">
      <h2 className="text-2xl font-bold tracking-tight text-zinc-100 flex items-center gap-2 mb-4 shrink-0">
        <Layers className="w-6 h-6 text-sky-400" />
        Benchmark Ảo Hóa 1.000 Tin Nhắn
      </h2>
      <div
        ref={parentRef}
        className="flex-1 overflow-y-auto bg-zinc-950 border border-zinc-800 rounded-xl p-4"
      >
        <div
          style={{
            height: `${rowVirtualizer.getTotalSize()}px`,
            width: '100%',
            position: 'relative',
          }}
        >
          {rowVirtualizer.getVirtualItems().map((virtualRow) => {
            const item = items[virtualRow.index];
            return (
              <div
                key={virtualRow.index}
                style={{
                  position: 'absolute',
                  top: 0,
                  left: 0,
                  width: '100%',
                  height: `${virtualRow.size}px`,
                  transform: `translateY(${virtualRow.start}px)`,
                }}
                className="py-1"
              >
                <div className="p-2.5 bg-zinc-900 border border-zinc-800/80 rounded-lg text-xs flex justify-between">
                  <span className="font-semibold text-indigo-400">{item.role}:</span>
                  <span className="text-zinc-300 flex-1 px-3 truncate">{item.content}</span>
                  <span className="text-zinc-500 font-mono text-[10px]">#{item.id}</span>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
