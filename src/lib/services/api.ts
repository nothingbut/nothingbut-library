import { invoke } from '@tauri-apps/api/core';
import type { Book, Category, Chapter } from '../types';

export interface ImportPreview {
	title: string;
	author: string;
	category: string;
	chapters: Array<{
		chapter_number: number;
		title: string;
		word_count: number;
	}>;
	total_chapters: number;
	total_words: number;
}

/**
 * Preview import - show first 3 chapters without importing
 */
export async function previewImport(
	filePath: string,
	title: string,
	author: string,
	category: string
): Promise<ImportPreview> {
	return await invoke('preview_import', {
		file_path: filePath,
		title,
		author,
		category
	});
}

/**
 * Complete import flow: parse → save files → insert to DB
 */
export async function importNovel(
	workspacePath: string,
	filePath: string,
	title: string,
	author?: string,
	description?: string,
	categoryId?: number
): Promise<number> {
	return await invoke('import_novel', {
		workspace_path: workspacePath,
		file_path: filePath,
		title,
		author,
		description,
		category_id: categoryId
	});
}

/**
 * List all books
 */
export async function listBooks(): Promise<Book[]> {
	return await invoke('list_books');
}

/**
 * List chapters by book_id
 */
export async function listChapters(bookId: number): Promise<Chapter[]> {
	return await invoke('list_chapters', {
		book_id: bookId
	});
}

/**
 * Create a new category
 */
export async function createCategory(
	name: string,
	parentId?: number,
	sortOrder: number = 0
): Promise<number> {
	return await invoke('create_category', {
		name,
		parent_id: parentId,
		sort_order: sortOrder
	});
}

/**
 * List all categories
 */
export async function listCategories(): Promise<Category[]> {
	return await invoke('list_categories');
}
