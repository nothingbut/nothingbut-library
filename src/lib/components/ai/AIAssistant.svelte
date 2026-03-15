<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as ai from '$lib/services/ai';
  import type { StoredMessage } from '$lib/services/ai';
  import type { UnlistenFn } from '@tauri-apps/api/event';

  interface Props {
    isOpen?: boolean;
    currentBookId?: number | null;
    currentChapterId?: number | null;
  }

  let { isOpen = false, currentBookId, currentChapterId }: Props = $props();

  let conversationId = $state<number | null>(null);
  let messages = $state<StoredMessage[]>([]);
  let inputMessage = $state('');
  let loading = $state(false);
  let error = $state<string | null>(null);
  let ollamaStatus = $state<boolean | null>(null);
  let streamingMessage = $state<string>(''); // 流式响应累积的消息
  let unlisteners = $state<UnlistenFn[]>([]); // 事件监听器清理函数

  // 检查 Ollama 状态
  async function checkStatus() {
    try {
      ollamaStatus = await ai.checkOllamaStatus();
    } catch (e) {
      ollamaStatus = false;
    }
  }

  // 创建或加载对话
  async function initConversation() {
    try {
      const contextType = currentChapterId ? 'chapter' : currentBookId ? 'book' : 'general';
      const contextId = currentChapterId || currentBookId || undefined;

      conversationId = await ai.createConversation(
        contextType === 'general' ? '新对话' : '书籍讨论',
        contextType,
        contextId
      );

      await loadMessages();
    } catch (e) {
      error = '创建对话失败';
      console.error(e);
    }
  }

  // 加载消息
  async function loadMessages() {
    if (!conversationId) return;
    try {
      messages = await ai.getConversationHistory(conversationId);
    } catch (e) {
      console.error('加载消息失败:', e);
    }
  }

  // 发送消息（使用流式响应）
  async function sendMessage() {
    if (!inputMessage.trim() || !conversationId || loading) return;

    const userMsg = inputMessage.trim();
    inputMessage = '';
    loading = true;
    error = null;
    streamingMessage = '';

    try {
      // 立即显示用户消息
      messages = [...messages, {
        id: Date.now(),
        conversation_id: conversationId,
        role: 'user',
        content: userMsg,
        timestamp: new Date().toISOString()
      }];

      // 滚动到底部
      setTimeout(scrollToBottom, 100);

      // 使用流式 API 获取 AI 响应
      unlisteners = await ai.sendMessageStream(conversationId, userMsg, {
        onChunk: (chunk: string) => {
          streamingMessage += chunk;
          // 实时滚动到底部
          setTimeout(scrollToBottom, 10);
        },
        onDone: () => {
          // 流式完成，添加完整消息
          if (streamingMessage) {
            messages = [...messages, {
              id: Date.now() + 1,
              conversation_id: conversationId!,
              role: 'assistant',
              content: streamingMessage,
              timestamp: new Date().toISOString()
            }];
          }
          streamingMessage = '';
          loading = false;
          // 清理监听器
          cleanupListeners();
        },
        onError: (err: string) => {
          error = '发送失败：' + err;
          console.error('Stream error:', err);
          streamingMessage = '';
          loading = false;
          // 清理监听器
          cleanupListeners();
        }
      });
    } catch (e) {
      error = '发送失败，请重试';
      console.error(e);
      loading = false;
      streamingMessage = '';
    }
  }

  // 清理事件监听器
  function cleanupListeners() {
    unlisteners.forEach(unlisten => unlisten());
    unlisteners = [];
  }

  function scrollToBottom() {
    const container = document.querySelector('.messages-container');
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  }

  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
    }
  }

  onMount(() => {
    checkStatus();
    if (ollamaStatus !== false) {
      initConversation();
    }
  });

  onDestroy(() => {
    // 组件销毁时清理监听器
    cleanupListeners();
  });
</script>

