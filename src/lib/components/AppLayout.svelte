<script lang="ts">
  import { type Snippet } from 'svelte';
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import AIAssistant from './AIAssistant.svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // Derived states
  let isHomePage = $derived($page.url.pathname === '/');
  let isReaderPage = $derived($page.url.pathname.startsWith('/reader/'));
  let isEpubReader = $derived($page.url.pathname.startsWith('/reader/epub/'));
  let currentLibraryName = $derived(() => {
    const path = $page.url.pathname;
    if (path === '/novel') return '网络小说';
    if (path.startsWith('/novel')) return '网络小说';
    if (path === '/epub') return 'EPUB 书库';
    if (path.startsWith('/epub')) return 'EPUB 书库';
    if (path === '/music') return '音乐库';
    if (path.startsWith('/music')) return '音乐库';
    return '';
  });

  function goToHome() {
    if (!isHomePage) {
      window.location.href = '/';
    }
  }

  // 阅读器控制函数
  function goBackFromReader() {
    if (isEpubReader) {
      goto('/epub');
    } else {
      goto('/novel');
    }
  }

  function toggleReaderSidebar() {
    // 发送自定义事件到阅读器页面
    window.dispatchEvent(new CustomEvent('toggle-reader-sidebar'));
  }
</script>

<div class="app-layout">
  <!-- Toolbar -->
  <header class="toolbar">
    <div class="toolbar-left">
      <button
        class="toolbar-btn"
        class:disabled={isHomePage}
        onclick={goToHome}
        title={isHomePage ? '当前在首页' : '返回首页'}
        disabled={isHomePage}
      >
        📚 资料库
      </button>
    </div>

    <div class="toolbar-center">
      {#if currentLibraryName()}
        <h1 class="app-title">{currentLibraryName()}</h1>
      {/if}
    </div>

    <div class="toolbar-right">
      {#if isReaderPage}
        <!-- 阅读器页面的返回和目录按钮 -->
        <button class="toolbar-icon-btn" onclick={goBackFromReader} title="返回图书馆">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
            <path d="M12.707 5.293a1 1 0 010 1.414L9.414 10l3.293 3.293a1 1 0 01-1.414 1.414l-4-4a1 1 0 010-1.414l4-4a1 1 0 011.414 0z"/>
          </svg>
        </button>
        <button class="toolbar-icon-btn" onclick={toggleReaderSidebar} title="目录">
          <svg width="20" height="20" viewBox="0 0 20 20" fill="none" stroke="currentColor">
            <path d="M3 5h14M3 10h14M3 15h14" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
      {/if}
    </div>
  </header>

  <!-- Main content area -->
  <main class="main-content">
    <div class="content-area">
      {@render children()}
    </div>
  </main>
</div>

<!-- AI 助手（浮动按钮，在 app-layout 外部） -->
<AIAssistant />

<style>
  .app-layout {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
  }

  /* Toolbar styles */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: 48px;
    padding: 0 16px;
    background-color: var(--color-bg-primary);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .toolbar-left,
  .toolbar-right {
    flex: 1;
  }

  .toolbar-left {
    display: flex;
    justify-content: flex-start;
  }

  .toolbar-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .toolbar-right {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
  }

  .app-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  /* Toolbar button styles */
  .toolbar-btn {
    padding: 6px 12px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
    background-color: var(--color-bg-secondary);
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .toolbar-btn:hover {
    background-color: var(--color-bg-hover);
    transform: translateY(-1px);
  }

  .toolbar-btn:active {
    transform: translateY(0);
  }

  .toolbar-btn:disabled,
  .toolbar-btn.disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
  }

  /* Icon button styles for reader controls */
  .toolbar-icon-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 6px;
    background-color: var(--color-bg-secondary);
    color: var(--color-text-primary);
    transition: all 0.2s ease;
    cursor: pointer;
    border: none;
  }

  .toolbar-icon-btn:hover {
    background-color: var(--color-bg-hover);
    transform: translateY(-1px);
  }

  .toolbar-icon-btn:active {
    transform: translateY(0);
  }

  /* Main content area */
  .main-content {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .content-area {
    flex: 1;
    overflow: auto;
    background-color: var(--color-bg-secondary);
  }
</style>
