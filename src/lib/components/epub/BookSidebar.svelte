<script lang="ts">
	import { onMount } from 'svelte';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import type { EpubBook, EpubBookWithDetails } from '$lib/types/epub';
	import { EpubService } from '$lib/services/epub';
	import { currentWorkspace } from '$lib/stores/workspace';
	import MetadataEditor from './MetadataEditor.svelte';

	interface Props {
		book: EpubBook;
		onClose: () => void;
		onDeleted: () => void;
	}

	let { book, onClose, onDeleted }: Props = $props();

	let bookDetails: EpubBookWithDetails | null = $state(null);
	let editMode: boolean = $state(false);
	let loading: boolean = $state(true);
	let error: string | null = $state(null);

	/**
	 * Load full book details on component mount
	 */
	async function loadBookDetails(): Promise<void> {
		loading = true;
		error = null;
		try {
			bookDetails = await EpubService.getBook(book.id);
			if (!bookDetails) {
				error = '无法加载书籍详情';
			}
		} catch (err) {
			const message = err instanceof Error ? err.message : '加载失败';
			error = `加载失败: ${message}`;
			console.error('Failed to load book details:', err);
		} finally {
			loading = false;
		}
	}

	/**
	 * Convert a file path to a usable Tauri asset URL
	 */
	function getCoverUrl(coverPath: string | null): string {
		if (coverPath) {
			try {
				return convertFileSrc(coverPath);
			} catch (e) {
				console.warn(`Failed to convert cover path for book ${book.id}:`, e);
				return '';
			}
		}
		return '';
	}

	/**
	 * Get first letter for placeholder
	 */
	function getInitial(title: string): string {
		return title.charAt(0).toUpperCase();
	}

	/**
	 * Convert rating number (0-5) to star representation
	 */
	function getRatingStars(rating: number | null): string {
		if (rating === null || rating < 0 || rating > 5) {
			return '未评分';
		}
		const filledStars = Math.round(rating);
		const emptyStars = 5 - filledStars;
		return '★'.repeat(filledStars) + '☆'.repeat(emptyStars);
	}

	/**
	 * Format file size to MB
	 */
	function formatFileSizeMB(bytes: number): string {
		const mb = bytes / (1024 * 1024);
		return mb.toFixed(2) + ' MB';
	}

	/**
	 * Format date
	 */
	function formatDate(dateStr: string): string {
		try {
			const date = new Date(dateStr);
			return date.toLocaleDateString('zh-CN', {
				year: 'numeric',
				month: '2-digit',
				day: '2-digit'
			});
		} catch (e) {
			return '日期格式错误';
		}
	}

	/**
	 * Handle delete action with confirmation
	 */
	async function handleDelete(): Promise<void> {
		if (!confirm('确定要删除这本书籍吗？此操作无法撤销。')) {
			return;
		}

		const workspace = $currentWorkspace;
		if (!workspace) {
			error = '未选择工作空间';
			return;
		}

		try {
			await EpubService.deleteBook(workspace.path, book.id);
			onDeleted();
		} catch (err) {
			const message = err instanceof Error ? err.message : '删除失败';
			error = `删除失败: ${message}`;
			console.error('Failed to delete book:', err);
		}
	}

	/**
	 * Handle start reading
	 */
	function handleStartReading(): void {
		// TODO: Implement reading functionality
		console.log('Start reading:', book.id);
	}

	/**
	 * Handle save metadata
	 */
	async function handleSaveMetadata(updatedData: any): Promise<void> {
		try {
			await EpubService.updateMetadata(book.id, updatedData.book);
			await EpubService.setAuthors(book.id, updatedData.authors);
			await EpubService.setTags(book.id, updatedData.tags);
			await loadBookDetails();
			editMode = false;
		} catch (err) {
			const message = err instanceof Error ? err.message : '保存失败';
			error = `保存失败: ${message}`;
			console.error('Failed to save metadata:', err);
			throw err;
		}
	}

	onMount(() => {
		loadBookDetails();
	});
</script>

