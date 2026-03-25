export type BookStatus = 'completed' | 'ongoing' | 'abandoned';

export interface Category {
	id: number;
	name: string;
	parent_id: number | null;
	sort_order: number;
	created_at: string;
}

export interface CategoryNode extends Category {
	children: CategoryNode[];
	expanded: boolean;
}

export interface Book {
	id: number;
	title: string;
	author: string | null;
	description: string | null;
	cover_path: string | null;
	category_id: number | null;
	book_dir: string;
	file_size: number;
	word_count: number;
	chapter_count: number;
	status: BookStatus;
	reading_progress: number;
	last_read_at: string | null;
	created_at: string;
	updated_at: string;
}

export interface Chapter {
	id: number;
	book_id: number;
	title: string;
	preview?: string; // First line preview (up to 20 chars) - optional for backwards compatibility
	file_path: string;
	sort_order: number;
	word_count: number;
	created_at: string;
}

// EPUB Library types
export * from './types/epub';
