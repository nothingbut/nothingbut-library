export interface Category {
	id: string;
	name: string;
	parentId: string | null;
	sortOrder: number;
	createdAt: Date;
}

export interface CategoryNode extends Category {
	children: CategoryNode[];
	expanded: boolean;
}

export interface Book {
	id: string;
	title: string;
	author: string;
	description: string;
	coverUrl: string;
	categoryId: string;
	createdAt: Date;
	updatedAt: Date;
}

export interface Chapter {
	id: string;
	bookId: string;
	title: string;
	content: string;
	sortOrder: number;
	createdAt: Date;
	updatedAt: Date;
}
