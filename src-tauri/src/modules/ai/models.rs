use serde::{Deserialize, Serialize};

/// AI 对话消息角色
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

impl MessageRole {
    pub fn as_str(&self) -> &str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
        }
    }
}

/// AI 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// 数据库中的消息记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMessage {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

impl StoredMessage {
    pub fn to_chat_message(&self) -> ChatMessage {
        let role = match self.role.as_str() {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            _ => MessageRole::User,
        };
        ChatMessage {
            role,
            content: self.content.clone(),
        }
    }
}

/// 对话上下文类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    General,  // 通用对话
    Book,     // 书籍上下文
    Chapter,  // 章节上下文
}

impl ContextType {
    pub fn as_str(&self) -> &str {
        match self {
            ContextType::General => "general",
            ContextType::Book => "book",
            ContextType::Chapter => "chapter",
        }
    }
}

/// 对话会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub context_type: String,
    pub context_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
}

/// 摘要长度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SummaryLength {
    Short,   // 100-200 字
    Medium,  // 300-500 字
    Long,    // 800-1000 字
}

impl SummaryLength {
    pub fn as_str(&self) -> &str {
        match self {
            SummaryLength::Short => "short",
            SummaryLength::Medium => "medium",
            SummaryLength::Long => "long",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            SummaryLength::Short => "100-200字的简短",
            SummaryLength::Medium => "300-500字的中等",
            SummaryLength::Long => "800-1000字的详细",
        }
    }
}

/// 摘要缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: i64,
    pub target_type: String, // "chapter" or "book"
    pub target_id: i64,
    pub summary: String,
    pub length: String,
    pub model: String,
    pub created_at: String,
}

/// Ollama API 请求：生成
#[derive(Debug, Clone, Serialize)]
pub struct GenerateRequest {
    pub model: String,
    pub prompt: String,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

/// Ollama API 请求：对话
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

/// Ollama 生成选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
}

/// Ollama API 响应：生成（非流式）
#[derive(Debug, Clone, Deserialize)]
pub struct GenerateResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
}

/// Ollama API 响应：对话（流式）
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponseChunk {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessage,
    pub done: bool,
}

/// Ollama API 请求：嵌入
#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingsRequest {
    pub model: String,
    pub input: String,
}

/// Ollama API 响应：嵌入
#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingsResponse {
    pub model: String,
    pub embeddings: Vec<Vec<f32>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_role_serialization() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains(r#""role":"user"#));
    }

    #[test]
    fn test_summary_length_description() {
        assert_eq!(SummaryLength::Short.description(), "100-200字的简短");
        assert_eq!(SummaryLength::Medium.description(), "300-500字的中等");
        assert_eq!(SummaryLength::Long.description(), "800-1000字的详细");
    }
}

/// 语义搜索结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chapter_id: i64,
    pub chapter_title: String,
    pub chapter_number: i32,
    pub book_id: i64,
    pub book_title: String,
    pub similarity: f32, // 0.0 - 1.0
    pub preview: String, // 章节预览（前200字）
}

// ==================== AI 助手工具调用 ====================

/// 工具参数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    #[serde(rename = "type")]
    pub param_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
}

/// 工具函数定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// 工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: ToolFunction,
}

/// 工具调用请求（由 LLM 返回）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

/// 函数调用详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String, // JSON 字符串
}

/// 工具调用结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub output: String,
}

/// 带工具调用的聊天请求（扩展版）
#[derive(Debug, Clone, Serialize)]
pub struct ChatRequestWithTools {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<GenerateOptions>,
}

/// 带工具调用的聊天响应
#[derive(Debug, Clone, Deserialize)]
pub struct ChatResponseWithTools {
    pub model: String,
    pub created_at: String,
    pub message: ChatMessageWithTools,
    pub done: bool,
}

/// 带工具调用的消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageWithTools {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}
