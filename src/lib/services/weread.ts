import { invoke } from '@tauri-apps/api/core';
import type {
  WereadAccount,
  WereadBook,
  WereadChapter,
  WereadDownload,
  LoginStatus
} from '$lib/types/weread';

// --- 登录 ---

export async function getAccount(): Promise<WereadAccount | null> {
  return await invoke('weread_get_account');
}

export async function checkCookie(): Promise<LoginStatus> {
  return await invoke('weread_check_cookie');
}

export async function saveLogin(cookiesJson: string): Promise<WereadAccount> {
  return await invoke('weread_save_login', { cookiesJson });
}

export async function logout(): Promise<void> {
  return await invoke('weread_logout');
}

export async function openLogin(): Promise<void> {
  return await invoke('weread_open_login');
}

// --- 书籍 ---

export async function syncBooks(libraryId: number): Promise<WereadBook[]> {
  return await invoke('weread_sync_books', { libraryId });
}

export async function listBooks(libraryId: number): Promise<WereadBook[]> {
  return await invoke('weread_list_books', { libraryId });
}

export async function searchBooks(libraryId: number, query: string): Promise<WereadBook[]> {
  return await invoke('weread_search_books', { libraryId, query });
}

export async function getBookDetail(libraryId: number, bookId: number): Promise<WereadChapter[]> {
  return await invoke('weread_get_book_detail', { libraryId, bookId });
}

// --- 下载/导出 ---

export async function startExport(libraryId: number, bookId: number, outputDir: string): Promise<void> {
  return await invoke('weread_start_export', { libraryId, bookId, outputDir });
}

export async function getExportProgress(libraryId: number, bookId: number): Promise<WereadDownload | null> {
  return await invoke('weread_get_export_progress', { libraryId, bookId });
}

export async function listDownloads(libraryId: number): Promise<WereadDownload[]> {
  return await invoke('weread_list_downloads', { libraryId });
}

// --- 工具 ---

export function formatWordCount(count: number): string {
  if (count >= 10000) {
    return `${(count / 10000).toFixed(1)}万字`;
  }
  return `${count}字`;
}
