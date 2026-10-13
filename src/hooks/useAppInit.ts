import { useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Conversation, Agent } from '../types';

interface UseAppInitProps {
  setAppVersion: (ver: string) => void;
  setAgents: (agents: Agent[]) => void;
  setSelectedAgentId: (id: string) => void;
  setConversations: (convs: Conversation[]) => void;
  setActiveConversationId: (id: string) => void;
  loadMessages: (convId: string) => Promise<void>;
}

export function useAppInit({
  setAppVersion,
  setAgents,
  setSelectedAgentId,
  setConversations,
  setActiveConversationId,
  loadMessages,
}: UseAppInitProps) {
  useEffect(() => {
    let isMounted = true;

    async function init() {
      try {
        const ver = await invoke<string>('get_version');
        if (isMounted) setAppVersion(ver);
      } catch {
        // Fallback for browser preview
      }

      let loadedAgents: Agent[] = [];
      try {
        loadedAgents = await invoke<Agent[]>('list_agents');
        if (isMounted && loadedAgents.length > 0) {
          setAgents(loadedAgents);
          setSelectedAgentId(loadedAgents[0].id);
        }
      } catch (err) {
        console.error('Failed to list agents:', err);
      }

      try {
        let convs = await invoke<Conversation[]>('list_conversations');
        if (convs.length === 0) {
          const defaultAgent = loadedAgents[0]?.id || 'researcher';
          const newConv = await invoke<Conversation>('create_conversation', {
            title: 'Cuộc trò chuyện mới',
            agentId: defaultAgent,
          });
          convs = [newConv];
        }

        if (isMounted) {
          setConversations(convs);
          if (convs[0]) {
            setActiveConversationId(convs[0].id);
            if (convs[0].agent_id) {
              setSelectedAgentId(convs[0].agent_id);
            }
            loadMessages(convs[0].id);
          }
        }
      } catch (err) {
        console.error('Failed to load conversations:', err);
      }
    }

    init();
    return () => {
      isMounted = false;
    };
  }, [setAppVersion, setAgents, setSelectedAgentId, setConversations, setActiveConversationId, loadMessages]);
}
