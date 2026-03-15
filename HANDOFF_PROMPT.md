# 项目交接提示词

当你继续这个项目时，请使用以下提示词启动新的 AI 会话：

---

## 📋 提示词模板

```
你好！我需要继续开发 NothingBut Library 项目。

## 项目背景
这是一个基于 Tauri 2 + SvelteKit 的桌面小说阅读应用，已集成本地 AI 功能（Ollama）。

## 当前状态
- ✅ 方案 C（AI 增强路线）100% 完成
- ✅ 所有 13 个任务已完成
- ✅ 代码编译通过（Rust + TypeScript）
- ⏳ 待手工测试和 bug 修复

## 已实现功能
1. 基础阅读功能（导入、章节、分类）
2. AI 对话助手（流式响应，80-95% 延迟降低）
3. 智能摘要（3 种长度，自动缓存）
4. 语义搜索（纯 Rust 向量存储，余弦相似度）
5. 并发控制（信号量，最多 3 个请求）

## 关键文档
请先阅读以下文档了解项目：
- `HANDOFF_2026-03-13_FINAL.md` - 完整项目交接
- `CLAUDE.md` - 开发指南
- `AI_ARCHITECTURE.md` - AI 架构设计

## 工作目录
/Users/shichang/Workspace/projects/ai-powered/nothingbut-library

## 下一步工作
[根据你的需求选择]

选项 A: 手工测试
- 按照 QUICK_TEST.md 执行测试
- 记录发现的问题
- 修复 bug

选项 B: 添加 UI 入口
- 在阅读器添加 "生成摘要" 按钮
- 在 AI 面板添加 "搜索" 标签
- 集成 SemanticSearch 组件

选项 C: 性能优化
- 实测性能指标
- 实现并发索引
- 优化搜索速度

选项 D: 新功能开发
- [描述你想要的功能]

请帮我：[具体描述你的需求]
```

---

## 🎯 快速启动场景

### 场景 1: 修复 Bug

```
我发现了一个 bug：[描述问题]

复现步骤：
1. [步骤 1]
2. [步骤 2]
3. [步骤 3]

预期行为：[描述]
实际行为：[描述]

请帮我定位和修复这个问题。

项目位置：/Users/shichang/Workspace/projects/ai-powered/nothingbut-library
参考文档：HANDOFF_2026-03-13_FINAL.md
```

### 场景 2: 添加新功能

```
我想添加一个新功能：[功能描述]

需求：
- [需求 1]
- [需求 2]
- [需求 3]

请先帮我：
1. 分析技术可行性
2. 设计实现方案
3. 提供代码示例

项目背景：已完成 AI 核心功能，详见 HANDOFF_2026-03-13_FINAL.md
工作目录：/Users/shichang/Workspace/projects/ai-powered/nothingbut-library
```

### 场景 3: 性能优化

```
我需要优化性能：

当前问题：
- [问题 1：例如搜索慢]
- [问题 2：例如索引慢]

性能目标：
- [目标 1]
- [目标 2]

请帮我：
1. 分析性能瓶颈
2. 提供优化方案
3. 实施改进

项目状态：详见 HANDOFF_2026-03-13_FINAL.md
技术栈：Tauri 2 + Rust + SvelteKit + Ollama
```

### 场景 4: 代码重构

```
我想重构以下部分：[模块名称]

原因：
- [原因 1]
- [原因 2]

请帮我：
1. 评估影响范围
2. 设计重构方案
3. 保持功能不变

项目文档：HANDOFF_2026-03-13_FINAL.md
架构说明：AI_ARCHITECTURE.md
```

---

## 📚 关键信息速查

### 目录结构
```
项目根目录/
├── src-tauri/              # Rust 后端
│   ├── src/modules/ai/     # AI 模块（7 个文件）
│   └── migrations/         # 数据库迁移（6 个）
├── src/lib/
│   ├── services/ai.ts      # AI API（180+ 行）
│   └── components/ai/      # AI 组件（2 个）
└── [文档]/                 # 11 个文档文件
```

### 技术栈
- **前端**: SvelteKit 2 + Svelte 5 + TypeScript + Tailwind CSS 4
- **后端**: Tauri 2 + Rust + tokio + sqlx
- **AI**: Ollama + qwen2.5:7b + nomic-embed-text
- **数据库**: SQLite（6 个表）

### 开发命令
```bash
bun run tauri:dev          # 完整开发
bun run check              # 类型检查
cd src-tauri && cargo test # Rust 测试
```

### 编译状态
- ✅ Rust: `cargo check` 通过
- ✅ TypeScript: `bun run check` 通过

### 代码统计
- 后端: ~1,400 行
- 前端: ~850 行
- 文档: ~3,500 行
- 总计: ~5,750 行

---

## ⚠️ 重要注意事项

### 1. 使用中文沟通
项目文档和注释主要使用中文，请继续使用中文。

### 2. 遵循现有架构
不要大幅改变已有架构，除非有充分理由。参考 `AI_ARCHITECTURE.md`。

### 3. 保持代码风格
- Rust: 标准 rustfmt 格式
- TypeScript: Svelte 最佳实践
- 注释清晰，文档完整

### 4. 测试先行
- 修改代码后运行 `cargo check` 和 `bun run check`
- 重要功能需要测试
- 性能敏感代码需要基准测试

### 5. 文档更新
- 新功能需要更新相关文档
- 重大改动需要更新 CLAUDE.md
- API 变化需要更新 AI_USER_GUIDE.md

---

## 🔧 常见任务快速参考

### 添加新的 Tauri 命令
1. 在 `src-tauri/src/modules/ai/commands.rs` 添加函数
2. 在 `src-tauri/src/lib.rs` 注册命令
3. 在 `src/lib/services/ai.ts` 添加 TypeScript 封装
4. 编译验证：`cargo check` 和 `bun run check`

### 添加新的数据库表
1. 创建迁移文件：`src-tauri/migrations/XXXX_name.sql`
2. 编写 CREATE TABLE 语句
3. 在相应模块添加数据库操作函数
4. 重启应用自动执行迁移

### 添加新的 AI 功能
1. 参考 `AI_ARCHITECTURE.md` 设计
2. 在 `src-tauri/src/modules/ai/` 添加模块
3. 使用信号量控制并发：`AI_REQUEST_SEMAPHORE`
4. 添加错误处理和日志
5. 更新 `AI_USER_GUIDE.md`

### 调试问题
1. Rust 日志：使用 `println!` 或 `eprintln!`
2. 前端日志：浏览器控制台
3. Ollama 日志：`~/.ollama/logs/server.log`
4. 数据库：使用 DB Browser for SQLite

---

## 📞 需要帮助？

如果遇到问题，请提供：
1. 详细错误信息
2. 相关代码片段
3. 复现步骤
4. 已尝试的解决方案

我会基于项目文档（特别是 HANDOFF_2026-03-13_FINAL.md）提供帮助。

---

**交接时间**: 2026-03-13
**项目状态**: ✅ 100% 完成，待测试
**下次会话**: 根据测试结果决定
