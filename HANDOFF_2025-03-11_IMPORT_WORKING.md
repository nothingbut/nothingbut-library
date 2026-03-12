# NothingBut Library - 导入功能可用交接

**日期**: 2025-03-11
**分支**: feature/nothingbut-library-mvp
**状态**: ✅ 导入功能可用，待添加分类选择器

---

## 当前状态

### ✅ 已完成功能
- [x] 数据库初始化和迁移
- [x] UI 布局（两栏小说模块）
- [x] 前端数据集成（API 调用）
- [x] 导入对话框（完整工作流）
- [x] 文件选择和解析
- [x] 章节提取和存储
- [x] 导入成功并验证（已导入 book-1，21+ 章节）

### ⚠️ 已知限制
1. **分类输入方式**：当前为自由文本输入，需要改为选择器
2. **元数据解析**：只解析章节，不解析作者/简介（需手动输入）
3. **Workspace 路径**：硬编码在前端代码中

### 🎯 待实现功能
- [ ] 分类选择器（使用 bsconfig.json 数据）
- [ ] 子分类联动选择
- [ ] 作者/简介智能提取（可选）
- [ ] Workspace 配置管理

---

## 关键信息

### Git 提交记录
```
58d7980 fix: correct chapterId variable name in error message
28322e2 fix: use camelCase for all Tauri command parameters
98eb206 refactor: improve import flow - parse file immediately
c508195 fix: add dialog plugin permissions
3b78241 feat: implement novel import functionality
27f4fe0 feat: integrate frontend with backend API
```

### 测试文件位置
```
/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library/test-novel.txt
```

### 分类数据源
```
/Users/shichang/Workspace/program/data/bsconfig.json
```

**结构**：
```json
{
  "attr.tagsJson": [
    {
      "category": "玄幻",
      "subcategories": ["东方玄幻", "异世大陆", ...]
    },
    {
      "category": "奇幻",
      "subcategories": ["现代魔法", "剑与魔法", ...]
    }
    // ... 共 14 个主分类
  ]
}
```

---

## 导入流程说明

### 当前工作流
1. **选择文件** → 自动触发解析
2. **显示解析结果** → 章节统计（总章节、总字数）
3. **编辑元数据** → 书名、作者、分类、简介
4. **确认导入** → 存储到数据库和文件系统
5. **刷新界面** → 自动更新分类树

### 元数据处理
- **书名**：从文件名自动提取（可编辑）
- **作者**：默认"未知作者"（需手动输入）
- **分类**：默认"未分类"（需手动输入/选择）
- **简介**：默认空（需手动输入）

### 章节解析规则
支持的章节标题格式（parser.rs）：
- `第一章 标题`
- `第1章 标题`
- `第一章：标题`
- `Chapter 1 Title`

---

## 参数命名约定

**重要**：所有 Tauri 命令使用 **camelCase** 参数名

### 已修复的命令
```rust
// Rust 端
pub async fn preview_import(
    filePath: String,  // ✅ camelCase
    title: String,
    author: String,
    category: String,
)

// JavaScript 端
await invoke('preview_import', {
    filePath,  // ✅ camelCase
    title,
    author,
    category
});
```

### 所有命令参数
- `preview_import`: filePath, title, author, category
- `import_novel`: workspacePath, filePath, title, author, description, categoryId
- `list_books`: 无参数
- `list_chapters`: bookId
- `get_chapter_content`: workspacePath, chapterId
- `create_category`: name, parentId, sortOrder
- `list_categories`: 无参数

---

## 关键文件位置

### 前端
- `src/lib/components/ImportDialog.svelte` - 导入对话框（700+ 行）
- `src/lib/services/api.ts` - API 封装
- `src/lib/types.ts` - 类型定义
- `src/routes/novel/+page.svelte` - 小说模块页面
- `src/lib/components/CategoryTree.svelte` - 分类树

### 后端
- `src-tauri/src/modules/novel/commands.rs` - Tauri Commands
- `src-tauri/src/modules/novel/parser.rs` - 章节解析器
- `src-tauri/src/modules/novel/database.rs` - 数据库操作
- `src-tauri/src/modules/novel/models.rs` - 数据模型
- `src-tauri/capabilities/default.json` - 权限配置

### 配置
- `src-tauri/Cargo.toml` - Rust 依赖
- `package.json` - 前端依赖
- `src-tauri/migrations/` - 数据库迁移

---

## 下次会话启动

### 工作目录
```bash
cd /Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library
```

### 启动应用
```bash
bun run tauri:dev
# 访问: http://localhost:1420/
```

