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
	libraryId: number,
	workspacePath: string,
	filePath: string,
	title: string,
	author?: string,
	description?: string,
	categoryId?: number,
	sourceSite?: string
): Promise<number> {
	return await invoke('import_novel', {
		libraryId,
		workspacePath,
		filePath,
		title,
		author,
		description,
		categoryId,
		sourceSite
	});
}

/**
 * List all books in a specific library
 */
export async function listBooks(libraryId: number): Promise<Book[]> {
	return await invoke('list_books', { libraryId });
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
	libraryId: number,
	name: string,
	parentId?: number,
	sortOrder: number = 0
): Promise<number> {
	return await invoke('create_category', {
		libraryId,
		name,
		parentId,
		sortOrder
	});
}

/**
 * List all categories for a specific library
 */
export async function listCategories(libraryId: number): Promise<Category[]> {
	return await invoke('list_categories', { libraryId });
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

/**
 * Fetch book metadata from source website
 */
export async function fetchBookMetadata(
	workspacePath: string,
	bookId: number | null,
	sourceSite: string,
	title: string,
	author?: string
): Promise<{
	description?: string;
	author?: string;
	category?: string;
	coverPath?: string;
	coverUrl?: string;
}> {
	return await invoke('fetch_book_metadata', {
		workspacePath,
		bookId,
		sourceSite,
		title,
		author
	});
}

/**
 * Delete a book
 */
export async function deleteBook(
	libraryId: number,
	workspacePath: string,
	bookId: number
): Promise<void> {
	return await invoke('delete_book', {
		libraryId,
		workspacePath,
		bookId
	});
}
