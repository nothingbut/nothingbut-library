<script lang="ts">
	import type { SearchQuery } from '$lib/types/epub';

	interface Props {
		onSearch: (query: SearchQuery) => void;
	}

	let { onSearch }: Props = $props();

	let keyword = $state('');
	let showAdvanced = $state(false);

	// Advanced search fields
	let advancedQuery = $state({
		title: '',
		author: '',
		publisher: '',
		series: '',
		rating_min: '' as string | number,
		rating_max: '' as string | number,
	});

	let searchError = $state('');

	function handleBasicSearch() {
		showAdvanced = false;
		searchError = '';
		if (keyword.trim()) {
			onSearch({ keyword: keyword.trim() });
		} else {
			onSearch({});
		}
	}

	function handleAdvancedSearch() {
		searchError = '';
		const query: SearchQuery = {};

		// Trim and add text fields if non-empty
		const title = advancedQuery.title?.trim() || '';
		const author = advancedQuery.author?.trim() || '';
		const publisher = advancedQuery.publisher?.trim() || '';
		const series = advancedQuery.series?.trim() || '';

		if (title) query.title = title;
		if (author) query.author = author;
		if (publisher) query.publisher = publisher;
		if (series) query.series = series;

		// Handle rating fields (empty string means unlimited)
		const ratingMin = advancedQuery.rating_min !== '' ? Number(advancedQuery.rating_min) : undefined;
		const ratingMax = advancedQuery.rating_max !== '' ? Number(advancedQuery.rating_max) : undefined;

		// Validate rating range: min <= max
		if (ratingMin !== undefined && ratingMax !== undefined && ratingMin > ratingMax) {
			searchError = '最低评分不能高于最高评分';
			return;
		}

		if (ratingMin !== undefined) query.rating_min = ratingMin;
		if (ratingMax !== undefined) query.rating_max = ratingMax;

		onSearch(query);
		showAdvanced = false;
	}

	function handleClearAdvanced() {
		searchError = '';
		advancedQuery = {
			title: '',
			author: '',
			publisher: '',
			series: '',
			rating_min: '',
			rating_max: '',
		};
		onSearch({});
	}
</script>

