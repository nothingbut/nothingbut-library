<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import type { EpubBook, Author, Tag } from '$lib/types/epub';
	import { currentWorkspace } from '$lib/stores/workspace';
	import { EpubService } from '$lib/services/epub';

	interface Props {
		book: EpubBook;
		authors: Author[];
		tags: Tag[];
		onSave: (updatedData: any) => Promise<void>;
		onCancel: () => void;
	}

	let { book, authors, tags, onSave, onCancel }: Props = $props();

	// Edited state - initialized from props
	let editedBook: EpubBook = $state(structuredClone(book));
	let editedAuthors: string[] = $state(authors.map((a) => a.name));
	let editedTags: string[] = $state(tags.map((t) => t.name));

	// UI state
	let saving: boolean = $state(false);
	let newAuthor: string = $state('');
	let newTag: string = $state('');
	let coverFile: string | null = $state(null);

	/**
	 * Get the cover image URL with error handling
	 */
	function getCoverUrl(): string {
		if (book.cover_path) {
			try {
				return convertFileSrc(book.cover_path);
			} catch (e) {
				console.warn('Failed to convert cover path:', e);
				return '/placeholder-cover.svg';
			}
		}
		return '/placeholder-cover.svg';
	}

	/**
	 * Handle cover file selection
	 */
	async function handleCoverUpload(): Promise<void> {
		const selected = await open({
			multiple: false,
			filters: [
				{
					name: 'Images',
					extensions: ['jpg', 'jpeg', 'png', 'webp']
				}
			]
		});

		if (selected) {
			coverFile = selected;
		}
	}

	/**
	 * Add an author if it's not a duplicate
	 */
	function addAuthor(): void {
		const trimmed = newAuthor.trim();
		if (trimmed && !editedAuthors.includes(trimmed)) {
			editedAuthors = [...editedAuthors, trimmed];
			newAuthor = '';
		}
	}

	/**
	 * Remove author by index
	 */
	function removeAuthor(index: number): void {
		editedAuthors = editedAuthors.filter((_, i) => i !== index);
	}

	/**
	 * Add a tag if it's not a duplicate
	 */
	function addTag(): void {
		const trimmed = newTag.trim();
		if (trimmed && !editedTags.includes(trimmed)) {
			editedTags = [...editedTags, trimmed];
			newTag = '';
		}
	}

	/**
	 * Remove tag by index
	 */
	function removeTag(index: number): void {
		editedTags = editedTags.filter((_, i) => i !== index);
	}

	/**
	 * Set rating (1-5 stars)
	 */
	function setRating(rating: number): void {
		editedBook.rating = rating;
	}

	/**
	 * Clear rating
	 */
	function clearRating(): void {
		editedBook.rating = null;
	}

	/**
	 * Handle save action
	 */
	async function handleSave(): Promise<void> {
		// Validate title is not empty
		if (!editedBook.title.trim()) {
			alert('标题不能为空');
			return;
		}

		saving = true;
		try {
			// Upload cover first if selected
			if (coverFile) {
				const workspace = $currentWorkspace;
				if (!workspace) {
					throw new Error('未选择工作空间');
				}
				await EpubService.updateCover(workspace.path, book.id, coverFile);
			}

			// Save metadata
			await onSave({
				book: editedBook,
				authors: editedAuthors,
				tags: editedTags
			});
		} catch (err) {
			const message = err instanceof Error ? err.message : '保存失败';
			alert(`保存失败: ${message}`);
			console.error('Failed to save metadata:', err);
		} finally {
			saving = false;
		}
	}

	/**
	 * Handle Enter key in author/tag inputs
	 */
	function handleAuthorKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter') {
			e.preventDefault();
			addAuthor();
		}
	}

	function handleTagKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter') {
			e.preventDefault();
			addTag();
		}
	}
</script>

