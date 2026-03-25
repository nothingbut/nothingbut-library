<script lang="ts">
	import type { SearchQuery } from '$lib/types/epub';

	interface Props {
		onSearch: (query: SearchQuery) => void;
	}

	let { onSearch }: Props = $props();

	let keyword = $state('');
	let showAdvanced = $state(false);

	// 高级搜索字段
	let advancedQuery = $state<SearchQuery>({
		title: '',
		author: '',
		publisher: '',
		series: '',
		rating_min: undefined,
		rating_max: undefined,
	});

	function handleBasicSearch() {
		if (keyword.trim()) {
			onSearch({ keyword: keyword.trim() });
		} else {
			onSearch({});
		}
	}

	function handleAdvancedSearch() {
		const query: SearchQuery = {};

		if (advancedQuery.title) query.title = advancedQuery.title;
		if (advancedQuery.author) query.author = advancedQuery.author;
		if (advancedQuery.publisher) query.publisher = advancedQuery.publisher;
		if (advancedQuery.series) query.series = advancedQuery.series;
		if (advancedQuery.rating_min !== undefined) query.rating_min = advancedQuery.rating_min;
		if (advancedQuery.rating_max !== undefined) query.rating_max = advancedQuery.rating_max;

		onSearch(query);
		showAdvanced = false;
	}

	function handleClearAdvanced() {
		advancedQuery = {
			title: '',
			author: '',
			publisher: '',
			series: '',
			rating_min: undefined,
			rating_max: undefined,
		};
		onSearch({});
	}
</script>

<div class="relative">
	<!-- 基础搜索栏 -->
	<div class="flex gap-2">
		<div class="relative flex-1">
			<input
				type="text"
				bind:value={keyword}
				placeholder="搜索书名、作者、出版社..."
				class="w-full rounded-lg border border-gray-300 px-4 py-2 pr-10 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				onkeydown={(e) => e.key === 'Enter' && handleBasicSearch()}
			/>
			<button
				class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
				onclick={handleBasicSearch}
			>
				🔍
			</button>
		</div>

		<button
			class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50"
			onclick={() => (showAdvanced = !showAdvanced)}
		>
			高级搜索
		</button>
	</div>

	<!-- 高级搜索面板 -->
	{#if showAdvanced}
		<div class="absolute left-0 right-0 top-full z-10 mt-2 rounded-lg border bg-white p-6 shadow-lg">
			<h3 class="mb-4 text-lg font-semibold">高级搜索</h3>

			<div class="grid grid-cols-2 gap-4">
				<!-- 标题 -->
				<div>
					<label for="title" class="mb-1 block text-sm font-medium text-gray-700">标题</label>
					<input
						id="title"
						type="text"
						bind:value={advancedQuery.title}
						placeholder="输入标题关键词"
						class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</div>

				<!-- 作者 -->
				<div>
					<label for="author" class="mb-1 block text-sm font-medium text-gray-700">作者</label>
					<input
						id="author"
						type="text"
						bind:value={advancedQuery.author}
						placeholder="输入作者名称"
						class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</div>

				<!-- 出版社 -->
				<div>
					<label for="publisher" class="mb-1 block text-sm font-medium text-gray-700">出版社</label>
					<input
						id="publisher"
						type="text"
						bind:value={advancedQuery.publisher}
						placeholder="输入出版社名称"
						class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</div>

				<!-- 系列 -->
				<div>
					<label for="series" class="mb-1 block text-sm font-medium text-gray-700">系列</label>
					<input
						id="series"
						type="text"
						bind:value={advancedQuery.series}
						placeholder="输入系列名称"
						class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					/>
				</div>

				<!-- 最低评分 -->
				<div>
					<label for="rating_min" class="mb-1 block text-sm font-medium text-gray-700">最低评分</label>
					<select
						id="rating_min"
						bind:value={advancedQuery.rating_min}
						class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					>
						<option value={undefined}>不限</option>
						<option value={1}>1 星</option>
						<option value={2}>2 星</option>
						<option value={3}>3 星</option>
						<option value={4}>4 星</option>
						<option value={5}>5 星</option>
					</select>
				</div>

				<!-- 最高评分 -->
				<div>
					<label for="rating_max" class="mb-1 block text-sm font-medium text-gray-700">最高评分</label>
					<select
						id="rating_max"
						bind:value={advancedQuery.rating_max}
						class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
					>
						<option value={undefined}>不限</option>
						<option value={1}>1 星</option>
						<option value={2}>2 星</option>
						<option value={3}>3 星</option>
						<option value={4}>4 星</option>
						<option value={5}>5 星</option>
					</select>
				</div>
			</div>

			<!-- 按钮 -->
			<div class="mt-6 flex justify-end gap-2">
				<button
					class="rounded-lg border border-gray-300 px-4 py-2 hover:bg-gray-50"
					onclick={handleClearAdvanced}
				>
					清除
				</button>
				<button
					class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
					onclick={handleAdvancedSearch}
				>
					搜索
				</button>
			</div>
		</div>
	{/if}
</div>
