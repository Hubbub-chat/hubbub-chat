export type MessageRole = 'user' | 'assistant' | 'system' | 'tool';

export interface MessagePart {
  type: string;
  content?: unknown;
  id?: string;
  name?: string;
  arguments?: Record<string, unknown> | string;
  tool_call_id?: string;
  result?: string;
  path?: string;
}

export interface Message {
  id: string;
  conversation_id: string;
  run_id?: string | null;
  role: MessageRole;
  parts: MessagePart[];
  created_at: string;
}

export interface Conversation {
  id: string;
  title: string;
  agent_id: string;
  created_at: string;
  updated_at: string;
  archived: boolean;
}

export interface Agent {
  id: string;
  name: string;
  model: string;
  system_prompt: string;
  tools: {
    builtin: string[];
    mcp: string[];
  };
  budget: {
    max_steps: number;
    max_tokens: number;
    max_cost_usd: number;
    timeout_s: number;
  };
}

export interface Run {
  id: string;
  conversation_id: string;
  agent_id: string;
  status: 'running' | 'completed' | 'failed' | 'cancelled';
  usage?: {
    prompt_tokens: number;
    completion_tokens: number;
    total_tokens: number;
  };
}

export type RunEvent =
  | { type: 'MessageDelta'; data: { content: string } }
  | { type: 'ToolStarted'; data: { tool_name: string; args_preview: string } }
  | { type: 'ToolFinished'; data: { tool_name: string; success: boolean; summary: string } }
  | { type: 'RunFinished'; data: { run_id: string; status: string } }
  | { type: 'Error'; data: { message: string; recoverable: boolean } };

export interface ToolLog {
  id: string;
  name: string;
  preview: string;
  status: 'running' | 'completed' | 'failed';
  summary?: string;
}

export interface WorkspaceDocument {
  id: string;
  path: string;
  title: string;
  tags: string[];
  agent_id?: string | null;
  run_id?: string | null;
  content_hash: string;
  size_bytes: number;
  created_at: string;
  updated_at: string;
}

export interface AuditLog {
  id: string;
  timestamp: string;
  run_id?: string | null;
  tool_name: string;
  args_digest: string;
  decision: string;
  result_digest?: string | null;
}

export interface ModelUsageStat {
  model: string;
  provider: string;
  total_prompt_tokens: number;
  total_completion_tokens: number;
  total_tokens: number;
  total_runs: number;
  total_cost_usd: number;
  requests_today: number;
  tokens_today: number;
  rpm_limit: number;
  rpd_limit: number;
  is_free_tier: boolean;
  last_used_at?: string | null;
}

export interface ToolStep {
  id: string;
  toolName: string;
  args?: unknown;
  result?: string;
  status: 'running' | 'completed' | 'failed';
}

export interface ChatTurn {
  id: string;
  userMessage?: Message;
  assistantMessages: Message[];
  toolSteps: ToolStep[];
  finalText: string;
  createdAt: string;
  runId?: string | null;
}


