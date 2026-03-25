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
	 * Falls back to placeholder if cover is not available
	 */
	function getCoverUrl(coverPath: string | null): string {
		if (coverPath) {
			try {
				return convertFileSrc(coverPath);
			} catch (e) {
				console.warn(`Failed to convert cover path for book ${book.id}:`, e);
				return '/placeholder-cover.svg';
			}
		}
		return '/placeholder-cover.svg';
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

	/**
	 * Convert rating number (0-5) to star representation
	 * Returns "未评分" if no rating
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
	 * Format file size specifically to MB with 2 decimals
	 */
	function formatFileSizeMB(bytes: number): string {
		const mb = bytes / (1024 * 1024);
		return mb.toFixed(2) + ' MB';
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
			return '日期格式错误';
		}
	}

	/**
	 * Handle delete action with confirmation dialog
	 */
	async function handleDelete(): Promise<void> {
		// Show confirmation dialog
		if (!confirm('确定要删除这本书籍吗？此操作无法撤销。')) {
			return;
		}

		// Validate workspace is selected
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
	 * Handle start reading action (placeholder)
	 */
	function handleStartReading(): void {
		// TODO: Implement reading functionality (Week 4)
		console.log('Start reading:', book.id);
	}

	/**
	 * Handle save metadata action
	 */
	async function handleSaveMetadata(updatedData: any): Promise<void> {
		try {
			// TODO: Call save API (placeholder for now)
			// await EpubService.updateBook(updatedData.book, updatedData.authors, updatedData.tags);
			console.log('Saving metadata:', updatedData);

			// Reload book details
			await loadBookDetails();

			// Exit edit mode
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

<!-- Sidebar container -->
<div class="flex h-full w-96 flex-col border-l border-gray-200 bg-white shadow-lg">
	<!-- Header -->
	<div class="flex items-center justify-between border-b border-gray-200 px-6 py-4">
		<h2 class="text-lg font-bold text-gray-900">书籍详情</h2>
		<button
			onclick={onClose}
			class="text-gray-400 hover:text-gray-600"
			aria-label="Close sidebar"
			title="关闭"
		>
			✕
		</button>
	</div>

	<!-- Content area (scrollable) -->
	<div class="flex-1 overflow-y-auto px-6 py-4">
		{#if loading}
			<div class="flex items-center justify-center py-8">
				<span class="text-gray-500">加载中...</span>
			</div>
		{:else if error}
			<div class="rounded-md bg-red-50 p-3">
				<p class="text-sm text-red-800">{error}</p>
			</div>
		{:else if bookDetails}
			<div class="space-y-4">
				<!-- Cover image -->
				<div class="flex justify-center">
					<div class="h-80 w-56 overflow-hidden rounded-lg shadow-md">
						<img
							src={getCoverUrl(bookDetails.book.cover_path)}
							alt={bookDetails.book.title}
							class="h-full w-full object-cover"
							onerror={handleImageError}
						/>
					</div>
				</div>

				<!-- Title -->
				<div>
					<h3 class="text-xl font-bold text-gray-900">{bookDetails.book.title}</h3>
				</div>

				<!-- Sort title -->
				{#if bookDetails.book.sort_title}
					<div>
						<p class="text-sm text-gray-600">
							<span class="font-semibold">排序标题:</span> {bookDetails.book.sort_title}
						</p>
					</div>
				{/if}

				<!-- Authors -->
				<div>
					<p class="text-sm text-gray-600">
						<span class="font-semibold">作者:</span>
						{bookDetails.authors.length > 0
							? bookDetails.authors.map((a) => a.name).join(', ')
							: '-'}
					</p>
				</div>

				<!-- Series -->
				{#if bookDetails.book.series}
					<div>
						<p class="text-sm text-gray-600">
							<span class="font-semibold">系列:</span> {bookDetails.book.series}
							{#if bookDetails.book.series_index !== null}
								#{bookDetails.book.series_index}
							{/if}
						</p>
					</div>
				{/if}

				<!-- Publisher and publication date -->
				<div>
					<p class="text-sm text-gray-600">
						<span class="font-semibold">出版社:</span>
						{bookDetails.book.publisher || '-'}
					</p>
					{#if bookDetails.book.pubdate}
						<p class="text-sm text-gray-600">
							<span class="font-semibold">出版日期:</span> {bookDetails.book.pubdate}
						</p>
					{/if}
				</div>

				<!-- ISBN -->
				{#if bookDetails.book.isbn}
					<div>
						<p class="text-sm text-gray-600">
							<span class="font-semibold">ISBN:</span> {bookDetails.book.isbn}
						</p>
					</div>
				{/if}

				<!-- Language -->
				{#if bookDetails.book.language}
					<div>
						<p class="text-sm text-gray-600">
							<span class="font-semibold">语言:</span> {bookDetails.book.language}
						</p>
					</div>
				{/if}

				<!-- Rating -->
				<div>
					<p class="text-sm text-gray-600">
						<span class="font-semibold">评分:</span>
						<span class="text-yellow-500">{getRatingStars(bookDetails.book.rating)}</span>
					</p>
				</div>

				<!-- Tags -->
				{#if bookDetails.tags.length > 0}
					<div>
						<p class="mb-2 text-sm font-semibold text-gray-600">标签:</p>
						<div class="flex flex-wrap gap-2">
							{#each bookDetails.tags as tag (tag.id)}
								<span
									class="rounded-full bg-blue-100 px-3 py-1 text-sm font-medium text-blue-800"
								>
									{tag.name}
								</span>
							{/each}
						</div>
					</div>
				{/if}

				<!-- Description -->
				{#if bookDetails.book.description}
					<div>
						<p class="mb-2 text-sm font-semibold text-gray-600">简介:</p>
						<p class="whitespace-pre-wrap text-sm text-gray-700">{bookDetails.book.description}</p>
					</div>
				{/if}

				<!-- File size -->
				<div>
					<p class="text-sm text-gray-600">
						<span class="font-semibold">文件大小:</span> {formatFileSizeMB(bookDetails.book.file_size)}
					</p>
				</div>

				<!-- Created date -->
				<div>
					<p class="text-sm text-gray-600">
						<span class="font-semibold">添加日期:</span> {formatDate(bookDetails.book.created_at)}
					</p>
				</div>
			</div>
		{:else}
			<div class="rounded-md bg-yellow-50 p-3">
				<p class="text-sm text-yellow-800">无法加载书籍详情</p>
			</div>
		{/if}
	</div>

	<!-- Footer buttons -->
	<div class="space-y-2 border-t border-gray-200 px-6 py-4">
		{#if editMode && bookDetails}
			<MetadataEditor
				book={bookDetails.book}
				authors={bookDetails.authors}
				tags={bookDetails.tags}
				onSave={handleSaveMetadata}
				onCancel={() => (editMode = false)}
			/>
		{:else}
			<!-- Start reading button -->
			<button
				onclick={handleStartReading}
				class="w-full rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 transition-colors"
				disabled={loading || !bookDetails}
			>
				开始阅读
			</button>

			<!-- Edit button -->
			<button
				onclick={() => (editMode = !editMode)}
				class="w-full rounded-lg bg-gray-100 px-4 py-2 font-medium text-gray-900 hover:bg-gray-200 transition-colors"
				disabled={loading || !bookDetails}
			>
				{editMode ? '取消编辑' : '编辑'}
			</button>

			<!-- Delete button -->
			<button
				onclick={handleDelete}
				class="w-full rounded-lg border border-red-600 px-4 py-2 font-medium text-red-600 hover:bg-red-50 transition-colors"
				disabled={loading}
			>
				删除书籍
			</button>
		{/if}
	</div>
</div>
