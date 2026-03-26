<script lang="ts">
	import type { EpubBook } from '$lib/types/epub';
	import { convertFileSrc } from '@tauri-apps/api/core';

	interface Props {
		books: EpubBook[];
		onSelect: (book: EpubBook) => void;
	}

	let { books, onSelect }: Props = $props();

	/**
	 * Convert a file path to a usable Tauri asset URL
	 */
	function getCoverUrl(book: EpubBook): string {
		if (book.cover_path) {
			try {
				return convertFileSrc(book.cover_path);
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
		if (rating === null || rating === undefined || rating < 0 || rating > 5) {
			return '';
		}
		const filledStars = Math.round(rating);
		const emptyStars = 5 - filledStars;
		return '★'.repeat(filledStars) + '☆'.repeat(emptyStars);
	}

	/**
	 * Format file size from bytes
	 */
	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
	}
</script>

<div class="book-list">
	{#each books as book (book.id)}
		<button class="list-item" onclick={() => onSelect(book)}>
			<!-- Cover thumbnail -->
			<div class="cover-thumb">
				{#if getCoverUrl(book)}
					<img src={getCoverUrl(book)} alt={book.title} class="cover-image" />
				{:else}
					<div class="cover-placeholder">
						{getInitial(book.title)}
					</div>
				{/if}
			</div>

			<!-- Book information -->
			<div class="item-content">
				<!-- Title -->
				<h3 class="item-title" title={book.title}>
					{book.title}
				</h3>

				<!-- Meta row -->
				<div class="item-meta">
					{#if book.publisher}
						<span class="meta-item">{book.publisher}</span>
					{/if}
					{#if book.pubdate}
						<span class="meta-separator">•</span>
						<span class="meta-item">{book.pubdate}</span>
					{/if}
					{#if book.file_size}
						<span class="meta-separator">•</span>
						<span class="meta-item">{formatFileSize(book.file_size)}</span>
					{/if}
				</div>

				<!-- Series information -->
				{#if book.series}
					<p class="item-series">
						系列: {book.series}
						{#if book.series_index !== null}#{book.series_index}{/if}
					</p>
				{/if}
			</div>

			<!-- Right side info -->
			<div class="item-actions">
				<!-- Rating stars -->
				{#if book.rating && book.rating > 0}
					<div class="item-rating">
						{getRatingStars(book.rating)}
					</div>
				{/if}
			</div>
		</button>
	{/each}
</div>

<style>
	.book-list {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.list-item {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 16px;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.list-item:hover {
		background-color: var(--color-bg-hover);
		border-color: var(--color-primary);
		box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
	}

	.cover-thumb {
		width: 80px;
		height: 120px;
		flex-shrink: 0;
		background-color: var(--color-bg-secondary);
		border-radius: 4px;
		overflow: hidden;
	}

	.cover-image {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.cover-placeholder {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 36px;
		font-weight: 600;
		color: var(--color-text-tertiary);
		background: linear-gradient(135deg, var(--color-bg-secondary) 0%, var(--color-bg-hover) 100%);
	}

	.item-content {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-width: 0;
	}

	.item-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		text-align: left;
	}

	.item-meta {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
		font-size: 13px;
		color: var(--color-text-secondary);
	}

	.meta-item {
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		max-width: 200px;
	}

	.meta-separator {
		color: var(--color-text-tertiary);
	}

	.item-series {
		font-size: 12px;
		color: var(--color-text-tertiary);
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.item-actions {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
		gap: 8px;
		flex-shrink: 0;
	}

	.item-rating {
		font-size: 16px;
		color: #fbbf24;
	}
</style>
