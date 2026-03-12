<script lang="ts">
	import { onMount } from 'svelte';

	interface Chapter {
		id: string;
		title: string;
	}

	interface Props {
		chapter: Chapter | null;
		bookDir: string;
	}

	let { chapter = null, bookDir = '' } = $props();

	let content = $state('');
	let loading = $state(false);
	let fontSize = $state(16);
	let lineHeight = $state(1.6);
	let theme = $state<'light' | 'sepia' | 'dark'>('light');

	async function loadChapterContent() {
		// TODO: Replace with actual file loading from bookDir
		if (!chapter) {
			content = '';
			return;
		}

		loading = true;
		try {
			// Simulated content loading
			content = `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.

Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.

Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.`;
		} finally {
			loading = false;
		}
	}

	function increaseFontSize() {
		if (fontSize < 32) {
			fontSize += 2;
		}
	}

	function decreaseFontSize() {
		if (fontSize > 12) {
			fontSize -= 2;
		}
	}

	function changeTheme(newTheme: 'light' | 'sepia' | 'dark') {
		theme = newTheme;
	}

	$effect(() => {
		if (chapter) {
			loadChapterContent();
		}
	});

	onMount(() => {
		if (chapter) {
			loadChapterContent();
		}
	});
</script>

<div class="reader" data-theme={theme}>
	<div class="toolbar">
		<div class="chapter-title">
			{#if chapter}
				<h1>{chapter.title}</h1>
			{:else}
				<h1>No chapter selected</h1>
			{/if}
		</div>

		<div class="toolbar-controls">
			<div class="font-controls">
				<button
					class="control-button"
					onclick={decreaseFontSize}
					title="Decrease font size"
					disabled={fontSize <= 12}
				>
					A−
				</button>
				<span class="font-size-display">{fontSize}px</span>
				<button
					class="control-button"
					onclick={increaseFontSize}
					title="Increase font size"
					disabled={fontSize >= 32}
				>
					A+
				</button>
			</div>

			<div class="theme-controls">
				<button
					class="theme-button {theme === 'light' ? 'active' : ''}"
					onclick={() => changeTheme('light')}
					title="Light theme"
				>
					☀️
				</button>
				<button
					class="theme-button {theme === 'sepia' ? 'active' : ''}"
					onclick={() => changeTheme('sepia')}
					title="Sepia theme"
				>
					📄
				</button>
				<button
					class="theme-button {theme === 'dark' ? 'active' : ''}"
					onclick={() => changeTheme('dark')}
					title="Dark theme"
				>
					🌙
				</button>
			</div>
		</div>
	</div>

	<div class="content-area">
		{#if loading}
			<div class="loading">Loading chapter...</div>
		{:else if !content}
			<div class="empty">Select a chapter to start reading</div>
		{:else}
			<div
				class="content"
				style="
					font-size: {fontSize}px;
					line-height: {lineHeight};
				"
			>
				{content}
			</div>
		{/if}
	</div>
</div>

<style>
	.reader {
		flex: 1;
		display: flex;
		flex-direction: column;
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
		overflow: hidden;
	}

	/* Theme-specific CSS variables */
	.reader[data-theme='light'] {
		--reader-bg: #ffffff;
		--reader-text: #1a1a1a;
		--reader-accent: #f0f0f0;
	}

	.reader[data-theme='sepia'] {
		--reader-bg: #f4ecd8;
		--reader-text: #5c4033;
		--reader-accent: #e8dcc8;
	}

	.reader[data-theme='dark'] {
		--reader-bg: #1a1a1a;
		--reader-text: #e0e0e0;
		--reader-accent: #2a2a2a;
	}

	.toolbar {
		padding: 20px 24px;
		border-bottom: 1px solid var(--color-border);
		background-color: var(--reader-accent);
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 24px;
	}

	.chapter-title {
		flex: 1;
	}

	.chapter-title h1 {
		font-size: 24px;
		font-weight: 600;
		margin: 0;
		color: var(--reader-text);
	}

	.toolbar-controls {
		display: flex;
		gap: 20px;
		align-items: center;
	}

	.font-controls {
		display: flex;
		align-items: center;
		gap: 8px;
		background-color: var(--reader-bg);
		padding: 4px 12px;
		border-radius: 6px;
	}

	.control-button {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: none;
		color: var(--reader-text);
		cursor: pointer;
		font-weight: 600;
		font-size: 14px;
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.control-button:hover:not(:disabled) {
		background-color: var(--color-primary);
		color: white;
	}

	.control-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.font-size-display {
		font-size: 12px;
		color: var(--reader-text);
		min-width: 40px;
		text-align: center;
	}

	.theme-controls {
		display: flex;
		gap: 8px;
		background-color: var(--reader-bg);
		padding: 4px;
		border-radius: 6px;
	}

	.theme-button {
		width: 32px;
		height: 32px;
		display: flex;
		align-items: center;
		justify-content: center;
		background: transparent;
		border: none;
		font-size: 16px;
		cursor: pointer;
		border-radius: 4px;
		transition: background-color 0.2s;
	}

	.theme-button:hover {
		background-color: var(--color-bg-hover);
	}

	.theme-button.active {
		background-color: var(--color-primary);
	}

	.content-area {
		flex: 1;
		overflow-y: auto;
		background-color: var(--reader-bg);
		color: var(--reader-text);
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 40px 24px;
	}

	.content {
		max-width: 800px;
		width: 100%;
		white-space: pre-wrap;
		word-wrap: break-word;
	}

	.loading,
	.empty {
		font-size: 16px;
		color: var(--reader-text);
		opacity: 0.6;
	}

	/* Scrollbar styling */
	.content-area::-webkit-scrollbar {
		width: 8px;
	}

	.content-area::-webkit-scrollbar-track {
		background: var(--reader-accent);
	}

	.content-area::-webkit-scrollbar-thumb {
		background: var(--color-border);
		border-radius: 4px;
	}

	.content-area::-webkit-scrollbar-thumb:hover {
		background: var(--color-text-secondary);
	}
</style>
