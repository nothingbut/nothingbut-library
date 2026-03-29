<script lang="ts">
	import { assistantChat, handleAssistantAction, checkOllamaStatus, type ChatMessage, type AssistantResponse } from '$lib/services/assistant';
	import { onMount } from 'svelte';

	// 状态
	let isOpen = $state(false);
	let userInput = $state('');
	let isLoading = $state(false);
	let ollamaAvailable = $state(false);
	let conversationHistory: ChatMessage[] = $state([]);
	let messages: Array<{ role: string; content: string; isError?: boolean }> = $state([]);

	// 检查 Ollama 状态
	onMount(async () => {
		ollamaAvailable = await checkOllamaStatus();
		if (!ollamaAvailable) {
			messages.push({
				role: 'system',
				content: '⚠️ Ollama 服务未运行。请启动 Ollama 后再使用 AI 助手。',
				isError: true
			});
		}
	});

	// 切换助手面板
	function toggleAssistant() {
		isOpen = !isOpen;
	}

	// 发送消息
	async function sendMessage() {
		if (!userInput.trim() || isLoading || !ollamaAvailable) return;

		const message = userInput.trim();
		userInput = '';
		isLoading = true;

		// 添加用户消息到界面
		messages.push({
			role: 'user',
			content: message
		});

		try {
			// 调用 AI 助手
			const response = await assistantChat(message, conversationHistory);

			console.log('📥 AI 助手响应:', response);

			// 更新对话历史
			conversationHistory = response.conversation_history;

			// 处理工具调用结果（如导航）
			const hasAction = handleAssistantAction(response);

			// 如果有导航动作，尝试从响应中提取友好消息
			let displayMessage = response.message;
			if (hasAction) {
				try {
					const action = JSON.parse(response.message);
					if (action.message) {
						displayMessage = action.message;
					}
					console.log('✨ 检测到导航动作:', action);
				} catch (e) {
					// 保持原始消息
				}
			}

			// 添加 AI 响应到界面
			messages.push({
				role: 'assistant',
				content: displayMessage
			});

			console.log('💬 显示消息:', displayMessage);
		} catch (error) {
			console.error('AI 助手错误:', error);

			// 更好的错误处理
			let errorMessage = '抱歉，发生了错误';

			if (error instanceof Error) {
				errorMessage += ': ' + error.message;
			} else if (typeof error === 'string') {
				errorMessage += ': ' + error;
			} else if (error && typeof error === 'object') {
				// 尝试序列化错误对象
				try {
					errorMessage += ': ' + JSON.stringify(error);
				} catch {
					errorMessage += ': 未知错误';
				}
			}

			messages.push({
				role: 'assistant',
				content: errorMessage,
				isError: true
			});
		} finally {
			isLoading = false;
		}
	}

	// 快捷示例
	const examples = [
		'列出所有的库',
		'切换到测试书库',
		'搜索三体小说',
		'播放周杰伦的歌'
	];

	function useExample(example: string) {
		userInput = example;
		sendMessage();
	}

	// 清空对话
	function clearConversation() {
		messages = [];
		conversationHistory = [];
	}
</script>

<!-- AI 助手按钮 -->
<button
	onclick={toggleAssistant}
	class="ai-button"
	title="AI 助手"
>
	<svg class="ai-icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
		<path
			stroke-linecap="round"
			stroke-linejoin="round"
			stroke-width="2"
			d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z"
		/>
	</svg>
</button>

