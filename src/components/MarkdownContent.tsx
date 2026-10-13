import { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { Copy, Check } from 'lucide-react';

interface MarkdownContentProps {
  content: string;
}

const CodeBlock = ({ language, codeString }: { language: string; codeString: string }) => {
  const [copied, setCopied] = useState(false);

  const handleCopy = () => {
    navigator.clipboard.writeText(codeString);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="my-3 rounded-xl overflow-hidden border border-white/[0.08] bg-[#0a0c12] shadow-md shadow-black/30">
      <div className="bg-[#121520] px-3.5 py-1.5 text-[11px] font-mono text-zinc-400 border-b border-white/[0.06] flex justify-between items-center select-none">
        <span className="text-zinc-300 font-semibold">{language || 'code'}</span>
        <button
          onClick={handleCopy}
          className="flex items-center gap-1.5 px-2 py-0.5 rounded-md hover:bg-white/[0.06] text-zinc-400 hover:text-zinc-200 transition-colors cursor-pointer"
        >
          {copied ? (
            <>
              <Check className="w-3 h-3 text-emerald-400" />
              <span className="text-[10px] text-emerald-400">Đã sao chép</span>
            </>
          ) : (
            <>
              <Copy className="w-3 h-3" />
              <span className="text-[10px]">Sao chép</span>
            </>
          )}
        </button>
      </div>
      <pre className="p-3.5 text-xs font-mono overflow-x-auto text-emerald-300/90 leading-relaxed bg-[#07090e]">
        <code>{codeString}</code>
      </pre>
    </div>
  );
};

export const MarkdownContent = ({ content }: MarkdownContentProps) => {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={{
        h1: ({ children }) => (
          <h1 className="text-base font-bold text-zinc-100 mt-4 mb-2 pb-1.5 border-b border-white/[0.08]">
            {children}
          </h1>
        ),
        h2: ({ children }) => (
          <h2 className="text-sm font-semibold text-zinc-100 mt-3.5 mb-1.5">{children}</h2>
        ),
        h3: ({ children }) => (
          <h3 className="text-xs font-semibold text-zinc-200 mt-2.5 mb-1">{children}</h3>
        ),
        p: ({ children }) => <p className="mb-2.5 leading-relaxed text-zinc-200 font-normal">{children}</p>,
        ul: ({ children }) => (
          <ul className="list-disc pl-5 mb-2.5 space-y-1 text-zinc-200">{children}</ul>
        ),
        ol: ({ children }) => (
          <ol className="list-decimal pl-5 mb-2.5 space-y-1 text-zinc-200">{children}</ol>
        ),
        li: ({ children }) => <li className="leading-relaxed">{children}</li>,
        blockquote: ({ children }) => (
          <blockquote className="border-l-2 border-indigo-500/80 pl-3.5 py-1.5 italic text-zinc-400 my-2.5 bg-white/[0.02] rounded-r-lg">
            {children}
          </blockquote>
        ),
        code: ({ className, children, ...props }) => {
          const match = /language-(\w+)/.exec(className || '');
          const isInline = !match && !String(children).includes('\n');
          if (isInline) {
            return (
              <code
                className="bg-white/[0.06] text-amber-300 px-1.5 py-0.5 rounded text-[12px] font-mono border border-white/[0.08]"
                {...props}
              >
                {children}
              </code>
            );
          }
          return <CodeBlock language={match ? match[1] : ''} codeString={String(children).replace(/\n$/, '')} />;
        },
        table: ({ children }) => (
          <div className="my-3 overflow-x-auto rounded-xl border border-white/[0.08]">
            <table className="min-w-full divide-y divide-white/[0.08] text-xs text-zinc-200">
              {children}
            </table>
          </div>
        ),
        th: ({ children }) => (
          <th className="bg-[#121520] px-3.5 py-2 text-left font-semibold text-zinc-300 border-b border-white/[0.08]">
            {children}
          </th>
        ),
        td: ({ children }) => (
          <td className="px-3.5 py-2 border-b border-white/[0.04] bg-[#0c0e16]/40">{children}</td>
        ),
        a: ({ href, children }) => (
          <a
            href={href}
            target="_blank"
            rel="noopener noreferrer"
            className="text-indigo-400 hover:text-indigo-300 underline underline-offset-2 transition-colors"
          >
            {children}
          </a>
        ),
      }}
    >
      {content}
    </ReactMarkdown>
  );
};
