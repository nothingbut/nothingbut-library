<script lang="ts">
	import { onMount } from 'svelte';

	interface Chapter {
		id: string;
		title: string;
		content?: string; // 可选：外部传入的内容
	}

	interface Props {
		chapter: Chapter | null;
		bookDir: string;
		externalContent?: string; // 可选：外部传入的章节内容（用于/novel页面）
		onBack?: () => void; // 可选：返回按钮回调
	}

	interface ReaderSettings {
		fontSize: number;
		lineHeight: number;
		theme: 'light' | 'sepia' | 'dark';
		paragraphIndent: number; // 段首缩进（汉字数量，0表示不缩进）
		paragraphSpacing: number;
		pageMargin: number;
		fontFamily: string;
		backgroundColor: string;
	}

	let { chapter = null, bookDir = '', externalContent = undefined, onBack = undefined } = $props();

	let content = $state('');
	let paragraphs = $state<string[]>([]); // 段落数组
	let loading = $state(false);
	let showSettings = $state(false);

	// 阅读设置
	let settings = $state<ReaderSettings>({
		fontSize: 16,
		lineHeight: 1.8,
		theme: 'light',
		paragraphIndent: 2, // 默认缩进2个汉字
		paragraphSpacing: 1,
		pageMargin: 80,
		fontFamily: 'system-ui',
		backgroundColor: '#ffffff'
	});

	// 加载设置从 localStorage
	function loadSettings() {
		const saved = localStorage.getItem('reader-settings');
		if (saved) {
			try {
				settings = { ...settings, ...JSON.parse(saved) };
			} catch (e) {
				console.error('Failed to load settings:', e);
			}
		}
	}

	// 保存设置到 localStorage
	function saveSettings() {
		localStorage.setItem('reader-settings', JSON.stringify(settings));
	}

	// 处理内容：去除段首空格，返回段落数组
	function processContent(raw: string): string[] {
		if (!raw) return [];

		// 按段落分割，过滤空行
		const paras = raw.split('\n').filter(p => p.trim());

		// 处理每个段落：去除段首空格
		return paras.map(p => p.trim());
	}

	async function loadChapterContent() {
		if (!chapter) {
			paragraphs = [];
			return;
		}

		// 如果有外部传入的内容，直接使用
		if (externalContent !== undefined) {
			paragraphs = processContent(externalContent);
			return;
		}

		loading = true;
		try {
			// Simulated content loading (仅用于独立阅读器页面)
			const rawContent = `  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
  Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
  Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris.`;

			paragraphs = processContent(rawContent);
		} finally {
			loading = false;
		}
	}

	function increaseFontSize() {
		console.log('[Reader] Increase font size, current:', settings.fontSize);
		if (settings.fontSize < 32) {
			settings = { ...settings, fontSize: settings.fontSize + 2 };
			saveSettings();
			console.log('[Reader] New font size:', settings.fontSize);
		}
	}

	function decreaseFontSize() {
		console.log('[Reader] Decrease font size, current:', settings.fontSize);
		if (settings.fontSize > 12) {
			settings = { ...settings, fontSize: settings.fontSize - 2 };
			saveSettings();
			console.log('[Reader] New font size:', settings.fontSize);
		}
	}

	function changeTheme(newTheme: 'light' | 'sepia' | 'dark') {
		console.log('[Reader] Change theme from', settings.theme, 'to', newTheme);
		settings = { ...settings, theme: newTheme };
		saveSettings();
		console.log('[Reader] Theme changed to:', settings.theme);
	}

	function toggleSettings() {
		console.log('[Reader] Toggle settings, current:', showSettings);
		showSettings = !showSettings;
	}

	function updateSetting<K extends keyof ReaderSettings>(key: K, value: ReaderSettings[K]) {
		console.log('[Reader] Update setting:', key, '=', value);
		settings = { ...settings, [key]: value };
		saveSettings();
	}

	$effect(() => {
		if (chapter) {
			loadChapterContent();
		}
	});

	// 当外部内容变化时，重新加载
	$effect(() => {
		if (externalContent !== undefined) {
			loadChapterContent();
		}
	});

	onMount(() => {
		loadSettings();
		if (chapter) {
			loadChapterContent();
		}
	});
