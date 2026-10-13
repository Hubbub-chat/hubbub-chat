import React from 'react'
import { ShieldCheck } from 'lucide-react'
import { useLanguage } from '../context/LanguageContext.tsx'
import { Logo } from './Logo.tsx'

export const Footer: React.FC = () => {
  const { t } = useLanguage()

  return (
    <footer className="border-t border-slate-200 dark:border-white/[0.07] bg-slate-50/80 dark:bg-[#050609] pt-14 sm:pt-16 pb-10 sm:pb-12 transition-colors duration-200">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-8 sm:gap-10 pb-10 sm:pb-12 border-b border-slate-200 dark:border-white/[0.05]">
          {/* Brand Col */}
          <div className="sm:col-span-2 space-y-4">
            <div className="flex items-center gap-3">
              <Logo className="w-9 h-9 sm:w-10 sm:h-10 shrink-0" />
              <div>
                <span className="text-lg sm:text-xl font-extrabold tracking-tight text-slate-900 dark:text-white block">
                  Hubbub <span className="text-indigo-600 dark:text-indigo-400 font-semibold">Chat</span>
                </span>
                <span className="text-[11px] text-slate-500 dark:text-slate-400">{t('nav.subtitle')}</span>
              </div>
            </div>

            <p className="text-xs text-slate-600 dark:text-slate-400 leading-relaxed max-w-sm">
              {t('footer.tagline')}
            </p>

            <div className="pt-1 flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 font-mono">
              <ShieldCheck className="w-4 h-4 text-emerald-600 dark:text-emerald-400 shrink-0" />
              <span>{t('footer.zeroTelemetry')}</span>
            </div>
          </div>

          {/* Links Column 1: Product */}
          <div className="space-y-3">
            <div className="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-slate-300">
              {t('footer.colProduct')}
            </div>
            <ul className="space-y-2 text-xs text-slate-600 dark:text-slate-400">
              <li>
                <a href="#agents" className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors">
                  {t('nav.agents')}
                </a>
              </li>
              <li>
                <a href="#features" className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors">
                  {t('nav.features')}
                </a>
              </li>
              <li>
                <a href="#security" className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors">
                  {t('nav.security')}
                </a>
              </li>
              <li>
                <a href="#architecture" className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors">
                  {t('nav.architecture')}
                </a>
              </li>
              <li>
                <a href="#download" className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors">
                  {t('nav.download')}
                </a>
              </li>
            </ul>
          </div>

          {/* Links Column 2: Architecture & Security */}
          <div className="space-y-3">
            <div className="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-slate-300">
              {t('footer.colSecurity')}
            </div>
            <ul className="space-y-2 text-xs text-slate-600 dark:text-slate-400">
              <li>
                <span className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors cursor-pointer">
                  PathGuard Sandboxing
                </span>
              </li>
              <li>
                <span className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors cursor-pointer">
                  UrlGuard & SSRF Blocker
                </span>
              </li>
              <li>
                <span className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors cursor-pointer">
                  SQLite FTS5 Local Storage
                </span>
              </li>
              <li>
                <span className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors cursor-pointer">
                  Approval Manager (HITL)
                </span>
              </li>
            </ul>
          </div>

          {/* Links Column 3: Community & Source */}
          <div className="space-y-3">
            <div className="text-xs font-bold uppercase tracking-wider text-slate-900 dark:text-slate-300">
              {t('footer.colEcosystem')}
            </div>
            <ul className="space-y-2 text-xs text-slate-600 dark:text-slate-400">
              <li>
                <a
                  href="https://github.com/thanghn14/hubbub-chat"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors flex items-center gap-1.5"
                >
                  <svg className="w-3.5 h-3.5" viewBox="0 0 24 24" fill="currentColor">
                    <path fillRule="evenodd" clipRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.53 1.032 1.53 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" />
                  </svg>
                  <span>GitHub Repository</span>
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/thanghn14/hubbub-chat#readme"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors"
                >
                  Documentation & Roadmap
                </a>
              </li>
              <li>
                <a
                  href="https://github.com/thanghn14/hubbub-chat/releases"
                  target="_blank"
                  rel="noreferrer"
                  className="hover:text-indigo-600 dark:hover:text-violet-300 transition-colors"
                >
                  Changelog & Releases
                </a>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom copyright */}
        <div className="pt-6 sm:pt-8 flex flex-col sm:flex-row items-center justify-between gap-3 text-xs text-slate-500">
          <div>
            © {new Date().getFullYear()} {t('footer.copyright')}
          </div>
          <div className="flex items-center gap-3">
            {/* <span>{t('footer.builtWith')}</span> */}
            <span>•</span>
            <span className="text-indigo-600 dark:text-violet-300">{t('footer.privacyFirst')}</span>
          </div>
        </div>
      </div>
    </footer>
  )
}
