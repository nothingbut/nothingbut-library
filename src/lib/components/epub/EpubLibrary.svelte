<script lang="ts">
	import { onMount } from 'svelte';
	import type { EpubBook, SearchQuery, ViewMode } from '$lib/types/epub';
	import { EpubService } from '$lib/services/epub';
	import BookGrid from './BookGrid.svelte';
	import BookList from './BookList.svelte';
	import BookDetailList from './BookDetailList.svelte';
	import SearchBar from './SearchBar.svelte';
	import BookSidebar from './BookSidebar.svelte';
	import EpubImportDialog from './EpubImportDialog.svelte';

	// State
	let books: EpubBook[] = $state([]);
	let selectedBook: EpubBook | null = $state(null);
	let viewMode: ViewMode = $state('grid');
	let loading: boolean = $state(false);
	let error: string | null = $state(null);
	let showImportDialog: boolean = $state(false);

	/**
	 * Load all books from the EPUB library
	 */
	async function loadBooks(): Promise<void> {
		loading = true;
		error = null;
		try {
			books = await EpubService.listBooks();
		} catch (err) {
			const message = err instanceof Error ? err.message : '加载书籍失败';
			error = `加载失败: ${message}`;
			console.error('Failed to load books:', err);
		} finally {
			loading = false;
		}
	}

	/**
	 * Handle book selection
	 */
	function handleBookSelect(book: EpubBook): void {
		selectedBook = book;
	}

	/**
	 * Handle book deselection
	 */
	function handleBookDeselect(): void {
		selectedBook = null;
	}

	/**
	 * Handle book deletion - clear selection and reload
	 */
	async function handleBookDeleted(): Promise<void> {
		selectedBook = null;
		await loadBooks();
	}

	/**
	 * Handle search query
	 */
	async function handleSearch(query: SearchQuery): Promise<void> {
		loading = true;
		error = null;
		try {
			books = await EpubService.searchBooks(query);
		} catch (err) {
			const message = err instanceof Error ? err.message : '搜索失败';
			error = `搜索失败: ${message}`;
			console.error('Failed to search books:', err);
		} finally {
			loading = false;
		}
	}

	/**
	 * Handle import button click
	 */
	function handleImport(): void {
		showImportDialog = true;
	}

	/**
	 * Handle import success
	 */
	async function handleImportSuccess(): Promise<void> {
		await loadBooks();
	}

	onMount(() => {
		loadBooks();
	});
</script>

<div class="epub-library">
	<!-- Header -->
	<div class="library-header">
		<div class="header-controls">
			<!-- Search bar -->
			<div class="search-wrapper">
				<SearchBar onSearch={handleSearch} />
			</div>

			<!-- Right side controls -->
			<div class="header-actions">
				<!-- Import button -->
				<button
					onclick={handleImport}
					class="action-btn primary"
					title="导入书籍"
				>
					📥 导入书籍
				</button>

				<!-- View mode buttons -->
				<div class="view-mode-selector">
					<button
						onclick={() => (viewMode = 'grid')}
						class="view-btn"
						class:active={viewMode === 'grid'}
						title="网格视图"
					>
						⊞
					</button>
					<button
						onclick={() => (viewMode = 'list')}
						class="view-btn"
						class:active={viewMode === 'list'}
						title="列表视图"
					>
						☰
					</button>
					<button
						onclick={() => (viewMode = 'detail')}
						class="view-btn"
						class:active={viewMode === 'detail'}
						title="详细视图"
					>
						≡
					</button>
				</div>
			</div>
		</div>
	</div>

	<!-- Main content area -->
	<div class="library-content">
		<!-- Books display area -->
		<div class="books-area">
			{#if loading}
				<div class="empty-state">
					<span class="loading-text">加载中...</span>
				</div>
			{:else if error}
				<div class="empty-state">
					<div class="error-box">
						<p class="error-text">{error}</p>
					</div>
				</div>
			{:else if books.length === 0}
				<div class="empty-state">
					<div class="empty-message">
						<div class="empty-icon">📚</div>
						<p class="empty-title">暂无书籍</p>
						<p class="empty-hint">点击"导入书籍"开始添加</p>
					</div>
				</div>
			{:else}
				<!-- Dynamic view based on viewMode -->
				{#if viewMode === 'grid'}
					<BookGrid {books} onSelect={handleBookSelect} />
				{:else if viewMode === 'list'}
					<BookList {books} onSelect={handleBookSelect} />
				{:else if viewMode === 'detail'}
					<BookDetailList {books} onSelect={handleBookSelect} />
				{/if}
			{/if}
		</div>

		<!-- Sidebar -->
		{#if selectedBook}
			<BookSidebar book={selectedBook} onClose={handleBookDeselect} onDeleted={handleBookDeleted} />
		{/if}
	</div>

	<!-- Import Dialog -->
	<EpubImportDialog
		bind:isOpen={showImportDialog}
		onClose={() => (showImportDialog = false)}
		onSuccess={handleImportSuccess}
	/>
</div>

<style>
	.epub-library {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		background-color: var(--color-bg-secondary);
	}

	/* Header */
	.library-header {
		background-color: var(--color-bg-primary);
		border-bottom: 1px solid var(--color-border);
		padding: 16px 24px;
		flex-shrink: 0;
	}

	.header-controls {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.search-wrapper {
		flex: 1;
		max-width: 500px;
	}

	.header-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	/* Action button */
	.action-btn {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text-primary);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.action-btn:hover {
		background-color: var(--color-bg-hover);
	}

	.action-btn.primary {
		background-color: var(--color-primary);
		color: white;
		border-color: var(--color-primary);
	}

	.action-btn.primary:hover {
		opacity: 0.9;
	}

	/* View mode selector */
	.view-mode-selector {
		display: flex;
		gap: 4px;
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		padding: 4px;
	}

	.view-btn {
		padding: 6px 12px;
		border-radius: 4px;
		font-size: 16px;
		color: var(--color-text-secondary);
		background-color: transparent;
		border: none;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.view-btn:hover {
		color: var(--color-text-primary);
		background-color: var(--color-bg-hover);
	}

	.view-btn.active {
		color: var(--color-text-primary);
		background-color: var(--color-bg-primary);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	/* Main content */
	.library-content {
		display: flex;
		flex: 1;
		overflow: hidden;
	}

	.books-area {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	/* Empty states */
	.empty-state {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		padding: 48px;
	}

	.loading-text {
		font-size: 16px;
		color: var(--color-text-secondary);
	}

	.error-box {
		background-color: var(--color-bg-primary);
		border: 1px solid #ef4444;
		border-radius: 8px;
		padding: 24px;
		text-align: center;
	}

	.error-text {
		color: #dc2626;
		font-size: 14px;
		margin: 0;
	}

	.empty-message {
		text-align: center;
	}

	.empty-icon {
		font-size: 64px;
		margin-bottom: 16px;
	}

	.empty-title {
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0 0 8px 0;
	}

	.empty-hint {
		font-size: 14px;
		color: var(--color-text-secondary);
		margin: 0;
	}
</style>
