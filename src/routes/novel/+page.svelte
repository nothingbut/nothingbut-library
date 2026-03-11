<script lang="ts">
  import CategoryTree from '$lib/components/CategoryTree.svelte';
  import ChapterList from '$lib/components/ChapterList.svelte';
  import LibraryGrid from '$lib/components/LibraryGrid.svelte';

  let selectedBookId = $state<number | null>(null);
  let selectedChapterId = $state<number | null>(null);
  let showBookGrid = $state(true);

  function handleBookSelect(bookId: number) {
    selectedBookId = bookId;
    showBookGrid = false;
  }

  function handleChapterSelect(chapterId: number) {
    selectedChapterId = chapterId;
    // Navigate to reader page
    window.location.href = `/reader/${selectedBookId}`;
  }

  function handleBackToGrid() {
    selectedBookId = null;
    selectedChapterId = null;
    showBookGrid = true;
  }
</script>

<div class="novel-module">
  <!-- Left: Category Tree -->
  <aside class="category-sidebar">
    <div class="sidebar-header">
      <h2>分类</h2>
      <button class="add-category-btn" title="添加分类">+</button>
    </div>
    <div class="sidebar-content">
      <CategoryTree />
    </div>
  </aside>

  <!-- Middle: Chapter List (shown when book is selected) -->
  {#if selectedBookId && !showBookGrid}
    <aside class="chapter-sidebar">
      <div class="sidebar-header">
        <button class="back-btn" onclick={handleBackToGrid}>← 返回</button>
        <h2>章节列表</h2>
      </div>
      <div class="sidebar-content">
        <ChapterList onSelectChapter={handleChapterSelect} />
      </div>
    </aside>
  {/if}

  <!-- Right: Book Grid or Reader -->
  <main class="main-area">
    {#if showBookGrid}
      <LibraryGrid />
    {:else if selectedBookId}
      <div class="reader-placeholder">
        <div class="placeholder-icon">📖</div>
        <h3>阅读器占位</h3>
        <p>选择章节开始阅读</p>
        <button class="back-grid-btn" onclick={handleBackToGrid}>
          返回书籍列表
        </button>
      </div>
    {:else}
      <div class="empty-state">
        <div class="empty-icon">📚</div>
        <p>从左侧选择分类查看书籍</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .novel-module {
    display: flex;
    height: 100%;
    width: 100%;
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
    overflow: hidden;
  }

  /* Chapter Sidebar */
  .chapter-sidebar {
    width: 320px;
    background-color: var(--color-bg-primary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
  }

  /* Sidebar Common Styles */
  .sidebar-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .sidebar-header h2 {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .add-category-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    font-size: 18px;
    font-weight: 600;
    color: var(--color-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .add-category-btn:hover {
    background-color: var(--color-bg-hover);
    border-color: var(--color-primary);
  }

  .back-btn {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-text-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .back-btn:hover {
    background-color: var(--color-bg-hover);
  }

  .sidebar-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
  }

  /* Main Area */
  .main-area {
    flex: 1;
    overflow: auto;
    background-color: var(--color-bg-secondary);
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 48px 24px;
    text-align: center;
  }

  .empty-icon {
    font-size: 64px;
    margin-bottom: 16px;
  }

  .empty-state p {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  /* Reader Placeholder */
  .reader-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 48px 24px;
    text-align: center;
  }

  .placeholder-icon {
    font-size: 64px;
    margin-bottom: 16px;
  }

  .reader-placeholder h3 {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .reader-placeholder p {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0 0 24px 0;
  }

  .back-grid-btn {
    padding: 10px 24px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-primary);
    background-color: transparent;
    border: 1px solid var(--color-primary);
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .back-grid-btn:hover {
    color: white;
    background-color: var(--color-primary);
  }
</style>
