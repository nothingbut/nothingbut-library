<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { Library } from '$lib/types/library';
  import type { WereadAccount, WereadBook, WereadChapter, WereadDownload } from '$lib/types/weread';
  import { listLibraries, getCurrentLibrary, createLibrary, setCurrentLibrary } from '$lib/services/library';
  import * as weread from '$lib/services/weread';

  let libraries: Library[] = [];
  let currentLibrary: Library | null = null;

  let account: WereadAccount | null = null;
  let books: WereadBook[] = [];
  let searchQuery = '';
  let syncing = false;
  let loading = false;
  let viewMode: 'cover' | 'list' = 'cover';
  let selectedBookIds = new Set<number>();
  let outputDir = '~/Downloads/WeRead';

  async function initLibrary() {
    try {
      libraries = await listLibraries('weread');
      if (libraries.length === 0) {
        await createLibrary({
          name: '微信读书',
          moduleType: 'weread',
          storagePath: 'weread',
          description: '微信读书导出'
        });
        libraries = await listLibraries('weread');
      }
      currentLibrary = await getCurrentLibrary('weread').catch(() => libraries[0] || null);
      if (currentLibrary) {
        await loadBooks();
      }
    } catch (e) {
      console.error('[weread] initLibrary 失败:', e);
    }
  }

  onMount(async () => {
    try {
      const status = await weread.checkCookie();
      if (status.loggedIn) {
        account = status.account;
      }
    } catch (e) {
      console.error('[weread] checkCookie 失败:', e);
    }

    await initLibrary();
  });

  async function handleLogin() {
    await weread.openLogin();
    const pollLogin = setInterval(async () => {
      try {
        const acc = await weread.getAccount();
        if (acc) {
          account = acc;
          clearInterval(pollLogin);
          await initLibrary();
        }
      } catch { /* ignore */ }
    }, 3000);
    setTimeout(() => clearInterval(pollLogin), 300000);
  }

  async function handleLogout() {
    if (!confirm('确定退出登录？')) return;
    await weread.logout();
    account = null;
  }

  async function loadBooks() {
    if (!currentLibrary) return;
    loading = true;
    try {
      books = await weread.listBooks(currentLibrary.id);
    } finally {
      loading = false;
    }
  }

  async function handleSync() {
    if (!currentLibrary) return;
    syncing = true;
    try {
      books = await weread.syncBooks(currentLibrary.id);
    } catch (e) {
      alert(`同步失败: ${e}`);
    } finally {
      syncing = false;
    }
  }

  async function handleSearch() {
    if (!currentLibrary || !searchQuery.trim()) {
      await loadBooks();
      return;
    }
    loading = true;
    try {
      books = await weread.searchBooks(currentLibrary.id, searchQuery);
    } finally {
      loading = false;
    }
  }

  async function handleLibrarySwitch(e: Event) {
    const target = e.target as HTMLSelectElement;
    const id = parseInt(target.value);
    await setCurrentLibrary(id);
    currentLibrary = libraries.find(l => l.id === id) ?? null;
    books = [];
    await loadBooks();
  }

  function toggleBookSelect(id: number) {
    const next = new Set(selectedBookIds);
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    selectedBookIds = next;
  }

  function selectAllBooks() {
    selectedBookIds = new Set(filteredBooks.map((b: WereadBook) => b.id));
  }

  function deselectAllBooks() {
    selectedBookIds = new Set<number>();
  }

  async function pickOutputDir() {
    const selected = await open({ directory: true, title: '选择 EPUB 输出目录' });
    if (selected) {
      outputDir = selected as string;
    }
  }

  async function handleBatchExport() {
    if (!currentLibrary || selectedBookIds.size === 0) return;
    const count = selectedBookIds.size;
    for (const bookId of selectedBookIds) {
      try {
        await weread.startExport(currentLibrary.id, bookId, outputDir);
      } catch (e) {
        console.error(`[weread] export book ${bookId} failed:`, e);
      }
    }
    selectedBookIds = new Set<number>();
    alert(`已启动 ${count} 本书的导出`);
  }

  // 书籍详情/导出
  let selectedBook: WereadBook | null = null;
  let chapters: WereadChapter[] = [];
  let loadingChapters = false;
  let exporting = false;
  let exportProgress: WereadDownload | null = null;
  let progressTimer: ReturnType<typeof setInterval> | null = null;

  async function selectBook(book: WereadBook) {
    selectedBook = book;
    chapters = [];
    loadingChapters = true;
    try {
      if (currentLibrary) {
        chapters = await weread.getBookDetail(currentLibrary.id, book.id);
      }
    } catch (e) {
      console.error('[weread] getBookDetail 失败:', e);
    } finally {
      loadingChapters = false;
    }
  }

  function closeDetail() {
    selectedBook = null;
    chapters = [];
    exportProgress = null;
    if (progressTimer) clearInterval(progressTimer);
  }

  async function handleExport() {
    if (!currentLibrary || !selectedBook) return;
    exporting = true;
    try {
      await weread.startExport(currentLibrary.id, selectedBook.id, outputDir);
      startExportPolling();
    } catch (e) {
      alert(`导出失败: ${e}`);
      exporting = false;
    }
  }

  function startExportPolling() {
    if (progressTimer) clearInterval(progressTimer);
    progressTimer = setInterval(async () => {
      if (!currentLibrary || !selectedBook) return;
      exportProgress = await weread.getExportProgress(currentLibrary.id, selectedBook.id);
      if (exportProgress && (exportProgress.status === 'completed' || exportProgress.status === 'failed')) {
        exporting = false;
        if (progressTimer) clearInterval(progressTimer);
      }
    }, 2000);
  }

  $: filteredBooks = books;