<div class="space-y-4 rounded-lg bg-gray-50 p-4">
	<h3 class="font-semibold text-gray-900">编辑书籍信息</h3>

	<!-- Cover upload -->
	<div>
		<label class="mb-1 block text-sm font-medium text-gray-700">封面</label>
		<div class="flex items-center gap-4">
			<img
				src={getCoverUrl()}
				alt="Current cover"
				class="h-32 w-24 rounded object-cover"
			/>
			<div class="flex-1">
				<button
					type="button"
					class="rounded bg-gray-600 px-4 py-2 text-white hover:bg-gray-700 disabled:bg-gray-400"
					onclick={handleCoverUpload}
					disabled={saving}
				>
					选择新封面
				</button>
				{#if coverFile}
					<p class="mt-2 text-sm text-green-600">
						已选择新封面
					</p>
				{/if}
			</div>
		</div>
	</div>

	<!-- Title (required) -->
	<div>
		<label for="title" class="mb-1 block text-sm font-medium text-gray-700">
			标题 <span class="text-red-600">*</span>
		</label>
		<input
			id="title"
			type="text"
			bind:value={editedBook.title}
			placeholder="输入书籍标题"
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		/>
	</div>

	<!-- Sort title -->
	<div>
		<label for="sort_title" class="mb-1 block text-sm font-medium text-gray-700">排序标题</label>
		<input
			id="sort_title"
			type="text"
			bind:value={editedBook.sort_title}
			placeholder="用于排序的标题"
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		/>
	</div>

	<!-- Authors -->
	<div>
		<label for="authors-input" class="mb-2 block text-sm font-medium text-gray-700">作者</label>
		<div class="mb-2 flex flex-wrap gap-2">
			{#each editedAuthors as author, index (index)}
				<div class="flex items-center gap-1 rounded-full bg-blue-100 px-3 py-1">
					<span class="text-sm font-medium text-blue-800">{author}</span>
					<button
						onclick={() => removeAuthor(index)}
						class="ml-1 text-blue-600 hover:text-blue-800"
						disabled={saving}
						title="删除"
					>
						✕
					</button>
				</div>
			{/each}
		</div>
		<div class="flex gap-2">
			<input
				id="authors-input"
				type="text"
				bind:value={newAuthor}
				placeholder="输入作者名称"
				class="flex-1 rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				disabled={saving}
				onkeydown={handleAuthorKeydown}
			/>
			<button
				onclick={addAuthor}
				class="rounded bg-blue-600 px-3 py-2 text-sm font-medium text-white hover:bg-blue-700 disabled:bg-gray-400"
				disabled={saving}
			>
				添加
			</button>
		</div>
	</div>

	<!-- Series and series index (2-column grid) -->
	<div class="grid grid-cols-2 gap-4">
		<div>
			<label for="series" class="mb-1 block text-sm font-medium text-gray-700">系列</label>
			<input
				id="series"
				type="text"
				bind:value={editedBook.series}
				placeholder="系列名称"
				class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				disabled={saving}
			/>
		</div>

		<div>
			<label for="series_index" class="mb-1 block text-sm font-medium text-gray-700">系列序号</label>
			<input
				id="series_index"
				type="number"
				bind:value={editedBook.series_index}
				placeholder="序号"
				class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				disabled={saving}
			/>
		</div>
	</div>

	<!-- Publisher -->
	<div>
		<label for="publisher" class="mb-1 block text-sm font-medium text-gray-700">出版社</label>
		<input
			id="publisher"
			type="text"
			bind:value={editedBook.publisher}
			placeholder="出版社名称"
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		/>
	</div>

	<!-- Publication date -->
	<div>
		<label for="pubdate" class="mb-1 block text-sm font-medium text-gray-700">出版日期</label>
		<input
			id="pubdate"
			type="date"
			bind:value={editedBook.pubdate}
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		/>
	</div>

	<!-- ISBN -->
	<div>
		<label for="isbn" class="mb-1 block text-sm font-medium text-gray-700">ISBN</label>
		<input
			id="isbn"
			type="text"
			bind:value={editedBook.isbn}
			placeholder="ISBN 号"
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		/>
	</div>

	<!-- Language -->
	<div>
		<label for="language" class="mb-1 block text-sm font-medium text-gray-700">语言</label>
		<select
			id="language"
			bind:value={editedBook.language}
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		>
			<option value={null}>未设置</option>
			<option value="zh">中文</option>
			<option value="en">英文</option>
			<option value="ja">日文</option>
			<option value="fr">法文</option>
			<option value="de">德文</option>
			<option value="es">西班牙文</option>
		</select>
	</div>

	<!-- Rating (star buttons) -->
	<fieldset>
		<legend class="mb-2 block text-sm font-medium text-gray-700">评分</legend>
		<div class="flex items-center gap-2">
			<div class="flex gap-1">
				{#each [1, 2, 3, 4, 5] as star (star)}
					<button
						type="button"
						onclick={() => setRating(star)}
						class="text-2xl transition-colors {editedBook.rating !== null && editedBook.rating >= star
							? 'text-yellow-500'
							: 'text-gray-300 hover:text-yellow-400'}"
						disabled={saving}
						title={`${star} 星`}
					>
						★
					</button>
				{/each}
			</div>
			<button
				type="button"
				onclick={clearRating}
				class="ml-2 rounded border border-gray-300 px-2 py-1 text-sm text-gray-600 hover:bg-gray-100 disabled:bg-gray-100"
				disabled={saving}
			>
				清除
			</button>
			{#if editedBook.rating !== null}
				<span class="text-sm text-gray-600">{editedBook.rating.toFixed(1)}</span>
			{/if}
		</div>
	</fieldset>

	<!-- Tags -->
	<div>
		<label for="tags-input" class="mb-2 block text-sm font-medium text-gray-700">标签</label>
		<div class="mb-2 flex flex-wrap gap-2">
			{#each editedTags as tag, index (index)}
				<div class="flex items-center gap-1 rounded-full bg-green-100 px-3 py-1">
					<span class="text-sm font-medium text-green-800">{tag}</span>
					<button
						onclick={() => removeTag(index)}
						class="ml-1 text-green-600 hover:text-green-800"
						disabled={saving}
						title="删除"
					>
						✕
					</button>
				</div>
			{/each}
		</div>
		<div class="flex gap-2">
			<input
				id="tags-input"
				type="text"
				bind:value={newTag}
				placeholder="输入标签名称"
				class="flex-1 rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
				disabled={saving}
				onkeydown={handleTagKeydown}
			/>
			<button
				onclick={addTag}
				class="rounded bg-green-600 px-3 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:bg-gray-400"
				disabled={saving}
			>
				添加
			</button>
		</div>
	</div>

	<!-- Description -->
	<div>
		<label for="description" class="mb-1 block text-sm font-medium text-gray-700">简介</label>
		<textarea
			id="description"
			bind:value={editedBook.description}
			placeholder="书籍简介"
			rows="6"
			class="w-full rounded border border-gray-300 px-3 py-2 focus:border-blue-500 focus:outline-none focus:ring-1 focus:ring-blue-500"
			disabled={saving}
		></textarea>
	</div>

	<!-- Save and cancel buttons -->
	<div class="flex gap-2 border-t border-gray-200 pt-4">
		<button
			onclick={handleSave}
			class="flex-1 rounded-lg bg-blue-600 px-4 py-2 font-medium text-white hover:bg-blue-700 disabled:bg-gray-400"
			disabled={saving}
		>
			{saving ? '保存中...' : '保存'}
		</button>
		<button
			onclick={onCancel}
			class="flex-1 rounded-lg border border-gray-300 px-4 py-2 font-medium text-gray-900 hover:bg-gray-50 disabled:bg-gray-50"
			disabled={saving}
		>
			取消
		</button>
	</div>
</div>
