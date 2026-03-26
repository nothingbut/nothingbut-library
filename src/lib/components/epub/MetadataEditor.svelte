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

	// Edited state
	let editedBook: EpubBook = $state(structuredClone(book));
	let editedAuthors: string[] = $state(authors.map((a) => a.name));
	let editedTags: string[] = $state(tags.map((t) => t.name));

	// UI state
	let saving: boolean = $state(false);
	let newAuthor: string = $state('');
	let newTag: string = $state('');
	let coverFile: string | null = $state(null);

	/**
	 * Get cover URL
	 */
	function getCoverUrl(): string {
		if (book.cover_path) {
			try {
				return convertFileSrc(book.cover_path);
			} catch (e) {
				return '';
			}
		}
		return '';
	}

	/**
	 * Get initial for placeholder
	 */
	function getInitial(): string {
		return book.title.charAt(0).toUpperCase();
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
			coverFile = selected as string;
		}
	}

	/**
	 * Add author
	 */
	function addAuthor(): void {
		const trimmed = newAuthor.trim();
		if (trimmed && !editedAuthors.includes(trimmed)) {
			editedAuthors = [...editedAuthors, trimmed];
			newAuthor = '';
		}
	}

	/**
	 * Remove author
	 */
	function removeAuthor(index: number): void {
		editedAuthors = editedAuthors.filter((_, i) => i !== index);
	}

	/**
	 * Add tag
	 */
	function addTag(): void {
		const trimmed = newTag.trim();
		if (trimmed && !editedTags.includes(trimmed)) {
			editedTags = [...editedTags, trimmed];
			newTag = '';
		}
	}

	/**
	 * Remove tag
	 */
	function removeTag(index: number): void {
		editedTags = editedTags.filter((_, i) => i !== index);
	}

	/**
	 * Set rating
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
	 * Handle save
	 */
	async function handleSave(): Promise<void> {
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
</script>

<div class="editor-container">
	<h3 class="editor-title">编辑书籍信息</h3>

	<!-- Cover upload -->
	<div class="form-group">
		<label class="form-label">封面</label>
		<div class="cover-upload">
			{#if getCoverUrl()}
				<img src={getCoverUrl()} alt="Current cover" class="cover-preview" />
			{:else}
				<div class="cover-placeholder">
					{getInitial()}
				</div>
			{/if}
			<div class="upload-actions">
				<button
					type="button"
					class="upload-btn"
					onclick={handleCoverUpload}
					disabled={saving}
				>
					选择新封面
				</button>
				{#if coverFile}
					<p class="upload-success">已选择新封面</p>
				{/if}
			</div>
		</div>
	</div>

	<!-- Title -->
	<div class="form-group">
		<label for="title" class="form-label">
			标题 <span class="required">*</span>
		</label>
		<input
			id="title"
			type="text"
			bind:value={editedBook.title}
			placeholder="输入书籍标题"
			class="form-input"
			disabled={saving}
		/>
	</div>

	<!-- Sort title -->
	<div class="form-group">
		<label for="sort_title" class="form-label">排序标题</label>
		<input
			id="sort_title"
			type="text"
			bind:value={editedBook.sort_title}
			placeholder="用于排序的标题"
			class="form-input"
			disabled={saving}
		/>
	</div>

	<!-- Authors -->
	<div class="form-group">
		<label class="form-label">作者</label>
		<div class="chips-list">
			{#each editedAuthors as author, index (index)}
				<div class="chip author-chip">
					<span class="chip-text">{author}</span>
					<button
						onclick={() => removeAuthor(index)}
						class="chip-remove"
						disabled={saving}
						title="删除"
					>
						✕
					</button>
				</div>
			{/each}
		</div>
		<div class="input-with-button">
			<input
				type="text"
				bind:value={newAuthor}
				placeholder="输入作者名称"
				class="form-input"
				disabled={saving}
				onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addAuthor())}
			/>
			<button onclick={addAuthor} class="add-btn" disabled={saving}>
				添加
			</button>
		</div>
	</div>

	<!-- Series -->
	<div class="form-row">
		<div class="form-group">
			<label for="series" class="form-label">系列</label>
			<input
				id="series"
				type="text"
				bind:value={editedBook.series}
				placeholder="系列名称"
				class="form-input"
				disabled={saving}
			/>
		</div>

		<div class="form-group">
			<label for="series_index" class="form-label">系列序号</label>
			<input
				id="series_index"
				type="number"
				bind:value={editedBook.series_index}
				placeholder="序号"
				class="form-input"
				disabled={saving}
			/>
		</div>
	</div>

	<!-- Publisher -->
	<div class="form-group">
		<label for="publisher" class="form-label">出版社</label>
		<input
			id="publisher"
			type="text"
			bind:value={editedBook.publisher}
			placeholder="出版社名称"
			class="form-input"
			disabled={saving}
		/>
	</div>

	<!-- Publication date -->
	<div class="form-group">
		<label for="pubdate" class="form-label">出版日期</label>
		<input
			id="pubdate"
			type="date"
			bind:value={editedBook.pubdate}
			class="form-input"
			disabled={saving}
		/>
	</div>

	<!-- ISBN -->
	<div class="form-group">
		<label for="isbn" class="form-label">ISBN</label>
		<input
			id="isbn"
			type="text"
			bind:value={editedBook.isbn}
			placeholder="ISBN 号"
			class="form-input"
			disabled={saving}
		/>
	</div>

	<!-- Language -->
	<div class="form-group">
		<label for="language" class="form-label">语言</label>
		<select
			id="language"
			bind:value={editedBook.language}
			class="form-select"
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

	<!-- Rating -->
	<div class="form-group">
		<label class="form-label">评分</label>
		<div class="rating-control">
			<div class="rating-stars">
				{#each [1, 2, 3, 4, 5] as star (star)}
					<button
						type="button"
						onclick={() => setRating(star)}
						class="star-btn"
						class:active={editedBook.rating !== null && editedBook.rating >= star}
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
				class="clear-rating-btn"
				disabled={saving}
			>
				清除
			</button>
			{#if editedBook.rating !== null}
				<span class="rating-value">{editedBook.rating.toFixed(1)}</span>
			{/if}
		</div>
	</div>

	<!-- Tags -->
	<div class="form-group">
		<label class="form-label">标签</label>
		<div class="chips-list">
			{#each editedTags as tag, index (index)}
				<div class="chip tag-chip">
					<span class="chip-text">{tag}</span>
					<button
						onclick={() => removeTag(index)}
						class="chip-remove"
						disabled={saving}
						title="删除"
					>
						✕
					</button>
				</div>
			{/each}
		</div>
		<div class="input-with-button">
			<input
				type="text"
				bind:value={newTag}
				placeholder="输入标签名称"
				class="form-input"
				disabled={saving}
				onkeydown={(e) => e.key === 'Enter' && (e.preventDefault(), addTag())}
			/>
			<button onclick={addTag} class="add-btn tag" disabled={saving}>
				添加
			</button>
		</div>
	</div>

	<!-- Description -->
	<div class="form-group">
		<label for="description" class="form-label">简介</label>
		<textarea
			id="description"
			bind:value={editedBook.description}
			placeholder="书籍简介"
			rows="6"
			class="form-textarea"
			disabled={saving}
		></textarea>
	</div>

	<!-- Actions -->
	<div class="editor-actions">
		<button
			onclick={handleSave}
			class="action-btn primary"
			disabled={saving}
		>
			{saving ? '保存中...' : '保存'}
		</button>
		<button
			onclick={onCancel}
			class="action-btn secondary"
			disabled={saving}
		>
			取消
		</button>
	</div>
</div>

<style>
	.editor-container {
		display: flex;
		flex-direction: column;
		gap: 20px;
		background-color: var(--color-bg-secondary);
		border-radius: 8px;
		padding: 20px;
	}

	.editor-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
	}

	/* Form groups */
	.form-group {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.form-row {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}

	.form-label {
		font-size: 13px;
		font-weight: 500;
		color: var(--color-text-secondary);
	}

	.required {
		color: #dc2626;
	}

	.form-input,
	.form-select,
	.form-textarea {
		padding: 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: 6px;
		font-size: 14px;
		color: var(--color-text-primary);
		background-color: var(--color-bg-primary);
		transition: all 0.2s ease;
	}

	.form-input:focus,
	.form-select:focus,
	.form-textarea:focus {
		outline: none;
		border-color: var(--color-primary);
		box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1);
	}

	.form-input::placeholder,
	.form-textarea::placeholder {
		color: var(--color-text-tertiary);
	}

	.form-input:disabled,
	.form-select:disabled,
	.form-textarea:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.form-textarea {
		resize: vertical;
		min-height: 100px;
		font-family: inherit;
		line-height: 1.5;
	}

	/* Cover upload */
	.cover-upload {
		display: flex;
		align-items: center;
		gap: 16px;
	}

	.cover-preview {
		width: 80px;
		height: 120px;
		object-fit: cover;
		border-radius: 4px;
	}

	.cover-placeholder {
		width: 80px;
		height: 120px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 32px;
		font-weight: 600;
		color: var(--color-text-tertiary);
		background: linear-gradient(135deg, var(--color-bg-secondary) 0%, var(--color-bg-hover) 100%);
		border-radius: 4px;
	}

	.upload-actions {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.upload-btn {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		color: white;
		background-color: #6b7280;
		border: none;
		cursor: pointer;
		transition: background-color 0.2s ease;
	}

	.upload-btn:hover:not(:disabled) {
		background-color: #4b5563;
	}

	.upload-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.upload-success {
		font-size: 12px;
		color: #16a34a;
		margin: 0;
	}

	/* Chips */
	.chips-list {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		min-height: 32px;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 12px;
		border-radius: 16px;
		font-size: 12px;
		font-weight: 500;
	}

	.author-chip {
		background-color: #dbeafe;
		color: #1e40af;
	}

	.tag-chip {
		background-color: #d1fae5;
		color: #065f46;
	}

	.chip-text {
		line-height: 1.4;
	}

	.chip-remove {
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		font-size: 14px;
		line-height: 1;
		transition: opacity 0.2s ease;
	}

	.author-chip .chip-remove {
		color: #1e40af;
	}

	.tag-chip .chip-remove {
		color: #065f46;
	}

	.chip-remove:hover:not(:disabled) {
		opacity: 0.7;
	}

	.chip-remove:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}

	/* Input with button */
	.input-with-button {
		display: flex;
		gap: 8px;
	}

	.input-with-button .form-input {
		flex: 1;
	}

	.add-btn {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 13px;
		font-weight: 500;
		color: white;
		background-color: #2563eb;
		border: none;
		cursor: pointer;
		transition: background-color 0.2s ease;
		white-space: nowrap;
	}

	.add-btn:hover:not(:disabled) {
		background-color: #1d4ed8;
	}

	.add-btn.tag {
		background-color: #16a34a;
	}

	.add-btn.tag:hover:not(:disabled) {
		background-color: #15803d;
	}

	.add-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	/* Rating */
	.rating-control {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.rating-stars {
		display: flex;
		gap: 4px;
	}

	.star-btn {
		font-size: 24px;
		color: var(--color-border);
		background: none;
		border: none;
		cursor: pointer;
		padding: 0;
		transition: color 0.2s ease;
		line-height: 1;
	}

	.star-btn:hover:not(:disabled) {
		color: #fbbf24;
	}

	.star-btn.active {
		color: #fbbf24;
	}

	.star-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.clear-rating-btn {
		padding: 4px 12px;
		border: 1px solid var(--color-border);
		border-radius: 4px;
		font-size: 12px;
		color: var(--color-text-secondary);
		background-color: var(--color-bg-primary);
		cursor: pointer;
		transition: all 0.2s ease;
	}

	.clear-rating-btn:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
	}

	.clear-rating-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.rating-value {
		font-size: 13px;
		color: var(--color-text-secondary);
	}

	/* Actions */
	.editor-actions {
		display: flex;
		gap: 12px;
		padding-top: 16px;
		border-top: 1px solid var(--color-border);
	}

	.action-btn {
		flex: 1;
		padding: 10px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s ease;
		border: 1px solid;
	}

	.action-btn.primary {
		background-color: var(--color-primary);
		color: white;
		border-color: var(--color-primary);
	}

	.action-btn.primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.action-btn.secondary {
		background-color: var(--color-bg-primary);
		color: var(--color-text-primary);
		border-color: var(--color-border);
	}

	.action-btn.secondary:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}
</style>
