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
        if (part.content.trim()) {
          if (currentTurn.finalText) {
            currentTurn.finalText += '\n\n';
          }
          currentTurn.finalText += part.content;
        }
      } else if (part.type === 'ToolCall') {
        const { id, name, arguments: args } = part.content;
        const existing = currentTurn.toolSteps.find((t) => t.id === id);
        if (existing) {
          existing.toolName = name;
          existing.args = args;
        } else {
          currentTurn.toolSteps.push({ id, toolName: name, args, status: 'running' });
        }
      } else if (part.type === 'ToolResult') {
        const { tool_call_id, result } = part.content;
        const resStr = typeof result === 'string' ? result : JSON.stringify(result);
        const existing = currentTurn.toolSteps.find((t) => t.id === tool_call_id);
        if (existing) {
          existing.status = 'completed';
          existing.result = resStr;
        } else {
          currentTurn.toolSteps.push({
            id: tool_call_id,
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

  // History is stored messages only (live tools are tracked separately), so a call
  // still without a result belongs to a run that was cancelled or failed
  for (const turn of turns) {
    for (const step of turn.toolSteps) {
      if (step.status === 'running') step.status = 'failed';
    }
  }

  return turns;
}
