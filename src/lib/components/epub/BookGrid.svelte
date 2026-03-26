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
</script>

<div class="book-grid">
	{#each books as book (book.id)}
		<button class="book-card" onclick={() => onSelect(book)} title={book.title}>
			<!-- Cover image -->
			<div class="cover-container">
				{#if getCoverUrl(book)}
					<img src={getCoverUrl(book)} alt={book.title} class="cover-image" />
				{:else}
					<div class="cover-placeholder">
						{getInitial(book.title)}
					</div>
				{/if}
			</div>

			<!-- Book information -->
			<div class="book-info">
				<!-- Title -->
				<h3 class="book-title" title={book.title}>
					{book.title}
				</h3>

				<!-- Publisher -->
				{#if book.publisher}
					<p class="book-publisher">{book.publisher}</p>
				{/if}

				<!-- Series information -->
				{#if book.series}
					<p class="book-series">
						{book.series}
						{#if book.series_index !== null}#{book.series_index}{/if}
					</p>
				{/if}

				<!-- Rating stars -->
				{#if book.rating && book.rating > 0}
					<div class="book-rating">
						{getRatingStars(book.rating)}
					</div>
				{/if}
			</div>
		</button>
	{/each}
</div>

<style>
	.book-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
		gap: 24px;
	}

	.book-card {
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
		border-radius: 8px;
		overflow: hidden;
		border: 1px solid var(--color-border);
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.book-card:hover {
		transform: translateY(-4px);
		box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
		border-color: var(--color-primary);
	}

	.cover-container {
		width: 100%;
		aspect-ratio: 2/3;
		background-color: var(--color-bg-secondary);
		position: relative;
		overflow: hidden;
	}

	.cover-image {
		width: 100%;
		height: 100%;
		object-fit: cover;
		transition: transform 0.3s ease;
	}

	.book-card:hover .cover-image {
		transform: scale(1.05);
	}

	.cover-placeholder {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 64px;
		font-weight: 600;
		color: var(--color-text-tertiary);
		background: linear-gradient(135deg, var(--color-bg-secondary) 0%, var(--color-bg-hover) 100%);
	}

	.book-info {
		padding: 12px;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.book-title {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		line-height: 1.4;
		min-height: 2.8em;
	}

	.book-publisher {
		font-size: 12px;
		color: var(--color-text-secondary);
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.book-series {
		font-size: 11px;
		color: var(--color-text-tertiary);
		margin: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.book-rating {
		font-size: 14px;
		color: #fbbf24;
		margin-top: 4px;
	}
</style>
