<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { previewImport, importNovel, createCategory } from '$lib/services/api';
  import type { ImportPreview } from '$lib/services/api';

  // Props
  interface Props {
    isOpen?: boolean;
    onClose?: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = $bindable(false), onClose, onSuccess }: Props = $props();

  // State
  let step = $state<'select' | 'preview' | 'importing' | 'success' | 'error'>('select');
  let selectedFile = $state<string | null>(null);
  let preview = $state<ImportPreview | null>(null);
  let error = $state<string | null>(null);
  let importing = $state(false);

  // Form data
  let title = $state('');
  let author = $state('');
  let description = $state('');
  let category = $state('');

  // Workspace path (hardcoded for now)
  const workspacePath = '/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library';

  // Select file
  async function selectFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Text',
          extensions: ['txt']
        }]
      });

      if (selected && typeof selected === 'string') {
        selectedFile = selected;
        // Extract filename as default title
        const filename = selected.split('/').pop() || '';
        title = filename.replace('.txt', '');
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to select file';
      console.error('File selection error:', e);
    }
  }

  // Preview import
  async function handlePreview() {
    if (!selectedFile || !title) {
      error = 'Please select a file and enter a title';
      return;
    }

    try {
      importing = true;
      error = null;

      preview = await previewImport(
        selectedFile,
        title,
        author || '未知作者',
        category || '未分类'
      );

      step = 'preview';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to preview import';
      console.error('Preview error:', e);
    } finally {
      importing = false;
    }
  }

  // Import novel
  async function handleImport() {
    if (!selectedFile || !title || !preview) {
      error = 'Invalid import data';
      return;
    }

    try {
      importing = true;
      step = 'importing';
      error = null;

      // Create category if needed
      let categoryId: number | undefined;
      if (category) {
        try {
          categoryId = await createCategory(category, undefined, 0);
        } catch (e) {
          console.warn('Failed to create category, continuing without it:', e);
        }
      }

      // Import the novel
      await importNovel(
        workspacePath,
        selectedFile,
        title,
        author || undefined,
        description || undefined,
        categoryId
      );

      step = 'success';

      // Close dialog after 1.5 seconds
      setTimeout(() => {
        handleClose();
        onSuccess?.();
      }, 1500);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to import novel';
      step = 'error';
      console.error('Import error:', e);
    } finally {
      importing = false;
    }
  }

  // Go back to file selection
  function handleBack() {
    step = 'select';
    preview = null;
    error = null;
  }

  // Close dialog
  function handleClose() {
    isOpen = false;
    step = 'select';
    selectedFile = null;
    preview = null;
    error = null;
    title = '';
    author = '';
    description = '';
    category = '';
    onClose?.();
  }

  // Format file size
  function formatFileSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  // Format number with commas
  function formatNumber(num: number): string {
    return num.toLocaleString('zh-CN');
  }
</script>

