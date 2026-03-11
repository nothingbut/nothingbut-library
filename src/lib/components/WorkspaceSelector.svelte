<script lang="ts">
  import { onMount } from 'svelte';

  interface Workspace {
    id: string;
    name: string;
    moduletype: string;
    lastOpenedAt: string;
  }

  let workspaces = $state<Workspace[]>([]);
  let selectedWorkspace = $state<Workspace | null>(null);

  function loadWorkspaces() {
    // TODO: Load workspaces from backend
    workspaces = [
      {
        id: '1',
        name: '网络文学',
        moduletype: 'novel',
        lastOpenedAt: '2026-03-10',
      },
      {
        id: '2',
        name: '科技资讯',
        moduletype: 'article',
        lastOpenedAt: '2026-03-08',
      },
      {
        id: '3',
        name: '个人笔记',
        moduletype: 'note',
        lastOpenedAt: '2026-03-05',
      },
    ];

    if (workspaces.length > 0) {
      selectedWorkspace = workspaces[0];
    }
  }

  function selectWorkspace(workspace: Workspace) {
    selectedWorkspace = workspace;
  }

  function createNewWorkspace() {
    // TODO: Implement new workspace creation
    console.log('Creating new workspace...');
  }

  onMount(() => {
    loadWorkspaces();
  });
</script>

<aside class="workspace-selector">
  <div class="workspace-header">
    <h2>工作区</h2>
  </div>

  <div class="workspace-list">
    {#each workspaces as workspace (workspace.id)}
      <button
        class="workspace-item"
        class:active={selectedWorkspace?.id === workspace.id}
        onclick={() => selectWorkspace(workspace)}
      >
        <div class="workspace-icon">
          {#if workspace.moduletype === 'novel'}
            📚
          {:else if workspace.moduletype === 'article'}
            📰
          {:else}
            📝
          {/if}
        </div>
        <div class="workspace-info">
          <div class="workspace-name">{workspace.name}</div>
          <div class="workspace-date">
            {new Date(workspace.lastOpenedAt).toLocaleDateString('zh-CN')}
          </div>
        </div>
      </button>
    {/each}
  </div>

  <button class="new-workspace-btn" onclick={createNewWorkspace}>
    + 新建工作区
  </button>
</aside>

<style>
  .workspace-selector {
    width: 240px;
    background-color: var(--color-bg-primary);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    flex-shrink: 0;
    overflow: hidden;
  }

  .workspace-header {
    padding: 16px;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .workspace-header h2 {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .workspace-list {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 8px;
  }

  .workspace-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 6px;
    margin-bottom: 8px;
    background-color: transparent;
    transition: all 0.2s ease;
  }

  .workspace-item:hover {
    background-color: var(--color-bg-secondary);
  }

  .workspace-item.active {
    background-color: var(--color-primary);
    color: white;
  }

  .workspace-icon {
    font-size: 20px;
    flex-shrink: 0;
  }

  .workspace-info {
    flex: 1;
    min-width: 0;
  }

  .workspace-name {
    font-size: 13px;
    font-weight: 500;
    color: inherit;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .workspace-item.active .workspace-name {
    color: white;
  }

  .workspace-date {
    font-size: 12px;
    color: var(--color-text-secondary);
    margin-top: 4px;
  }

  .workspace-item.active .workspace-date {
    color: rgba(255, 255, 255, 0.8);
  }

  .new-workspace-btn {
    margin: 8px;
    padding: 12px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    color: var(--color-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .new-workspace-btn:hover {
    background-color: var(--color-bg-hover);
    border-color: var(--color-primary);
  }
</style>
