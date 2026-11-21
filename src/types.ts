export interface Conversation {
  id: string;
  platform: string;
  title: string;
  created_at: number | null;
  updated_at: number | null;
  private: boolean;
  sensitive: boolean;
  message_count: number;
  preview: string;
}
export interface Message {
  id: string;
  conversation_id: string;
  role: "user" | "assistant";
  content: string;
  timestamp: number | null;
  branch_id: string | null;
  parent_id: string | null;
  is_active: boolean;
  sensitive: boolean;
  position: number;
}
export interface Stats {
  conversations: number;
  messages: number;
  private: number;
  sensitive: number;
  database_path: string;
}
export interface ImportReport {
  conversations: number;
  messages: number;
  new_conversations: number;
  updated_conversations: number;
  skipped_messages: number;
  warnings: string[];
}

export interface AgentSessionStatus {
  remaining_seconds: number | null;
}

export interface PrivacyUpdate {
  flagged: number;
  approvals_reset: number;
}
