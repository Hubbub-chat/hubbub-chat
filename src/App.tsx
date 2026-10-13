import { useState, useEffect, useCallback, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { ActivityDock, type MainNavView } from './components/ActivityDock';
import { Sidebar } from './components/Sidebar';
import { ChatView } from './components/ChatView';
import { AgentsHubView } from './components/AgentsHubView';
import { WorkspaceReportsView } from './components/WorkspaceReportsView';
import { AgentSkillsDrawer } from './components/AgentSkillsDrawer';
import { ModelQuotaModal } from './components/ModelQuotaModal';
import { SettingsModal } from './components/SettingsModal';
import { AgentModal } from './components/AgentModal';
import { BenchmarkSpike } from './components/BenchmarkSpike';
import { useRunEvents } from './hooks/useRunEvents';
import { useAppInit } from './hooks/useAppInit';
import type { Conversation, Message, Agent, Run, ToolLog } from './types';

export default function App() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeConversationId, setActiveConversationId] = useState<string | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [selectedAgentId, setSelectedAgentId] = useState<string>('analyst');
  const [messages, setMessages] = useState<Message[]>([]);

  // Navigation View State
  const [activeNavView, setActiveNavView] = useState<MainNavView>('chat');

  // Streaming & Execution State
  const [isStreaming, setIsStreaming] = useState<boolean>(false);
  const [streamingText, setStreamingText] = useState<string>('');
  const [activeTools, setActiveTools] = useState<ToolLog[]>([]);
  const [activeRunId, setActiveRunId] = useState<string | null>(null);
  const [errorMsg, setErrorMsg] = useState<string | null>(null);
  const isSendingRef = useRef(false);

  // Modals & Panels
  const [isSidebarOpen, setIsSidebarOpen] = useState<boolean>(() => {
    return typeof window !== 'undefined' ? window.innerWidth >= 900 : true;
  });
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [isQuotaModalOpen, setIsQuotaModalOpen] = useState<boolean>(false);
  const [isAgentModalOpen, setIsAgentModalOpen] = useState<boolean>(false);
  const [isAgentDrawerOpen, setIsAgentDrawerOpen] = useState<boolean>(false);
  const [drawerAgent, setDrawerAgent] = useState<Agent | null>(null);
  const [showBenchmark, setShowBenchmark] = useState<boolean>(false);
  const [appVersion, setAppVersion] = useState<string>('0.1.0');

  const activeConversationIdRef = useRef<string | null>(null);
  useEffect(() => {
    activeConversationIdRef.current = activeConversationId;
  }, [activeConversationId]);

  // Load messages for a conversation
  const loadMessages = useCallback(async (convId: string) => {
    try {
      const msgs = await invoke<Message[]>('list_messages', {
        conversationId: convId,
        limit: 100,
        offset: 0,
      });
      setMessages(msgs);
    } catch (err) {
      console.error('Failed to load messages:', err);
    }
  }, []);

  // Initialize data on mount via custom hook
  useAppInit({
    setAppVersion,
    setAgents,
    setSelectedAgentId,
    setConversations,
    setActiveConversationId,
    loadMessages,
  });

  // Listen for Tauri streaming run events via custom hook
  useRunEvents({
    onMessageDelta: useCallback((delta: string) => {
      setStreamingText((prev) => prev + delta);
    }, []),
    onToolStarted: useCallback((toolName: string, argsPreview: string) => {
      setActiveTools((prev) => [
        ...prev,
        {
          id: `${toolName}-${Date.now()}`,
          name: toolName,
          preview: argsPreview,
          status: 'running',
        },
      ]);
    }, []),
    onToolFinished: useCallback((toolName: string, success: boolean, summary: string) => {
      setActiveTools((prev) =>
        prev.map((tool) =>
          tool.name === toolName && tool.status === 'running'
            ? {
                ...tool,
                status: success ? 'completed' : 'failed',
                summary,
              }
            : tool
        )
      );
    }, []),
    onRunFinished: useCallback(() => {
      if (activeConversationIdRef.current) {
        loadMessages(activeConversationIdRef.current).finally(() => {
          setIsStreaming(false);
          isSendingRef.current = false;
          setActiveRunId(null);
          setStreamingText('');
          setActiveTools([]);
        });
      } else {
        setIsStreaming(false);
        isSendingRef.current = false;
        setActiveRunId(null);
        setStreamingText('');
        setActiveTools([]);
      }
    }, [loadMessages]),
    onError: useCallback((message: string) => {
      setErrorMsg(message);
      setIsStreaming(false);
      isSendingRef.current = false;
      setActiveRunId(null);
    }, []),
  });

  // Handle switching conversation
  const handleSelectConversation = useCallback(
    (id: string) => {
      setActiveNavView('chat');
      setActiveConversationId(id);
      const conv = conversations.find((c) => c.id === id);
      if (conv?.agent_id) {
        setSelectedAgentId(conv.agent_id);
      }
      setIsStreaming(false);
      isSendingRef.current = false;
      setStreamingText('');
      setActiveTools([]);
      setErrorMsg(null);
      loadMessages(id);
    },
    [conversations, loadMessages]
  );

  // Handle creating a new conversation
  const handleNewConversation = useCallback(async () => {
    try {
      const newConv = await invoke<Conversation>('create_conversation', {
        title: 'Cuộc trò chuyện mới',
        agentId: selectedAgentId,
      });
      setConversations((prev) => [newConv, ...prev]);
      setActiveConversationId(newConv.id);
      setActiveNavView('chat');
      setMessages([]);
      setIsStreaming(false);
      isSendingRef.current = false;
      setStreamingText('');
      setActiveTools([]);
      setErrorMsg(null);
    } catch (err) {
      console.error('Failed to create new conversation:', err);
    }
  }, [selectedAgentId]);

  // Handle sending a message with strict in-flight send lock
  const handleSendMessage = useCallback(
    async (prompt: string) => {
      if (!activeConversationId || isSendingRef.current || isStreaming) return;
      isSendingRef.current = true;

      setErrorMsg(null);
      setIsStreaming(true);
      setStreamingText('');
      setActiveTools([]);

      const optimisticMsg: Message = {
        id: `temp-${Date.now()}`,
        conversation_id: activeConversationId,
        role: 'user',
        parts: [{ type: 'Text', content: prompt }],
        created_at: new Date().toISOString(),
      };
      setMessages((prev) => [...prev, optimisticMsg]);

      try {
        const run = await invoke<Run>('send_message', {
          conversationId: activeConversationId,
          agentId: selectedAgentId,
          prompt,
        });
        setActiveRunId(run.id);
      } catch (err) {
        console.error('Failed to send message:', err);
        setIsStreaming(false);
        setActiveRunId(null);
        setErrorMsg(String(err));
      } finally {
        isSendingRef.current = false;
      }
    },
    [activeConversationId, selectedAgentId, isStreaming]
  );

  // Handle cancelling a run
  const handleCancelRun = useCallback(async () => {
    if (!activeRunId) return;
    try {
      await invoke('cancel_run', { runId: activeRunId });
    } catch (err) {
      console.error('Failed to cancel run:', err);
    } finally {
      setIsStreaming(false);
      isSendingRef.current = false;
      setActiveRunId(null);
    }
  }, [activeRunId]);

  // Handle changing agent model
  const handleSelectModel = useCallback(
    async (model: string) => {
      if (!selectedAgentId) return;
      try {
        await invoke('set_agent_model', { agentId: selectedAgentId, model });
        setAgents((prev) =>
          prev.map((a) => (a.id === selectedAgentId ? { ...a, model } : a))
        );
      } catch (err) {
        alert(`Lỗi khi đổi model cho agent: ${String(err)}`);
      }
    },
    [selectedAgentId]
  );

  // Handle saving new or updated functional agent
  const handleSaveAgent = useCallback(async (newAgent: Agent) => {
    try {
      await invoke('upsert_agent', { agent: newAgent });
      setAgents((prev) => {
        const existingIdx = prev.findIndex((a) => a.id === newAgent.id);
        if (existingIdx >= 0) {
          const next = [...prev];
          next[existingIdx] = newAgent;
          return next;
        }
        return [...prev, newAgent];
      });
      setSelectedAgentId(newAgent.id);
    } catch (err) {
      console.error('Failed to save agent:', err);
      throw err;
    }
  }, []);

  const handleSelectNavView = (view: MainNavView) => {
    if (view === 'quota') {
      setIsQuotaModalOpen(true);
      return;
    }
    setActiveNavView(view);
  };

  const activeConversation = conversations.find((c) => c.id === activeConversationId);
  const currentAgent = agents.find((a) => a.id === selectedAgentId) || agents[0] || null;

  if (showBenchmark) {
    return (
      <div className="flex h-screen w-screen bg-[#08090d] text-zinc-100 font-sans select-none overflow-hidden">
        <BenchmarkSpike onBackToChat={() => setShowBenchmark(false)} />
      </div>
    );
  }

  return (
    <div className="flex h-screen w-screen bg-[#08090d] text-zinc-100 font-sans select-none overflow-hidden">
      {/* 1. Leftmost Activity Dock (56px) */}
      <ActivityDock
        activeView={activeNavView}
        onSelectView={handleSelectNavView}
        onOpenSettings={() => setIsSettingsOpen(true)}
        onOpenBenchmark={() => setShowBenchmark(true)}
        showBenchmark={showBenchmark}
        appVersion={appVersion}
      />

      {/* 2. Secondary Sidebar (256px) - Visible in Chat view when toggled open */}
      {activeNavView === 'chat' && isSidebarOpen && (
        <Sidebar
          conversations={conversations}
          activeId={activeConversationId}
          onSelectConversation={handleSelectConversation}
          onNewConversation={handleNewConversation}
          agents={agents}
          selectedAgentId={selectedAgentId}
          onSelectAgent={(agentId: string) => setSelectedAgentId(agentId)}
          onToggleCollapse={() => setIsSidebarOpen(false)}
        />
      )}

      {/* 3. Main Workspace Area */}
      <main className="flex-1 flex flex-col min-w-0 h-full overflow-hidden bg-[#08090d]">
        {activeNavView === 'chat' && (
          <ChatView
            conversationId={activeConversationId || ''}
            conversationTitle={activeConversation?.title || 'Cuộc trò chuyện'}
            messages={messages}
            agent={currentAgent}
            onSendMessage={handleSendMessage}
            onCancelRun={handleCancelRun}
            onSelectModel={handleSelectModel}
            isStreaming={isStreaming}
            streamingText={streamingText}
            activeTools={activeTools}
            errorMsg={errorMsg}
            onOpenSettings={() => setIsSettingsOpen(true)}
            onOpenAgentDrawer={() => {
              setDrawerAgent(currentAgent);
              setIsAgentDrawerOpen(true);
            }}
            onOpenQuotaModal={() => setIsQuotaModalOpen(true)}
            isSidebarOpen={isSidebarOpen}
            onToggleSidebar={() => setIsSidebarOpen((prev) => !prev)}
          />
        )}

        {activeNavView === 'agents' && (
          <AgentsHubView
            agents={agents}
            onSelectAgentForChat={(agentId) => {
              setSelectedAgentId(agentId);
              setActiveNavView('chat');
            }}
            onOpenAgentDrawer={(agent) => {
              setDrawerAgent(agent);
              setIsAgentDrawerOpen(true);
            }}
            onOpenCreateAgent={() => setIsAgentModalOpen(true)}
          />
        )}

        {activeNavView === 'workspace' && (
          <WorkspaceReportsView onBackToChat={() => setActiveNavView('chat')} />
        )}
      </main>

      {/* Slide-over Agent & Skills Inspector Drawer */}
      <AgentSkillsDrawer
        isOpen={isAgentDrawerOpen}
        onClose={() => setIsAgentDrawerOpen(false)}
        agent={drawerAgent || currentAgent}
        onSaveAgent={handleSaveAgent}
      />

      {/* Model Quota & Usage Manager Modal */}
      <ModelQuotaModal
        isOpen={isQuotaModalOpen}
        onClose={() => setIsQuotaModalOpen(false)}
      />

      {/* Settings Modal (API Keys Vault) */}
      <SettingsModal
        isOpen={isSettingsOpen}
        onClose={() => setIsSettingsOpen(false)}
      />

      {/* Create Agent Modal */}
      <AgentModal
        isOpen={isAgentModalOpen}
        onClose={() => setIsAgentModalOpen(false)}
        onSaveAgent={handleSaveAgent}
      />
    </div>
  );
}