</script>

<div class="h-full flex flex-col bg-gray-50">
  <!-- 顶部栏 -->
  <div class="flex items-center gap-4 px-6 py-3 bg-white border-b">
    <h1 class="text-xl font-bold text-green-600">微信读书</h1>

    <select
      class="px-3 py-1.5 border rounded-lg text-sm"
      value={currentLibrary?.id ?? ''}
      on:change={handleLibrarySwitch}
    >
      {#each libraries as lib}
        <option value={lib.id}>{lib.name}</option>
      {/each}
    </select>

    <div class="flex-1"></div>

    {#if account}
      <div class="flex items-center gap-2 text-sm">
        {#if account.avatarUrl}
          <img src={account.avatarUrl} alt="" class="w-6 h-6 rounded-full" />
        {/if}
        <span class="text-gray-700">{account.username}</span>
        <button class="text-gray-400 hover:text-red-500 text-xs" on:click={handleLogout}>退出</button>
      </div>
    {:else}
      <button
        class="px-4 py-1.5 bg-green-500 text-white rounded-lg text-sm hover:bg-green-600"
        on:click={handleLogin}
      >
        扫码登录
      </button>
    {/if}
  </div>

  <!-- 主内容 -->
  {#if !account}
    <div class="flex-1 flex flex-col items-center justify-center gap-4">
      <div class="text-6xl">📚</div>
      <p class="text-gray-500">请先登录微信读书账号</p>
      <button
        class="px-6 py-2.5 bg-green-500 text-white rounded-lg text-base hover:bg-green-600"
        on:click={handleLogin}
      >
        扫码登录
      </button>
    </div>
  {:else}
    <!-- 工具栏 -->
    <div class="flex items-center gap-3 px-6 py-2.5 bg-white border-b">
      <button
        class="px-4 py-1.5 text-sm border rounded-lg hover:bg-gray-50 disabled:opacity-50"
        on:click={handleSync}
        disabled={syncing}
      >
        {syncing ? '同步中...' : '同步书架'}
      </button>

      <button
        class="px-3 py-1.5 text-sm border rounded-lg hover:bg-gray-50 truncate max-w-48"
        on:click={pickOutputDir}
        title="点击修改输出目录: {outputDir}"
      >
        📁 {outputDir.split('/').pop()}
      </button>

      <input
        type="text"
        placeholder="搜索书籍..."
        bind:value={searchQuery}
        class="px-3 py-1.5 border rounded-lg text-sm w-64"
        on:keydown={(e) => e.key === 'Enter' && handleSearch()}
      />

      <div class="flex-1"></div>

      {#if viewMode === 'list' && selectedBookIds.size > 0}
        <button class="text-sm text-green-600 hover:underline" on:click={handleBatchExport}>
          导出选中 ({selectedBookIds.size})
        </button>
        <button class="text-sm text-gray-500 hover:underline" on:click={selectAllBooks}>全选</button>
        <button class="text-sm text-gray-500 hover:underline" on:click={deselectAllBooks}>取消</button>
      {/if}

      <span class="text-sm text-gray-500">{books.length} 本书</span>

      <div class="flex border rounded-lg overflow-hidden">
        <button
          class="px-2.5 py-1 text-sm {viewMode === 'cover' ? 'bg-green-500 text-white' : 'bg-white text-gray-600 hover:bg-gray-50'}"
          on:click={() => viewMode = 'cover'}
          title="封面模式"
        >▦</button>
        <button
          class="px-2.5 py-1 text-sm {viewMode === 'list' ? 'bg-green-500 text-white' : 'bg-white text-gray-600 hover:bg-gray-50'}"
          on:click={() => viewMode = 'list'}
          title="列表模式"
        >☰</button>
      </div>
    </div>

    <!-- 书籍列表 -->
    <div class="flex-1 overflow-y-auto p-6">
      {#if loading || syncing}
        <div class="text-center text-gray-400 py-12">
          {syncing ? '正在同步书架...' : '加载中...'}
        </div>
      {:else if filteredBooks.length === 0}
        <div class="text-center text-gray-400 py-12">
          {books.length === 0 ? '点击「同步书架」获取书籍列表' : '没有匹配的书籍'}
        </div>
      {:else if viewMode === 'cover'}
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4">
          {#each filteredBooks as book}
            <button
              class="bg-white rounded-lg shadow-sm border hover:shadow-md transition-shadow p-3 cursor-pointer text-left"
              on:click={() => selectBook(book)}
            >
              {#if book.coverUrl}
                <img
                  src={book.coverUrl}
                  alt={book.title}
                  class="w-full aspect-[3/4] object-cover rounded mb-2"
                />
              {:else}
                <div class="w-full aspect-[3/4] bg-gradient-to-br from-green-100 to-green-200 rounded mb-2 flex items-center justify-center">
                  <span class="text-2xl">📖</span>
                </div>
              {/if}
              <div class="text-sm font-medium truncate" title={book.title}>{book.title}</div>
              <div class="text-xs text-gray-400 truncate mt-0.5">{book.author || '未知作者'}</div>
              <div class="text-xs text-gray-300 mt-1">{weread.formatWordCount(book.wordCount)}</div>
            </button>
          {/each}
        </div>
      {:else}
        <div class="space-y-1">
          {#each filteredBooks as book}
            <label
              class="flex items-center gap-3 px-4 py-3 bg-white border-b hover:bg-gray-50 cursor-pointer
                {selectedBookIds.has(book.id) ? 'bg-green-50' : ''}"
            >
              <input
                type="checkbox"
                checked={selectedBookIds.has(book.id)}
                on:change={() => toggleBookSelect(book.id)}
                class="w-4 h-4 accent-green-500"
              />
              {#if book.coverUrl}
                <img src={book.coverUrl} alt="" class="w-10 h-14 object-cover rounded flex-shrink-0" />
              {:else}
                <div class="w-10 h-14 bg-green-100 rounded flex-shrink-0 flex items-center justify-center text-xs">📖</div>
              {/if}
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium truncate">{book.title}</div>
                <div class="text-xs text-gray-400 mt-0.5">{book.author || '未知作者'} · {weread.formatWordCount(book.wordCount)} · {book.chapterCount} 章</div>
              </div>
              <button
                class="text-xs text-green-600 hover:underline px-2 py-1"
                on:click|stopPropagation={() => selectBook(book)}
              >详情</button>
            </label>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- 书籍详情弹窗 -->
  {#if selectedBook}
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div class="bg-white rounded-xl max-w-2xl w-full mx-4 max-h-[80vh] flex flex-col">
        <div class="flex items-start gap-4 p-6 border-b">
          {#if selectedBook.coverUrl}
            <img src={selectedBook.coverUrl} alt="" class="w-20 h-28 object-cover rounded flex-shrink-0" />
          {/if}
          <div class="flex-1 min-w-0">
            <h2 class="text-lg font-bold">{selectedBook.title}</h2>
            <p class="text-sm text-gray-500 mt-1">{selectedBook.author || '未知作者'}</p>
            <p class="text-xs text-gray-400 mt-1">{weread.formatWordCount(selectedBook.wordCount)} · {selectedBook.chapterCount} 章</p>
            {#if selectedBook.intro}
              <p class="text-xs text-gray-400 mt-2 line-clamp-3">{selectedBook.intro}</p>
            {/if}
          </div>
          <button class="text-gray-400 hover:text-gray-600 text-xl" on:click={closeDetail}>✕</button>
        </div>

        <div class="flex-1 overflow-y-auto p-6">
          {#if loadingChapters}
            <p class="text-center text-gray-400">加载章节列表...</p>
          {:else if chapters.length > 0}
            <h3 class="text-sm font-medium text-gray-600 mb-2">章节列表 ({chapters.length})</h3>
            <div class="space-y-1 max-h-60 overflow-y-auto">
              {#each chapters as ch}
                <div class="text-sm py-1 px-2 rounded hover:bg-gray-50" style="padding-left: {(ch.level - 1) * 16 + 8}px">
                  {ch.title}
                  {#if ch.extractedAt}
                    <span class="text-green-500 text-xs ml-1">&#10003;</span>
                  {/if}
                </div>
              {/each}
            </div>
          {:else}
            <p class="text-center text-gray-400 text-sm">暂无章节信息</p>
          {/if}
        </div>

        <div class="p-4 border-t bg-gray-50 rounded-b-xl">
          {#if exportProgress}
            <div class="mb-2">
              {#if exportProgress.status === 'completed'}
                <p class="text-sm text-green-600">导出完成: {exportProgress.outputPath}</p>
              {:else if exportProgress.status === 'failed'}
                <p class="text-sm text-red-500">导出失败: {exportProgress.errorMessage}</p>
              {:else}
                <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                  <span>提取章节中...</span>
                  <span>{exportProgress.progressCurrent}/{exportProgress.progressTotal}</span>
                </div>
                <div class="w-full bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-green-500 h-2 rounded-full transition-all"
                    style="width: {exportProgress.progressTotal > 0 ? (exportProgress.progressCurrent / exportProgress.progressTotal * 100) : 0}%"
                  ></div>
                </div>
              {/if}
            </div>
          {/if}
          <button
            class="w-full px-4 py-2.5 bg-green-500 text-white rounded-lg text-sm font-medium hover:bg-green-600 disabled:opacity-50"
            on:click={handleExport}
            disabled={exporting || chapters.length === 0}
          >
            {exporting ? '导出中...' : '导出为 EPUB'}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>
