<script lang="ts">
  import { type Snippet } from 'svelte';
  import { page } from '$app/stores';
  import AIAssistant from './ai/AIAssistant.svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // Local state using Svelte 5 runes
  let showAIPanel = $state(false);

  // Derived states
  let isHomePage = $derived($page.url.pathname === '/');
  let currentLibraryName = $derived(() => {
    const path = $page.url.pathname;
    if (path === '/novel') return '网络小说';
    if (path.startsWith('/novel')) return '网络小说';
    return '';
  });

  function toggleAIPanel() {
    showAIPanel = !showAIPanel;
  }

  function goToHome() {
    if (!isHomePage) {
      window.location.href = '/';
    }
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
      <button class="toolbar-btn" onclick={toggleAIPanel}>
        {showAIPanel ? '🤖 关闭AI' : '🤖 打开AI'}
      </button>
    </div>
  </header>

  <!-- Main content area -->
  <main class="main-content" class:with-ai-panel={showAIPanel}>
    <!-- AI Panel (toggleable) - 右侧 -->
    {#if showAIPanel}
      <aside class="ai-panel">
        <AIAssistant isOpen={showAIPanel} />
      </aside>
    {/if}

    <div class="content-area">
      {@render children()}
    </div>
  </main>
</div>

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
    justify-content: flex-end;
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

  /* AI Panel styles */
  .ai-panel {
    width: 320px;
    background-color: var(--color-bg-primary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .ai-panel-header {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
  }

  .ai-panel-header h2 {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .ai-panel-content {
    flex: 1;
    padding: 16px;
    overflow: auto;
  }

  .placeholder-text {
    color: var(--color-text-secondary);
    font-size: 13px;
    text-align: center;
    margin-top: 24px;
  }
</style>
