<script lang="ts">
  import { type Snippet } from 'svelte';

  interface Props {
    children: Snippet;
  }

  let { children }: Props = $props();

  // Local state using Svelte 5 runes
  let showAIPanel = $state(false);
  let currentView = $state('library');

  function toggleAIPanel() {
    showAIPanel = !showAIPanel;
  }

  function changeView(view: string) {
    currentView = view;
  }
</script>

<div class="app-layout">
  <!-- Toolbar -->
  <header class="toolbar">
    <div class="toolbar-left">
      <button class="icon-button" aria-label="Menu">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M2 4H18M2 10H18M2 16H18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
      <span class="app-title">NothingBut Library</span>
    </div>

    <div class="toolbar-center">
      <nav class="nav-tabs">
        <button
          class="nav-tab"
          class:active={currentView === 'library'}
          onclick={() => changeView('library')}
        >
          资料库
        </button>
        <button
          class="nav-tab"
          class:active={currentView === 'reader'}
          onclick={() => changeView('reader')}
        >
          阅读器
        </button>
      </nav>
    </div>

    <div class="toolbar-right">
      <button class="icon-button" aria-label="Search">
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <circle cx="9" cy="9" r="6" stroke="currentColor" stroke-width="2"/>
          <path d="M13.5 13.5L17 17" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
        </svg>
      </button>
      <button
        class="icon-button ai-button"
        class:active={showAIPanel}
        onclick={toggleAIPanel}
        aria-label="AI Assistant"
      >
        <svg width="20" height="20" viewBox="0 0 20 20" fill="none" xmlns="http://www.w3.org/2000/svg">
          <path d="M10 2L12 8H18L13 12L15 18L10 14L5 18L7 12L2 8H8L10 2Z" stroke="currentColor" stroke-width="2" stroke-linejoin="round"/>
        </svg>
      </button>
    </div>
  </header>

  <!-- Main content area -->
  <main class="main-content" class:with-ai-panel={showAIPanel}>
    <div class="content-area">
      {@render children()}
    </div>

    <!-- AI Panel (toggleable) -->
    {#if showAIPanel}
      <aside class="ai-panel">
        <div class="ai-panel-header">
          <h2>AI 助手</h2>
          <button class="icon-button" onclick={toggleAIPanel} aria-label="Close AI Panel">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M2 2L14 14M14 2L2 14" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
            </svg>
          </button>
        </div>
        <div class="ai-panel-content">
          <p class="placeholder-text">AI 助手面板占位</p>
        </div>
      </aside>
    {/if}
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
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 200px;
  }

  .toolbar-left {
    justify-content: flex-start;
  }

  .toolbar-right {
    justify-content: flex-end;
  }

  .toolbar-center {
    flex: 1;
    display: flex;
    justify-content: center;
  }

  .app-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  /* Icon button styles */
  .icon-button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 6px;
    color: var(--color-text-secondary);
    transition: all 0.2s ease;
  }

  .icon-button:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .icon-button.active {
    background-color: var(--color-primary);
    color: white;
  }

  .ai-button.active {
    background-color: var(--color-primary);
    color: white;
  }

  /* Navigation tabs */
  .nav-tabs {
    display: flex;
    gap: 8px;
  }

  .nav-tab {
    padding: 6px 16px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-secondary);
    transition: all 0.2s ease;
  }

  .nav-tab:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .nav-tab.active {
    background-color: var(--color-bg-secondary);
    color: var(--color-primary);
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

  .main-content.with-ai-panel .content-area {
    flex: 1;
  }

  /* AI Panel styles */
  .ai-panel {
    width: 320px;
    background-color: var(--color-bg-primary);
    border-left: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
  }

  .ai-panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
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

  /* Responsive adjustments */
  @media (max-width: 768px) {
    .ai-panel {
      width: 280px;
    }

    .toolbar-left,
    .toolbar-right {
      min-width: 100px;
    }

    .app-title {
      display: none;
    }
  }
</style>
