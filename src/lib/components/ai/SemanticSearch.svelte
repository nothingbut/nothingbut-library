<script lang="ts">
  import { semanticSearch, type SearchResult } from '$lib/services/ai';

  interface Props {
    bookId?: number | null;
    workspacePath: string;
  }

  let { bookId = null, workspacePath }: Props = $props();

  let query = $state('');
  let results = $state<SearchResult[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleSearch() {
    if (!query.trim()) {
      error = '请输入搜索内容';
      return;
    }

    loading = true;
    error = null;

    try {
      results = await semanticSearch(query, bookId ?? undefined, 10, 0.6);
    } catch (e) {
      error = '搜索失败：' + (e as Error).message;
      console.error(e);
    } finally {
      loading = false;
    }
  }

  function handleKeyPress(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSearch();
    }
  }

  function formatSimilarity(similarity: number): string {
    return (similarity * 100).toFixed(0) + '%';
  }
</script>

<div class="semantic-search">
  <div class="search-input">
    <input
      type="text"
      bind:value={query}
      onkeydown={handleKeyPress}
      placeholder="输入内容描述，例如：主角获得神秘力量的章节"
      disabled={loading}
    />
    <button onclick={handleSearch} disabled={loading || !query.trim()}>
      {loading ? '搜索中...' : '🔍 搜索'}
    </button>
  </div>

  {#if error}
    <div class="error-message">{error}</div>
  {/if}

  {#if results.length > 0}
    <div class="results">
      <div class="results-header">
        找到 {results.length} 个相关章节
      </div>
      {#each results as result}
        <div class="result-item">
          <div class="result-header">
            <div class="result-title">
              <h4>{result.chapter_title}</h4>
              <span class="chapter-number">第 {result.chapter_number} 章</span>
            </div>
            <span class="similarity" style="opacity: {result.similarity}">
              {formatSimilarity(result.similarity)} 相关
            </span>
          </div>
          <div class="result-book">{result.book_title}</div>
          <p class="result-preview">{result.preview}...</p>
          <button
            class="read-btn"
            onclick={() => {
              // TODO: 导航到章节阅读页面
              console.log('Navigate to chapter:', result.chapter_id);
            }}
          >
            跳转阅读
          </button>
        </div>
      {/each}
    </div>
  {:else if !loading && query}
    <div class="empty-state">
      <div class="empty-icon">🔍</div>
      <p>未找到相关章节</p>
      <p class="hint">尝试使用不同的关键词或降低相似度要求</p>
    </div>
  {/if}
</div>

<style>
  .semantic-search {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .search-input {
    display: flex;
    gap: 8px;
  }

  .search-input input {
    flex: 1;
    padding: 10px 14px;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    font-size: 14px;
  }

  .search-input button {
    padding: 10px 20px;
    background: var(--color-primary);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
    white-space: nowrap;
  }

  .search-input button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .error-message {
    padding: 12px;
    background: #fee;
    color: #c33;
    border-radius: 6px;
    font-size: 14px;
  }

  .results {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .results-header {
    font-size: 14px;
    color: var(--color-text-secondary);
    padding: 8px 0;
  }

  .result-item {
    padding: 16px;
    border: 1px solid var(--color-border);
    border-radius: 8px;
    background: var(--color-bg-secondary);
  }

  .result-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    margin-bottom: 8px;
  }

  .result-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .result-title h4 {
    margin: 0;
    font-size: 16px;
    font-weight: 600;
  }

  .chapter-number {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .similarity {
    font-size: 12px;
    padding: 4px 8px;
    background: var(--color-primary);
    color: white;
    border-radius: 4px;
    white-space: nowrap;
  }

  .result-book {
    font-size: 13px;
    color: var(--color-text-secondary);
    margin-bottom: 8px;
  }

  .result-preview {
    font-size: 14px;
    line-height: 1.6;
    color: var(--color-text-primary);
    margin: 12px 0;
  }

  .read-btn {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid var(--color-primary);
    color: var(--color-primary);
    border-radius: 4px;
    cursor: pointer;
    font-size: 13px;
  }

  .read-btn:hover {
    background: var(--color-primary);
    color: white;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    color: var(--color-text-secondary);
  }

  .empty-icon {
    font-size: 48px;
    margin-bottom: 12px;
  }

  .hint {
    font-size: 12px;
    opacity: 0.7;
  }
</style>
