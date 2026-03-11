<script lang="ts">
	interface Chapter {
		id: string;
		order: number;
		title: string;
		wordCount: number;
	}

	interface Props {
		chapters: Chapter[];
		currentChapterId: string | null;
		onSelectChapter: (id: string) => void;
	}

	let { chapters = [], currentChapterId = null, onSelectChapter } = $props();
</script>

<div class="chapter-list">
	<div class="list-header">
		<h2>Chapters</h2>
		<span class="chapter-count">{chapters.length}</span>
	</div>

	<div class="chapters-content">
		{#each chapters as chapter (chapter.id)}
			{@const isActive = currentChapterId === chapter.id}
			<button
				class="chapter-item {isActive ? 'active' : ''}"
				onclick={() => onSelectChapter(chapter.id)}
			>
				<div class="chapter-order">{chapter.order}</div>
				<div class="chapter-info">
					<div class="chapter-title">{chapter.title}</div>
					<div class="chapter-words">{chapter.wordCount.toLocaleString()} words</div>
				</div>
			</button>
		{/each}
	</div>
</div>

<style>
	.chapter-list {
		width: 280px;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-secondary);
		border-right: 1px solid var(--color-border);
		overflow: hidden;
	}

	.list-header {
		padding: 12px 16px;
		border-bottom: 1px solid var(--color-border);
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.list-header h2 {
		font-size: 14px;
		font-weight: 600;
		margin: 0;
		color: var(--color-text-primary);
	}

	.chapter-count {
		font-size: 12px;
		color: var(--color-text-secondary);
		background-color: var(--color-bg-hover);
		padding: 2px 8px;
		border-radius: 12px;
	}

	.chapters-content {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
		padding: 8px 0;
	}

	.chapter-item {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px 12px;
		text-align: left;
		background: transparent;
		border: none;
		color: var(--color-text-primary);
		cursor: pointer;
		transition: background-color 0.2s;
		border-left: 3px solid transparent;
	}

	.chapter-item:hover {
		background-color: var(--color-bg-hover);
	}

	.chapter-item.active {
		background-color: var(--color-primary);
		color: white;
		border-left-color: var(--color-primary);
	}

	.chapter-order {
		flex-shrink: 0;
		width: 24px;
		height: 24px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 12px;
		font-weight: 600;
		background-color: var(--color-bg-hover);
		border-radius: 4px;
		color: var(--color-text-secondary);
	}

	.chapter-item.active .chapter-order {
		background-color: rgba(255, 255, 255, 0.2);
		color: white;
	}

	.chapter-info {
		flex: 1;
		min-width: 0;
	}

	.chapter-title {
		font-size: 14px;
		font-weight: 500;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		margin-bottom: 2px;
	}

	.chapter-words {
		font-size: 12px;
		color: var(--color-text-secondary);
	}

	.chapter-item.active .chapter-words {
		color: rgba(255, 255, 255, 0.7);
	}

	/* Scrollbar styling */
	.chapters-content::-webkit-scrollbar {
		width: 6px;
	}

	.chapters-content::-webkit-scrollbar-track {
		background: transparent;
	}

	.chapters-content::-webkit-scrollbar-thumb {
		background: var(--color-border);
		border-radius: 3px;
	}

	.chapters-content::-webkit-scrollbar-thumb:hover {
		background: var(--color-text-secondary);
	}
</style>
