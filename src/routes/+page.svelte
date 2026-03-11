<script lang="ts">
  import { onMount } from 'svelte';

  interface Module {
    id: string;
    name: string;
    icon: string;
    available: boolean;
  }

  interface RecentWorkspace {
    id: string;
    name: string;
    type: string;
    lastOpenedAt: string;
  }

  const modules: Module[] = [
    { id: 'novel', name: '网络小说', icon: '📚', available: true },
    { id: 'music', name: '音乐库', icon: '🎵', available: false },
    { id: 'ebook', name: '电子书', icon: '📖', available: false },
    { id: 'note', name: '笔记', icon: '📝', available: false },
  ];

  let recentWorkspaces = $state<RecentWorkspace[]>([]);

  function loadRecentWorkspaces() {
    // TODO: Load from backend
    recentWorkspaces = [
      {
        id: '1',
        name: '网络文学',
        type: '网络小说',
        lastOpenedAt: '2026-03-11',
      },
      {
        id: '2',
        name: '个人笔记',
        type: '笔记',
        lastOpenedAt: '2026-03-08',
      },
    ];
  }

  function navigateToModule(moduleId: string) {
    window.location.href = `/${moduleId}`;
  }

  onMount(() => {
    loadRecentWorkspaces();
  });
</script>

<div class="home-page">
  <div class="home-header">
    <h1 class="main-title">NothingBut Library</h1>
    <p class="subtitle">选择资料类型开始管理</p>
  </div>

  <div class="module-grid">
    {#each modules as module (module.id)}
      {#if module.available}
        <button
          class="module-card"
          onclick={() => navigateToModule(module.id)}
        >
          <div class="module-icon">{module.icon}</div>
          <h3 class="module-name">{module.name}</h3>
        </button>
      {:else}
        <div class="module-card disabled">
          <div class="module-icon">{module.icon}</div>
          <h3 class="module-name">{module.name}</h3>
          <span class="coming-soon">即将推出</span>
        </div>
      {/if}
    {/each}
  </div>

  <!-- Recent Workspaces -->
  {#if recentWorkspaces.length > 0}
    <div class="recent-section">
      <h2 class="recent-title">最近使用</h2>
      <div class="recent-list">
        {#each recentWorkspaces as workspace (workspace.id)}
          <div class="recent-item">
            <div class="recent-info">
              <div class="recent-name">{workspace.name}</div>
              <div class="recent-meta">
                <span class="recent-type">{workspace.type}</span>
                <span class="recent-date">
                  {new Date(workspace.lastOpenedAt).toLocaleDateString('zh-CN')}
                </span>
              </div>
            </div>
            <button class="recent-open-btn">打开</button>
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .home-page {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 48px 24px;
    min-height: 100%;
    background-color: var(--color-bg-secondary);
  }

  .home-header {
    text-align: center;
    margin-bottom: 48px;
  }

  .main-title {
    font-size: 32px;
    font-weight: 700;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }

  .subtitle {
    font-size: 16px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .module-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 24px;
    width: 100%;
    max-width: 880px;
    margin-bottom: 64px;
  }

  .module-card {
    aspect-ratio: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 24px;
    border-radius: 12px;
    background-color: var(--color-bg-primary);
    border: 2px solid var(--color-border);
    transition: all 0.2s ease;
    cursor: pointer;
    position: relative;
  }

  .module-card:hover:not(.disabled) {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
    border-color: var(--color-primary);
  }

  .module-card.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .module-icon {
    font-size: 64px;
    line-height: 1;
  }

  .module-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .coming-soon {
    position: absolute;
    top: 16px;
    right: 16px;
    font-size: 12px;
    color: var(--color-text-secondary);
    background-color: var(--color-bg-secondary);
    padding: 4px 12px;
    border-radius: 12px;
  }

  .recent-section {
    width: 100%;
    max-width: 880px;
  }

  .recent-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 16px 0;
  }

  .recent-list {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .recent-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px;
    border-radius: 8px;
    background-color: var(--color-bg-primary);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
  }

  .recent-item:hover {
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  }

  .recent-info {
    flex: 1;
  }

  .recent-name {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin-bottom: 4px;
  }

  .recent-meta {
    display: flex;
    gap: 12px;
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .recent-type {
    font-weight: 500;
  }

  .recent-open-btn {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    color: var(--color-primary);
    background-color: transparent;
    border: 1px solid var(--color-primary);
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .recent-open-btn:hover {
    color: white;
    background-color: var(--color-primary);
  }
</style>
