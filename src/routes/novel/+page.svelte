<script lang="ts">
  import CategoryTree from '$lib/components/CategoryTree.svelte';
  import { onMount } from 'svelte';

  // Types
  interface Book {
    id: number;
    title: string;
    author: string;
    description: string;
    coverPath?: string;
    category: string;
    status: 'completed' | 'ongoing' | 'abandoned';
    wordCount: number;
    chapterCount: number;
  }

  interface Chapter {
    id: number;
    order: number;
    title: string;
    lineCount: number;
    firstLine: string;
    content?: string;
  }

  // State
  let selectedBook = $state<Book | null>(null);
  let selectedChapter = $state<Chapter | null>(null);
  let chapters = $state<Chapter[]>([]);

  // Mock data
  const mockBook: Book = {
    id: 1,
    title: '三体',
    author: '刘慈欣',
    description:
      '文化大革命如火如荼地进行，天文学家叶文洁在期间历经劫难，被带到军方绝秘计划"红岸工程"。叶文洁以太阳为天线，向宇宙发出地球文明的第一声啼鸣，取得了探寻外星文明的突破性进展。',
    category: '科幻 / 太空歌剧',
    status: 'completed',
    wordCount: 280000,
    chapterCount: 46,
  };

  const mockChapters: Chapter[] = [
    {
      id: 1,
      order: 1,
      title: '第一章 疯狂年代',
      lineCount: 125,
      firstLine: '那是一个疯狂的年代，红色的旗帜飘扬在每一个角落...',
    },
    {
      id: 2,
      order: 2,
      title: '第二章 寂静的春天',
      lineCount: 98,
      firstLine: '清晨的阳光透过树叶的缝隙，洒在林间小道上...',
    },
    {
      id: 3,
      order: 3,
      title: '第三章 红岸之一',
      lineCount: 156,
      firstLine: '红岸基地位于大兴安岭深处，这里人迹罕至...',
    },
    {
      id: 4,
      order: 4,
      title: '第四章 三体世界',
      lineCount: 142,
      firstLine: '三体文明处于一个拥有三颗太阳的星系之中...',
    },
    {
      id: 5,
      order: 5,
      title: '第五章 叶文洁',
      lineCount: 188,
      firstLine: '叶文洁站在雷达峰的顶端，凝视着远方的天空...',
    },
  ];

  // Handlers
  function handleBookSelect(bookId: number) {
    // TODO: Load from backend
    selectedBook = mockBook;
    chapters = mockChapters;
    selectedChapter = null;
  }

  function handleChapterSelect(chapterId: number) {
    const chapter = chapters.find((ch) => ch.id === chapterId);
    if (chapter) {
      // Load chapter content (mock)
      selectedChapter = {
        ...chapter,
        content: `# ${chapter.title}\n\n${'这是章节内容的占位文本。'.repeat(50)}`,
      };
    }
  }

  function getStatusLabel(status: Book['status']): string {
    const labels = {
      completed: '✓ 已完本',
      ongoing: '⏳ 连载中',
      abandoned: '⚠ 已断更',
    };
    return labels[status];
  }

  function getStatusColor(status: Book['status']): string {
    const colors = {
      completed: 'green',
      ongoing: 'orange',
      abandoned: 'red',
    };
    return colors[status];
  }

  function truncateText(text: string, maxLength: number = 20): string {
    if (text.length <= maxLength) return text;
    return text.slice(0, maxLength) + '...';
  }

  // Simulate book selection on mount for demo
  onMount(() => {
    // Uncomment to auto-select a book for testing
    // setTimeout(() => handleBookSelect(1), 500);
  });
</script>

<div class="novel-module">
  <!-- Left Column: Category Tree -->
  <aside class="category-sidebar">
    <div class="sidebar-header">
      <h2>分类</h2>
      <button class="add-btn" title="添加分类">+</button>
    </div>
    <div class="sidebar-content">
      <CategoryTree onSelectBook={handleBookSelect} />
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
              {#if selectedBook.coverPath}
                <img
                  src={selectedBook.coverPath}
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
                  <span class="value">{selectedBook.author}</span>
                </div>

                <div class="metadata-row">
                  <span class="label">分类：</span>
                  <span class="value">{selectedBook.category}</span>
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
                    >{(selectedBook.wordCount / 10000).toFixed(1)} 万字</span
                  >
                </div>

                <div class="metadata-row">
                  <span class="label">章节：</span>
                  <span class="value">{selectedBook.chapterCount} 章</span>
                </div>

                <div class="description">
                  <p class="label">简介：</p>
                  <p class="description-text">{selectedBook.description}</p>
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
            {#each chapters as chapter (chapter.id)}
              <button
                class="chapter-item"
                class:active={selectedChapter?.id === chapter.id}
                onclick={() => handleChapterSelect(chapter.id)}
              >
                <div class="chapter-item-header">
                  <span class="chapter-order">{chapter.order}.</span>
                  <span class="chapter-item-title">{chapter.title}</span>
                  <span class="chapter-length">[{chapter.lineCount}行]</span>
                </div>
                <div class="chapter-preview">
                  {truncateText(chapter.firstLine, 40)}
                </div>
              </button>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  </main>
</div>

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

  .add-btn {
    width: 28px;
    height: 28px;
    border-radius: 6px;
    font-size: 18px;
    color: var(--color-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .add-btn:hover {
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

  .chapter-preview {
    font-size: 12px;
    color: var(--color-text-secondary);
    line-height: 1.5;
  }

  .chapter-item.active .chapter-preview {
    color: rgba(255, 255, 255, 0.7);
  }
</style>
