# NothingBut Library AI 集成架构设计

## 概述

集成 Ollama 本地 AI，为小说阅读提供智能辅助功能。

### 核心目标
1. 智能摘要（章节/书籍）
2. 对话问答（基于上下文）
3. 角色分析
4. 语义搜索（相似内容查找）

### 技术选型
- **AI 服务**: Ollama（本地部署，支持多种开源模型）
- **通信方式**: HTTP API
- **向量数据库**: SQLite-VSS（轻量级，与现有 SQLite 集成）
- **嵌入模型**: nomic-embed-text（Ollama 内置）
- **对话模型**: qwen2.5:7b 或 llama3.1:8b（推荐）

---

## 1. Ollama 服务通信

### 1.1 API 端点
```
Base URL: http://localhost:11434
```

| 端点 | 方法 | 用途 |
|------|------|------|
| `/api/generate` | POST | 文本生成（非流式/流式） |
| `/api/chat` | POST | 对话（支持上下文） |
| `/api/embeddings` | POST | 向量嵌入 |
| `/api/tags` | GET | 列出可用模型 |

### 1.2 请求示例

**文本生成**:
```json
{
  "model": "qwen2.5:7b",
  "prompt": "为以下内容生成摘要：\n\n{content}",
  "stream": false,
  "options": {
    "temperature": 0.7,
    "top_p": 0.9
  }
}
```

**对话**:
```json
{
  "model": "qwen2.5:7b",
  "messages": [
    {"role": "system", "content": "你是小说阅读助手"},
    {"role": "user", "content": "这本书的主角是谁？"}
  ],
  "stream": true
}
```

**嵌入向量**:
```json
{
  "model": "nomic-embed-text",
  "input": "第一章内容..."
}
```

### 1.3 流式响应处理
```rust
// Server-Sent Events (SSE) 格式
// 每行一个 JSON 对象，以 \n 分隔
{
  "model": "qwen2.5:7b",
  "created_at": "2024-01-01T00:00:00Z",
  "message": {
    "role": "assistant",
    "content": "这是"
  },
  "done": false
}
```

---

## 2. Rust 后端架构

### 2.1 模块结构
```
src-tauri/src/
├── modules/
│   └── ai/
│       ├── mod.rs              # 模块入口
│       ├── ollama.rs           # Ollama HTTP 客户端
│       ├── chat.rs             # 对话管理
│       ├── embeddings.rs       # 向量嵌入
│       ├── summarize.rs        # 摘要生成
│       ├── search.rs           # 语义搜索
│       ├── models.rs           # 数据模型
│       ├── database.rs         # AI 相关数据库操作
│       └── commands.rs         # Tauri Commands
```

### 2.2 核心类型定义

```rust
// models.rs

/// AI 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// 对话会话
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: i64,
    pub title: String,
    pub context_type: ContextType,
    pub context_id: Option<i64>, // book_id or chapter_id
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextType {
    General,      // 通用对话
    Book,         // 书籍上下文
    Chapter,      // 章节上下文
}

/// 章节向量索引
#[derive(Debug, Clone)]
pub struct ChapterEmbedding {
    pub chapter_id: i64,
    pub embedding: Vec<f32>, // 768 维向量
    pub created_at: DateTime<Utc>,
}

/// 摘要缓存
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    pub id: i64,
    pub target_type: String, // "chapter" or "book"
    pub target_id: i64,
    pub summary: String,
    pub length: SummaryLength,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SummaryLength {
    Short,   // 100-200 字
    Medium,  // 300-500 字
    Long,    // 800-1000 字
}
```

### 2.3 Ollama 客户端

```rust
// ollama.rs

pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// 生成文本（非流式）
    pub async fn generate(&self, prompt: &str, model: &str) -> Result<String> {
        // POST /api/generate
    }

    /// 对话（流式）
    pub async fn chat_stream(
        &self,
        messages: Vec<ChatMessage>,
        model: &str,
    ) -> Result<impl Stream<Item = Result<String>>> {
        // POST /api/chat with stream=true
    }

    /// 生成嵌入向量
    pub async fn embeddings(&self, text: &str, model: &str) -> Result<Vec<f32>> {
        // POST /api/embeddings
    }

    /// 检查 Ollama 服务状态
    pub async fn health_check(&self) -> Result<bool> {
        // GET /api/tags
    }
}
```