<div class="search-container">
	<!-- Basic search bar -->
	<div class="search-bar">
		<div class="search-input-wrapper">
			<input
				type="text"
				bind:value={keyword}
				placeholder="搜索书名、作者、出版社..."
				class="search-input"
				onkeydown={(e) => e.key === 'Enter' && handleBasicSearch()}
			/>
			<button class="search-icon-btn" onclick={handleBasicSearch} title="搜索">
				🔍
			</button>
		</div>

		<button class="advanced-toggle" onclick={() => (showAdvanced = !showAdvanced)}>
			高级搜索
		</button>
	</div>

	<!-- Advanced search panel -->
	{#if showAdvanced}
		<div class="advanced-panel">
			<h3 class="advanced-title">高级搜索</h3>

			{#if searchError}
				<div class="error-message">
					{searchError}
				</div>
			{/if}

			<div class="advanced-grid">
				<!-- Title -->
				<div class="form-field">
					<label for="title" class="field-label">标题</label>
					<input
						id="title"
						type="text"
						bind:value={advancedQuery.title}
						placeholder="输入标题关键词"
						class="field-input"
					/>
				</div>

				<!-- Author -->
				<div class="form-field">
					<label for="author" class="field-label">作者</label>
					<input
						id="author"
						type="text"
						bind:value={advancedQuery.author}
						placeholder="输入作者名称"
						class="field-input"
					/>
				</div>

				<!-- Publisher -->
				<div class="form-field">
					<label for="publisher" class="field-label">出版社</label>
					<input
						id="publisher"
						type="text"
						bind:value={advancedQuery.publisher}
						placeholder="输入出版社名称"
						class="field-input"
					/>
				</div>

				<!-- Series -->
				<div class="form-field">
					<label for="series" class="field-label">系列</label>
					<input
						id="series"
						type="text"
						bind:value={advancedQuery.series}
						placeholder="输入系列名称"
						class="field-input"
					/>
				</div>

				<!-- Min rating -->
				<div class="form-field">
					<label for="rating_min" class="field-label">最低评分</label>
					<select id="rating_min" bind:value={advancedQuery.rating_min} class="field-select">
						<option value="">不限</option>
						<option value={1}>1 星</option>
						<option value={2}>2 星</option>
						<option value={3}>3 星</option>
						<option value={4}>4 星</option>
						<option value={5}>5 星</option>
					</select>
				</div>

				<!-- Max rating -->
				<div class="form-field">
					<label for="rating_max" class="field-label">最高评分</label>
					<select id="rating_max" bind:value={advancedQuery.rating_max} class="field-select">
						<option value="">不限</option>
						<option value={1}>1 星</option>
						<option value={2}>2 星</option>
						<option value={3}>3 星</option>
						<option value={4}>4 星</option>
						<option value={5}>5 星</option>
					</select>
				</div>
			</div>

			<!-- Action buttons -->
			<div class="advanced-actions">
				<button class="action-btn secondary" onclick={handleClearAdvanced}>
					清除
				</button>
				<button class="action-btn primary" onclick={handleAdvancedSearch}>
					搜索
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
	.search-container {
		position: relative;
	}

	/* Basic search bar */
	.search-bar {
		display: flex;
		gap: 12px;
	}

	.search-input-wrapper {
		position: relative;
		flex: 1;
	}

	.search-input {
		width: 100%;
		padding: 8px 40px 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 14px;
		color: var(--color-text-primary);
		background-color: var(--color-bg-primary);
		transition: all 0.2s ease;
	}

	.search-input:focus {
		outline: none;
		border-color: var(--color-primary);
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.search-input::placeholder {
		color: var(--color-text-tertiary);
	}

	.search-icon-btn {
		position: absolute;
		right: 8px;
		top: 50%;
		transform: translateY(-50%);
		font-size: 16px;
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px;
		color: var(--color-text-tertiary);
		transition: color 0.2s ease;
	}

	.search-icon-btn:hover {
		color: var(--color-text-primary);
	}

	.advanced-toggle {
		padding: 8px 16px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text-primary);
		background-color: var(--color-bg-primary);
		cursor: pointer;
		transition: all 0.2s ease;
		white-space: nowrap;
	}

	.advanced-toggle:hover {
		background-color: var(--color-bg-hover);
		border-color: var(--color-primary);
	}

	/* Advanced panel */
	.advanced-panel {
		position: absolute;
		left: 0;
		right: 0;
		top: calc(100% + 8px);
		z-index: 10;
		background-color: var(--color-bg-primary);
		border: 1px solid var(--color-border);
		border-radius: 8px;
		padding: 20px;
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
	}

	.advanced-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0 0 16px 0;
	}

	.error-message {
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 6px;
		padding: 12px;
		margin-bottom: 16px;
		color: #dc2626;
		font-size: 13px;
	}

	.advanced-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: 16px;
		margin-bottom: 20px;
	}

	.form-field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}

	.field-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.field-input,
	.field-select {
		padding: 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 14px;
		color: var(--color-text-primary);
		background-color: var(--color-bg-primary);
		transition: all 0.2s ease;
	}

	.field-input:focus,
	.field-select:focus {
		outline: none;
		border-color: var(--color-primary);
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.field-input::placeholder {
		color: var(--color-text-tertiary);
	}

	/* Action buttons */
	.advanced-actions {
		display: flex;
		justify-content: flex-end;
		gap: 12px;
	}

	.action-btn {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.action-btn.secondary {
		color: var(--color-text-primary);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
	}

	.action-btn.secondary:hover {
		background-color: var(--color-bg-hover);
	}

	.action-btn.primary {
		color: white;
		background-color: var(--color-primary);
		border: 1px solid var(--color-primary);
	}

	.action-btn.primary:hover {
		opacity: 0.9;
	}
</style>
