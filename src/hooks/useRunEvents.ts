import { useEffect } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { RunEvent } from '../types';

interface UseRunEventsProps {
  onRunStarted: (runId: string, conversationId: string) => void;
  /** Events from any other run (e.g. one left running in another conversation) are dropped. */
  isActiveRun: (runId: string) => boolean;
  onMessageDelta: (delta: string) => void;
  onToolStarted: (toolName: string, argsPreview: string) => void;
  onToolFinished: (toolName: string, success: boolean, summary: string) => void;
  onRunFinished: () => void;
  onError: (message: string) => void;
}

export function useRunEvents({
  onRunStarted,
  isActiveRun,
  onMessageDelta,
  onToolStarted,
  onToolFinished,
  onRunFinished,
  onError,
}: UseRunEventsProps) {
  useEffect(() => {
    let isMounted = true;
    let unlisten: UnlistenFn | null = null;

    async function subscribe() {
      try {
        unlisten = await listen<RunEvent>('run_event', (event) => {
          if (!isMounted) return;
          const payload = event.payload;
          if (!payload) return;

          if (payload.type === 'RunStarted') {
            onRunStarted(payload.data.run_id, payload.data.conversation_id);
            return;
          }
          if (!isActiveRun(payload.data.run_id)) return;

          switch (payload.type) {
            case 'MessageDelta':
              onMessageDelta(payload.data.content);
              break;
            case 'ToolStarted':
              onToolStarted(payload.data.tool_name, payload.data.args_preview);
              break;
            case 'ToolFinished':
              onToolFinished(payload.data.tool_name, payload.data.success, payload.data.summary);
              break;
            case 'RunFinished':
              onRunFinished();
              break;
            case 'Error':
              onError(payload.data.message);
              break;
          }
        });
      } catch (err) {
        console.warn('Tauri event listener registration failed:', err);
      }
    }

    subscribe();

    return () => {
      isMounted = false;
      if (unlisten) unlisten();
    };
  }, [
    onRunStarted,
    isActiveRun,
    onMessageDelta,
    onToolStarted,
    onToolFinished,
    onRunFinished,
    onError,
  ]);
}
