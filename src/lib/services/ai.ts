import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface StoredMessage {
	id: number;
	conversation_id: number;
	role: 'system' | 'user' | 'assistant';
	content: string;
	timestamp: string;
}

export interface StreamCallbacks {
	onChunk: (chunk: string) => void;
	onDone: () => void;
	onError: (error: string) => void;
}

/**
 * 检查 Ollama 服务状态
 */
export async function checkOllamaStatus(): Promise<boolean> {
	return await invoke('check_ollama_status');
}

/**
 * 测试 Ollama 生成
 */
export async function testOllamaGenerate(prompt: string, model?: string): Promise<string> {
	return await invoke('test_ollama_generate', { prompt, model });
}

/**
 * 创建新对话
 */
export async function createConversation(
	title: string,
	contextType: 'general' | 'book' | 'chapter',
	contextId?: number
): Promise<number> {
	return await invoke('create_conversation', {
		title,
		contextType,
		contextId
	});
}

/**
 * 发送消息并获取 AI 响应（非流式，保留用于测试）
 */
export async function sendMessage(conversationId: number, message: string): Promise<string> {
	return await invoke('send_message', {
		conversationId,
		message
	});
}

/**
 * 发送消息并获取 AI 响应（流式版本）
 * 返回一个取消监听的函数
 */
export async function sendMessageStream(
	conversationId: number,
	message: string,
	callbacks: StreamCallbacks
): Promise<UnlistenFn[]> {
	const unlisteners: UnlistenFn[] = [];

	// 监听流式片段
	const unlistenChunk = await listen<{ conversation_id: number; chunk: string }>(
		'ai-message-chunk',
		(event) => {
			if (event.payload.conversation_id === conversationId) {
				callbacks.onChunk(event.payload.chunk);
			}
		}
	);
	unlisteners.push(unlistenChunk);

	// 监听完成事件
	const unlistenDone = await listen<{ conversation_id: number }>('ai-message-done', (event) => {
		if (event.payload.conversation_id === conversationId) {
			callbacks.onDone();
		}
	});
	unlisteners.push(unlistenDone);

	// 监听错误事件
	const unlistenError = await listen<{ conversation_id: number; error: string }>(
		'ai-message-error',
		(event) => {
			if (event.payload.conversation_id === conversationId) {
				callbacks.onError(event.payload.error);
			}
		}
	);
	unlisteners.push(unlistenError);

	// 发起请求
	try {
		await invoke('send_message_stream', {
			conversationId,
			message
		});
	} catch (error) {
		callbacks.onError(error as string);
		// 清理监听器
		unlisteners.forEach((unlisten) => unlisten());
		throw error;
	}

	return unlisteners;
}

/**
 * 获取对话历史
 */
export async function getConversationHistory(
	conversationId: number,
	limit?: number
): Promise<StoredMessage[]> {
	return await invoke('get_conversation_history', {
		conversationId,
		limit
	});
}

/**
 * 生成章节摘要
 */
export async function summarizeChapter(
	workspacePath: string,
	chapterId: number,
	length: 'short' | 'medium' | 'long' = 'medium'
): Promise<string> {
	return await invoke('summarize_chapter', {
		workspacePath,
		chapterId,
		length
	});
}

/**
 * 为单个章节建立向量索引
 */
export async function indexChapter(
	workspacePath: string,
	chapterId: number
): Promise<void> {
	return await invoke('index_chapter', {
		workspacePath,
		chapterId
	});
}

/**
 * 批量索引整本书的所有章节
 */
export async function indexBook(
	workspacePath: string,
	bookId: number
): Promise<number[]> {
	return await invoke('index_book', {
		workspacePath,
		bookId
	});
}

/**
 * 语义搜索结果
 */
export interface SearchResult {
	chapter_id: number;
	chapter_title: string;
	chapter_number: number;
	book_id: number;
	book_title: string;
	similarity: number; // 0.0 - 1.0
	preview: string;
}

/**
 * 语义搜索
 */
export async function semanticSearch(
	query: string,
	bookId?: number,
	limit: number = 10,
	minSimilarity: number = 0.7
): Promise<SearchResult[]> {
	return await invoke('semantic_search', {
		query,
		bookId,
		limit,
		minSimilarity
	});
}