{#if isOpen}
  <div class="dialog-overlay" onclick={handleClose}>
    <div class="dialog-content" onclick={(e) => e.stopPropagation()}>
      <!-- Dialog Header -->
      <div class="dialog-header">
        <h2 class="dialog-title">
          {#if step === 'select'}
            导入小说
          {:else if step === 'preview'}
            预览导入
          {:else if step === 'importing'}
            导入中...
          {:else if step === 'success'}
            导入成功
          {:else}
            导入失败
          {/if}
        </h2>
        <button class="close-btn" onclick={handleClose}>×</button>
      </div>

      <!-- Dialog Body -->
      <div class="dialog-body">
        {#if step === 'select'}
          <!-- Step 1: File Selection -->
          <div class="form-section">
            <div class="form-group">
              <label class="form-label">选择文件</label>
              <div class="file-input-group">
                <input
                  type="text"
                  class="form-input"
                  readonly
                  value={selectedFile || ''}
                  placeholder="请选择 .txt 文件"
                />
                <button class="btn btn-secondary" onclick={selectFile}>
                  浏览...
                </button>
              </div>
            </div>

            <div class="form-group">
              <label class="form-label">书名 *</label>
              <input
                type="text"
                class="form-input"
                bind:value={title}
                placeholder="请输入书名"
              />
            </div>

            <div class="form-group">
              <label class="form-label">作者</label>
              <input
                type="text"
                class="form-input"
                bind:value={author}
                placeholder="作者姓名（可选）"
              />
            </div>

            <div class="form-group">
              <label class="form-label">分类</label>
              <input
                type="text"
                class="form-input"
                bind:value={category}
                placeholder="例如：科幻、历史（可选）"
              />
            </div>

            <div class="form-group">
              <label class="form-label">简介</label>
              <textarea
                class="form-textarea"
                bind:value={description}
                placeholder="书籍简介（可选）"
                rows="3"
              ></textarea>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}
          </div>
        {:else if step === 'preview' && preview}
          <!-- Step 2: Preview -->
          <div class="preview-section">
            <div class="preview-info">
              <div class="preview-row">
                <span class="preview-label">书名：</span>
                <span class="preview-value">{preview.title}</span>
              </div>
              <div class="preview-row">
                <span class="preview-label">作者：</span>
                <span class="preview-value">{preview.author}</span>
              </div>
              <div class="preview-row">
                <span class="preview-label">分类：</span>
                <span class="preview-value">{preview.category}</span>
              </div>
              <div class="preview-row">
                <span class="preview-label">总章节：</span>
                <span class="preview-value">{formatNumber(preview.total_chapters)} 章</span>
              </div>
              <div class="preview-row">
                <span class="preview-label">总字数：</span>
                <span class="preview-value">{formatNumber(preview.total_words)} 字</span>
              </div>
            </div>

            <div class="preview-chapters">
              <h4 class="preview-chapters-title">章节预览（前 {preview.chapters.length} 章）</h4>
              <div class="chapter-preview-list">
                {#each preview.chapters as chapter}
                  <div class="chapter-preview-item">
                    <span class="chapter-number">{chapter.chapter_number}.</span>
                    <span class="chapter-title">{chapter.title}</span>
                    <span class="chapter-words">{formatNumber(chapter.word_count)} 字</span>
                  </div>
                {/each}
              </div>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}
          </div>
        {:else if step === 'importing'}
          <!-- Step 3: Importing -->
          <div class="status-section">
            <div class="spinner"></div>
            <p class="status-text">正在导入小说，请稍候...</p>
            <p class="status-subtext">这可能需要几秒钟到几分钟，取决于文件大小。</p>
          </div>
        {:else if step === 'success'}
          <!-- Step 4: Success -->
          <div class="status-section">
            <div class="success-icon">✓</div>
            <p class="status-text">导入成功！</p>
            <p class="status-subtext">小说已成功导入到您的资料库中。</p>
          </div>
        {:else if step === 'error'}
          <!-- Step 5: Error -->
          <div class="status-section">
            <div class="error-icon">✕</div>
            <p class="status-text">导入失败</p>
            {#if error}
              <p class="status-subtext error">{error}</p>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Dialog Footer -->
      <div class="dialog-footer">
        {#if step === 'select'}
          <button class="btn btn-secondary" onclick={handleClose}>取消</button>
          <button
            class="btn btn-primary"
            onclick={handlePreview}
            disabled={!selectedFile || !title || importing}
          >
            {importing ? '处理中...' : '下一步'}
          </button>
        {:else if step === 'preview'}
          <button class="btn btn-secondary" onclick={handleBack}>上一步</button>
          <button
            class="btn btn-primary"
            onclick={handleImport}
            disabled={importing}
          >
            确认导入
          </button>
        {:else if step === 'error'}
          <button class="btn btn-secondary" onclick={handleClose}>关闭</button>
          <button class="btn btn-primary" onclick={handleBack}>重试</button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .dialog-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    padding: 24px;
  }

  .dialog-content {
    background-color: var(--color-bg-primary);
    border-radius: 12px;
    width: 100%;
    max-width: 600px;
    max-height: 90vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  }

  .dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--color-border);
  }

  .dialog-title {
    font-size: 20px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0;
  }

  .close-btn {
    width: 32px;
    height: 32px;
    border-radius: 6px;
    font-size: 24px;
    line-height: 1;
    color: var(--color-text-secondary);
    background-color: transparent;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .close-btn:hover {
    background-color: var(--color-bg-hover);
    color: var(--color-text-primary);
  }

  .dialog-body {
    flex: 1;
    padding: 24px;
    overflow-y: auto;
  }

  .dialog-footer {
    display: flex;
    gap: 12px;
    justify-content: flex-end;
    padding: 16px 24px;
    border-top: 1px solid var(--color-border);
  }

  /* Form Styles */
  .form-section {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .form-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .form-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .form-input,
  .form-textarea {
    width: 100%;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 14px;
    color: var(--color-text-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
    transition: all 0.2s ease;
  }

  .form-input:focus,
  .form-textarea:focus {
    outline: none;
    border-color: var(--color-primary);
  }

  .form-input:read-only {
    cursor: default;
  }

  .form-textarea {
    resize: vertical;
    font-family: inherit;
  }

  .file-input-group {
    display: flex;
    gap: 8px;
  }

  .file-input-group .form-input {
    flex: 1;
  }

  /* Preview Styles */
  .preview-section {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .preview-info {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
    border-radius: 8px;
    background-color: var(--color-bg-secondary);
  }

  .preview-row {
    display: flex;
    gap: 12px;
    font-size: 14px;
  }

  .preview-label {
    font-weight: 500;
    color: var(--color-text-secondary);
    min-width: 70px;
  }

  .preview-value {
    color: var(--color-text-primary);
  }

  .preview-chapters-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 12px 0;
  }

  .chapter-preview-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .chapter-preview-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 12px;
    border-radius: 6px;
    background-color: var(--color-bg-secondary);
    font-size: 14px;
  }

  .chapter-number {
    font-weight: 600;
    color: var(--color-text-secondary);
    min-width: 30px;
  }

  .chapter-title {
    flex: 1;
    color: var(--color-text-primary);
  }

  .chapter-words {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  /* Status Styles */
  .status-section {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 48px 24px;
    text-align: center;
  }

  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid var(--color-bg-secondary);
    border-top-color: var(--color-primary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    margin-bottom: 24px;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .success-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background-color: #4caf50;
    color: white;
    font-size: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 24px;
  }

  .error-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background-color: #f44336;
    color: white;
    font-size: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 24px;
  }

  .status-text {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0 0 8px 0;
  }

  .status-subtext {
    font-size: 14px;
    color: var(--color-text-secondary);
    margin: 0;
  }

  .status-subtext.error {
    color: #f44336;
  }

  /* Button Styles */
  .btn {
    padding: 10px 20px;
    border-radius: 6px;
    font-size: 14px;
    font-weight: 500;
    border: none;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .btn-primary {
    color: white;
    background-color: var(--color-primary);
  }

  .btn-primary:hover:not(:disabled) {
    background-color: var(--color-primary-dark);
  }

  .btn-secondary {
    color: var(--color-text-primary);
    background-color: var(--color-bg-secondary);
    border: 1px solid var(--color-border);
  }

  .btn-secondary:hover:not(:disabled) {
    background-color: var(--color-bg-hover);
  }

  /* Error Message */
  .error-message {
    padding: 12px;
    border-radius: 6px;
    background-color: #ffebee;
    color: #c62828;
    font-size: 14px;
    border: 1px solid #ef9a9a;
  }
</style>
