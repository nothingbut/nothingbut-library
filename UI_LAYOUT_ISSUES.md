# UI 布局问题分析

**日期**: 2025-03-11
**对比**: Spec vs 实际实现

---

## 问题总结

实际实现的 UI 与 spec 设计存在以下主要差异：

1. **AI 面板位置错误** - 应该在左侧，实际在右侧
2. **缺少资料库首页** - 应该显示 4 种资料类型卡片，实际直接进入书籍列表
3. **小说模块布局混淆** - WorkspaceSelector 不应该在这个位置

---

## 详细对比

### 1. AI 面板位置

**Spec (section 5.1)**:
```
主界面布局：
- 左侧/顶部 AI 面板：320px 宽（可隐藏）
- 右侧/底部主窗口：自适应宽度

桌面端 (>1024px): 左右布局，AI 在左侧
```

**实际实现** (`AppLayout.svelte`):
```svelte
<!-- AI Panel (toggleable) -->
{#if showAIPanel}
  <aside class="ai-panel">
    <div class="ai-panel-header">
      <h2>AI 助手</h2>
    </div>
    ...
  </aside>
{/if}
```

```css
.ai-panel {
  width: 320px;
  background-color: var(--color-bg-primary);
  border-left: 1px solid var(--color-border);  /* ❌ 左边框 = 在右侧 */
  ...
}
```

**问题**:
- `border-left` 表示 AI 面板在右侧
- Spec 要求在左侧

**正确实现**:
```css
.ai-panel {
  width: 320px;
  border-right: 1px solid var(--color-border);  /* ✅ 右边框 = 在左侧 */
  order: -1;  /* 确保在 content-area 之前 */
}
```

---

### 2. 首页布局错误

**Spec (section 5.2 - 资料库首页)**:
```
网格布局:
- 展示四种资料类型卡片（网络小说、音乐库、电子书、笔记）
- 卡片大小：200px x 200px，正方形
- 网络小说卡片可点击，其他显示"即将推出"徽章

最近使用:
- 列表展示最近打开的工作区
- 显示工作区名称、类型、最后打开时间
```

**实际实现** (`+page.svelte`):
```svelte
<div class="library-home">
  <WorkspaceSelector />  <!-- ❌ 左侧工作区选择器 -->
  <LibraryGrid />        <!-- ❌ 书籍网格 -->
</div>
```

**问题**:
1. 缺少 4 种资料类型的卡片网格
2. `WorkspaceSelector` 是为小说模块设计的，不应该在首页
3. `LibraryGrid` 显示的是书籍列表，不是资料类型选择

**正确实现** (应该是):
```svelte
<!-- 首页: +page.svelte -->
<div class="home-page">
  <h1>选择资料类型</h1>

  <div class="module-grid">
    <!-- 4 个资料类型卡片 -->
    <a href="/novel" class="module-card novel">
      <div class="icon">📚</div>
      <h3>网络小说</h3>
    </a>

    <div class="module-card coming-soon">
      <div class="icon">🎵</div>
      <h3>音乐库</h3>
      <span class="badge">即将推出</span>
    </div>

    <div class="module-card coming-soon">
      <div class="icon">📖</div>
      <h3>电子书</h3>
      <span class="badge">即将推出</span>
    </div>

    <div class="module-card coming-soon">
      <div class="icon">📝</div>
      <h3>笔记</h3>
      <span class="badge">即将推出</span>
    </div>
  </div>

  <!-- 最近使用 -->
  <div class="recent-workspaces">
    <h2>最近使用</h2>
    <ul>
      <!-- 最近工作区列表 -->
    </ul>
  </div>
</div>
```

---

### 3. 小说模块布局

**Spec (section 5.3 - 小说模块布局（三栏）)**:
```
左栏 - 分类树 (280px):
  - 树状结构展示分类和书籍
  - 四层层级

中栏 - 章节列表 (320px):
  - 显示选中书籍的所有章节
  - 章节卡片显示：标题、字数

右栏 - 阅读器 (自适应):
  - 章节标题栏 + 工具按钮
  - 内容区：最大宽度 800px，居中显示
```

**实际实现** (`+page.svelte`):
```
目前首页就是小说模块的实现：
- 左侧：WorkspaceSelector (240px)  ← 应该是 CategoryTree
- 右侧：LibraryGrid             ← 应该是 ChapterList + Reader
```

**问题**:
1. 首页不应该直接是小说模块
2. 小说模块应该是三栏（分类树 + 章节列表 + 阅读器）
3. `WorkspaceSelector` 应该在顶部工具栏，不是左侧栏

**正确实现** (应该是):
```
首页路由: /
└─ 资料类型网格（4 个卡片）

小说模块路由: /novel
├─ 分类树 (280px) - CategoryTree.svelte
├─ 章节列表 (320px) - ChapterList.svelte（当选中书籍时显示）
└─ 阅读器 (自适应) - Reader.svelte（当选中章节时显示）

阅读器路由: /reader/[bookId]
└─ 全屏阅读器
```

---

## 修复优先级

### P0 - 关键问题（必须修复）

1. **创建正确的首页**
   - 4 种资料类型卡片网格
   - 最近使用工作区列表
   - 路由: `/`

2. **修复 AI 面板位置**
   - 从右侧移到左侧
   - 修改 `AppLayout.svelte`

3. **重构小说模块路由**
   - 创建 `/novel` 路由
   - 实现三栏布局
   - 左栏: CategoryTree（已存在）
   - 中栏: ChapterList（已存在）
   - 右栏: LibraryGrid 改造为书籍网格视图

### P1 - 重要改进

4. **工作区选择器位置**
   - 从左侧栏移到顶部工具栏
   - 作为下拉菜单

