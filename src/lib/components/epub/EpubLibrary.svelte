<script lang="ts">
	import { onMount } from 'svelte';
	import type { EpubBook, SearchQuery, ViewMode } from '$lib/types/epub';
	import { EpubService } from '$lib/services/epub';
	import BookGrid from './BookGrid.svelte';
	import BookList from './BookList.svelte';
	import BookDetailList from './BookDetailList.svelte';
	import SearchBar from './SearchBar.svelte';
	import BookSidebar from './BookSidebar.svelte';

	// State
	let books: EpubBook[] = $state([]);
	let selectedBook: EpubBook | null = $state(null);
	let viewMode: ViewMode = $state('grid');
	let loading: boolean = $state(false);
	let error: string | null = $state(null);

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
	 * Handle import button click (placeholder)
	 */
	function handleImport(): void {
		// TODO: Open import dialog (implement in next phase)
	}

	onMount(() => {
		loadBooks();
	});
</script>

<div class="flex h-screen flex-col bg-gray-50">
	<!-- Header -->
	<div class="border-b border-gray-200 bg-white px-6 py-4 shadow-sm">
		<div class="mb-4 flex items-center justify-between">
			<!-- Title -->
			<h1 class="text-2xl font-bold text-gray-900">EPUB 书库</h1>

			<!-- Right side controls -->
			<div class="flex items-center gap-3">
				<!-- Import button -->
				<button
					onclick={handleImport}
					class="rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 transition-colors"
					title="导入书籍"
				>
					📥 导入书籍
				</button>

				<!-- View mode buttons -->
				<div class="flex gap-1 rounded-lg border border-gray-300 bg-gray-50 p-1">
					<button
						onclick={() => (viewMode = 'grid')}
						class={`rounded px-3 py-1 transition-colors ${
							viewMode === 'grid'
								? 'bg-white shadow-sm text-gray-900'
								: 'text-gray-600 hover:text-gray-900'
						}`}
						title="网格视图"
					>
						⊞
					</button>
					<button
						onclick={() => (viewMode = 'list')}
						class={`rounded px-3 py-1 transition-colors ${
							viewMode === 'list'
								? 'bg-white shadow-sm text-gray-900'
								: 'text-gray-600 hover:text-gray-900'
						}`}
						title="列表视图"
					>
						☰
					</button>
					<button
						onclick={() => (viewMode = 'detail')}
						class={`rounded px-3 py-1 transition-colors ${
							viewMode === 'detail'
								? 'bg-white shadow-sm text-gray-900'
								: 'text-gray-600 hover:text-gray-900'
						}`}
						title="详细视图"
					>
						≡
					</button>
				</div>
			</div>
		</div>

		<!-- Search bar -->
		<SearchBar onSearch={handleSearch} />
	</div>

	<!-- Main content area -->
	<div class="flex flex-1 overflow-hidden">
		<!-- Books display area -->
		<div class="flex-1 overflow-auto bg-gray-50 p-6">
			{#if loading}
				<div class="flex h-full items-center justify-center">
					<span class="text-lg text-gray-500">加载中...</span>
				</div>
			{:else if error}
				<div class="flex h-full items-center justify-center">
					<div class="rounded-lg bg-red-50 p-6 text-center">
						<p class="text-red-700">{error}</p>
					</div>
				</div>
			{:else if books.length === 0}
				<div class="flex h-full items-center justify-center">
					<div class="text-center">
						<p class="mb-2 text-lg font-semibold text-gray-600">暂无书籍</p>
						<p class="text-gray-500">点击"导入书籍"开始添加</p>
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
</div>