<div class="ai-assistant" class:open={isOpen}>
  <div class="assistant-header">
    <h3>🤖 AI 助手</h3>
    {#if ollamaStatus === false}
      <span class="status-badge offline">服务离线</span>
    {:else if ollamaStatus === true}
      <span class="status-badge online">在线</span>
    {/if}
  </div>

  <div class="assistant-body">
    {#if ollamaStatus === false}
      <div class="empty-state">
        <div class="empty-icon">⚠️</div>
        <p>Ollama 服务未运行</p>
        <p class="hint">请先启动 Ollama 服务</p>
      </div>
    {:else if messages.length === 0}
      <div class="empty-state">
        <div class="empty-icon">💬</div>
        <p>开始对话</p>
        <p class="hint">我可以帮你分析书籍内容</p>
      </div>
    {:else}
      <div class="messages-container">
        {#each messages as msg (msg.id)}
          <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
            <div class="message-avatar">
              {msg.role === 'user' ? '👤' : '🤖'}
            </div>
            <div class="message-content">
              <div class="message-text">{msg.content}</div>
            </div>
          </div>
        {/each}
        {#if loading}
          <div class="message assistant">
            <div class="message-avatar">🤖</div>
            <div class="message-content">
              {#if streamingMessage}
                <div class="message-text streaming">
                  {streamingMessage}
                  <span class="cursor">▊</span>
                </div>
              {:else}
                <div class="typing-indicator">
                  <span></span><span></span><span></span>
                </div>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if ollamaStatus !== false}
    <div class="assistant-footer">
      {#if error}
        <div class="error-message">{error}</div>
      {/if}
      <div class="input-area">
        <textarea
          bind:value={inputMessage}
          onkeydown={handleKeyPress}
          placeholder="输入消息... (Enter 发送, Shift+Enter 换行)"
          disabled={loading || !conversationId}
          rows="3"
        ></textarea>
        <button
          onclick={sendMessage}
          disabled={!inputMessage.trim() || loading || !conversationId}
          class="send-btn"
        >
          {loading ? '⏳' : '📤'}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .ai-assistant {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--color-bg-primary);
    border-left: 1px solid var(--color-border);
  }

  .assistant-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .assistant-header h3 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .status-badge {
    font-size: 12px;
    padding: 4px 8px;
    border-radius: 4px;
  }

  .status-badge.online {
    background: #10b981;
    color: white;
  }

  .status-badge.offline {
    background: #ef4444;
    color: white;
  }

  .assistant-body {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--color-text-secondary);
  }

  .empty-icon {
    font-size: 48px;
  }

  .hint {
    font-size: 12px;
    opacity: 0.7;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .message {
    display: flex;
    gap: 12px;
  }

  .message.user {
    flex-direction: row-reverse;
  }

  .message-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 18px;
    flex-shrink: 0;
  }

  .message-content {
    flex: 1;
    max-width: 80%;
  }

  .message-text {
    padding: 12px;
    border-radius: 8px;
    line-height: 1.5;
    white-space: pre-wrap;
  }

  .message.user .message-text {
    background: var(--color-primary);
    color: white;
  }

  .message.assistant .message-text {
    background: var(--color-bg-secondary);
    color: var(--color-text-primary);
  }

  .message-text.streaming {
    position: relative;
  }

  .message-text .cursor {
    display: inline-block;
    animation: blink 1s infinite;
    margin-left: 2px;
  }

  @keyframes blink {
    0%, 50% { opacity: 1; }
    51%, 100% { opacity: 0; }
  }

  .typing-indicator {
    display: flex;
    gap: 4px;
    padding: 12px;
  }

  .typing-indicator span {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-text-secondary);
    animation: typing 1.4s infinite;
  }

  .typing-indicator span:nth-child(2) {
    animation-delay: 0.2s;
  }

  .typing-indicator span:nth-child(3) {
    animation-delay: 0.4s;
  }

  @keyframes typing {
    0%, 60%, 100% { opacity: 0.3; }
    30% { opacity: 1; }
  }

  .assistant-footer {
    border-top: 1px solid var(--color-border);
    padding: 16px;
  }

  .error-message {
    color: #ef4444;
    font-size: 12px;
    margin-bottom: 8px;
  }

  .input-area {
    display: flex;
    gap: 8px;
  }

  .input-area textarea {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    resize: none;
    font-family: inherit;
    font-size: 14px;
  }

  .send-btn {
    width: 40px;
    height: 40px;
    border: none;
    background: var(--color-primary);
    color: white;
    border-radius: 6px;
    cursor: pointer;
    font-size: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .send-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
