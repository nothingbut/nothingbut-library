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
 * with comprehensive error handling and input validation
 */
export class EpubService {
	/**
	 * Validates that a string is non-empty
	 * @throws Error if validation fails
	 */
	private static validateNonEmptyString(value: unknown, fieldName: string): asserts value is string {
		if (typeof value !== 'string' || value.trim() === '') {
			throw new Error(`Invalid input: ${fieldName} must be a non-empty string`);
		}
	}

	/**
	 * Validates that a number is positive
	 * @throws Error if validation fails
	 */
	private static validatePositiveNumber(value: unknown, fieldName: string): asserts value is number {
		if (typeof value !== 'number' || value <= 0 || !Number.isInteger(value)) {
			throw new Error(`Invalid input: ${fieldName} must be a positive integer`);
		}
	}

	/**
	 * Validates that an array is non-empty
	 * @throws Error if validation fails
	 */
	private static validateNonEmptyArray<T>(value: unknown, fieldName: string): asserts value is T[] {
		if (!Array.isArray(value) || value.length === 0) {
			throw new Error(`Invalid input: ${fieldName} must be a non-empty array`);
		}
	}

	/**
	 * 导入单个 EPUB 文件
	 * @param workspacePath - 工作空间路径
	 * @param sourceFilePath - EPUB 文件路径
	 * @returns 新导入的书籍 ID
	 * @throws Error if validation fails or import operation fails
	 */
	static async importEpub(
		workspacePath: string,
		sourceFilePath: string
	): Promise<number> {
		try {
			this.validateNonEmptyString(workspacePath, 'workspacePath');
			this.validateNonEmptyString(sourceFilePath, 'sourceFilePath');

			return await invoke<number>('import_epub', {
				workspacePath,
				sourceFilePath,
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to import EPUB file';
			throw new Error(`Failed to import EPUB file: ${message}`);
		}
	}

	/**
	 * 批量导入 EPUB 文件
	 * @param workspacePath - 工作空间路径
	 * @param filePaths - EPUB 文件路径数组
	 * @returns 导入结果数组
	 * @throws Error if validation fails or import operation fails
	 */
	static async batchImportEpub(
		workspacePath: string,
		filePaths: string[]
	): Promise<ImportResult[]> {
		try {
			this.validateNonEmptyString(workspacePath, 'workspacePath');
			this.validateNonEmptyArray<string>(filePaths, 'filePaths');

			return await invoke<ImportResult[]>('batch_import_epub', {
				workspacePath,
				filePaths,
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to batch import EPUB files';
			throw new Error(`Failed to batch import EPUB files: ${message}`);
		}
	}

	/**
	 * 获取书籍详情
	 * @param bookId - 书籍 ID
	 * @returns 书籍详情（包含作者和标签），未找到时返回 null
	 * @throws Error if validation fails or fetch operation fails
	 */
	static async getBook(bookId: number): Promise<EpubBookWithDetails | null> {
		try {
			this.validatePositiveNumber(bookId, 'bookId');

			return await invoke<EpubBookWithDetails | null>('get_epub_book', {
				bookId,
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to fetch book details';
			throw new Error(`Failed to fetch book details: ${message}`);
		}
	}

	/**
	 * 列出所有书籍
	 * @returns 书籍数组
	 * @throws Error if operation fails
	 */
	static async listBooks(): Promise<EpubBook[]> {
		try {
			return await invoke<EpubBook[]>('list_epub_books');
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to fetch books';
			throw new Error(`Failed to fetch books: ${message}`);
		}
	}

	/**
	 * 搜索书籍
	 * @param query - 搜索查询条件
	 * @returns 匹配的书籍数组
	 * @throws Error if operation fails
	 */
	static async searchBooks(query: SearchQuery): Promise<EpubBook[]> {
		try {
			if (!query || typeof query !== 'object') {
				throw new Error('Invalid input: query must be a valid search object');
			}

			return await invoke<EpubBook[]>('search_epub_books', { query });
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to search books';
			throw new Error(`Failed to search books: ${message}`);
		}
	}

	/**
	 * 删除书籍
	 * @param workspacePath - 工作空间路径
	 * @param bookId - 书籍 ID
	 * @throws Error if validation fails or delete operation fails
	 */
	static async deleteBook(
		workspacePath: string,
		bookId: number
	): Promise<void> {
		try {
			this.validateNonEmptyString(workspacePath, 'workspacePath');
			this.validatePositiveNumber(bookId, 'bookId');

			await invoke('delete_epub_book', { workspacePath, bookId });
		} catch (error) {
			const message = error instanceof Error ? error.message : 'Failed to delete book';
			throw new Error(`Failed to delete book: ${message}`);
		}
	}
}