<!-- AI 助手面板 -->
{#if isOpen}
	<div class="ai-panel">
		<!-- 标题栏 -->
		<div class="panel-header">
			<div class="header-left">
				<div class="status-indicator {ollamaAvailable ? 'status-online' : 'status-offline'}"></div>
				<h3 class="panel-title">AI 助手</h3>
			</div>
			<div class="header-actions">
				<button onclick={clearConversation} class="icon-btn" title="清空对话">
					<svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
					</svg>
				</button>
				<button onclick={toggleAssistant} class="icon-btn">
					<svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
					</svg>
				</button>
			</div>
		</div>

		<!-- 消息列表 -->
		<div class="messages-container">
			{#if messages.length === 0}
				<div class="empty-state">
					<p class="greeting">👋 你好！我可以帮你：</p>
					<ul class="features-list">
						<li>• 切换和管理库</li>
						<li>• 搜索和打开书籍</li>
						<li>• 搜索和播放音乐</li>
					</ul>
					<p class="examples-title">试试下面的示例：</p>
					<div class="examples-list">
						{#each examples as example}
							<button onclick={() => useExample(example)} class="example-btn">
								{example}
							</button>
						{/each}
					</div>
				</div>
			{:else}
				{#each messages as msg}
					<div class="message-row {msg.role === 'user' ? 'message-user' : 'message-assistant'}">
						<div class="message-bubble {msg.role === 'user' ? 'bubble-user' : msg.isError ? 'bubble-error' : 'bubble-assistant'}">
							<p class="message-text">{msg.content}</p>
						</div>
					</div>
				{/each}
			{/if}

			{#if isLoading}
				<div class="message-row message-assistant">
					<div class="message-bubble bubble-assistant">
						<div class="typing-indicator">
							<div class="typing-dot"></div>
							<div class="typing-dot"></div>
							<div class="typing-dot"></div>
						</div>
					</div>
				</div>
			{/if}
		</div>

		<!-- 输入框 -->
		<div class="input-container">
			<form onsubmit={(e) => { e.preventDefault(); sendMessage(); }} class="input-form">
				<input
					type="text"
					bind:value={userInput}
					placeholder={ollamaAvailable ? "输入你的请求..." : "Ollama 未运行"}
					disabled={!ollamaAvailable || isLoading}
					class="text-input"
				/>
				<button
					type="submit"
					disabled={!ollamaAvailable || isLoading || !userInput.trim()}
					class="send-btn"
				>
					<svg class="icon" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
					</svg>
				</button>
			</form>
		</div>
	</div>
{/if}

<style>
	/* AI 助手按钮 */
	.ai-button {
		position: fixed !important;
		bottom: 24px !important;
		right: 24px !important;
		width: 56px !important;
		height: 56px !important;
		background-color: #3b82f6 !important;
		color: white !important;
		border: none !important;
		border-radius: 50% !important;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15) !important;
		display: flex !important;
		align-items: center !important;
		justify-content: center !important;
		cursor: pointer !important;
		transition: all 0.2s ease !important;
		z-index: 99999 !important;
		opacity: 1 !important;
		visibility: visible !important;
		pointer-events: auto !important;
	}

	.ai-button:hover {
		background-color: #2563eb;
		transform: scale(1.05);
		box-shadow: 0 6px 16px rgba(0, 0, 0, 0.2);
	}

	.ai-icon {
		width: 24px;
		height: 24px;
	}

	/* AI 助手面板 */
	.ai-panel {
		position: fixed !important;
		bottom: 96px !important;
		right: 24px !important;
		width: 384px !important;
		height: 600px !important;
		background-color: white !important;
		border-radius: 12px !important;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15) !important;
		display: flex !important;
		flex-direction: column !important;
		z-index: 99998 !important;
		border: 1px solid #e5e7eb !important;
		opacity: 1 !important;
		visibility: visible !important;
	}

	/* 标题栏 */
	.panel-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px;
		border-bottom: 1px solid #e5e7eb;
	}

	.header-left {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.status-indicator {
		width: 12px;
		height: 12px;
		border-radius: 50%;
	}

	.status-online {
		background-color: #10b981;
	}

	.status-offline {
		background-color: #ef4444;
	}

	.panel-title {
		font-size: 14px;
		font-weight: 600;
		color: #111827;
		margin: 0;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.icon-btn {
		padding: 4px;
		background: none;
		border: none;
		border-radius: 4px;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.icon-btn:hover {
		background-color: #f3f4f6;
	}

	.icon {
		width: 20px;
		height: 20px;
		color: #6b7280;
	}

	/* 消息容器 */
	.messages-container {
		flex: 1;
		overflow-y: auto;
		padding: 16px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	/* 空状态 */
	.empty-state {
		text-align: center;
		color: #6b7280;
		margin-top: 32px;
	}

	.greeting {
		margin-bottom: 16px;
		font-size: 14px;
	}

	.features-list {
		font-size: 13px;
		list-style: none;
		padding: 0;
		margin: 0 auto;
		max-width: 240px;
		text-align: left;
	}

	.features-list li {
		margin-bottom: 8px;
	}

	.examples-title {
		margin-top: 24px;
		margin-bottom: 8px;
		font-size: 12px;
	}

	.examples-list {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.example-btn {
		padding: 8px 12px;
		font-size: 12px;
		text-align: left;
		background-color: #f3f4f6;
		border: none;
		border-radius: 6px;
		cursor: pointer;
		transition: background-color 0.2s;
	}

	.example-btn:hover {
		background-color: #e5e7eb;
	}

	/* 消息行 */
	.message-row {
		display: flex;
	}

	.message-user {
		justify-content: flex-end;
	}

	.message-assistant {
		justify-content: flex-start;
	}

	/* 消息气泡 */
	.message-bubble {
		max-width: 80%;
		padding: 10px 14px;
		border-radius: 12px;
	}

	.bubble-user {
		background-color: #3b82f6;
		color: white;
	}

	.bubble-assistant {
		background-color: #f3f4f6;
		color: #111827;
	}

	.bubble-error {
		background-color: #fee2e2;
		color: #991b1b;
	}

	.message-text {
		font-size: 13px;
		margin: 0;
		white-space: pre-wrap;
		word-wrap: break-word;
	}

	/* 输入指示器 */
	.typing-indicator {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.typing-dot {
		width: 8px;
		height: 8px;
		background-color: #9ca3af;
		border-radius: 50%;
		animation: bounce 1.4s infinite ease-in-out both;
	}

	.typing-dot:nth-child(1) {
		animation-delay: -0.32s;
	}

	.typing-dot:nth-child(2) {
		animation-delay: -0.16s;
	}

	@keyframes bounce {
		0%, 80%, 100% {
			transform: scale(0);
		}
		40% {
			transform: scale(1);
		}
	}

	/* 输入容器 */
	.input-container {
		padding: 16px;
		border-top: 1px solid #e5e7eb;
	}

	.input-form {
		display: flex;
		gap: 8px;
	}

	.text-input {
		flex: 1;
		padding: 10px 14px;
		border: 1px solid #d1d5db;
		border-radius: 8px;
		font-size: 14px;
		outline: none;
		transition: border-color 0.2s;
	}

	.text-input:focus {
		border-color: #3b82f6;
		box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
	}

	.text-input:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.send-btn {
		padding: 10px 14px;
		background-color: #3b82f6;
		color: white;
		border: none;
		border-radius: 8px;
		cursor: pointer;
		transition: background-color 0.2s;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.send-btn:hover:not(:disabled) {
		background-color: #2563eb;
	}

	.send-btn:disabled {
		background-color: #9ca3af;
		cursor: not-allowed;
	}
</style>