### 2.4 数据库 Schema

```sql
-- migrations/0005_ai.sql

-- AI 对话会话
CREATE TABLE ai_conversations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    context_type TEXT NOT NULL, -- 'general', 'book', 'chapter'
    context_id INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- AI 对话消息
CREATE TABLE ai_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    conversation_id INTEGER NOT NULL,
    role TEXT NOT NULL, -- 'system', 'user', 'assistant'
    content TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES ai_conversations(id) ON DELETE CASCADE
);

-- 摘要缓存
CREATE TABLE ai_summaries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    target_type TEXT NOT NULL, -- 'chapter', 'book'
    target_id INTEGER NOT NULL,
    summary TEXT NOT NULL,
    length TEXT NOT NULL, -- 'short', 'medium', 'long'
    model TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(target_type, target_id, length)
);

-- 章节向量索引（使用 SQLite-VSS）
CREATE VIRTUAL TABLE chapter_embeddings USING vss0(
    embedding(768) -- nomic-embed-text 输出 768 维
);

CREATE TABLE chapter_embedding_metadata (
    rowid INTEGER PRIMARY KEY,
    chapter_id INTEGER NOT NULL UNIQUE,
    created_at TEXT NOT NULL,
    FOREIGN KEY (chapter_id) REFERENCES novel_chapters(id) ON DELETE CASCADE
);
```

### 2.5 Tauri Commands

```rust
// commands.rs

/// 创建对话会话
#[tauri::command]
pub async fn create_conversation(
    pool: State<'_, SqlitePool>,
    title: String,
    context_type: String,
    context_id: Option<i64>,
) -> AppResult<i64> {
    // ...
}

/// 发送消息并获取 AI 响应（流式）
#[tauri::command]
pub async fn send_message(
    pool: State<'_, SqlitePool>,
    conversation_id: i64,
    message: String,
) -> AppResult<()> {
    // 返回一个事件 ID，通过 Tauri 事件系统发送流式响应
}

/// 获取对话历史
#[tauri::command]
pub async fn get_conversation_history(
    pool: State<'_, SqlitePool>,
    conversation_id: i64,
    limit: Option<i32>,
) -> AppResult<Vec<ChatMessage>> {
    // ...
}

/// 生成章节摘要
#[tauri::command]
pub async fn summarize_chapter(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    chapter_id: i64,
    length: String, // "short", "medium", "long"
) -> AppResult<String> {
    // 检查缓存，如果没有则生成
}

/// 生成书籍摘要
#[tauri::command]
pub async fn summarize_book(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
    length: String,
) -> AppResult<String> {
    // ...
}

/// 对章节内容建立向量索引
#[tauri::command]
pub async fn index_chapter(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    chapter_id: i64,
) -> AppResult<()> {
    // ...
}

/// 批量索引整本书
#[tauri::command]
pub async fn index_book(
    pool: State<'_, SqlitePool>,
    workspace_path: String,
    book_id: i64,
) -> AppResult<()> {
    // ...
}

/// 语义搜索
#[tauri::command]
pub async fn semantic_search(
    pool: State<'_, SqlitePool>,
    query: String,
    book_id: Option<i64>, // 限制在某本书内搜索
    limit: Option<i32>,
) -> AppResult<Vec<SearchResult>> {
    // 1. 生成查询向量
    // 2. 使用 VSS 查找相似向量
    // 3. 返回章节信息和相似度
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub chapter_id: i64,
    pub chapter_title: String,
    pub book_title: String,
    pub similarity: f32, // 0.0 - 1.0
    pub preview: String, // 章节预览
}
```

---

## 3. 前端 UI 架构

