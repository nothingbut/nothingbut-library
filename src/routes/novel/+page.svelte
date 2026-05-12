<script lang="ts">
  import { onMount } from 'svelte';
  import CategoryTree from '$lib/components/CategoryTree.svelte';
  import ImportDialog from '$lib/components/ImportDialog.svelte';
  import Reader from '$lib/components/Reader.svelte';
  import { listBooks, listChapters, getChapterContent, listCategories } from '$lib/services/api';
  import { listLibraries, getCurrentLibrary, setCurrentLibrary, createLibrary } from '$lib/services/library';
  import type { Book, Chapter, BookStatus, Category } from '$lib/types';
  import type { Library } from '$lib/types/library';

  // State
  let selectedBook = $state<Book | null>(null);
  let selectedChapter = $state<Chapter & { content?: string } | null>(null);
  let chapters = $state<Chapter[]>([]);
  let categories = $state<Category[]>([]);
  let categoryMap = $state<Map<number, string>>(new Map());
  let loading = $state(false);
  let error = $state<string | null>(null);
  let showImportDialog = $state(false);
  let categoryTreeKey = $state(0); // For forcing re-render

  // Library state
  let libraries = $state<Library[]>([]);
  let currentLibrary = $state<Library | null>(null);
  let showCreateLibraryDialog = $state(false);
  let newLibraryName = $state('');
  let newLibraryPath = $state('');
  let newLibraryDescription = $state('');

  // Workspace path (hardcoded for now, should come from config)
  const workspacePath = '/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library';

  // Load categories and libraries on mount
  onMount(async () => {
    try {
      // Load libraries first
      console.log('开始加载库列表...');
      libraries = await listLibraries('novel');
      console.log('库列表加载成功，数量:', libraries.length);
      console.log('库列表内容:', JSON.stringify(libraries));

      console.log('开始获取当前库...');
      try {
        currentLibrary = await getCurrentLibrary('novel');
        console.log('当前库:', JSON.stringify(currentLibrary));
        console.log('显示条件:', libraries.length > 0, '&&', currentLibrary !== null);
      } catch (err) {
        console.error('获取当前库失败:', err);
        alert(`获取当前库失败: ${err}`);
        // 如果获取当前库失败，使用第一个库
        currentLibrary = libraries[0] || null;
        console.log('使用第一个库作为当前库:', currentLibrary);
      }

      // Then load categories
      if (currentLibrary) {
        categories = await listCategories(currentLibrary.id);
        // Build category map for quick lookup
        categoryMap = new Map(categories.map(cat => [cat.id, cat.name]));
      }
    } catch (e) {
      console.error('加载数据失败:', e);
      alert(`加载库信息失败: ${e}`);
    }
  });

  // Handlers
  async function handleBookSelect(bookId: number) {
    if (!currentLibrary) {
      error = '请先选择一个库';
      return;
    }

    try {
      loading = true;
      error = null;

      // Load book and chapters from backend
      const [books, chapterList] = await Promise.all([
        listBooks(currentLibrary.id),
        listChapters(bookId)
      ]);

      const book = books.find(b => b.id === bookId);
      if (book) {
        selectedBook = book;
        chapters = chapterList;
        selectedChapter = null;
      } else {
        error = 'Book not found';
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load book';
      console.error('Failed to load book:', e);
    } finally {
      loading = false;
    }
  }

  async function handleChapterSelect(chapterId: number) {
    try {
      loading = true;
      error = null;

      const chapter = chapters.find((ch) => ch.id === chapterId);
      if (!chapter) {
        error = 'Chapter not found';
        return;
      }

      console.log('Loading chapter:', {
        chapterId,
        workspacePath,
        chapter
      });

      // Load chapter content from file
      const content = await getChapterContent(workspacePath, chapterId);

      console.log('Chapter content loaded, length:', content.length);

      selectedChapter = {
        ...chapter,
        content
      };
    } catch (e) {
      console.error('Failed to load chapter - full error:', e);
      console.error('Error type:', typeof e);
      if (e && typeof e === 'object') {
        console.error('Error keys:', Object.keys(e));
        console.error('Error values:', Object.values(e));
        console.error('Error stringified:', JSON.stringify(e, null, 2));
        // Check if it's a Tauri error response
        if ('message' in e) {
          console.error('Error message:', (e as any).message);
          error = (e as any).message;
        } else if ('error' in e) {
          console.error('Error.error:', (e as any).error);
          error = (e as any).error;
        } else {
          error = JSON.stringify(e);
        }
      } else {
        error = e instanceof Error ? e.message : String(e);
      }
    } finally {
      loading = false;
    }
  }

  function getStatusLabel(status: BookStatus): string {
    const labels = {
      completed: '✓ 已完本',
      ongoing: '⏳ 连载中',
      abandoned: '⚠ 已断更',
    };
    return labels[status];
  }

  function getStatusColor(status: BookStatus): string {
    const colors = {
      completed: 'green',
      ongoing: 'orange',
      abandoned: 'red',
    };
    return colors[status];
  }

  function getCategoryName(book: Book): string {
    if (!book.category_id) return '未分类';
    return categoryMap.get(book.category_id) || `未知分类 (${book.category_id})`;
  }

  function getLineCount(chapter: Chapter): number {
    // Estimate line count from word count
    return Math.ceil(chapter.word_count / 15);
  }

  function getFirstLine(content: string): string {
    const lines = content.split('\n').filter(line => line.trim());
    return lines[0] || '';
  }

  // Open import dialog
  function openImportDialog() {
    if (!currentLibrary) {
      alert('请先选择一个库');
      return;
    }
    showImportDialog = true;
  }

  // Handle import success - refresh the tree
  function handleImportSuccess() {
    // Force re-render of CategoryTree by changing key
    categoryTreeKey += 1;
    // Clear selection
    selectedBook = null;
    selectedChapter = null;
    chapters = [];
  }

  // Handle library switch
  async function handleLibrarySwitch(event: Event) {
    const target = event.target as HTMLSelectElement;
    const libraryId = Number(target.value);

    if (libraryId === currentLibrary?.id) return;

    try {
      await setCurrentLibrary(libraryId);
      currentLibrary = libraries.find(lib => lib.id === libraryId) || null;

      // Refresh page data
      categoryTreeKey += 1;
      selectedBook = null;
      selectedChapter = null;
      chapters = [];
      if (currentLibrary) {
        categories = await listCategories(currentLibrary.id);
        categoryMap = new Map(categories.map(cat => [cat.id, cat.name]));
      }
    } catch (e) {
      console.error('Failed to switch library:', e);
      error = e instanceof Error ? e.message : '切换库失败';
    }
  }

  // Open create library dialog
  function openCreateLibraryDialog() {
    newLibraryName = '';
    newLibraryPath = `/novels-${Date.now()}`;
    newLibraryDescription = '';
    showCreateLibraryDialog = true;
  }

  // Create new library
  async function handleCreateLibrary() {
    if (!newLibraryName.trim()) {
      alert('请输入库名称');
      return;
    }

    try {
      loading = true;
      const libraryId = await createLibrary({
        name: newLibraryName,
        moduleType: 'novel',
        storagePath: newLibraryPath,
        description: newLibraryDescription || undefined
      });

      // Reload libraries
      libraries = await listLibraries('novel');

      // Switch to new library
      await setCurrentLibrary(libraryId);
      currentLibrary = libraries.find(lib => lib.id === libraryId) || null;

      // Refresh page data
      categoryTreeKey += 1;
      selectedBook = null;
      selectedChapter = null;
      chapters = [];
      if (currentLibrary) {
        categories = await listCategories(currentLibrary.id);
        categoryMap = new Map(categories.map(cat => [cat.id, cat.name]));
      }

      showCreateLibraryDialog = false;
      alert(`成功创建库：${newLibraryName}`);
    } catch (e) {
      console.error('Failed to create library:', e);
      alert(`创建库失败: ${e instanceof Error ? e.message : String(e)}`);
    } finally {
      loading = false;
    }
  }

</script>

<div class="novel-module">
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
      {#if libraries.length === 1}
        <span class="library-hint">（单库模式）</span>
      {/if}
      <button class="create-library-btn" onclick={openCreateLibraryDialog} title="创建新库">
        ➕ 新建库
      </button>
    </div>
  {/if}

  <!-- Main content wrapper -->
  <div class="main-content">
    <!-- Left Column: Category Tree -->
    <aside class="category-sidebar">
    <div class="sidebar-header">
      <h2>分类</h2>
      <div class="sidebar-actions">
        <button class="action-btn" title="导入小说" onclick={openImportDialog}>
          📥
        </button>
        <button class="action-btn" title="添加分类">+</button>
      </div>
    </div>
    <div class="sidebar-content">
      {#key categoryTreeKey}
        {#if currentLibrary}
          <CategoryTree libraryId={currentLibrary.id} onSelectBook={handleBookSelect} />
        {:else}
          <div class="empty-state">请选择一个库</div>
        {/if}
      {/key}
    </div>
  </aside>

  <!-- Right Column: Content Area -->
  <main class="content-area">
    {#if !selectedBook}
      <!-- State 1: No book selected -->
      <div class="empty-state">
        <div class="empty-icon">📚</div>
        <p class="empty-text">请从左侧选择一本书</p>
      </div>
    {:else}
      <!-- State 2 & 3: Book selected -->
      <div class="book-view">
        <!-- Upper section: Book metadata or Chapter content -->
        <div class="upper-section">
          {#if !selectedChapter}
            <!-- State 2: Book metadata -->
            <div class="book-metadata">
              <div class="metadata-card">
                {#if selectedBook.cover_path}
                  <img
                    src={selectedBook.cover_path}
                    alt={selectedBook.title}
                    class="book-cover"
                  />
                {:else}
                  <div class="cover-placeholder">
                    {selectedBook.title.charAt(0)}
                  </div>
                {/if}

                <div class="metadata-info">
                  <h1 class="book-title">{selectedBook.title}</h1>

                  <div class="metadata-row">
                    <span class="label">作者：</span>
                    <span class="value">{selectedBook.author || '未知'}</span>
                  </div>

                  <div class="metadata-row">
                    <span class="label">分类：</span>
                    <span class="value">{getCategoryName(selectedBook)}</span>
                  </div>

                  <div class="metadata-row">
                    <span class="label">状态：</span>
                    <span
                      class="status-badge"
                      style="color: {getStatusColor(selectedBook.status)}"
                    >
                      {getStatusLabel(selectedBook.status)}
                    </span>
                  </div>

                  <div class="metadata-row">
                    <span class="label">字数：</span>
                    <span class="value"
                      >{(selectedBook.word_count / 10000).toFixed(1)} 万字</span
                    >
                  </div>

                  <div class="metadata-row">
                    <span class="label">章节：</span>
                    <span class="value">{selectedBook.chapter_count} 章</span>
                  </div>

                  <div class="description">
                    <p class="label">简介：</p>
                    <p class="description-text">{selectedBook.description || '暂无简介'}</p>
                  </div>
                </div>
              </div>
            </div>
          {:else}
            <!-- State 3: Chapter content - 使用完整的Reader组件 -->
            <div class="chapter-reader-wrapper">
              <Reader
                chapter={{
                  id: String(selectedChapter.id),
                  title: selectedChapter.title
                }}
                bookDir={workspacePath}
                externalContent={selectedChapter.content}
                onBack={() => (selectedChapter = null)}
              />
            </div>
          {/if}
        </div>

        <!-- Lower section: Chapter List (always visible when book selected) -->
        <div class="lower-section">
          <div class="chapter-list">
            <h3 class="chapter-list-title">章节目录</h3>
            <div class="chapter-items">
              {#if loading}
                <div class="chapter-loading">Loading...</div>
              {:else if chapters.length === 0}
                <div class="chapter-empty">暂无章节</div>
              {:else}
                {#each chapters as chapter (chapter.id)}
                  <button
                    class="chapter-item"
                    class:active={selectedChapter?.id === chapter.id}
                    onclick={() => handleChapterSelect(chapter.id)}
                  >
                    <div class="chapter-item-header">
                      <span class="chapter-order">{chapter.sort_order}.</span>
                      <span class="chapter-item-title">{chapter.title}</span>
                      <span class="chapter-length">[{chapter.word_count}字]</span>
                    </div>
                    {#if chapter.preview}
                      <div class="chapter-preview">
                        {chapter.preview}
                      </div>
                    {/if}
                  </button>
                {/each}
              {/if}
            </div>
          </div>
        </div>
      </div>
    {/if}
  </main>
  </div> <!-- end main-content -->
</div>

<!-- Import Dialog -->
{#if currentLibrary}
  <ImportDialog
    bind:isOpen={showImportDialog}
    libraryId={currentLibrary.id}
    onSuccess={handleImportSuccess}
  />
{/if}

<!-- Create Library Dialog -->
{#if showCreateLibraryDialog}
  <div class="dialog-overlay" onclick={() => showCreateLibraryDialog = false}>
    <div class="dialog-content" onclick={(e) => e.stopPropagation()}>
      <div class="dialog-header">
        <h3>创建新的小说库</h3>
        <button class="dialog-close" onclick={() => showCreateLibraryDialog = false}>✕</button>
      </div>
      <div class="dialog-body">
        <div class="form-field">
          <label for="library-name">库名称 *</label>
          <input
            id="library-name"
            type="text"
            bind:value={newLibraryName}
            placeholder="例如：工作小说库"
          />
        </div>
        <div class="form-field">
          <label for="library-path">存储路径</label>
          <input
            id="library-path"
            type="text"
            bind:value={newLibraryPath}
            placeholder="/novels"
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
        <button class="btn-secondary" onclick={() => showCreateLibraryDialog = false}>
          取消
        </button>
        <button class="btn-primary" onclick={handleCreateLibrary} disabled={loading}>
          {loading ? '创建中...' : '创建'}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .novel-module {
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
    background-color: var(--color-bg-hover);
  }

  .library-selector select:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 1px;
  }

  .library-hint {
    font-size: 12px;
    color: var(--color-text-tertiary);
    font-style: italic;
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
    white-space: nowrap;
  }

  .create-library-btn:hover {
    background-color: var(--color-primary);
    color: white;
  }

  /* Dialog Overlay */
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }

  .dialog-content {
    background: var(--color-bg-primary);
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
  }

  .dialog-header h3 {
    margin: 0;
    font-size: 16px;
    color: var(--color-text-primary);
  }

  .dialog-close {
    width: 28px;
    height: 28px;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 20px;
    cursor: pointer;
    border-radius: 4px;
  }

  .dialog-close:hover {
    background: var(--color-bg-hover);
  }

  .dialog-body {
    padding: 20px;
  }

  .form-field {
    margin-bottom: 16px;
  }

  .form-field:last-child {
    margin-bottom: 0;
  }

  .form-field label {
    display: block;
    margin-bottom: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .form-field input,
  .form-field textarea {
    width: 100%;
    padding: 8px 12px;
    font-size: 14px;
    color: var(--color-text-primary);
    background: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    border-radius: 4px;
  }

  .form-field input:focus,
  .form-field textarea:focus {
    outline: 2px solid var(--color-primary);
    outline-offset: 1px;
  }

  .form-field textarea {
    resize: vertical;
    font-family: inherit;
  }

  .field-hint {
    display: block;
    margin-top: 4px;
    font-size: 12px;
    color: var(--color-text-tertiary);
  }

  .dialog-footer {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    padding: 16px 20px;
    border-top: 1px solid var(--color-border);
  }

  .btn-secondary,
  .btn-primary {
    padding: 8px 16px;
    font-size: 14px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .btn-secondary {
    color: var(--color-text-primary);
    background: var(--color-bg-secondary);
  }

  .btn-secondary:hover {
    background: var(--color-bg-hover);
  }

  .btn-primary {
    color: white;
    background: var(--color-primary);
  }

  .btn-primary:hover {
    opacity: 0.9;
  }

  .btn-primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* Main content wrapper for sidebar and content area */
  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* Category Sidebar */
  .category-sidebar {
    width: 280px;
    background-color: var(--color-bg-primary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .sidebar-header h2 {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .sidebar-actions {
    display: flex;
    gap: 8px;
  }

  .action-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    font-size: 16px;
    color: var(--color-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .action-btn:hover {
    background-color: var(--color-bg-hover);
    border-color: var(--color-primary);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
  }

  /* Content Area */
  .content-area {
    flex: 1;
    overflow: hidden;
    background-color: var(--color-bg-secondary);
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 48px;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 16px;
  }

  .empty-text {
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  /* Book View */
  .book-view {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* Upper Section (Book metadata or Chapter content) */
  .upper-section {
    flex: 1;
    overflow-y: auto;
    min-height: 0; /* Important for flex overflow */
  }

  /* 当显示Reader时，禁用上层滚动 */
  .upper-section:has(.chapter-reader-wrapper) {
    overflow: hidden;
  }

  /* Lower Section (Chapter List) */
  .lower-section {
    height: 350px;
    flex-shrink: 0;
    border-top: 1px solid var(--color-border);
    background-color: var(--color-bg-primary);
  }

  /* Book Metadata */
  .book-metadata {
    padding: 24px;
    height: 100%;
  }

  .metadata-card {
    display: flex;
    gap: 24px;
    max-width: 900px;
    margin: 0 auto;
  }

  .book-cover {
    width: 180px;
    height: 240px;
    object-fit: cover;
    border-radius: 8px;
    flex-shrink: 0;
  }

  .cover-placeholder {
    width: 180px;
    height: 240px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 72px;
    font-weight: 600;
    color: var(--color-text-secondary);
    background: linear-gradient(
      135deg,
      var(--color-bg-hover) 0%,
      var(--color-bg-secondary) 100%
    );
    border-radius: 8px;
    flex-shrink: 0;
  }

  .metadata-info {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .book-title {
    font-size: 28px;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .metadata-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
  }

  .label {
    font-weight: 500;
    color: var(--color-text-secondary);
  }

  .value {
    color: var(--color-text-primary);
  }

  .status-badge {
    font-weight: 600;
  }

  .description {
    margin-top: 16px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .description-text {
    font-size: 14px;
    line-height: 1.8;
    color: var(--color-text-primary);
    margin: 0;
  }

  /* 旧的章节预览样式已移除，现在使用Reader组件 */

  /* Chapter List */
  .chapter-list {
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .chapter-list-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    padding: 16px 24px;
    margin: 0;
    border-bottom: 1px solid var(--color-border);
  }

  .chapter-items {
    flex: 1;
    overflow-y: auto;
    padding: 8px;
  }

  .chapter-item {
    width: 100%;
    padding: 12px 16px;
    margin-bottom: 4px;
    border-radius: 6px;
    background-color: transparent;
    border: 1px solid transparent;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .chapter-item:hover {
    background-color: var(--color-bg-secondary);
    border-color: var(--color-border);
  }

  .chapter-item.active {
    background-color: var(--color-primary);
    border-color: var(--color-primary);
    color: white;
  }

  .chapter-item-header {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .chapter-preview {
    font-size: 12px;
    color: var(--color-text-tertiary);
    margin-top: 4px;
    padding-left: 20px;
    line-height: 1.4;
  }

  .chapter-item.active .chapter-preview {
    color: rgba(255, 255, 255, 0.7);
  }

  .chapter-order {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
  }

  .chapter-item.active .chapter-order {
    color: rgba(255, 255, 255, 0.8);
  }

  .chapter-item-title {
    flex: 1;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .chapter-item.active .chapter-item-title {
    color: white;
  }

  .chapter-length {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .chapter-item.active .chapter-length {
    color: rgba(255, 255, 255, 0.8);
  }

  /* Chapter list states */
  .chapter-loading,
  .chapter-empty {
    padding: 24px;
    text-align: center;
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  /* Chapter Reader Wrapper - 使用Reader组件时的样式 */
  .chapter-reader-wrapper {
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--color-bg-secondary);
    overflow: hidden; /* 防止外部滚动，让Reader内部处理滚动 */
  }

  /* Reader组件占据剩余空间 */
  .chapter-reader-wrapper :global(.reader) {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-height: 0; /* 允许flex子元素收缩 */
  }

  /* 确保Reader内部的content-area可以正确滚动 */
  .chapter-reader-wrapper :global(.content-area) {
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }
</style>
