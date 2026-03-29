import { invoke } from '@tauri-apps/api/core';
import { goto } from '$app/navigation';

// ==================== 类型定义 ====================

export interface ChatMessage {
	role: 'system' | 'user' | 'assistant';
	content: string;
}

export interface ToolCall {
	id: string;
	type: string;
	function: {
		name: string;
		arguments: string;
	};
}

export interface ToolResult {
	tool_call_id: string;
	output: string;
}

export interface AssistantResponse {
	message: string;
	tool_calls?: ToolCall[];
	tool_results?: ToolResult[];
	conversation_history: ChatMessage[];
}

// ==================== AI 助手服务 ====================

/**
 * 发送消息给 AI 助手
 */
export async function assistantChat(
	userMessage: string,
	conversationHistory?: ChatMessage[]
): Promise<AssistantResponse> {
	return await invoke<AssistantResponse>('assistant_chat', {
		userMessage,
		conversationHistory
	});
}

/**
 * 处理 AI 助手的响应（执行导航等操作）
 */
export function handleAssistantAction(response: AssistantResponse): boolean {
	let hasAction = false;

	// 检查主消息中是否有导航指令
	try {
		const action = JSON.parse(response.message);
		if (action.action === 'navigate' && action.route) {
			console.log('🚀 AI Assistant: Navigating to:', action.route);

			// 使用 setTimeout 确保在下一个事件循环执行，避免被其他操作阻塞
			setTimeout(() => {
				console.log('🎯 Executing navigation to:', action.route);
				// 使用原生 location 替换确保导航成功
				window.location.href = action.route;
			}, 100);

			hasAction = true;
		}
	} catch (e) {
		// 主消息不是 JSON，继续检查工具结果
	}

	// 检查工具结果中是否有导航指令
	if (response.tool_results) {
		for (const result of response.tool_results) {
			try {
				const action = JSON.parse(result.output);

				// 处理导航动作
				if (action.action === 'navigate' && action.route) {
					console.log('🚀 AI Assistant: Navigating to:', action.route);

					// 使用 setTimeout 确保在下一个事件循环执行
					setTimeout(() => {
						console.log('🎯 Executing navigation to:', action.route);
						window.location.href = action.route;
					}, 100);

					hasAction = true;
					break;
				}

				// 处理播放音乐动作
				if (action.action === 'play_music' && action.track_id) {
					console.log('Playing track:', action.track_id);
					// TODO: 实现音乐播放逻辑
					// 可以发送自定义事件给音乐播放器组件
					window.dispatchEvent(
						new CustomEvent('play-track', {
							detail: { trackId: action.track_id }
						})
					);
					hasAction = true;
					break;
				}
			} catch (e) {
				// 如果不是 JSON，忽略
				continue;
			}
		}
	}

	return hasAction;
}

/**
 * 格式化对话历史用于显示
 */
export function formatConversationHistory(history: ChatMessage[]): string {
	return history
		.filter((msg) => msg.role !== 'system')
		.map((msg) => {
			const role = msg.role === 'user' ? '用户' : 'AI 助手';
			return `${role}: ${msg.content}`;
		})
		.join('\n\n');
}

/**
 * 检查 Ollama 服务状态
 */
export async function checkOllamaStatus(): Promise<boolean> {
	try {
		return await invoke<boolean>('check_ollama_status');
	} catch (error) {
		console.error('Failed to check Ollama status:', error);
		return false;
	}
}
