import { writable } from 'svelte/store';
import type { Book, Category, Chapter } from '../types';

export const books = writable<Book[]>([]);
export const categories = writable<Category[]>([]);
export const currentBook = writable<Book | null>(null);
export const currentChapter = writable<Chapter | null>(null);

export function setBooks(bookList: Book[]) {
	books.set(bookList);
}

export function setCategories(categoryList: Category[]) {
	categories.set(categoryList);
}

export function setCurrentBook(book: Book | null) {
	currentBook.set(book);
}

export function setCurrentChapter(chapter: Chapter | null) {
	currentChapter.set(chapter);
}

export function addBook(book: Book) {
	books.update((list) => [...list, book]);
}

export function updateBook(book: Book) {
	books.update((list) => list.map((b) => (b.id === book.id ? book : b)));
}

export function removeBook(bookId: string) {
	books.update((list) => list.filter((b) => b.id !== bookId));
}

export function addCategory(category: Category) {
	categories.update((list) => [...list, category]);
}

export function updateCategory(category: Category) {
	categories.update((list) => list.map((c) => (c.id === category.id ? category : c)));
}

export function removeCategory(categoryId: string) {
	categories.update((list) => list.filter((c) => c.id !== categoryId));
}