<div class="sidebar">
	<!-- Header -->
	<div class="sidebar-header">
		<h2 class="sidebar-title">书籍详情</h2>
		<button onclick={onClose} class="close-btn" title="关闭">
			✕
		</button>
	</div>

	<!-- Content -->
	<div class="sidebar-content">
		{#if loading}
			<div class="loading-state">
				<span class="loading-text">加载中...</span>
			</div>
		{:else if error}
			<div class="error-state">
				<p class="error-text">{error}</p>
			</div>
		{:else if bookDetails}
			{#if editMode}
				<MetadataEditor
					book={bookDetails.book}
					authors={bookDetails.authors}
					tags={bookDetails.tags}
					onSave={handleSaveMetadata}
					onCancel={() => (editMode = false)}
				/>
			{:else}
				<div class="details-container">
					<!-- Cover -->
					<div class="cover-section">
						{#if getCoverUrl(bookDetails.book.cover_path)}
							<img
								src={getCoverUrl(bookDetails.book.cover_path)}
								alt={bookDetails.book.title}
								class="cover-image"
							/>
						{:else}
							<div class="cover-placeholder">
								{getInitial(bookDetails.book.title)}
							</div>
						{/if}
					</div>

					<!-- Title -->
					<h3 class="book-title">{bookDetails.book.title}</h3>

					<!-- Details grid -->
					<div class="details-grid">
						<!-- Sort title -->
						{#if bookDetails.book.sort_title}
							<div class="detail-item">
								<span class="detail-label">排序标题:</span>
								<span class="detail-value">{bookDetails.book.sort_title}</span>
							</div>
						{/if}

						<!-- Authors -->
						<div class="detail-item">
							<span class="detail-label">作者:</span>
							<span class="detail-value">
								{bookDetails.authors.length > 0
									? bookDetails.authors.map((a) => a.name).join(', ')
									: '-'}
							</span>
						</div>

						<!-- Series -->
						{#if bookDetails.book.series}
							<div class="detail-item">
								<span class="detail-label">系列:</span>
								<span class="detail-value">
									{bookDetails.book.series}
									{#if bookDetails.book.series_index !== null}
										#{bookDetails.book.series_index}
									{/if}
								</span>
							</div>
						{/if}

						<!-- Publisher -->
						<div class="detail-item">
							<span class="detail-label">出版社:</span>
							<span class="detail-value">{bookDetails.book.publisher || '-'}</span>
						</div>

						<!-- Pubdate -->
						{#if bookDetails.book.pubdate}
							<div class="detail-item">
								<span class="detail-label">出版日期:</span>
								<span class="detail-value">{bookDetails.book.pubdate}</span>
							</div>
						{/if}

						<!-- ISBN -->
						{#if bookDetails.book.isbn}
							<div class="detail-item">
								<span class="detail-label">ISBN:</span>
								<span class="detail-value">{bookDetails.book.isbn}</span>
							</div>
						{/if}

						<!-- Language -->
						{#if bookDetails.book.language}
							<div class="detail-item">
								<span class="detail-label">语言:</span>
								<span class="detail-value">{bookDetails.book.language}</span>
							</div>
						{/if}

						<!-- Rating -->
						<div class="detail-item">
							<span class="detail-label">评分:</span>
							<span class="rating-stars">{getRatingStars(bookDetails.book.rating)}</span>
						</div>

						<!-- File size -->
						<div class="detail-item">
							<span class="detail-label">文件大小:</span>
							<span class="detail-value">{formatFileSizeMB(bookDetails.book.file_size)}</span>
						</div>

						<!-- Created date -->
						<div class="detail-item">
							<span class="detail-label">添加日期:</span>
							<span class="detail-value">{formatDate(bookDetails.book.created_at)}</span>
						</div>
					</div>

					<!-- Tags -->
					{#if bookDetails.tags.length > 0}
						<div class="tags-section">
							<div class="section-label">标签:</div>
							<div class="tags-list">
								{#each bookDetails.tags as tag (tag.id)}
									<span class="tag-badge">{tag.name}</span>
								{/each}
							</div>
						</div>
					{/if}

					<!-- Description -->
					{#if bookDetails.book.description}
						<div class="description-section">
							<div class="section-label">简介:</div>
							<p class="description-text">{bookDetails.book.description}</p>
						</div>
					{/if}
				</div>
			{/if}
		{:else}
			<div class="warning-state">
				<p class="warning-text">无法加载书籍详情</p>
			</div>
		{/if}
	</div>

	<!-- Footer buttons -->
	{#if !editMode}
		<div class="sidebar-footer">
			<button
				onclick={handleStartReading}
				class="action-btn primary"
				disabled={loading || !bookDetails}
			>
				开始阅读
			</button>

			<button
				onclick={() => (editMode = true)}
				class="action-btn secondary"
				disabled={loading || !bookDetails}
			>
				编辑
			</button>

			<button
				onclick={handleDelete}
				class="action-btn danger"
				disabled={loading}
			>
				删除书籍
			</button>
		</div>
	{/if}
</div>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		width: 400px;
		height: 100%;
		background-color: var(--color-bg-primary);
		border-left: 1px solid var(--color-border);
		box-shadow: -2px 0 8px rgba(0, 0, 0, 0.05);
	}

	/* Header */
	.sidebar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.sidebar-title {
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
	}

	.close-btn {
		font-size: 20px;
		color: var(--color-text-tertiary);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px;
		transition: color 0.2s ease;
	}

	.close-btn:hover {
		color: var(--color-text-primary);
	}

	/* Content */
	.sidebar-content {
		flex: 1;
		overflow-y: auto;
		padding: 20px;
	}

	/* States */
	.loading-state,
	.error-state,
	.warning-state {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 32px 16px;
	}

	.loading-text {
		color: var(--color-text-secondary);
		font-size: 14px;
	}

	.error-state {
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 6px;
		padding: 16px;
	}

	.error-text {
		color: #dc2626;
		font-size: 13px;
		margin: 0;
	}

	.warning-state {
		background-color: #fffbeb;
		border: 1px solid #fde68a;
		border-radius: 6px;
		padding: 16px;
	}

	.warning-text {
		color: #d97706;
		font-size: 13px;
		margin: 0;
	}

	/* Details */
	.details-container {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.cover-section {
		display: flex;
		justify-content: center;
	}

	.cover-image {
		width: 200px;
		height: 300px;
		object-fit: cover;
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
	}

	.cover-placeholder {
		width: 200px;
		height: 300px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 80px;
		font-weight: 600;
		color: var(--color-text-tertiary);
		background: linear-gradient(135deg, var(--color-bg-secondary) 0%, var(--color-bg-hover) 100%);
		border-radius: 8px;
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
	}

	.book-title {
		font-size: 20px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
		line-height: 1.4;
	}

	.details-grid {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.detail-item {
		display: flex;
		gap: 8px;
		font-size: 13px;
		line-height: 1.5;
	}

	.detail-label {
		font-weight: 500;
		color: var(--color-text-secondary);
		flex-shrink: 0;
	}

	.detail-value {
		color: var(--color-text-primary);
		flex: 1;
	}

	.rating-stars {
		color: #fbbf24;
		font-size: 14px;
	}

	.tags-section,
	.description-section {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.section-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.tags-list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.tag-badge {
		display: inline-block;
		padding: 4px 12px;
		border-radius: 16px;
		background-color: #dbeafe;
		color: #1e40af;
		font-size: 12px;
		font-weight: 500;
	}

	.description-text {
		font-size: 13px;
		color: var(--color-text-primary);
		line-height: 1.6;
		white-space: pre-wrap;
		margin: 0;
	}

	/* Footer */
	.sidebar-footer {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 20px;
		border-top: 1px solid var(--color-border);
		flex-shrink: 0;
	}

	.action-btn {
		width: 100%;
		padding: 10px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid;
	}

	.action-btn.primary {
		background-color: var(--color-primary);
		color: white;
		border-color: var(--color-primary);
	}

	.action-btn.primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.action-btn.secondary {
		background-color: var(--color-bg-secondary);
		color: var(--color-text-primary);
		border-color: var(--color-border);
	}

	.action-btn.secondary:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
	}

	.action-btn.danger {
		background-color: transparent;
		color: #dc2626;
		border-color: #dc2626;
	}

	.action-btn.danger:hover:not(:disabled) {
		background-color: #fef2f2;
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
