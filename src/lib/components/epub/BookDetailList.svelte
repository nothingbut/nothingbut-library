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
	 * Format file size from bytes
	 */
	function formatFileSize(bytes: number): string {
		if (bytes === 0) return '0 B';
		const k = 1024;
		const sizes = ['B', 'KB', 'MB', 'GB'];
		const i = Math.floor(Math.log(bytes) / Math.log(k));
		return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
	}

	/**
	 * Format date to Chinese locale format
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
			return dateStr;
		}
	}

	/**
	 * Convert rating number (0-5) to star representation
	 */
	function getRatingStars(rating: number | null): string {
		if (rating === null || rating === undefined || rating < 0 || rating > 5) {
			return '-';
		}
		const filledStars = Math.round(rating);
		const emptyStars = 5 - filledStars;
		return '★'.repeat(filledStars) + '☆'.repeat(emptyStars);
	}
</script>

<div class="detail-list">
	<table class="detail-table">
		<thead>
			<tr>
				<th class="col-cover">封面</th>
				<th class="col-title">标题</th>
				<th class="col-series">系列</th>
				<th class="col-publisher">出版社</th>
				<th class="col-pubdate">出版日期</th>
				<th class="col-rating">评分</th>
				<th class="col-date">添加日期</th>
				<th class="col-size">大小</th>
			</tr>
		</thead>
		<tbody>
			{#each books as book (book.id)}
				<tr class="detail-row" onclick={() => onSelect(book)}>
					<!-- Cover thumbnail -->
					<td class="col-cover">
						<div class="cover-thumb">
							{#if getCoverUrl(book)}
								<img src={getCoverUrl(book)} alt={book.title} class="cover-image" />
							{:else}
								<div class="cover-placeholder">
									{getInitial(book.title)}
								</div>
							{/if}
						</div>
					</td>

					<!-- Title -->
					<td class="col-title">
						<div class="title-cell" title={book.title}>
							{book.title}
						</div>
					</td>

					<!-- Series -->
					<td class="col-series">
						{#if book.series}
							<div class="series-cell">
								{book.series}
								{#if book.series_index !== null}#{book.series_index}{/if}
							</div>
						{:else}
							<span class="empty-cell">-</span>
						{/if}
					</td>

					<!-- Publisher -->
					<td class="col-publisher">
						{#if book.publisher}
							<div class="publisher-cell">{book.publisher}</div>
						{:else}
							<span class="empty-cell">-</span>
						{/if}
					</td>

					<!-- Pubdate -->
					<td class="col-pubdate">
						{#if book.pubdate}
							<span class="date-cell">{book.pubdate}</span>
						{:else}
							<span class="empty-cell">-</span>
						{/if}
					</td>

					<!-- Rating -->
					<td class="col-rating">
						<span class="rating-cell">{getRatingStars(book.rating)}</span>
					</td>

					<!-- Created date -->
					<td class="col-date">
						<span class="date-cell">{formatDate(book.created_at)}</span>
					</td>

					<!-- File size -->
					<td class="col-size">
						<span class="size-cell">{formatFileSize(book.file_size)}</span>
					</td>
				</tr>
			{/each}
		</tbody>
	</table>
</div>

<style>
	.detail-list {
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		overflow: hidden;
	}

	.detail-table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}

	thead {
		background-color: var(--color-bg-secondary);
		border-bottom: 1px solid var(--color-border);
		position: sticky;
		top: 0;
		z-index: 1;
	}

	th {
		padding: 12px 16px;
		text-align: left;
		font-weight: 600;
		color: var(--color-text-primary);
		white-space: nowrap;
	}

	.detail-row {
		border-bottom: 1px solid var(--color-border);
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.detail-row:last-child {
		border-bottom: none;
	}

	.detail-row:hover {
		background-color: var(--color-bg-hover);
	}

	td {
		padding: 12px 16px;
		color: var(--color-text-primary);
	}

	/* Column widths */
	.col-cover {
		width: 80px;
	}

	.col-title {
		min-width: 200px;
		max-width: 300px;
	}

	.col-series {
		width: 150px;
	}

	.col-publisher {
		width: 150px;
	}

	.col-pubdate {
		width: 100px;
	}

	.col-rating {
		width: 100px;
	}

	.col-date {
		width: 100px;
	}

	.col-size {
		width: 80px;
	}

	/* Cover */
	.cover-thumb {
		width: 48px;
		height: 72px;
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
		font-size: 24px;
		font-weight: 600;
		color: var(--color-text-tertiary);
		background: linear-gradient(135deg, var(--color-bg-secondary) 0%, var(--color-bg-hover) 100%);
	}

	/* Cell content */
	.title-cell {
		overflow: hidden;
		text-overflow: ellipsis;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		-webkit-box-orient: vertical;
		font-weight: 500;
		line-height: 1.4;
	}

	.series-cell,
	.publisher-cell {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--color-text-secondary);
	}

	.date-cell,
	.size-cell {
		white-space: nowrap;
		color: var(--color-text-secondary);
		font-size: 12px;
	}

	.rating-cell {
		color: #fbbf24;
		font-size: 14px;
	}

	.empty-cell {
		color: var(--color-text-tertiary);
	}
</style>
