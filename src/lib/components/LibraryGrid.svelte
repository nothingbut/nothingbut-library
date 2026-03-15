<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { listBooks } from '$lib/services/api';
  import type { Book } from '$lib/types';

  // Props
  interface Props {
    onImport?: () => void;
  }

  let { onImport }: Props = $props();

  let books = $state<Book[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function loadBooks() {
    try {
      loading = true;
      error = null;
      books = await listBooks();
      console.log('Loaded books:', books.length);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load books';
      console.error('Failed to load books:', e);
    } finally {
      loading = false;
    }
  }

  function openBook(book: Book) {
    // Navigate to novel page with book context
    goto(`/novel?bookId=${book.id}`);
  }

  function importNovel() {
    if (onImport) {
      onImport();
    }
  }

  onMount(() => {
    loadBooks();
  });
</script>

<div class="library-grid">
  <div class="library-header">
    <h1 class="library-title">我的资料库</h1>
    <button class="import-btn" onclick={importNovel}>
      📥 导入小说
    </button>
  </div>

  {#if error}
    <div class="error-state">
      <div class="error-icon">⚠️</div>
      <p class="error-title">加载失败</p>
      <p class="error-message">{error}</p>
      <button class="primary-btn" onclick={loadBooks}>重试</button>
    </div>
  {:else if loading}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>加载中...</p>
    </div>
  {:else if books.length === 0}
    <div class="empty-state">
      <div class="empty-icon">📚</div>
      <p class="empty-title">还没有添加任何小说</p>
      <p class="empty-description">点击"导入小说"按钮开始添加</p>
      <button class="primary-btn" onclick={importNovel}>导入小说</button>
    </div>
  {:else}
    <div class="books-grid">
      {#each books as book (book.id)}
        <button
          class="book-card"
          onclick={() => openBook(book)}
          title={book.title}
        >
          <div class="book-cover">
            {#if book.cover_path}
              <img src={book.cover_path} alt={book.title} class="cover-image" />
            {:else}
              <div class="cover-placeholder">
                {book.title.charAt(0)}
              </div>
            {/if}
          </div>
          <div class="book-info">
            <h3 class="book-title">{book.title}</h3>
            <p class="book-author">{book.author || '未知作者'}</p>
            <div class="book-meta">
              <span class="chapter-count">{book.chapter_count} 章</span>
              <span class="reading-progress">
                {book.reading_progress}%
              </span>
            </div>
            <div class="progress-bar">
              <div
                class="progress-fill"
                style="width: {book.reading_progress}%"
              ></div>
            </div>
          </div>
        </button>
      {/each}
    </div>
  {/if}
</div>

<style>
  .library-grid {
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 24px;
    height: 100%;
    overflow: auto;
  }

  .library-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-shrink: 0;
  }

  .library-title {
    font-size: 24px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .import-btn {
    padding: 10px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: white;
    background-color: var(--color-primary);
    border: none;
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .import-btn:hover {
    background-color: var(--color-primary-hover);
  }

  .loading-state,
  .error-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 48px 24px;
    color: var(--color-text-secondary);
  }

  .error-icon {
    font-size: 64px;
    margin-bottom: 16px;
  }

  .error-title {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .error-message {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 48px 24px;
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
    margin: 0;
  }

  .empty-description {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .primary-btn {
    margin-top: 16px;
    padding: 10px 24px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: white;
    background-color: var(--color-primary);
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .primary-btn:hover {
    background-color: var(--color-primary-hover);
  }

  .books-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 16px;
    flex: 1;
  }

  .book-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
    border-radius: 8px;
    background-color: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
    cursor: pointer;
    text-align: left;
  }

  .book-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }

  .book-cover {
    width: 100%;
    aspect-ratio: 3 / 4;
    border-radius: 6px;
    overflow: hidden;
    background-color: var(--color-bg-secondary);
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
    font-size: 48px;
    font-weight: 600;
    color: var(--color-text-secondary);
    background: linear-gradient(135deg, var(--color-bg-secondary) 0%, var(--color-bg-hover) 100%);
  }

  .book-info {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .book-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .book-author {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .book-meta {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .chapter-count {
    flex: 1;
  }

  .reading-progress {
    font-weight: 500;
    color: var(--color-primary);
  }

  .progress-bar {
    width: 100%;
    height: 4px;
    background-color: var(--color-bg-secondary);
    border-radius: 2px;
    overflow: hidden;
  }

  .progress-fill {
    height: 100%;
    background-color: var(--color-primary);
    border-radius: 2px;
    transition: width 0.3s ease;
  }
</style>
