<script lang="ts">
  import CategoryTree from '$lib/components/CategoryTree.svelte';
  import ImportDialog from '$lib/components/ImportDialog.svelte';
  import { listBooks, listChapters, getChapterContent } from '$lib/services/api';
  import type { Book, Chapter, BookStatus } from '$lib/types';

  // State
  let selectedBook = $state<Book | null>(null);
  let selectedChapter = $state<Chapter & { content?: string } | null>(null);
  let chapters = $state<Chapter[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let showImportDialog = $state(false);
  let categoryTreeKey = $state(0); // For forcing re-render

  // Workspace path (hardcoded for now, should come from config)
  const workspacePath = '/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library';

  // Handlers
  async function handleBookSelect(bookId: number) {
    try {
      loading = true;
      error = null;

      // Load book and chapters from backend
      const [books, chapterList] = await Promise.all([
        listBooks(),
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

      // Load chapter content from file
      const content = await getChapterContent(workspacePath, chapterId);

      selectedChapter = {
        ...chapter,
        content
      };
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load chapter';
      console.error('Failed to load chapter:', e);
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
    // TODO: Resolve category name from category_id
    return book.category_id ? `分类 ${book.category_id}` : '未分类';
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

</script>

<div class="novel-module">
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
        <CategoryTree onSelectBook={handleBookSelect} />
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
          <!-- State 3: Chapter content -->
          <div class="chapter-content">
            <div class="chapter-header">
              <h2 class="chapter-title">{selectedChapter.title}</h2>
              <div class="chapter-toolbar">
                <button class="toolbar-btn" title="字体大小">A</button>
                <button class="toolbar-btn" title="主题">☀️</button>
                <button
                  class="toolbar-btn"
                  onclick={() => (selectedChapter = null)}
                >
                  返回书籍
                </button>
              </div>
            </div>
            <div class="chapter-body">
              <p class="chapter-text">{selectedChapter.content}</p>
            </div>
          </div>
        {/if}

        <!-- Chapter List (always visible when book selected) -->
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
                </button>
              {/each}
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </main>
</div>

<!-- Import Dialog -->
<ImportDialog
  bind:isOpen={showImportDialog}
  onSuccess={handleImportSuccess}
/>

<style>
  .novel-module {
    display: flex;
    height: 100%;
    overflow: hidden;
    background-color: var(--color-bg-secondary);
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

  /* Book Metadata */
  .book-metadata {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    border-bottom: 1px solid var(--color-border);
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

  /* Chapter Content */
  .chapter-content {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .chapter-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
    max-width: 900px;
    margin-left: auto;
    margin-right: auto;
  }

  .chapter-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .chapter-toolbar {
    display: flex;
    gap: 8px;
  }

  .toolbar-btn {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .toolbar-btn:hover {
    background-color: var(--color-bg-hover);
  }

  .chapter-body {
    max-width: 900px;
    margin: 0 auto;
  }

  .chapter-text {
    font-size: 16px;
    line-height: 1.8;
    color: var(--color-text-primary);
    white-space: pre-wrap;
    margin: 0;
  }

  /* Chapter List */
  .chapter-list {
    height: 400px;
    background-color: var(--color-bg-primary);
    border-top: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
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
    margin-bottom: 4px;
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
</style>