### 3.1 组件结构
```
src/lib/components/ai/
├── AIAssistant.svelte       # 主 AI 助手面板
├── ChatInterface.svelte     # 对话界面
├── MessageBubble.svelte     # 消息气泡
├── ToolPanel.svelte         # 功能面板（摘要、搜索等）
├── SummaryView.svelte       # 摘要显示
├── SearchResults.svelte     # 搜索结果
└── TypingIndicator.svelte   # 打字指示器
```

### 3.2 AIAssistant 组件

```svelte
<!-- AIAssistant.svelte -->
<script lang="ts">
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import ChatInterface from './ChatInterface.svelte';
  import ToolPanel from './ToolPanel.svelte';

  interface Props {
    isOpen?: boolean;
    currentBook?: Book | null;
    currentChapter?: Chapter | null;
  }

  let { isOpen = false, currentBook, currentChapter }: Props = $props();

  let activeTab = $state<'chat' | 'tools'>('chat');
  let conversationId = $state<number | null>(null);
  let messages = $state<ChatMessage[]>([]);

  // 创建或加载对话
  async function initConversation() {
    // 根据上下文（currentBook/currentChapter）创建对话
  }

  // 监听流式响应事件
  onMount(() => {
    const unlisten = listen<string>('ai-message-chunk', (event) => {
      // 追加到最后一条 AI 消息
    });
    return unlisten;
  });
</script>

<div class="ai-assistant" class:open={isOpen}>
  <div class="assistant-header">
    <h3>AI 助手</h3>
    <div class="header-tabs">
      <button
        class:active={activeTab === 'chat'}
        onclick={() => activeTab = 'chat'}
      >
        对话
      </button>
      <button
        class:active={activeTab === 'tools'}
        onclick={() => activeTab = 'tools'}
      >
        工具
      </button>
    </div>
  </div>

  <div class="assistant-body">
    {#if activeTab === 'chat'}
      <ChatInterface
        {conversationId}
        {messages}
        context={{ book: currentBook, chapter: currentChapter }}
      />
    {:else}
      <ToolPanel
        {currentBook}
        {currentChapter}
      />
    {/if}
  </div>
</div>
```

### 3.3 状态管理

```typescript
// src/lib/stores/ai.ts

import { writable, derived } from 'svelte/store';
import type { ChatMessage, Conversation } from '$lib/types';

export const conversations = writable<Conversation[]>([]);
export const currentConversation = writable<Conversation | null>(null);
export const messages = writable<Map<number, ChatMessage[]>>(new Map());

export const currentMessages = derived(
  [currentConversation, messages],
  ([$conversation, $messages]) => {
    if (!$conversation) return [];
    return $messages.get($conversation.id) || [];
  }
);

// Actions
export async function loadConversations() {
  // 加载所有对话
}

export async function sendMessage(conversationId: number, content: string) {
  // 发送消息并处理流式响应
}

export async function createConversation(title: string, context?: any) {
  // 创建新对话
}
```

### 3.4 API 服务层

```typescript
// src/lib/services/ai.ts

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export async function createConversation(
  title: string,
  contextType: 'general' | 'book' | 'chapter',
  contextId?: number
): Promise<number> {
  return await invoke('create_conversation', {
    title,
    contextType,
    contextId,
  });
}

export async function sendMessage(
  conversationId: number,
  message: string,
  onChunk: (chunk: string) => void
): Promise<void> {
  // 监听流式响应事件
  const unlisten = await listen<string>('ai-message-chunk', (event) => {
    onChunk(event.payload);
  });

  try {
    await invoke('send_message', { conversationId, message });
  } finally {
    unlisten();
  }
}

export async function summarizeChapter(
  chapterId: number,
  length: 'short' | 'medium' | 'long'
): Promise<string> {
  return await invoke('summarize_chapter', {
    workspacePath: getWorkspacePath(),
    chapterId,
    length,
  });
}

export async function semanticSearch(
  query: string,
  bookId?: number,
  limit: number = 10
): Promise<SearchResult[]> {
  return await invoke('semantic_search', { query, bookId, limit });
}
```

