# NothingBut Library - 启动指南

## 快速启动（开发模式）

```bash
cd /Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library
bun run tauri:dev
```

**首次启动约需 2-3 分钟**（需要编译 Rust 代码）

---

## 当前实现状态

### ✅ 后端功能（已完成）

1. **数据库系统**
   - SQLite 数据库（books, chapters, categories）
   - 完整的 CRUD 操作
   - 28 个测试全部通过

2. **文件处理**
   - TXT 文件解析（支持 UTF-8/GBK 编码）
   - 自动章节分割（识别多种章节标题格式）
   - 文件存储系统（目录管理）
   - 中英文字数统计

3. **Tauri Commands（后端 API）**
   - `preview_import` - 导入预览
   - `import_novel` - 完整导入流程
   - `list_books` - 查询所有书籍
   - `list_chapters` - 查询章节
   - `create_category` - 创建分类
   - `list_categories` - 查询分类

### ⏳ 前端 UI（待实现）

- 主界面布局
- 资料库首页
- 分类树组件
- 章节列表
- 阅读器界面
- 状态管理

**当前界面**：默认的 Tauri + Svelte 模板页面

---

## 测试后端功能

### 方法 1：通过浏览器控制台

1. 启动应用后，右键 → 检查（打开开发者工具）
2. 进入 Console 标签
3. 执行以下代码：

```javascript
const { invoke } = window.__TAURI__.core;

// 测试创建分类
invoke('create_category', {
  name: '玄幻',
  parentId: null,
  sortOrder: 1
}).then(id => {
  console.log('✅ 分类创建成功，ID:', id);

  // 测试查询分类
  return invoke('list_categories');
}).then(categories => {
  console.log('✅ 分类列表:', categories);
}).catch(err => {
  console.error('❌ 错误:', err);
});
```

### 方法 2：准备测试文件

创建一个测试小说文件：

```bash
cat > /tmp/test-novel.txt << 'EOF'
第一章 测试章节

这是第一章的内容，用于测试导入功能。
这是更多的测试内容。

第二章 另一个章节

这是第二章的内容。
继续测试。

第三章 最后一章

这是最后一章的内容。
EOF
```

然后在控制台测试导入预览：

```javascript
const { invoke } = window.__TAURI__.core;

invoke('preview_import', {
  workspacePath: '/tmp/test-workspace',
  filePath: '/tmp/test-novel.txt',
  title: '测试小说',
  author: '测试作者',
  category: '玄幻'
}).then(preview => {
  console.log('✅ 导入预览成功:', preview);
  console.log('章节数量:', preview.chapters.length);
  console.log('总字数:', preview.total_words);
}).catch(err => {
  console.error('❌ 错误:', err);
});
```

---

## 构建生产版本

如果需要独立的 .app 应用包：

```bash
cd /Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library
bun run tauri:build
```

生成的应用位于：
```
src-tauri/target/release/bundle/macos/NothingBut Library.app
```

---

## 验收建议

### 选项 1：现在验收后端功能
- ✅ 验证应用能正常启动
- ✅ 通过控制台测试后端 API
- ✅ 确认数据库操作正常
- ⏳ 等待前端 UI 完成后再次验收

### 选项 2：等待前端完成后统一验收
- 完成 Task 8-12（UI 基础实现）
- 完整的用户界面
- 完整的交互流程
- 端到端功能测试

---

## 项目进度

- **已完成**: 9/22 任务 (41%)
  - Chunk 1: 项目基础 ✅
  - Chunk 2: 后端功能 ✅
- **进行中**: Chunk 3: UI 基础实现
- **待完成**: Chunk 4: AI 集成, Chunk 5: 完善测试

---

## 问题排查

### 应用无法启动
```bash
# 检查依赖
cd /Users/shichang/Workspace/program/.worktrees/nothingbut-mvp/claude/nothingbut-library
bun install

# 检查 Rust 编译
cd src-tauri
cargo check
```

### 数据库位置
- 开发模式：`src-tauri/library.db`
- 生产模式：应用数据目录

### 日志位置
- 开发模式：终端输出
- 生产模式：`~/Library/Logs/com.nothingbut.library/`