</script>

<div class="reader" data-theme={settings.theme} style="--custom-bg: {settings.backgroundColor}">
	<div class="toolbar">
		<!-- ✨ UPDATED VERSION 2026-03-26 ✨ -->
		<div class="chapter-title">
			{#if chapter}
				<h1>{chapter.title}</h1>
			{:else}
				<h1>No chapter selected</h1>
			{/if}
		</div>

		<div class="toolbar-controls">
			<button
				class="control-button"
				onclick={decreaseFontSize}
				title="减小字号"
				disabled={settings.fontSize <= 12}
			>
				A−
			</button>
			<span class="font-size-display">{settings.fontSize}px</span>
			<button
				class="control-button"
				onclick={increaseFontSize}
				title="增大字号"
				disabled={settings.fontSize >= 32}
			>
				A+
			</button>

			<button
				class="theme-button {settings.theme === 'light' ? 'active' : ''}"
				onclick={() => changeTheme('light')}
				title="明亮"
			>
				☀️
			</button>
			<button
				class="theme-button {settings.theme === 'sepia' ? 'active' : ''}"
				onclick={() => changeTheme('sepia')}
				title="护眼"
			>
				📄
			</button>
			<button
				class="theme-button {settings.theme === 'dark' ? 'active' : ''}"
				onclick={() => changeTheme('dark')}
				title="夜间"
			>
				🌙
			</button>

			<button class="icon-button" onclick={toggleSettings} title="阅读设置">
				⚙️
			</button>

			{#if onBack}
				<button class="back-button" onclick={onBack} title="返回书籍">
					返回书籍
				</button>
			{/if}
		</div>
	</div>

	<div class="content-area" style="padding: 40px {settings.pageMargin}px">
		{#if loading}
			<div class="loading">加载中...</div>
		{:else if paragraphs.length === 0}
			<div class="empty">选择章节开始阅读</div>
		{:else}
			<div
				class="content"
				style="
					font-size: {settings.fontSize}px;
					line-height: {settings.lineHeight};
					font-family: {settings.fontFamily};
				"
			>
				{#each paragraphs as paragraph, i (i)}
					<p
						class="paragraph"
						style="
							text-indent: {settings.paragraphIndent}em;
							margin-bottom: {settings.paragraphSpacing}em;
						"
					>
						{paragraph}
					</p>
				{/each}
			</div>
		{/if}
	</div>

	<!-- 设置面板 -->
	{#if showSettings}
		<div class="settings-panel">
			<div class="settings-header">
				<h3>阅读设置</h3>
				<button class="close-button" onclick={toggleSettings}>✕</button>
			</div>

			<div class="settings-content">
				<!-- 段首缩进 -->
				<div class="setting-item">
					<label class="setting-label">段首缩进（汉字数）</label>
					<div class="indent-control">
						<input
							type="number"
							min="0"
							max="8"
							value={settings.paragraphIndent}
							oninput={(e) => updateSetting('paragraphIndent', parseInt(e.currentTarget.value) || 0)}
							class="indent-input"
						/>
						<span class="indent-unit">字</span>
					</div>
					<p class="setting-hint">设置为0表示不缩进，自动去除原文段首空格</p>
				</div>

				<!-- 段间距 -->
				<div class="setting-item">
					<label class="setting-label">段间距</label>
					<input
						type="range"
						min="0.5"
						max="3"
						step="0.5"
						value={settings.paragraphSpacing}
						oninput={(e) => updateSetting('paragraphSpacing', parseFloat(e.currentTarget.value))}
					/>
					<span class="setting-value">{settings.paragraphSpacing}em</span>
				</div>

				<!-- 页边距 -->
				<div class="setting-item">
					<label class="setting-label">页边距</label>
					<input
						type="range"
						min="20"
						max="200"
						step="20"
						value={settings.pageMargin}
						oninput={(e) => updateSetting('pageMargin', parseInt(e.currentTarget.value))}
					/>
					<span class="setting-value">{settings.pageMargin}px</span>
				</div>

				<!-- 行高 -->
				<div class="setting-item">
					<label class="setting-label">行高</label>
					<input
						type="range"
						min="1.2"
						max="2.5"
						step="0.1"
						value={settings.lineHeight}
						oninput={(e) => updateSetting('lineHeight', parseFloat(e.currentTarget.value))}
					/>
					<span class="setting-value">{settings.lineHeight}</span>
				</div>

				<!-- 字体 -->
				<div class="setting-item">
					<label class="setting-label">字体</label>
					<select
						value={settings.fontFamily}
						onchange={(e) => updateSetting('fontFamily', e.currentTarget.value)}
						class="setting-select"
					>
						<option value="system-ui">系统默认</option>
						<option value="'Songti SC', serif">宋体</option>
						<option value="'Heiti SC', sans-serif">黑体</option>
						<option value="'Kaiti SC', serif">楷体</option>
						<option value="'PingFang SC', sans-serif">苹方</option>
						<option value="Georgia, serif">Georgia</option>
						<option value="'Times New Roman', serif">Times New Roman</option>
					</select>
				</div>

				<!-- 背景颜色 -->
				<div class="setting-item">
					<label class="setting-label">背景颜色</label>
					<div class="color-picker">
						<input
							type="color"
							value={settings.backgroundColor}
							oninput={(e) => updateSetting('backgroundColor', e.currentTarget.value)}
						/>
						<span class="color-value">{settings.backgroundColor}</span>
					</div>
				</div>
			</div>
		</div>
		<div class="settings-overlay" onclick={toggleSettings}></div>
	{/if}
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
		--reader-bg: var(--custom-bg, #ffffff);
		--reader-text: #1a1a1a;
		--reader-accent: #f0f0f0;
	}

	.reader[data-theme='sepia'] {
		--reader-bg: var(--custom-bg, #f4ecd8);
		--reader-text: #5c4033;
		--reader-accent: #e8dcc8;
	}

	.reader[data-theme='dark'] {
		--reader-bg: var(--custom-bg, #1a1a1a);
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
		min-height: 80px;
		flex-shrink: 0;
	}

	.chapter-title {
		flex: 1;
		min-width: 0;
		overflow: hidden;
	}

	.chapter-title h1 {
		font-size: 24px;
		font-weight: 600;
		margin: 0;
		color: var(--reader-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.toolbar-controls {
		display: flex;
		gap: 12px;
		align-items: center;
		flex-wrap: nowrap;
		flex-shrink: 0;
	}

	.control-button {
		min-width: 32px;
		height: 32px;
		padding: 4px 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--reader-bg);
		border: 1px solid var(--color-border);
		color: var(--reader-text);
		cursor: pointer;
		font-weight: 600;
		font-size: 14px;
		border-radius: 6px;
		transition: all 0.2s;
	}

	.control-button:hover:not(:disabled) {
		background-color: var(--color-primary);
		color: white;
		border-color: var(--color-primary);
	}

	.control-button:active:not(:disabled) {
		transform: scale(0.95);
	}

	.control-button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.font-size-display {
		font-size: 12px;
		color: var(--reader-text);
		min-width: 40px;
		text-align: center;
	}

	.theme-button {
		min-width: 36px;
		height: 36px;
		padding: 4px 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--reader-bg);
		border: 1px solid var(--color-border);
		font-size: 16px;
		cursor: pointer;
		border-radius: 6px;
		transition: all 0.2s;
	}

	.theme-button:hover {
		background-color: var(--reader-accent);
		transform: scale(1.05);
	}

	.theme-button:active {
		transform: scale(0.95);
	}

	.theme-button.active {
		background-color: var(--color-primary);
		border-color: var(--color-primary);
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
	}

	.icon-button {
		min-width: 36px;
		height: 36px;
		padding: 4px 8px;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--reader-bg);
		border: 1px solid var(--color-border);
		font-size: 16px;
		cursor: pointer;
		border-radius: 6px;
		transition: all 0.2s;
	}

	.icon-button:hover {
		background-color: var(--reader-accent);
		transform: scale(1.05);
	}

	.icon-button:active {
		transform: scale(0.95);
	}

	.back-button {
		min-width: 80px;
		height: 36px;
		padding: 4px 12px;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		background-color: var(--color-primary);
		color: white;
		border: none;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		border-radius: 6px;
		transition: all 0.2s;
	}

	.back-button:hover {
		background-color: var(--color-primary);
		filter: brightness(1.1);
		transform: translateY(-1px);
	}

	.back-button:active {
		transform: translateY(0);
	}

	.content-area {
		flex: 1;
		overflow-y: auto;
		background-color: var(--reader-bg);
		color: var(--reader-text);
		display: flex;
		align-items: flex-start; /* 改为顶部对齐，避免居中时内容被遮挡 */
		justify-content: center;
		padding: 40px 24px;
	}

	.content {
		max-width: 800px;
		width: 100%;
		padding-top: 20px; /* 确保内容与container顶部有间距 */
		padding-bottom: 40px; /* 确保底部有足够空间 */
	}

	.paragraph {
		margin: 0; /* 移除默认margin */
		word-wrap: break-word;
		overflow-wrap: break-word;
	}

	/* 最后一个段落不需要底部间距 */
	.paragraph:last-child {
		margin-bottom: 0;
	}

	/* 设置面板 */
	.settings-overlay {
		position: fixed;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: rgba(0, 0, 0, 0.5);
		z-index: 999;
	}

	.settings-panel {
		position: fixed;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 90%;
		max-width: 500px;
		background: var(--reader-bg);
		border-radius: 12px;
		box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
		z-index: 1000;
	}

	.settings-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 20px;
		border-bottom: 1px solid var(--color-border);
	}

	.settings-header h3 {
		margin: 0;
		font-size: 18px;
		color: var(--reader-text);
	}

	.close-button {
		width: 32px;
		height: 32px;
		border: none;
		background: transparent;
		font-size: 20px;
		cursor: pointer;
		border-radius: 6px;
		color: var(--reader-text);
	}

	.close-button:hover {
		background: var(--reader-accent);
	}

	.settings-content {
		padding: 20px;
		max-height: 60vh;
		overflow-y: auto;
	}

	.setting-item {
		margin-bottom: 20px;
	}

	.setting-label {
		display: block;
		margin-bottom: 8px;
		font-size: 14px;
		font-weight: 500;
		color: var(--reader-text);
	}

	.setting-label input[type='checkbox'] {
		margin-right: 8px;
	}

	input[type='range'] {
		width: calc(100% - 80px);
		margin-right: 12px;
		vertical-align: middle;
	}

	.setting-value {
		display: inline-block;
		width: 60px;
		text-align: right;
		font-size: 14px;
		color: var(--reader-text);
	}

	.setting-select {
		width: 100%;
		padding: 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--reader-bg);
		color: var(--reader-text);
		font-size: 14px;
	}

	.indent-control {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.indent-input {
		width: 80px;
		padding: 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		background: var(--reader-bg);
		color: var(--reader-text);
		font-size: 14px;
		text-align: center;
	}

	.indent-unit {
		font-size: 14px;
		color: var(--reader-text);
	}

	.setting-hint {
		margin-top: 4px;
		font-size: 12px;
		color: var(--color-text-secondary);
		line-height: 1.4;
	}

	.color-picker {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.color-picker input[type='color'] {
		width: 60px;
		height: 36px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		cursor: pointer;
	}

	.color-value {
		font-size: 14px;
		color: var(--reader-text);
		font-family: monospace;
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
