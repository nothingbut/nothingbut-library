<script lang="ts">
	import { open } from '@tauri-apps/plugin-dialog';
	import { EpubService } from '$lib/services/epub';
	import { currentWorkspace } from '$lib/stores/workspace';
	import type { ImportResult } from '$lib/types/epub';

	interface Props {
		isOpen?: boolean;
		onClose: () => void;
		onSuccess: () => void;
	}

	let { isOpen = $bindable(false), onClose, onSuccess }: Props = $props();

	// State
	let importing = $state(false);
	let error = $state<string | null>(null);
	let selectedFiles = $state<string[]>([]);
	let importResults = $state<ImportResult[]>([]);
	let showResults = $state(false);

	/**
	 * Select EPUB files
	 */
	async function selectFiles() {
		try {
			error = null;
			const selected = await open({
				multiple: true,
				filters: [
					{
						name: 'EPUB',
						extensions: ['epub'],
					},
				],
			});

			if (selected) {
				selectedFiles = Array.isArray(selected) ? selected : [selected];
			}
		} catch (e) {
			error = e instanceof Error ? e.message : '选择文件失败';
			console.error('File selection error:', e);
		}
	}

	/**
	 * Import selected files
	 */
	async function importFiles() {
		const workspace = $currentWorkspace;
		if (!workspace) {
			error = '请先选择工作空间';
			return;
		}

		if (selectedFiles.length === 0) {
			error = '请先选择 EPUB 文件';
			return;
		}

		try {
			importing = true;
			error = null;

			if (selectedFiles.length === 1) {
				// Single file import
				const bookId = await EpubService.importEpub(workspace.path, selectedFiles[0]);
				importResults = [
					{
						type: 'success',
						book_id: bookId,
						file_path: selectedFiles[0],
					},
				];
			} else {
				// Batch import
				importResults = await EpubService.batchImportEpub(workspace.path, selectedFiles);
			}

			showResults = true;
		} catch (e) {
			error = e instanceof Error ? e.message : '导入失败';
			console.error('Import error:', e);
		} finally {
			importing = false;
		}
	}

	/**
	 * Close and reset dialog
	 */
	function handleClose() {
		selectedFiles = [];
		importResults = [];
		showResults = false;
		error = null;
		importing = false;
		onClose();
	}

	/**
	 * Finish import and notify success
	 */
	function handleFinish() {
		handleClose();
		onSuccess();
	}

	/**
	 * Get filename from path
	 */
	function getFileName(path: string): string {
		return path.split('/').pop() || path;
	}
</script>