### 停止应用
```bash
lsof -ti:1420 | xargs kill -9
```

---

## 下一步任务：添加分类选择器

### 优先级 1: 分类选择器（1-2 小时）

**目标**：将分类输入改为下拉选择，使用 bsconfig.json 数据

**实现步骤**：

1. **创建分类数据文件**
   ```typescript
   // src/lib/data/categories.ts
   export const CATEGORIES = [
     { category: "玄幻", subcategories: ["东方玄幻", "异世大陆", ...] },
     { category: "奇幻", subcategories: ["现代魔法", "剑与魔法", ...] },
     // ... 从 bsconfig.json 复制完整数据
   ];
   ```

2. **修改 ImportDialog.svelte**
   - 将分类输入框改为 `<select>` 下拉框
   - 添加子分类联动选择
   - 保存为 `category/subcategory` 格式

3. **UI 设计**
   ```html
   <div class="form-row">
     <select bind:value={mainCategory}>
       <option value="">选择分类</option>
       <option>玄幻</option>
       <option>奇幻</option>
       ...
     </select>

     {#if mainCategory}
     <select bind:value={subCategory}>
       <option value="">选择子分类（可选）</option>
       {#each subcategories as sub}
       <option>{sub}</option>
       {/each}
     </select>
     {/if}
   </div>
   ```

4. **数据处理**
   ```typescript
   // 保存时组合
   const category = subCategory
     ? `${mainCategory}/${subCategory}`
     : mainCategory;
   ```

### 优先级 2: 优化元数据输入（可选）

1. **作者提取**
   - 尝试从文件前几行提取"作者："字段
   - 如果失败，保持"未知作者"默认值

2. **简介提取**
   - 尝试从文件前几行提取简介段落
   - 如果失败，保持空默认值

### 优先级 3: Workspace 配置（1 小时）

1. 创建 workspace store
2. 添加配置界面
3. 替换硬编码路径

---

## 提示词模板

### 立即开始分类选择器
```
继续开发 NothingBut Library MVP。

当前进度：
- ✅ 导入功能已完成并验证
- ✅ 章节解析和存储正常
- 🎯 下一步：添加分类选择器

任务：将导入对话框的分类输入改为下拉选择器。

数据源：/Users/shichang/Workspace/program/data/bsconfig.json
- attr.tagsJson 包含 14 个主分类
- 每个分类有多个子分类

要求：
1. 创建分类数据文件（复制 bsconfig.json 数据）
2. 修改 ImportDialog.svelte 的分类输入为下拉选择
3. 实现主分类和子分类联动
4. 保存格式：主分类/子分类（例如：玄幻/东方玄幻）

交接文档：HANDOFF_2025-03-11_IMPORT_WORKING.md
工作分支：feature/nothingbut-library-mvp
```

### 或者：继续其他优化
```
继续开发 NothingBut Library MVP。

参考交接文档：HANDOFF_2025-03-11_IMPORT_WORKING.md

可选任务：
1. 添加分类选择器（优先级最高）
2. 实现作者/简介智能提取
3. 添加 Workspace 配置管理
4. 优化导入流程 UI

请根据优先级选择任务开始实现。
```

---

## 测试清单

### 验证导入功能
- [x] 文件选择对话框正常
- [x] 文件解析成功
- [x] 章节统计正确
- [x] 元数据可编辑
- [x] 导入成功
- [x] 分类树刷新
- [x] 书籍可查看
- [x] 章节可阅读

### 待测试功能
- [ ] 分类选择器（待实现）
- [ ] 子分类联动（待实现）
- [ ] 重复导入处理
- [ ] 大文件导入性能
- [ ] 错误处理完整性

---

## 已知问题

### 非阻塞问题
1. **A11y 警告**：ImportDialog 有 8 个可访问性警告（不影响功能）
2. **硬编码路径**：workspace 路径写死在代码中
3. **分类输入**：自由文本输入，容易出错

### 预期行为
1. **作者/简介为空**：设计如此，需要用户手动输入
2. **章节序号**：从文件名提取，可能不连续
3. **文件名即书名**：默认行为，可编辑

---

## 应用信息

**前端**: Svelte 5 + Vite
**后端**: Rust + Tauri 2.0
**数据库**: SQLite (library.db)
**文件存储**: books/book-{id}/chapters/

**端口**: http://localhost:1420/
**数据库位置**: `/Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library/library.db`

---

**会话结束时间**: 2025-03-11 22:30
**下次会话**: 添加分类选择器或其他优化