---

## 4. 向量数据库方案

### 4.1 SQLite-VSS 集成

**优势**:
- 与现有 SQLite 无缝集成
- 轻量级，无需额外服务
- 支持余弦相似度搜索
- 性能足够（中小规模数据）

**安装**:
```toml
# Cargo.toml
[dependencies]
sqlite-vss = "0.1"
```

### 4.2 索引策略

**何时建立索引**:
1. 导入新书时（后台任务）
2. 用户手动触发（"为此书建立索引"）
3. 首次使用语义搜索时（延迟索引）

**索引粒度**:
- 每个章节一个向量
- 长章节可以分段（每 1000 字一个向量）

**向量维度**:
- `nomic-embed-text`: 768 维
- 余弦相似度阈值: > 0.7 认为相关

### 4.3 性能优化

```rust
// 批量生成嵌入
pub async fn batch_embed(
    client: &OllamaClient,
    texts: Vec<String>,
) -> Result<Vec<Vec<f32>>> {
    // 并发请求，限制并发数为 5
    let tasks: Vec<_> = texts
        .chunks(5)
        .map(|chunk| {
            let futures = chunk.iter().map(|text| client.embeddings(text, "nomic-embed-text"));
            futures::future::join_all(futures)
        })
        .collect();

    let results = futures::future::join_all(tasks).await;
    // 展平结果
}
```

---

## 5. AI 功能详细设计

### 5.1 智能摘要

**提示词模板**:
```rust
const SUMMARY_PROMPT_TEMPLATE: &str = r#"
你是一位专业的小说分析助手。请为以下内容生成一个{length}摘要。

要求：
- 简洁明了，突出重点
- 保留关键情节和人物
- 不要添加原文没有的内容
- 使用第三人称叙述

内容：
{content}

摘要：
"#;

fn build_summary_prompt(content: &str, length: SummaryLength) -> String {
    let length_desc = match length {
        SummaryLength::Short => "100-200字的简短",
        SummaryLength::Medium => "300-500字的中等",
        SummaryLength::Long => "800-1000字的详细",
    };

    SUMMARY_PROMPT_TEMPLATE
        .replace("{length}", length_desc)
        .replace("{content}", content)
}
```

**缓存策略**:
- 同一内容+长度的摘要缓存到数据库
- 24小时有效期（可配置）
- 用户可手动刷新

### 5.2 对话问答

**系统提示词**:
```rust
const CHAT_SYSTEM_PROMPT: &str = r#"
你是一位专业的小说阅读助手，专门帮助读者理解和分析小说内容。

当前上下文：
- 书名：{book_title}
- 作者：{author}
- 当前章节：{chapter_title}

你可以：
1. 回答关于书籍内容的问题
2. 分析人物关系和性格
3. 总结情节发展
4. 预测后续走向
5. 解释书中的梗和典故

请基于提供的上下文信息回答，如果信息不足，请明确告知。
"#;
```

**上下文管理**:
- 保留最近 10 轮对话
- 超过限制时，保留系统提示词 + 最近对话
- 自动注入当前书籍/章节信息

### 5.3 角色分析

**功能**:
- 提取书中提到的角色名单
- 分析角色关系网络
- 生成角色性格描述

**实现**:
```rust
pub async fn analyze_characters(
    client: &OllamaClient,
    book_content: &str,
) -> Result<Vec<Character>> {
    let prompt = format!(
        "请从以下小说内容中提取所有角色，并分析每个角色的性格特点：\n\n{}",
        book_content
    );

    let response = client.generate(&prompt, "qwen2.5:7b").await?;
    // 解析 JSON 响应
}
```

### 5.4 语义搜索

**搜索流程**:
1. 用户输入查询："主角获得神秘力量的章节"
2. 生成查询向量
3. VSS 查找 top-K 相似章节
4. 返回章节列表 + 相似度 + 预览

