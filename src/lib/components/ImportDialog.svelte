<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { previewImport, importNovel, createCategory, listCategories, listBooks, fetchBookMetadata } from '$lib/services/api';
  import type { ImportPreview } from '$lib/services/api';
  import { CATEGORIES } from '$lib/data/categories';
  import { SOURCE_SITES } from '$lib/data/sourceSites';

  // Props
  interface Props {
    isOpen?: boolean;
    onClose?: () => void;
    onSuccess?: () => void;
  }

  let { isOpen = $bindable(false), onClose, onSuccess }: Props = $props();

  // State
  let step = $state<'select' | 'parsing' | 'edit' | 'importing' | 'success' | 'error' | 'duplicate-confirm'>('select');
  let selectedFile = $state<string | null>(null);
  let preview = $state<ImportPreview | null>(null);
  let error = $state<string | null>(null);
  let parsing = $state(false);
  let importing = $state(false);
  let refreshing = $state(false);
  let duplicateMessage = $state<string | null>(null);

  // Form data
  let title = $state('');
  let author = $state('');
  let description = $state('');
  let mainCategory = $state('');
  let subCategory = $state('');
  let sourceSite = $state('');

  // Workspace path (hardcoded for now)
  const workspacePath = '/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library';

  // Get subcategories for selected main category
  const subcategories = $derived(
    mainCategory ? CATEGORIES.find(c => c.category === mainCategory)?.subcategories || [] : []
  );

  // Select file and immediately parse
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

        // Immediately parse the file
        await parseFile();
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to select file';
      console.error('File selection error:', e);
    }
  }

  // Parse file and show preview
  async function parseFile() {
    if (!selectedFile || !title) {
      error = 'Please select a file';
      return;
    }

    try {
      parsing = true;
      step = 'parsing';
      error = null;

      console.log('Parsing file:', {
        file: selectedFile,
        title
      });

      const categoryStr = mainCategory && subCategory ? `${mainCategory}/${subCategory}` : (mainCategory || '未分类');
      preview = await previewImport(
        selectedFile,
        title,
        categoryStr
      );

      console.log('Parse successful:', preview);

      // Use auto-extracted metadata if available
      if (preview.author) {
        author = preview.author;
      }
      if (preview.description) {
        description = preview.description;
      }

      // Check for duplicates before continuing
      await checkDuplicates();
    } catch (e) {
      console.error('Parse error details:', e);
      // Try to get more detailed error message
      if (e && typeof e === 'object' && 'message' in e) {
        error = `解析失败: ${e.message}`;
      } else {
        error = `解析失败: ${String(e)}`;
      }
      step = 'error';
    } finally {
      parsing = false;
    }
  }

  // Check for duplicate books
  async function checkDuplicates() {
    try {
      const books = await listBooks();
      const duplicate = books.find(
        b => b.title === title && b.author === author
      );

      if (duplicate) {
        duplicateMessage = `已存在相同书名和作者的图书。\n书名：${duplicate.title}\n作者：${duplicate.author || '未知'}`;
        step = 'duplicate-confirm';
      } else {
        step = 'edit';
      }
    } catch (e) {
      console.error('Failed to check duplicates:', e);
      // Continue to edit step even if duplicate check fails
      step = 'edit';
    }
  }

  // Confirm duplicate import
  function confirmDuplicateImport() {
    duplicateMessage = null;
    step = 'edit';
  }

  // Refresh metadata from source site
  async function handleRefreshMetadata() {
    if (!sourceSite || !title) {
      return;
    }

    try {
      refreshing = true;
      error = null;

      console.log('Fetching metadata from:', sourceSite, 'for book:', title, 'author:', author);

      const metadata = await fetchBookMetadata(
        workspacePath,
        null, // No bookId yet (before import)
        sourceSite,
        title,
        author || undefined
      );

      console.log('Fetched metadata:', metadata);

      // Update form fields with fetched data
      if (metadata.description) {
        description = metadata.description;
      }
      if (metadata.author) {
        author = metadata.author;
      }

      // Parse and fill category information
      if (metadata.category) {
        // Try to match with existing categories
        const categoryText = metadata.category.trim();

        // Check if it matches any of our main categories
        for (const cat of CATEGORIES) {
          if (categoryText.includes(cat.category)) {
            mainCategory = cat.category;

            // Check if there's a subcategory match
            for (const subcat of cat.subcategories) {
              if (categoryText.includes(subcat)) {
                subCategory = subcat;
                break;
              }
            }
            break;
          }
        }

        // If no exact match, check subcategories first
        if (!mainCategory) {
          for (const cat of CATEGORIES) {
            for (const subcat of cat.subcategories) {
              if (categoryText.includes(subcat)) {
                mainCategory = cat.category;
                subCategory = subcat;
                break;
              }
            }
            if (mainCategory) break;
          }
        }
      }

      // Show success message
      const parts = [];
      if (metadata.description) parts.push('简介');
      if (metadata.author) parts.push('作者');
      if (metadata.category) parts.push('分类');
      if (metadata.coverUrl) parts.push('封面URL');

      if (parts.length > 0) {
        alert(`成功获取 ${parts.join('、')} 信息！`);
      } else {
        alert('未找到相关信息');
      }

    } catch (e) {
      console.error('Failed to refresh metadata:', e);
      if (e && typeof e === 'object' && 'message' in e) {
        error = `获取信息失败: ${e.message}`;
      } else {
        error = `获取信息失败: ${String(e)}`;
      }
    } finally {
      refreshing = false;
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

      // Handle category selection
      let categoryId: number | undefined;
      if (mainCategory) {
        try {
          // Check if main category exists, create if not
          const categories = await listCategories();
          let mainCat = categories.find(c => c.name === mainCategory && !c.parent_id);

          if (!mainCat) {
            const mainId = await createCategory(mainCategory, undefined, 0);
            mainCat = { id: mainId, name: mainCategory, parent_id: null, sort_order: 0, created_at: '' };
          }

          // If subcategory specified, handle it
          if (subCategory) {
            let subCat = categories.find(c => c.name === subCategory && c.parent_id === mainCat!.id);
            if (!subCat) {
              categoryId = await createCategory(subCategory, mainCat.id, 0);
            } else {
              categoryId = subCat.id;
            }
          } else {
            categoryId = mainCat.id;
          }
        } catch (e) {
          console.warn('Failed to handle category:', e);
        }
      }

      // Import the novel
      await importNovel(
        workspacePath,
        selectedFile,
        title,
        author || undefined,
        description || undefined,
        categoryId,
        sourceSite || undefined
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
    selectedFile = null;
    preview = null;
    error = null;
    duplicateMessage = null;
    title = '';
    author = '';
    description = '';
    mainCategory = '';
    subCategory = '';
    sourceSite = '';
  }

  // Close dialog
  function handleClose() {
    isOpen = false;
    step = 'select';
    selectedFile = null;
    preview = null;
    error = null;
    duplicateMessage = null;
    title = '';
    author = '';
    description = '';
    mainCategory = '';
    subCategory = '';
    sourceSite = '';
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
          {:else if step === 'parsing'}
            解析文件中...
          {:else if step === 'edit'}
            确认导入信息
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
            <div class="select-prompt">
              <div class="select-icon">📂</div>
              <p class="select-text">选择一个 .txt 格式的小说文件开始导入</p>
              <button class="btn btn-primary btn-large" onclick={selectFile}>
                选择文件
              </button>
            </div>

            {#if error}
              <div class="error-message">{error}</div>
            {/if}
          </div>
        {:else if step === 'parsing'}
          <!-- Step 2: Parsing -->
          <div class="status-section">
            <div class="spinner"></div>
            <p class="status-text">正在解析文件...</p>
            <p class="status-subtext">提取章节信息和元数据</p>
          </div>
        {:else if step === 'duplicate-confirm'}
          <!-- Duplicate Confirmation -->
          <div class="status-section">
            <div class="warning-icon">⚠</div>
            <p class="status-text">发现重复</p>
            {#if duplicateMessage}
              <p class="status-subtext warning">{duplicateMessage}</p>
            {/if}
            <p class="status-subtext">您仍然可以继续导入，或者选择重新选择文件。</p>
          </div>
        {:else if step === 'edit' && preview}
          <!-- Step 3: Edit Metadata & Preview -->
          <div class="edit-section">
            <!-- Statistics Summary -->
            <div class="stats-summary">
              <div class="stat-item">
                <span class="stat-label">总章节</span>
                <span class="stat-value">{formatNumber(preview.total_chapters)}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">总字数</span>
                <span class="stat-value">{formatNumber(preview.total_words)}</span>
              </div>
            </div>

            <!-- Metadata Form -->
            <div class="form-section">
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
                  placeholder="作者姓名"
                />
              </div>

              <div class="form-group">
                <label class="form-label">首发网站</label>
                <div class="form-group-with-action">
                  <select
                    class="form-select"
                    bind:value={sourceSite}
                  >
                    {#each SOURCE_SITES as site}
                      <option value={site === '--' ? '' : site}>{site}</option>
                    {/each}
                  </select>
                  <button
                    class="btn btn-secondary btn-icon"
                    onclick={handleRefreshMetadata}
                    disabled={!sourceSite || refreshing}
                    title="从网站抓取简介和封面"
                  >
                    {#if refreshing}
                      <span class="spinner-small"></span>
                    {:else}
                      🔄
                    {/if}
                  </button>
                </div>
              </div>

              <div class="form-group">
                <label class="form-label">主分类</label>
                <select
                  class="form-select"
                  bind:value={mainCategory}
                  onchange={() => { subCategory = ''; }}
                >
                  <option value="">-- 选择主分类 --</option>
                  {#each CATEGORIES as cat}
                    <option value={cat.category}>{cat.category}</option>
                  {/each}
                </select>
              </div>

              {#if mainCategory && subcategories.length > 0}
                <div class="form-group">
                  <label class="form-label">子分类</label>
                  <select
                    class="form-select"
                    bind:value={subCategory}
                  >
                    <option value="">-- 选择子分类（可选） --</option>
                    {#each subcategories as subcat}
                      <option value={subcat}>{subcat}</option>
                    {/each}
                  </select>
                </div>
              {/if}

              <div class="form-group">
                <label class="form-label">简介</label>
                <textarea
                  class="form-textarea"
                  bind:value={description}
                  placeholder="书籍简介（可选）"
                  rows="3"
                ></textarea>
              </div>
            </div>

            <!-- Chapter Preview -->
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
        {:else if step === 'duplicate-confirm'}
          <button class="btn btn-secondary" onclick={handleBack}>重新选择</button>
          <button class="btn btn-primary" onclick={confirmDuplicateImport}>
            继续导入
          </button>
        {:else if step === 'edit'}
          <button class="btn btn-secondary" onclick={handleBack}>重新选择</button>
          <button
            class="btn btn-primary"
            onclick={handleImport}
            disabled={!title || importing}
          >
            {importing ? '导入中...' : '确认导入'}
          </button>
        {:else if step === 'error'}
          <button class="btn btn-secondary" onclick={handleClose}>关闭</button>
          <button class="btn btn-primary" onclick={handleBack}>重新选择</button>
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

  /* Select Prompt */
  .select-prompt {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 64px 24px;
    text-align: center;
  }

  .select-icon {
    font-size: 72px;
    margin-bottom: 24px;
  }

  .select-text {
    font-size: 16px;
    color: var(--color-text-secondary);
    margin: 0 0 32px 0;
    max-width: 400px;
  }

  .btn-large {
    padding: 14px 32px;
    font-size: 16px;
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

  .form-group-with-action {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .form-group-with-action .form-select {
    flex: 1;
  }

  .form-label {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .form-input,
  .form-textarea,
  .form-select {
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
  .form-textarea:focus,
  .form-select:focus {
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

  .form-select {
    cursor: pointer;
  }

  /* Edit Section */
  .edit-section {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .stats-summary {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 16px;
    padding: 16px;
    border-radius: 8px;
    background-color: var(--color-bg-secondary);
  }

  .stat-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
  }

  .stat-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .stat-value {
    font-size: 24px;
    font-weight: 700;
    color: var(--color-primary);
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

  .warning-icon {
    width: 64px;
    height: 64px;
    border-radius: 50%;
    background-color: #ff9800;
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
    white-space: pre-line;
  }

  .status-subtext.error {
    color: #f44336;
  }

  .status-subtext.warning {
    color: #ff9800;
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

  .btn-icon {
    padding: 10px;
    min-width: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
  }

  .spinner-small {
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-bg-secondary);
    border-top-color: var(--color-text-secondary);
    border-radius: 50%;
    animation: spin 1s linear infinite;
    display: inline-block;
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
