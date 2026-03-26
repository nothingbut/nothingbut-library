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
	<div
		class="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50"
		onclick={handleClose}
		role="presentation"
	>
		<div
			class="w-full max-w-2xl rounded-lg bg-white p-6 shadow-xl"
			onclick={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
		>
			{#if !showResults}
				<!-- Import Form -->
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-xl font-bold text-gray-900">导入 EPUB 书籍</h2>
					<button
						onclick={handleClose}
						class="text-gray-400 hover:text-gray-600"
						title="关闭"
						type="button"
					>
						✕
					</button>
				</div>

				{#if error}
					<div class="mb-4 rounded-lg bg-red-50 p-4 text-sm text-red-600">
						{error}
					</div>
				{/if}

				<div class="mb-6 space-y-4">
					<div>
						<label class="mb-2 block text-sm font-medium text-gray-700">
							选择 EPUB 文件
						</label>
						<button
							onclick={selectFiles}
							class="w-full rounded-lg border-2 border-dashed border-gray-300 p-6 text-center hover:border-blue-500 hover:bg-blue-50"
							disabled={importing}
							type="button"
						>
							{#if selectedFiles.length === 0}
								<div class="text-gray-600">
									<div class="text-4xl mb-2">📚</div>
									<div>点击选择 EPUB 文件</div>
									<div class="text-xs text-gray-400 mt-1">支持单个或批量导入</div>
								</div>
							{:else}
								<div class="text-left">
									<div class="text-sm font-medium text-gray-900 mb-2">
										已选择 {selectedFiles.length} 个文件
									</div>
									<div class="max-h-40 overflow-y-auto space-y-1">
										{#each selectedFiles as file}
											<div class="text-sm text-gray-600">
												📖 {getFileName(file)}
											</div>
										{/each}
									</div>
								</div>
							{/if}
						</button>
					</div>

					{#if !$currentWorkspace}
						<div class="rounded-lg bg-yellow-50 p-4 text-sm text-yellow-700">
							⚠️ 未选择工作空间，请先在设置中选择工作空间
						</div>
					{/if}
				</div>

				<div class="flex gap-3 justify-end">
					<button
						onclick={handleClose}
						class="rounded-lg border border-gray-300 px-4 py-2 text-gray-700 hover:bg-gray-50"
						disabled={importing}
						type="button"
					>
						取消
					</button>
					<button
						onclick={importFiles}
						class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700 disabled:bg-gray-400"
						disabled={importing || selectedFiles.length === 0 || !$currentWorkspace}
						type="button"
					>
						{importing ? '导入中...' : '开始导入'}
					</button>
				</div>
			{:else}
				<!-- Import Results -->
				<div class="mb-4 flex items-center justify-between">
					<h2 class="text-xl font-bold text-gray-900">导入结果</h2>
					<button
						onclick={handleClose}
						class="text-gray-400 hover:text-gray-600"
						title="关闭"
						type="button"
					>
						✕
					</button>
				</div>

				<div class="mb-6 max-h-96 overflow-y-auto space-y-2">
					{#each importResults as result}
						{#if result.type === 'success'}
							<div class="flex items-center gap-3 rounded-lg bg-green-50 p-3">
								<span class="text-2xl">✅</span>
								<div class="flex-1">
									<div class="text-sm font-medium text-green-900">
										{getFileName(result.file_path || '')}
									</div>
									<div class="text-xs text-green-700">
										导入成功 (ID: {result.book_id})
									</div>
								</div>
							</div>
						{:else}
							<div class="flex items-center gap-3 rounded-lg bg-red-50 p-3">
								<span class="text-2xl">❌</span>
								<div class="flex-1">
									<div class="text-sm font-medium text-red-900">
										{getFileName(result.file_path || '')}
									</div>
									<div class="text-xs text-red-700">
										{result.error || '导入失败'}
									</div>
								</div>
							</div>
						{/if}
					{/each}
				</div>

				<div class="flex justify-end">
					<button
						onclick={handleFinish}
						class="rounded-lg bg-blue-600 px-4 py-2 text-white hover:bg-blue-700"
						type="button"
					>
						完成
					</button>
				</div>
			{/if}
		</div>
	</div>
{/if}
