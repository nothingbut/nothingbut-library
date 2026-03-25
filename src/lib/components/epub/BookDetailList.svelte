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
	 * Format file size from bytes to human-readable format
	 */
	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
	}

	/**
	 * Format date to Chinese locale format (YYYY-MM-DD)
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
			console.warn(`Failed to format date ${dateStr}:`, e);
			return dateStr;
		}
	}

	/**
	 * Convert rating number (0-5) to star representation
	 * Returns "-" if no rating
	 */
	function getRatingStars(rating: number | null): string {
		if (rating === null || rating === undefined || rating < 0 || rating > 5) {
			return '-';
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

<div class="overflow-x-auto rounded-lg border border-gray-200 bg-white">
	<table class="w-full text-sm">
		<thead class="border-b border-gray-200 bg-gray-50">
			<tr>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">封面</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">标题</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">作者</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">系列</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">出版社</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">评分</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">标签</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">添加日期</th>
				<th class="px-4 py-3 text-left font-semibold text-gray-900">大小</th>
			</tr>
		</thead>
		<tbody class="divide-y divide-gray-200">
			{#each books as book (book.id)}
				<tr
					class="cursor-pointer transition hover:bg-gray-50"
					onclick={() => onSelect(book)}
					role="button"
					tabindex="0"
					onkeydown={(e) => {
						if (e.key === 'Enter' || e.key === ' ') {
							onSelect(book);
						}
					}}
				>
					<!-- Cover thumbnail -->
					<td class="px-4 py-3">
						<div class="h-16 w-12 overflow-hidden rounded bg-gray-200">
							<img
								src={getCoverUrl(book)}
								alt={book.title}
								class="h-full w-full object-cover"
								onerror={handleImageError}
							/>
						</div>
					</td>

					<!-- Title -->
					<td class="px-4 py-3">
						<div class="line-clamp-1" title={book.title}>
							{book.title}
						</div>
					</td>

					<!-- Authors -->
					<td class="px-4 py-3">
						<div class="line-clamp-1 text-gray-600">作者信息加载中...</div>
					</td>

					<!-- Series -->
					<td class="px-4 py-3">
						{#if book.series}
							<div class="line-clamp-1">
								{book.series}
								{#if book.series_index !== null}#{book.series_index}{/if}
							</div>
						{:else}
							<div class="text-gray-400">-</div>
						{/if}
					</td>

					<!-- Publisher -->
					<td class="px-4 py-3">
						{#if book.publisher}
							<div class="line-clamp-1">{book.publisher}</div>
						{:else}
							<div class="text-gray-400">-</div>
						{/if}
					</td>

					<!-- Rating -->
					<td class="px-4 py-3">
						<div class="text-yellow-500">{getRatingStars(book.rating)}</div>
					</td>

					<!-- Tags -->
					<td class="px-4 py-3">
						<div class="text-gray-400">-</div>
					</td>

					<!-- Created date -->
					<td class="px-4 py-3 whitespace-nowrap">
						<div class="text-gray-600">{formatDate(book.created_at)}</div>
					</td>

					<!-- File size -->
					<td class="px-4 py-3 whitespace-nowrap">
						<div class="text-gray-600">{formatFileSize(book.file_size)}</div>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>