5. **导航流程**
   ```
   首页 (/)
     → 点击"网络小说"
     → 小说模块 (/novel)
     → 选择书籍
     → 显示章节列表
     → 点击章节
     → 阅读器 (/reader/[bookId])
   ```

---

## 修复计划

### Step 1: 创建正确的首页

**新建**: `src/routes/+page.svelte`
```svelte
<script lang="ts">
  const modules = [
    { id: 'novel', name: '网络小说', icon: '📚', available: true },
    { id: 'music', name: '音乐库', icon: '🎵', available: false },
    { id: 'ebook', name: '电子书', icon: '📖', available: false },
    { id: 'note', name: '笔记', icon: '📝', available: false },
  ];
</script>

<div class="home-page">
  <div class="home-header">
    <h1>NothingBut Library</h1>
    <p>选择资料类型开始</p>
  </div>

  <div class="module-grid">
    {#each modules as module}
      {#if module.available}
        <a href="/{module.id}" class="module-card">
          <div class="module-icon">{module.icon}</div>
          <h3 class="module-name">{module.name}</h3>
        </a>
      {:else}
        <div class="module-card disabled">
          <div class="module-icon">{module.icon}</div>
          <h3 class="module-name">{module.name}</h3>
          <span class="coming-soon">即将推出</span>
        </div>
      {/if}
    {/each}
  </div>

  <!-- 最近使用 -->
  <div class="recent-section">
    <h2>最近使用</h2>
    <div class="recent-list">
      <!-- TODO: 显示最近工作区 -->
    </div>
  </div>
</div>

<style>
  .module-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 24px;
    max-width: 880px;
    margin: 48px auto;
  }

  .module-card {
    aspect-ratio: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 16px;
    padding: 24px;
    border-radius: 12px;
    background-color: var(--color-bg-primary);
    border: 2px solid var(--color-border);
    transition: all 0.2s ease;
    cursor: pointer;
  }

  .module-card:hover:not(.disabled) {
    transform: translateY(-4px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.1);
    border-color: var(--color-primary);
  }

  .module-card.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .module-icon {
    font-size: 64px;
  }

  .module-name {
    font-size: 18px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .coming-soon {
    font-size: 12px;
    color: var(--color-text-secondary);
    background-color: var(--color-bg-secondary);
    padding: 4px 12px;
    border-radius: 12px;
  }
</style>
```

### Step 2: 修复 AI 面板位置

**修改**: `src/lib/components/AppLayout.svelte`

```diff
<main class="main-content" class:with-ai-panel={showAIPanel}>
+   <!-- AI Panel (toggleable) - 在左侧 -->
+   {#if showAIPanel}
+     <aside class="ai-panel">
+       <div class="ai-panel-header">
+         <h2>AI 助手</h2>
+       </div>
+       <div class="ai-panel-content">
+         <p class="placeholder-text">AI 助手面板占位</p>
+       </div>
+     </aside>
+   {/if}
+
    <div class="content-area">
      {@render children()}
    </div>
-
-   <!-- AI Panel (toggleable) -->
-   {#if showAIPanel}
-     <aside class="ai-panel">
-       ...
-     </aside>
-   {/if}
</main>
```

```diff
.ai-panel {
  width: 320px;
  background-color: var(--color-bg-primary);
-  border-left: 1px solid var(--color-border);
+  border-right: 1px solid var(--color-border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
```

### Step 3: 创建小说模块路由

**新建**: `src/routes/novel/+page.svelte`
```svelte
<script lang="ts">
  import CategoryTree from '$lib/components/CategoryTree.svelte';
  import ChapterList from '$lib/components/ChapterList.svelte';
  import Reader from '$lib/components/Reader.svelte';

  let selectedBookId = $state<number | null>(null);
  let selectedChapterId = $state<number | null>(null);
</script>

<div class="novel-module">
  <!-- 左栏: 分类树 -->
  <aside class="category-sidebar">
    <CategoryTree bind:selectedBookId />
  </aside>

  <!-- 中栏: 章节列表 -->
  {#if selectedBookId}
    <aside class="chapter-sidebar">
      <ChapterList
        bookId={selectedBookId}
        bind:selectedChapterId
      />
    </aside>
  {/if}

  <!-- 右栏: 阅读器 -->
  <main class="reader-area">
    {#if selectedChapterId}
      <Reader chapterId={selectedChapterId} />
    {:else}
      <div class="empty-state">
        <p>请选择一本书开始阅读</p>
      </div>
    {/if}
  </main>
</div>

<style>
  .novel-module {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .category-sidebar {
    width: 280px;
    border-right: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .chapter-sidebar {
    width: 320px;
    border-right: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .reader-area {
    flex: 1;
    overflow: auto;
  }
</style>
```

---

## 文件清单

### 需要创建
- [ ] `src/routes/+page.svelte` - 新首页（4 个卡片）
- [ ] `src/routes/novel/+page.svelte` - 小说模块（三栏布局）

### 需要修改
- [ ] `src/lib/components/AppLayout.svelte` - 修复 AI 面板位置
- [ ] `src/lib/components/WorkspaceSelector.svelte` - 改造为下拉组件

### 可以复用
- [x] `src/lib/components/CategoryTree.svelte` - 已存在
- [x] `src/lib/components/ChapterList.svelte` - 已存在
- [x] `src/lib/components/Reader.svelte` - 已存在
- [x] `src/lib/components/LibraryGrid.svelte` - 可改造为书籍网格视图

---

## 预估工作量

- **Step 1** (创建首页): 30 分钟
- **Step 2** (修复 AI 面板): 15 分钟
- **Step 3** (小说模块路由): 45 分钟
- **Step 4** (工作区选择器重构): 30 分钟

**总计**: 约 2 小时

---

**文档创建时间**: 2025-03-11
**下一步**: 修复 UI 布局问题