{#if isOpen}
	<div class="dialog-overlay" onclick={handleClose} role="presentation">
		<div class="dialog-content" onclick={(e) => e.stopPropagation()} role="dialog" aria-modal="true">
			{#if !showResults}
				<!-- Import Form -->
				<div class="dialog-header">
					<h2 class="dialog-title">导入 EPUB 书籍</h2>
					<button onclick={handleClose} class="close-btn" title="关闭" type="button">
						✕
					</button>
				</div>

				{#if error}
					<div class="error-message">
						{error}
					</div>
				{/if}

				<div class="dialog-body">
					<div class="form-group">
						<label class="form-label">选择 EPUB 文件</label>
						<button onclick={selectFiles} class="file-select-btn" disabled={importing} type="button">
							{#if selectedFiles.length === 0}
								<div class="file-select-empty">
									<div class="file-icon">📚</div>
									<div class="file-hint">点击选择 EPUB 文件</div>
									<div class="file-subhint">支持单个或批量导入</div>
								</div>
							{:else}
								<div class="file-select-list">
									<div class="file-count">
										已选择 {selectedFiles.length} 个文件
									</div>
									<div class="file-items">
										{#each selectedFiles as file}
											<div class="file-item">
												📖 {getFileName(file)}
											</div>
										{/each}
									</div>
								</div>
							{/if}
						</button>
					</div>

					{#if !$currentWorkspace}
						<div class="warning-message">
							⚠️ 未选择工作空间，请先在设置中选择工作空间
						</div>
					{/if}
				</div>

				<div class="dialog-footer">
					<button onclick={handleClose} class="dialog-btn secondary" disabled={importing} type="button">
						取消
					</button>
					<button
						onclick={importFiles}
						class="dialog-btn primary"
						disabled={importing || selectedFiles.length === 0 || !$currentWorkspace}
						type="button"
					>
						{importing ? '导入中...' : '开始导入'}
					</button>
				</div>
			{:else}
				<!-- Import Results -->
				<div class="dialog-header">
					<h2 class="dialog-title">导入结果</h2>
					<button onclick={handleClose} class="close-btn" title="关闭" type="button">
						✕
					</button>
				</div>

				<div class="results-body">
					{#each importResults as result}
						{#if result.type === 'success'}
							<div class="result-item success">
								<span class="result-icon">✅</span>
								<div class="result-info">
									<div class="result-title">
										{getFileName(result.file_path || '')}
									</div>
									<div class="result-detail">
										导入成功 (ID: {result.book_id})
									</div>
								</div>
							</div>
						{:else}
							<div class="result-item error">
								<span class="result-icon">❌</span>
								<div class="result-info">
									<div class="result-title">
										{getFileName(result.file_path || '')}
									</div>
									<div class="result-detail">
										{result.error || '导入失败'}
									</div>
								</div>
							</div>
						{/if}
					{/each}
				</div>

				<div class="dialog-footer">
					<button onclick={handleFinish} class="dialog-btn primary" type="button">
						完成
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	.dialog-overlay {
		position: fixed;
		inset: 0;
		z-index: 50;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: rgba(0, 0, 0, 0.5);
	}

	.dialog-content {
		width: 100%;
		max-width: 600px;
		background-color: var(--color-bg-primary);
		border-radius: 8px;
		padding: 24px;
		box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
	}

	.dialog-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 16px;
	}

	.dialog-title {
		font-size: 20px;
		font-weight: 600;
		color: var(--color-text-primary);
		margin: 0;
	}

	.close-btn {
		font-size: 20px;
		color: var(--color-text-secondary);
		background: none;
		border: none;
		cursor: pointer;
		padding: 4px;
		transition: color 0.2s;
	}

	.close-btn:hover {
		color: var(--color-text-primary);
	}

	.error-message {
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: 6px;
		padding: 12px;
		margin-bottom: 16px;
		color: #dc2626;
		font-size: 14px;
	}

	.warning-message {
		background-color: #fffbeb;
		border: 1px solid #fde68a;
		border-radius: 6px;
		padding: 12px;
		color: #d97706;
		font-size: 14px;
	}

	.dialog-body {
		margin-bottom: 24px;
	}

	.form-group {
		margin-bottom: 16px;
	}

	.form-label {
		display: block;
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text-secondary);
		margin-bottom: 8px;
	}

	.file-select-btn {
		width: 100%;
		border: 2px dashed var(--color-border);
		border-radius: 8px;
		padding: 32px;
		background-color: var(--color-bg-secondary);
		cursor: pointer;
		transition: all 0.2s;
	}

	.file-select-btn:hover:not(:disabled) {
		border-color: var(--color-primary);
		background-color: var(--color-bg-hover);
	}

	.file-select-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.file-select-empty {
		text-align: center;
		color: var(--color-text-secondary);
	}

	.file-icon {
		font-size: 48px;
		margin-bottom: 12px;
	}

	.file-hint {
		font-size: 14px;
		margin-bottom: 4px;
	}

	.file-subhint {
		font-size: 12px;
		color: var(--color-text-tertiary);
	}

	.file-select-list {
		text-align: left;
	}

	.file-count {
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text-primary);
		margin-bottom: 12px;
	}

	.file-items {
		max-height: 160px;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.file-item {
		font-size: 13px;
		color: var(--color-text-secondary);
	}

	.dialog-footer {
		display: flex;
		gap: 12px;
		justify-content: flex-end;
	}

	.dialog-btn {
		padding: 8px 16px;
		border-radius: 6px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.2s;
	}

	.dialog-btn.secondary {
		color: var(--color-text-primary);
		background-color: var(--color-bg-secondary);
		border: 1px solid var(--color-border);
	}

	.dialog-btn.secondary:hover:not(:disabled) {
		background-color: var(--color-bg-hover);
	}

	.dialog-btn.primary {
		color: white;
		background-color: var(--color-primary);
		border: 1px solid var(--color-primary);
	}

	.dialog-btn.primary:hover:not(:disabled) {
		opacity: 0.9;
	}

	.dialog-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.results-body {
		max-height: 400px;
		overflow-y: auto;
		margin-bottom: 24px;
		display: flex;
		flex-direction: column;
		gap: 8px;
	}

	.result-item {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px;
		border-radius: 6px;
		border: 1px solid;
	}

	.result-item.success {
		background-color: #f0fdf4;
		border-color: #86efac;
	}

	.result-item.error {
		background-color: #fef2f2;
		border-color: #fecaca;
	}

	.result-icon {
		font-size: 24px;
		flex-shrink: 0;
	}

	.result-info {
		flex: 1;
	}

	.result-title {
		font-size: 14px;
		font-weight: 500;
		margin-bottom: 4px;
	}

	.result-item.success .result-title {
		color: #166534;
	}

	.result-item.error .result-title {
		color: #991b1b;
	}

	.result-detail {
		font-size: 12px;
	}

	.result-item.success .result-detail {
		color: #15803d;
	}

	.result-item.error .result-detail {
		color: #dc2626;
	}
</style>
