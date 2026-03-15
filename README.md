# NothingBut Library

基于 Tauri 2 的桌面小说阅读和管理应用，集成本地 AI 功能。

## ✨ 特性

### 📚 核心功能
- 小说导入和管理（支持 TXT 文件，自动编码检测）
- 章节解析和阅读
- 分类管理（主分类 + 子分类）
- 阅读进度追踪

### 🤖 AI 功能（本地运行）
- **AI 对话助手**: 讨论书籍内容，回答问题
- **智能摘要**: 自动生成章节摘要（3 种长度）
- **语义搜索**: 用自然语言描述查找相关章节

## 🚀 快速开始

### 前置要求

- [Node.js](https://nodejs.org/) 18+
- [Bun](https://bun.sh/)
- [Rust](https://rustup.rs/)
- [Ollama](https://ollama.com/)（可选，用于 AI 功能）

### 安装

```bash
# 克隆项目
git clone <repository-url>
cd nothingbut-library

# 安装依赖
bun install
```

### 开发

```bash
# 前端开发（仅 Vite）
bun run dev

# 完整开发（Tauri + 前端）
bun run tauri:dev

# 类型检查
bun run check
```

### 构建

```bash
# 构建应用
bun run tauri:build
```

## 🤖 AI 功能设置

### 1. 安装 Ollama

```bash
# macOS/Linux
curl -fsSL https://ollama.com/install.sh | sh

# Windows: 访问 https://ollama.com 下载安装包
```

### 2. 下载模型

```bash
# 对话模型（4.7GB）
ollama pull qwen2.5:7b

# 嵌入模型（274MB，用于语义搜索）
ollama pull nomic-embed-text
```

### 3. 启动服务

```bash
ollama serve
```

### 4. 使用 AI 功能

1. 启动应用
2. 点击右上角 "🤖 打开 AI" 按钮
3. 开始对话或搜索

详细使用说明请参考 [AI_USER_GUIDE.md](./AI_USER_GUIDE.md)

## 📖 文档

- [CLAUDE.md](./CLAUDE.md) - 项目开发指南
- [AI_ARCHITECTURE.md](./AI_ARCHITECTURE.md) - AI 架构设计
- [AI_USER_GUIDE.md](./AI_USER_GUIDE.md) - AI 功能用户指南
- [TESTING_GUIDE.md](./TESTING_GUIDE.md) - 测试指南
- [HANDOFF_2026-03-13_FINAL.md](./HANDOFF_2026-03-13_FINAL.md) - 项目交接文档

## 🛠️ 技术栈

**前端**
- [SvelteKit 2](https://kit.svelte.dev/) + [Svelte 5](https://svelte.dev/)
- TypeScript
- Tailwind CSS 4

**后端**
- [Tauri 2](https://v2.tauri.app/)
- Rust with tokio
- SQLite + sqlx

**AI**
- [Ollama](https://ollama.com/) - 本地 AI 模型运行
- 纯 Rust 向量存储（余弦相似度搜索）

## 📊 项目状态

- ✅ 基础阅读功能 - 完成
- ✅ AI 对话助手 - 完成
- ✅ 智能摘要 - 完成
- ✅ 语义搜索 - 完成
- ✅ 流式响应 - 完成
- 📝 测试和文档 - 完成

## 📝 许可证

MIT

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台桌面应用框架
- [SvelteKit](https://kit.svelte.dev/) - Web 应用框架
- [Ollama](https://ollama.com/) - 本地 AI 模型运行平台