**UI 展示**:
```svelte
<SearchResults results={searchResults}>
  {#each results as result}
    <div class="result-item">
      <div class="result-header">
        <h4>{result.chapterTitle}</h4>
        <span class="similarity">{(result.similarity * 100).toFixed(0)}% 相关</span>
      </div>
      <p class="result-preview">{result.preview}</p>
      <button onclick={() => navigateToChapter(result.chapterId)}>
        跳转阅读
      </button>
    </div>
  {/each}
</SearchResults>
```

---

## 6. 错误处理和降级

### 6.1 Ollama 服务检测
```rust
pub async fn check_ollama_available() -> bool {
    let client = OllamaClient::new("http://localhost:11434".to_string());
    client.health_check().await.unwrap_or(false)
}
```

### 6.2 降级策略

| 场景 | 降级方案 |
|------|----------|
| Ollama 服务不可用 | 显示提示："AI 功能需要 Ollama 服务" |
| 模型未下载 | 提供下载命令提示 |
| 请求超时 | 显示错误 + 重试按钮 |
| 向量索引未建立 | 提示"请先为此书建立索引" |

### 6.3 用户反馈
- 所有 AI 操作显示加载状态
- 流式响应显示打字机效果
- 错误信息友好且可操作

---

## 7. 性能和优化

### 7.1 响应时间目标
- 对话响应首字: < 500ms
- 章节摘要生成: < 5s
- 语义搜索: < 2s
- 批量索引: 后台进行，不阻塞 UI

### 7.2 资源管理
- 限制并发 AI 请求数: 3
- 请求队列：FIFO
- 超时设置: 30s
- 取消支持：用户可中断长时间请求

### 7.3 缓存策略
- 摘要缓存到数据库
- 向量索引持久化
- 对话历史按需加载

---

## 8. 实施步骤

### Phase 1: 基础设施（Task 6）
- [ ] Ollama HTTP 客户端
- [ ] 数据库 schema 和迁移
- [ ] 基础 Tauri commands

### Phase 2: 对话功能（Task 7）
- [ ] 对话管理
- [ ] 流式响应处理
- [ ] 对话历史存储

### Phase 3: 向量搜索（Task 8）
- [ ] SQLite-VSS 集成
- [ ] 向量索引构建
- [ ] 语义搜索实现

### Phase 4: 前端集成（Task 9）
- [ ] AI 助手 UI 组件
- [ ] 对话界面
- [ ] 工具面板

### Phase 5: 高级功能（Task 10）
- [ ] 智能摘要
- [ ] 角色分析
- [ ] 增强搜索

### Phase 6: 优化和测试（Task 11-12）
- [ ] 性能优化
- [ ] 错误处理完善
- [ ] 端到端测试

---

## 9. 配置和部署

### 9.1 Ollama 安装
```bash
# macOS
curl -fsSL https://ollama.com/install.sh | sh

# 下载推荐模型
ollama pull qwen2.5:7b
ollama pull nomic-embed-text
```

### 9.2 应用配置
```toml
# src-tauri/Cargo.toml
[dependencies]
reqwest = { version = "0.11", features = ["json", "stream"] }
sqlite-vss = "0.1"
tokio-stream = "0.1"
```

### 9.3 环境变量
```bash
# .env (可选)
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_DEFAULT_MODEL=qwen2.5:7b
OLLAMA_EMBEDDING_MODEL=nomic-embed-text
```

---

## 10. 未来扩展

### 10.1 多模型支持
- 允许用户选择不同的模型
- 模型配置管理界面

### 10.2 高级功能
- 情节时间线生成
- 地图和世界观可视化
- 多书交叉搜索
- AI 写作助手（续写、改写）

### 10.3 性能增强
- 迁移到独立向量数据库（Qdrant）
- 使用更大的模型（Qwen 14B/32B）
- GPU 加速

---

## 参考资料

- [Ollama API 文档](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [SQLite-VSS](https://github.com/asg017/sqlite-vss)
- [Nomic Embed Text](https://huggingface.co/nomic-ai/nomic-embed-text-v1.5)
- [Qwen2.5](https://github.com/QwenLM/Qwen2.5)
