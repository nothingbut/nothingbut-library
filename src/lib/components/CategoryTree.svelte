<script lang="ts">
	import { onMount } from 'svelte';
	import type { CategoryNode } from '../types';

	let categories = $state<CategoryNode[]>([]);
	let selectedCategory = $state<CategoryNode | null>(null);

	function buildTree(flatCategories: any[]): CategoryNode[] {
		const nodeMap = new Map<string, CategoryNode>();

		// First pass: create all nodes
		flatCategories.forEach((cat) => {
			nodeMap.set(cat.id, {
				...cat,
				children: [],
				expanded: false
			});
		});

		// Second pass: build parent-child relationships
		const roots: CategoryNode[] = [];
		nodeMap.forEach((node) => {
			if (node.parentId === null) {
				roots.push(node);
			} else {
				const parent = nodeMap.get(node.parentId);
				if (parent) {
					parent.children.push(node);
				}
			}
		});

		// Sort by sortOrder
		const sortByOrder = (nodes: CategoryNode[]) => {
			nodes.sort((a, b) => a.sortOrder - b.sortOrder);
			nodes.forEach((node) => sortByOrder(node.children));
		};
		sortByOrder(roots);

		return roots;
	}

	function loadCategories() {
		// TODO: Replace with API call to backend
		const flatData = [
			{
				id: '1',
				name: 'Fiction',
				parentId: null,
				sortOrder: 1,
				createdAt: new Date()
			},
			{
				id: '2',
				name: 'Science Fiction',
				parentId: '1',
				sortOrder: 1,
				createdAt: new Date()
			},
			{
				id: '3',
				name: 'Fantasy',
				parentId: '1',
				sortOrder: 2,
				createdAt: new Date()
			},
			{
				id: '4',
				name: 'Non-Fiction',
				parentId: null,
				sortOrder: 2,
				createdAt: new Date()
			},
			{
				id: '5',
				name: 'History',
				parentId: '4',
				sortOrder: 1,
				createdAt: new Date()
			},
			{
				id: '6',
				name: 'Biography',
				parentId: '4',
				sortOrder: 2,
				createdAt: new Date()
			},
			{
				id: '7',
				name: 'Science',
				parentId: null,
				sortOrder: 3,
				createdAt: new Date()
			}
		];

		categories = buildTree(flatData);
	}

	function toggleExpand(node: CategoryNode) {
		node.expanded = !node.expanded;
		categories = categories; // trigger reactivity
	}

	function selectCategory(node: CategoryNode) {
		selectedCategory = selectedCategory?.id === node.id ? null : node;
	}

	onMount(() => {
		loadCategories();
	});
</script>

{#snippet renderNode(node: CategoryNode, level: number)}
	{@const hasChildren = node.children.length > 0}
	{@const isSelected = selectedCategory?.id === node.id}
	<div class="category-item" style="--level: {level}">
		<div class="category-header {isSelected ? 'selected' : ''}">
			{#if hasChildren}
				<button
					class="expand-toggle"
					onclick={() => toggleExpand(node)}
					aria-label={node.expanded ? 'Collapse' : 'Expand'}
				>
					{node.expanded ? '▼' : '▶'}
				</button>
			{:else}
				<div class="expand-toggle-placeholder"></div>
			{/if}
			<button class="category-name" onclick={() => selectCategory(node)}>
				{node.name}
			</button>
		</div>
		{#if hasChildren && node.expanded}
			<div class="category-children">
				{#each node.children as child (child.id)}
					{@render renderNode(child, level + 1)}
				{/each}
			</div>
		{/if}
	</div>
{/snippet}

<div class="category-tree">
	<div class="tree-header">
		<h2>Categories</h2>
		<button class="add-button" title="Add new category">+</button>
	</div>

	<div class="tree-content">
		{#each categories as root (root.id)}
			{@render renderNode(root, 0)}
		{/each}
	</div>
</div>



<style>
	.category-tree {
		width: 240px;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-secondary);
		border-right: 1px solid var(--color-border);
		overflow: hidden;
	}

	.tree-header {
		padding: 12px 16px;
		border-bottom: 1px solid var(--color-border);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.tree-header h2 {
		font-size: 14px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text-primary);
	}

	.add-button {
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 16px;
		color: var(--color-primary);
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.add-button:hover {
		background-color: var(--color-bg-hover);
	}

	.tree-content {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 8px 0;
	}

	.category-item {
		padding-left: calc(var(--level, 0) * 16px);
	}

	.category-header {
		display: flex;
		align-items: center;
		height: 32px;
		padding: 0 8px;
		transition: background-color 0.2s;
	}

	.category-header:hover {
		background-color: var(--color-bg-hover);
	}

	.category-header.selected {
		background-color: var(--color-primary);
		color: white;
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
		transition: color 0.2s;
	}

	.expand-toggle:hover {
		color: var(--color-text-primary);
	}

	.category-header.selected .expand-toggle {
		color: white;
	}

	.expand-toggle-placeholder {
		width: 20px;
		height: 20px;
		flex-shrink: 0;
	}

	.category-name {
		flex: 1;
		text-align: left;
		color: inherit;
		font-size: 14px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		padding: 0 4px;
		transition: color 0.2s;
	}

	.category-header.selected .category-name {
		color: white;
		font-weight: 500;
	}

	.category-children {
		display: contents;
	}

	/* Scrollbar styling */
	.tree-content::-webkit-scrollbar {
		width: 6px;
	}

	.tree-content::-webkit-scrollbar-track {
		background: transparent;
	}

	.tree-content::-webkit-scrollbar-thumb {
		background: var(--color-border);
		border-radius: 3px;
	}

	.tree-content::-webkit-scrollbar-thumb:hover {
		background: var(--color-text-secondary);
	}
</style>
