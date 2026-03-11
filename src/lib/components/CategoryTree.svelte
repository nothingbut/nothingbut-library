<script lang="ts">
  import { onMount } from 'svelte';
  import { listCategories, listBooks } from '$lib/services/api';
  import type { Category, Book, BookStatus } from '$lib/types';

  // Props
  interface Props {
    onSelectBook?: (bookId: number) => void;
  }

  let { onSelectBook }: Props = $props();

  // Types
  interface TreeNode {
    id: string | number;
    name: string;
    type: 'root' | 'category' | 'book';
    parentId: string | number | null;
    children: TreeNode[];
    expanded: boolean;
    // Book-specific fields
    bookId?: number;
    status?: BookStatus;
    // Category-specific fields
    categoryId?: number;
  }

  // State
  let tree = $state<TreeNode[]>([]);
  let selectedId = $state<string | number | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);

  // Build tree from API data
  async function loadTree() {
    try {
      loading = true;
      error = null;

      // Load data from API
      const [categories, books] = await Promise.all([
        listCategories(),
        listBooks()
      ]);

      // Build category map for easy lookup
      const categoryMap = new Map<number, TreeNode>();
      const rootNode: TreeNode = {
        id: 'root',
        name: '📚 全部小说',
        type: 'root',
        parentId: null,
        children: [],
        expanded: true,
      };

      // Create category nodes
      categories.forEach(cat => {
        const node: TreeNode = {
          id: `cat-${cat.id}`,
          categoryId: cat.id,
          name: `📁 ${cat.name}`,
          type: 'category',
          parentId: cat.parent_id ? `cat-${cat.parent_id}` : 'root',
          children: [],
          expanded: false,
        };
        categoryMap.set(cat.id, node);
      });

      // Build category tree hierarchy
      categoryMap.forEach(node => {
        if (node.parentId === 'root') {
          rootNode.children.push(node);
        } else {
          const parentCatId = typeof node.parentId === 'string'
            ? parseInt(node.parentId.replace('cat-', ''))
            : null;
          if (parentCatId !== null) {
            const parent = categoryMap.get(parentCatId);
            if (parent) {
              parent.children.push(node);
            }
          }
        }
      });

      // Add books to their categories
      books.forEach(book => {
        const bookNode: TreeNode = {
          id: `book-${book.id}`,
          bookId: book.id,
          name: book.title,
          type: 'book',
          status: book.status,
          parentId: book.category_id ? `cat-${book.category_id}` : 'root',
          children: [],
          expanded: false,
        };

        if (book.category_id) {
          const parent = categoryMap.get(book.category_id);
          if (parent) {
            parent.children.push(bookNode);
          } else {
            // If category not found, add to root
            rootNode.children.push(bookNode);
          }
        } else {
          // No category, add to root
          rootNode.children.push(bookNode);
        }
      });

      tree = [rootNode];
      loading = false;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to load data';
      loading = false;
      console.error('Failed to load tree data:', e);
    }
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
    loadTree();
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
  {#if loading}
    <div class="tree-status">Loading...</div>
  {:else if error}
    <div class="tree-error">{error}</div>
  {:else if tree.length === 0}
    <div class="tree-empty">No data available</div>
  {:else}
    {#each tree as root (root.id)}
      {@render renderNode(root, 0)}
    {/each}
  {/if}
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

  /* Status messages */
  .tree-status,
  .tree-error,
  .tree-empty {
    padding: 16px;
    text-align: center;
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  .tree-error {
    color: red;
  }
</style>
