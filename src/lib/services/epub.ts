import { invoke } from '@tauri-apps/api/core';
import type {
	EpubBook,
	EpubBookWithDetails,
	SearchQuery,
	ImportResult,
} from '$lib/types/epub';

/**
 * EPUB Library API Service
 * Provides type-safe interface for calling EPUB-related Tauri commands
 */
export class EpubService {
	/**
	 * 导入单个 EPUB 文件
	 * @param workspacePath - 工作空间路径
	 * @param sourceFilePath - EPUB 文件路径
	 * @returns 新导入的书籍 ID
	 */
	static async importEpub(
		workspacePath: string,
		sourceFilePath: string
	): Promise<number> {
		return await invoke<number>('import_epub', {
			workspacePath,
			sourceFilePath,
		});
	}

	/**
	 * 批量导入 EPUB 文件
	 * @param workspacePath - 工作空间路径
	 * @param filePaths - EPUB 文件路径数组
	 * @returns 导入结果数组
	 */
	static async batchImportEpub(
		workspacePath: string,
		filePaths: string[]
	): Promise<ImportResult[]> {
		return await invoke<ImportResult[]>('batch_import_epub', {
			workspacePath,
			filePaths,
		});
	}

	/**
	 * 获取书籍详情
	 * @param bookId - 书籍 ID
	 * @returns 书籍详情（包含作者和标签），未找到时返回 null
	 */
	static async getBook(bookId: number): Promise<EpubBookWithDetails | null> {
		return await invoke<EpubBookWithDetails | null>('get_epub_book', {
			bookId,
		});
	}

	/**
	 * 列出所有书籍
	 * @returns 书籍数组
	 */
	static async listBooks(): Promise<EpubBook[]> {
		return await invoke<EpubBook[]>('list_epub_books');
	}

	/**
	 * 搜索书籍
	 * @param query - 搜索查询条件
	 * @returns 匹配的书籍数组
	 */
	static async searchBooks(query: SearchQuery): Promise<EpubBook[]> {
		return await invoke<EpubBook[]>('search_epub_books', { query });
	}

	/**
	 * 删除书籍
	 * @param workspacePath - 工作空间路径
	 * @param bookId - 书籍 ID
	 */
	static async deleteBook(
		workspacePath: string,
		bookId: number
	): Promise<void> {
		await invoke('delete_epub_book', { workspacePath, bookId });
	}
}
