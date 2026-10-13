import type { Message, ChatTurn } from '../types';

/**
 * Consolidates individual database Message records into cohesive conversational Turns.
 * Each turn groups:
 * - 1 User prompt
 * - Multiple multi-step Tool calls and results (if any)
 * - The final synthesized Assistant text response
 */
export function groupMessagesIntoTurns(messages: Message[]): ChatTurn[] {
  const turns: ChatTurn[] = [];
  let currentTurn: ChatTurn | null = null;

  for (const msg of messages) {
    if (msg.role === 'user') {
      if (currentTurn) {
        turns.push(currentTurn);
      }
      currentTurn = {
        id: msg.id,
        userMessage: msg,
        assistantMessages: [],
        toolSteps: [],
        finalText: '',
        createdAt: msg.created_at,
        runId: msg.run_id,
      };
      continue;
    }

    // Role is Assistant or Tool
    if (!currentTurn) {
      currentTurn = {
        id: msg.id,
        assistantMessages: [],
        toolSteps: [],
        finalText: '',
        createdAt: msg.created_at,
        runId: msg.run_id,
      };
    }

    currentTurn.assistantMessages.push(msg);

    for (const part of msg.parts) {
      if (part.type === 'Text') {
        const textContent =
          typeof part.content === 'string'
            ? part.content
            : typeof part.content === 'object' && part.content !== null
            ? JSON.stringify(part.content)
            : String(part.content ?? '');
        if (textContent.trim()) {
          if (currentTurn.finalText) {
            currentTurn.finalText += '\n\n';
          }
          currentTurn.finalText += textContent;
        }
      } else if (part.type === 'ToolCall') {
        const callId = part.id || `tc-${Date.now()}-${Math.random()}`;
        const toolName =
          typeof part.content === 'object' && part.content !== null && 'name' in part.content
            ? String((part.content as Record<string, unknown>).name)
            : part.name || 'Công cụ';
        const existingIdx = currentTurn.toolSteps.findIndex((t) => t.id === callId);
        if (existingIdx >= 0) {
          currentTurn.toolSteps[existingIdx].toolName = toolName;
          currentTurn.toolSteps[existingIdx].args = part.arguments;
        } else {
          currentTurn.toolSteps.push({
            id: callId,
            toolName,
            args: part.arguments,
            status: 'running',
          });
        }
      } else if (part.type === 'ToolResult') {
        const toolCallId = part.tool_call_id || '';
        const resStr =
          typeof part.content === 'object' && part.content !== null && 'result' in part.content
            ? JSON.stringify((part.content as Record<string, unknown>).result)
            : String(part.result ?? part.content ?? '');

        const existing = currentTurn.toolSteps.find((t) => t.id === toolCallId);
        if (existing) {
          existing.status = 'completed';
          existing.result = resStr;
        } else {
          currentTurn.toolSteps.push({
            id: toolCallId || `tr-${Date.now()}`,
            toolName: 'Công cụ',
            result: resStr,
            status: 'completed',
          });
        }
      }
    }
  }

  if (currentTurn) {
    turns.push(currentTurn);
  }

  return turns;
}
