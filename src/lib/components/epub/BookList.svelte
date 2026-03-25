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
	 * Falls back to placeholder if cover is not available
	 */
	function getCoverUrl(book: EpubBook): string {
		if (book.cover_path) {
			try {
				return convertFileSrc(book.cover_path);
			} catch (e) {
				console.warn(`Failed to convert cover path for book ${book.id}:`, e);
				return '/placeholder-cover.svg';
			}
		}
		return '/placeholder-cover.svg';
	}

	/**
	 * Format author information
	 * Note: Authors are not directly available in EpubBook type.
	 * This is a placeholder that will be populated once EpubBookWithDetails is used.
	 */
	function formatAuthors(book: EpubBook): string {
		// TODO: Get authors from EpubBookWithDetails when available
		// For now, return placeholder
		return '作者信息加载中...';
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
	 * Handle image loading errors by falling back to placeholder
	 */
	function handleImageError(event: Event): void {
		const img = event.target as HTMLImageElement;
		if (img.src !== '/placeholder-cover.svg') {
			img.src = '/placeholder-cover.svg';
		}
	}
</script>

<div class="space-y-4">
	{#each books as book (book.id)}
		<button
			class="flex gap-4 rounded-lg bg-white p-4 shadow transition hover:shadow-lg"
			onclick={() => onSelect(book)}
			title={book.title}
		>
			<!-- Cover thumbnail -->
			<div class="h-32 w-24 flex-shrink-0 overflow-hidden rounded bg-gray-200">
				<img
					src={getCoverUrl(book)}
					alt={book.title}
					class="h-full w-full object-cover"
					onerror={handleImageError}
				/>
			</div>

			<!-- Book information -->
			<div class="flex flex-1 flex-col justify-between">
				<!-- Title -->
				<h3 class="line-clamp-2 font-semibold text-gray-900" title={book.title}>
					{book.title}
				</h3>

				<!-- Authors -->
				<p class="line-clamp-1 text-sm text-gray-600">
					{formatAuthors(book)}
				</p>

				<!-- Series information -->
				{#if book.series}
					<p class="line-clamp-1 text-xs text-gray-500">
						{book.series}
						{#if book.series_index !== null}#{book.series_index}{/if}
					</p>
				{/if}

				<!-- Tags placeholder -->
				<p class="line-clamp-1 text-xs text-gray-400">标签信息加载中...</p>

				<!-- Rating stars -->
				{#if book.rating && book.rating > 0}
					<div class="mt-2 text-sm text-yellow-500">
						{getRatingStars(book.rating)}
					</div>
				{/if}
			</div>

			<!-- Metadata (right side) -->
			<div class="flex flex-shrink-0 flex-col items-end justify-start gap-2 text-right text-xs text-gray-600">
				{#if book.publisher}
					<div class="line-clamp-1 max-w-32">
						<span class="font-medium">出版社:</span>
						{book.publisher}
					</div>
				{/if}
				{#if book.pubdate}
					<div class="whitespace-nowrap">
						<span class="font-medium">出版日期:</span>
						{book.pubdate}
					</div>
				{/if}
			</div>
		</button>
	{/each}
</div>
