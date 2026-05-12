<script lang="ts">
	import { onMount } from 'svelte';
	import type { EpubBook, EpubBookWithDetails, SearchQuery, ViewMode } from '$lib/types/epub';
	import type { Library, CreateLibraryRequest } from '$lib/types/library';
	import { EpubService } from '$lib/services/epub';
	import { listLibraries, getCurrentLibrary, setCurrentLibrary, createLibrary } from '$lib/services/library';
	import BookGrid from './BookGrid.svelte';
	import BookList from './BookList.svelte';
	import BookDetailList from './BookDetailList.svelte';
	import SearchBar from './SearchBar.svelte';
	import BookSidebar from './BookSidebar.svelte';
	import EpubImportDialog from './EpubImportDialog.svelte';

	// State
	let books = $state.raw<EpubBook[]>([]);
	let booksWithDetails = $state.raw<EpubBookWithDetails[]>([]);
	let selectedBook = $state<EpubBook | null>(null);
	let viewMode = $state<ViewMode>('grid');
	let loading = $state(false);
	let error = $state<string | null>(null);
	let showImportDialog = $state(false);

	// Library state
	let libraries = $state<Library[]>([]);
	let currentLibrary = $state<Library | null>(null);

	// Create library dialog state
	let showCreateLibraryDialog = $state(false);
	let newLibraryName = $state('');
	let newLibraryPath = $state('');
	let newLibraryDescription = $state('');

	/**
	 * Load all books from the EPUB library
	 */
	async function loadBooks(): Promise<void> {
		if (!currentLibrary) return;

		loading = true;
		error = null;
		try {
			// Load books with details for all views
			booksWithDetails = await EpubService.listBooksWithDetails(currentLibrary.id);
			// Extract simple books for grid and list views
			books = booksWithDetails.map((b) => b.book);
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

	/**
	 * Handle library switch
	 */
	async function handleLibrarySwitch(event: Event): Promise<void> {
		const target = event.target as HTMLSelectElement;
		const libraryId = Number(target.value);

		if (libraryId === currentLibrary?.id) return;

		try {
			await setCurrentLibrary(libraryId);
			currentLibrary = libraries.find(lib => lib.id === libraryId) || null;
			await loadBooks();
		} catch (e) {
			console.error('切换库失败:', e);
			error = e instanceof Error ? e.message : '切换库失败';
		}
	}

	/**
	 * Open create library dialog
	 */
	function openCreateLibraryDialog(): void {
		newLibraryName = '';
		newLibraryPath = `/epub-${Date.now()}`;
		newLibraryDescription = '';
		showCreateLibraryDialog = true;
	}

	/**
	 * Handle create library
	 */
	async function handleCreateLibrary(): Promise<void> {
		if (!newLibraryName.trim()) {
			alert('请输入库名称');
			return;
		}

		try {
			loading = true;
			const libraryId = await createLibrary({
				name: newLibraryName,
				moduleType: 'epub',
				storagePath: newLibraryPath,
				description: newLibraryDescription || undefined
			});

			libraries = await listLibraries('epub');
			await setCurrentLibrary(libraryId);
			currentLibrary = libraries.find(lib => lib.id === libraryId) || null;
			await loadBooks();

			showCreateLibraryDialog = false;
			alert(`成功创建库：${newLibraryName}`);
		} catch (e) {
			console.error('创建库失败:', e);
			alert(`创建库失败: ${e instanceof Error ? e.message : String(e)}`);
		} finally {
			loading = false;
		}
	}

	onMount(async () => {
		try {
			libraries = await listLibraries('epub');
			currentLibrary = await getCurrentLibrary('epub');
			if (currentLibrary) {
				await loadBooks();
			}
		} catch (err) {
			console.error('加载库失败:', err);
			error = '加载库失败';
		}
	});
</script>

<div class="epub-library">
	<!-- Library Selector -->
	{#if libraries.length > 0 && currentLibrary}
		<div class="library-selector">
			<label for="library-select">当前库：</label>
			<select
				id="library-select"
				value={currentLibrary.id}
				onchange={handleLibrarySwitch}
			>
				{#each libraries as lib}
					<option value={lib.id}>{lib.name}</option>
				{/each}
			</select>
			<button class="create-library-btn" onclick={openCreateLibraryDialog}>
				➕ 新建库
			</button>
		</div>
	{/if}

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
					<BookDetailList books={booksWithDetails} onSelect={handleBookSelect} />
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
		libraryId={currentLibrary?.id ?? 1}
		onClose={() => (showImportDialog = false)}
		onSuccess={handleImportSuccess}
	/>

	<!-- Create Library Dialog -->
	{#if showCreateLibraryDialog}
		<div class="dialog-overlay" onclick={() => showCreateLibraryDialog = false} role="presentation">
			<div class="dialog-content" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
				<div class="dialog-header">
					<h3>创建新的 EPUB 库</h3>
					<button class="dialog-close" onclick={() => showCreateLibraryDialog = false} type="button">✕</button>
				</div>
				<div class="dialog-body">
					<div class="form-field">
						<label for="library-name">库名称 *</label>
						<input
							id="library-name"
							type="text"
							bind:value={newLibraryName}
							placeholder="例如：工作电子书库"
						/>
					</div>
					<div class="form-field">
						<label for="library-path">存储路径</label>
						<input
							id="library-path"
							type="text"
							bind:value={newLibraryPath}
							placeholder="/epub"
						/>
						<span class="field-hint">相对于工作空间的路径</span>
					</div>
					<div class="form-field">
						<label for="library-desc">描述（可选）</label>
						<textarea
							id="library-desc"
							bind:value={newLibraryDescription}
							placeholder="库的简短描述"
							rows="3"
						></textarea>
					</div>
				</div>
				<div class="dialog-footer">
					<button class="btn-secondary" onclick={() => showCreateLibraryDialog = false} type="button">
						取消
					</button>
					<button class="btn-primary" onclick={handleCreateLibrary} disabled={loading} type="button">
						{loading ? '创建中...' : '创建'}
					</button>
				</div>
			</div>
		</div>
	{/if}
</div>

<style>
	.epub-library {
		display: flex;
		flex-direction: column;
		height: 100%;
		overflow: hidden;
		background-color: var(--color-bg-secondary);
	}

	/* Library Selector */
	.library-selector {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		background-color: var(--color-bg-primary);
		border-bottom: 1px solid var(--color-border);
	}

	.library-selector label {
		font-size: 13px;
		color: var(--color-text-secondary);
		white-space: nowrap;
	}

	.library-selector select {
		flex: 0 0 auto;
		min-width: 200px;
		padding: 4px 8px;
		font-size: 13px;
		color: var(--color-text-primary);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: 4px;
		cursor: pointer;
	}

	.library-selector select:hover {
		border-color: var(--color-primary);
	}

	.create-library-btn {
		margin-left: auto;
		padding: 4px 12px;
		font-size: 13px;
		color: var(--color-primary);
		background-color: transparent;
		border: 1px solid var(--color-primary);
		border-radius: 4px;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.create-library-btn:hover {
		background-color: var(--color-primary);
		color: white;
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

	/* Create Library Dialog */
	.dialog-overlay {
		position: fixed;
		inset: 0;
		z-index: 1000;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: rgba(0, 0, 0, 0.5);
		backdrop-filter: blur(4px);
	}

	.dialog-content {
		width: 90%;
		max-width: 500px;
		background-color: var(--color-bg-primary);
		border-radius: 12px;
		box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.3);
		overflow: hidden;
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 20px 24px;
		border-bottom: 1px solid var(--color-border);
	}

	.dialog-header h3 {
		margin: 0;
		font-size: 18px;
		font-weight: 600;
		color: var(--color-text-primary);
	}

	.dialog-close {
		padding: 4px 8px;
		font-size: 20px;
		color: var(--color-text-secondary);
		background: none;
		border: none;
		cursor: pointer;
		transition: color 0.2s;
	}

	.dialog-close:hover {
		color: var(--color-text-primary);
	}

	.dialog-body {
		padding: 24px;
		display: flex;
		flex-direction: column;
		gap: 20px;
	}

	.form-field {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.form-field label {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text-primary);
	}

	.form-field input,
	.form-field textarea {
		padding: 10px 12px;
		font-size: 14px;
		color: var(--color-text-primary);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
		border-radius: 6px;
		transition: border-color 0.2s;
	}

	.form-field input:focus,
	.form-field textarea:focus {
		outline: none;
		border-color: var(--color-primary);
	}

	.form-field textarea {
		resize: vertical;
		min-height: 80px;
	}

	.field-hint {
		font-size: 12px;
		color: var(--color-text-tertiary);
	}

	.dialog-footer {
		display: flex;
		gap: 12px;
		justify-content: flex-end;
		padding: 16px 24px;
		background-color: var(--color-bg-secondary);
		border-top: 1px solid var(--color-border);
	}

	.btn-secondary,
	.btn-primary {
		padding: 8px 16px;
		font-size: 14px;
		font-weight: 500;
		border-radius: 6px;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid;
	}

	.btn-secondary {
		color: var(--color-text-primary);
		background-color: var(--color-bg-primary);
		border-color: var(--color-border);
	}

	.btn-secondary:hover {
		background-color: var(--color-bg-hover);
	}

	.btn-primary {
		color: white;
		background-color: var(--color-primary);
		border-color: var(--color-primary);
	}

	.btn-primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.btn-primary:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
