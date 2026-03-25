/**
 * EPUB Library TypeScript Type Definitions
 * Matches Rust models in src-tauri/src/modules/epub/models.rs
 */

/** EPUB 书籍 */
export interface EpubBook {
	id: number;
	title: string;
	sort_title: string | null;
	isbn: string | null;
	publisher: string | null;
	pubdate: string | null;
	language: string | null;
	series: string | null;
	series_index: number | null;
	rating: number | null;
	file_path: string;
	file_size: number;
	cover_path: string | null;
	description: string | null;
	created_at: string;
	updated_at: string;
}

/** 作者 */
export interface Author {
	id: number;
	name: string;
	sort_name: string | null;
	created_at: string;
}

/** 标签 */
export interface Tag {
	id: number;
	name: string;
	created_at: string;
}

/** 书籍-作者关联 */
export interface BookAuthor {
	book_id: number;
	author_id: number;
	author_order: number;
}

/** 书籍-标签关联 */
export interface BookTag {
	book_id: number;
	tag_id: number;
}

/** 自定义字段类型 */
export type CustomFieldType =
	| 'text'
	| 'series'
	| 'enumeration'
	| 'number'
	| 'rating'
	| 'date'
	| 'bool'
	| 'comments';

/** 自定义字段定义 */
export interface CustomField {
	id: number;
	name: string;
	label: string;
	datatype: CustomFieldType;
	is_multiple: boolean;
	display_order: number;
	created_at: string;
}

/** 自定义字段值 */
export interface CustomFieldValue {
	book_id: number;
	field_id: number;
	value: string;
}

/** 阅读进度 */
export interface ReadingProgress {
	book_id: number;
	chapter_href: string;
	progress_percent: number;
	updated_at: string;
}

/** 书签 */
export interface Bookmark {
	id: number;
	book_id: number;
	chapter_href: string;
	cfi: string;
	note: string | null;
	created_at: string;
}

/** 高亮 */
export interface Highlight {
	id: number;
	book_id: number;
	chapter_href: string;
	cfi_range: string;
	text: string;
	color: string;
	note: string | null;
	created_at: string;
}

/** EPUB 章节信息 */
export interface EpubChapter {
	href: string;
	title: string;
	level: number;
	order_index: number;
}

/** 导入结果 */
export type ImportResult =
	| {
			type: 'success';
			book_id: number;
	  }
	| {
			type: 'failed';
			file_path: string;
			error: string;
	  };

/** 导入进度 */
export interface ImportProgress {
	current: number;
	total: number;
	file_name: string;
}

/** 搜索查询 */
export interface SearchQuery {
	keyword?: string | null;
	title?: string | null;
	author?: string | null;
	publisher?: string | null;
	isbn?: string | null;
	series?: string | null;
	tags?: string[] | null;
	rating_min?: number | null;
	rating_max?: number | null;
	sort_by?: string | null;
	sort_order?: string | null;
	limit?: number | null;
	offset?: number | null;
}

/** 书籍详情（包含作者和标签） */
export interface EpubBookWithDetails {
	book: EpubBook;
	authors: Author[];
	tags: Tag[];
}

/** 视图模式 */
export type ViewMode = 'grid' | 'list' | 'detail';
