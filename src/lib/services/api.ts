import { invoke } from '@tauri-apps/api/core';
import type { Book, Category, Chapter } from '../types';

export interface ImportPreview {
	title: string;
	author?: string | null; // Auto-extracted from file if found
	description?: string | null; // Auto-extracted from file if found
	category: string;
	chapters: Array<{
		chapter_number: number;
		title: string;
		preview: string; // First line preview
		word_count: number;
	}>;
	total_chapters: number;
	total_words: number;
}

/**
 * Preview import - show first 3 chapters without importing
 * Author and description are auto-extracted if found in the file
 */
export async function previewImport(
	filePath: string,
	title: string,
	category: string
): Promise<ImportPreview> {
	return await invoke('preview_import', {
		filePath,
		title,
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
		workspacePath,
		filePath,
		title,
		author,
		description,
		categoryId
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
		bookId
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
		parentId,
		sortOrder
	});
}

/**
 * List all categories
 */
export async function listCategories(): Promise<Category[]> {
	return await invoke('list_categories');
}

/**
 * Get chapter content
 */
export async function getChapterContent(
	workspacePath: string,
	chapterId: number
): Promise<string> {
	return await invoke('get_chapter_content', {
		workspacePath,
		chapterId
	});
}
