<script lang="ts">
  import { onMount } from 'svelte';

  // Props
  interface Props {
    onSelectBook?: (bookId: number) => void;
  }

  let { onSelectBook }: Props = $props();

  // Types
  interface TreeNode {
    id: string | number;
    name: string;
    type: 'root' | 'category-1' | 'category-2' | 'book';
    parentId: string | number | null;
    children: TreeNode[];
    expanded: boolean;
    // Book-specific fields
    bookId?: number;
    status?: 'completed' | 'ongoing' | 'abandoned';
  }

  // State
  let tree = $state<TreeNode[]>([]);
  let selectedId = $state<string | number | null>(null);

  // Build mock tree data
  function loadMockTree() {
    const mockData: TreeNode[] = [
      {
        id: 'root',
        name: '📚 全部小说',
        type: 'root',
        parentId: null,
        children: [],
        expanded: true,
      },
    ];

    // L2: First-level categories
    const cat1: TreeNode = {
      id: 'cat-scifi',
      name: '📁 科幻',
      type: 'category-1',
      parentId: 'root',
      children: [],
      expanded: false,
    };

    const cat2: TreeNode = {
      id: 'cat-history',
      name: '📁 历史',
      type: 'category-1',
      parentId: 'root',
      children: [],
      expanded: false,
    };

    const cat3: TreeNode = {
      id: 'cat-fantasy',
      name: '📁 玄幻',
      type: 'category-1',
      parentId: 'root',
      children: [],
      expanded: false,
    };

    // L3: Second-level categories (sub-categories)
    const subcat1: TreeNode = {
      id: 'subcat-space',
      name: '📂 太空歌剧',
      type: 'category-2',
      parentId: 'cat-scifi',
      children: [],
      expanded: false,
    };

    const subcat2: TreeNode = {
      id: 'subcat-apocalypse',
      name: '📂 末世幻想',
      type: 'category-2',
      parentId: 'cat-scifi',
      children: [],
      expanded: false,
    };

    const subcat3: TreeNode = {
      id: 'subcat-ancient',
      name: '📂 古代',
      type: 'category-2',
      parentId: 'cat-history',
      children: [],
      expanded: false,
    };

    // L4: Books
    const book1: TreeNode = {
      id: 'book-1',
      bookId: 1,
      name: '三体',
      type: 'book',
      status: 'completed',
      parentId: 'subcat-space',
      children: [],
      expanded: false,
    };

    const book2: TreeNode = {
      id: 'book-2',
      bookId: 2,
      name: '流浪地球',
      type: 'book',
      status: 'completed',
      parentId: 'subcat-space',
      children: [],
      expanded: false,
    };

    const book3: TreeNode = {
      id: 'book-3',
      bookId: 3,
      name: '全球高武',
      type: 'book',
      status: 'ongoing',
      parentId: 'subcat-apocalypse',
      children: [],
      expanded: false,
    };

    const book4: TreeNode = {
      id: 'book-4',
      bookId: 4,
      name: '明朝那些事儿',
      type: 'book',
      status: 'completed',
      parentId: 'subcat-ancient',
      children: [],
      expanded: false,
    };

    const book5: TreeNode = {
      id: 'book-5',
      bookId: 5,
      name: '某未完成小说',
      type: 'book',
      status: 'abandoned',
      parentId: 'subcat-ancient',
      children: [],
      expanded: false,
    };

    // Build tree structure
    subcat1.children.push(book1, book2);
    subcat2.children.push(book3);
    subcat3.children.push(book4, book5);

    cat1.children.push(subcat1, subcat2);
    cat2.children.push(subcat3);

    mockData[0].children.push(cat1, cat2, cat3);

    tree = mockData;
  }

  // Toggle expand/collapse
  function toggleExpand(node: TreeNode) {
    node.expanded = !node.expanded;
    tree = tree; // trigger reactivity
  }

  // Select node
  function selectNode(node: TreeNode) {
    selectedId = node.id;

    // If it's a book, call the callback
    if (node.type === 'book' && node.bookId && onSelectBook) {
      onSelectBook(node.bookId);
    }
  }

  // Get status icon
  function getStatusIcon(status?: TreeNode['status']): string {
    if (!status) return '';
    const icons = {
      completed: '✓',
      ongoing: '⏳',
      abandoned: '⚠',
    };
    return icons[status];
  }

  // Get status color
  function getStatusColor(status?: TreeNode['status']): string {
    if (!status) return '';
    const colors = {
      completed: 'green',
      ongoing: 'orange',
      abandoned: 'red',
    };
    return colors[status];
  }

  onMount(() => {
    loadMockTree();
  });
</script>

{#snippet renderNode(node: TreeNode, level: number)}
  {@const hasChildren = node.children.length > 0}
  {@const isSelected = selectedId === node.id}
  {@const isBook = node.type === 'book'}

  <div class="tree-node" style="--level: {level}">
    <div class="node-header {isSelected ? 'selected' : ''}">
      {#if hasChildren}
        <button
          class="expand-toggle"
          onclick={() => toggleExpand(node)}
          aria-label={node.expanded ? '收起' : '展开'}
        >
          {node.expanded ? '▼' : '▶'}
        </button>
      {:else}
        <div class="expand-placeholder"></div>
      {/if}

      <button class="node-content" onclick={() => selectNode(node)}>
        {#if isBook && node.status}
          <span
            class="status-icon"
            style="color: {getStatusColor(node.status)}"
          >
            {getStatusIcon(node.status)}
          </span>
        {/if}
        <span class="node-name">{node.name}</span>
      </button>
    </div>

    {#if hasChildren && node.expanded}
      <div class="node-children">
        {#each node.children as child (child.id)}
          {@render renderNode(child, level + 1)}
        {/each}
      </div>
    {/if}
  </div>
{/snippet}

<div class="category-tree">
  {#each tree as root (root.id)}
    {@render renderNode(root, 0)}
  {/each}
</div>

<style>
  .category-tree {
    width: 100%;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 8px 0;
  }

  .tree-node {
    padding-left: calc(var(--level, 0) * 16px);
  }

  .node-header {
    display: flex;
    align-items: center;
    height: 32px;
    padding: 0 8px;
    border-radius: 4px;
    transition: all 0.2s ease;
  }

  .node-header:hover {
    background-color: var(--color-bg-hover);
  }

  .node-header.selected {
    background-color: var(--color-primary);
  }

  .expand-toggle {
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    color: var(--color-text-secondary);
    flex-shrink: 0;
    transition: color 0.2s ease;
  }

  .expand-toggle:hover {
    color: var(--color-text-primary);
  }

  .node-header.selected .expand-toggle {
    color: white;
  }

  .expand-placeholder {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }

  .node-content {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 6px;
    text-align: left;
    padding: 0 4px;
    color: inherit;
  }

  .status-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .node-header.selected .status-icon {
    filter: brightness(2);
  }

  .node-name {
    flex: 1;
    font-size: 14px;
    color: var(--color-text-primary);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: color 0.2s ease;
  }

  .node-header.selected .node-name {
    color: white;
    font-weight: 500;
  }

  .node-children {
    display: contents;
  }

  /* Scrollbar styling */
  .category-tree::-webkit-scrollbar {
    width: 6px;
  }

  .category-tree::-webkit-scrollbar-track {
    background: transparent;
  }

  .category-tree::-webkit-scrollbar-thumb {
    background: var(--color-border);
    border-radius: 3px;
  }

  .category-tree::-webkit-scrollbar-thumb:hover {
    background: var(--color-text-secondary);
  }
</style>
