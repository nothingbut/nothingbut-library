# 快速手工测试步骤（15 分钟）

## 准备工作（5 分钟）

### 1. 启动 Ollama
```bash
# 新终端窗口
ollama serve
```

### 2. 验证服务
浏览器打开：http://localhost:11434
应该看到：`Ollama is running`

### 3. 启动应用
```bash
# 项目根目录
bun run tauri:dev
```

---

## 测试步骤

### 测试 1: 基础功能（2 分钟）✅

1. **导入书籍**
   - 点击 "导入小说" 按钮
   - 选择一个 TXT 文件
   - 等待解析完成
   - 验证：✅ 书籍出现在列表

2. **查看章节**
   - 点击书籍卡片
   - 验证：✅ 章节列表显示正确
   - 点击第一章
   - 验证：✅ 内容正常显示

---

### 测试 2: AI 对话（3 分钟）✅

1. **打开 AI 面板**
   - 点击右上角 "🤖 打开AI" 按钮
   - 验证：✅ 面板打开，显示 "在线" 徽章

2. **发送消息**
   - 输入："你好，请介绍一下你自己"
   - 按 Enter 键
   - 验证：
     - ✅ 立即显示打字指示器
     - ✅ 1 秒内看到第一个字
     - ✅ 文字逐渐出现（流式效果）
     - ✅ 光标动画（▊）闪烁
     - ✅ 完成后光标消失

3. **多轮对话**
   - 输入："1+1等于几？"
   - 等待回复
   - 输入："那 2+2 呢？"
   - 验证：✅ AI 能理解上下文

---

### 测试 3: 智能摘要（2 分钟）✅

**注意**: 当前摘要功能已实现，但 UI 入口待添加。使用浏览器控制台测试。

1. **打开浏览器控制台**
   - 按 F12 或 Cmd+Option+I (Mac)

2. **测试摘要**
   ```javascript
   // 获取工作空间路径（根据实际情况修改）
   const workspace = '/Users/shichang/Workspace/projects/ai-powered/nothingbut-library';

   // 导入 API
   const { summarizeChapter } = await import('$lib/services/ai');

   // 生成短摘要
   const summary = await summarizeChapter(workspace, 1, 'short');
   console.log('摘要:', summary);
   ```

3. **验证结果**
   - ✅ 2-5 秒内返回摘要
   - ✅ 摘要长度约 100-200 字
   - ✅ 内容相关且准确

4. **测试缓存**
   ```javascript
   // 再次生成相同摘要
   console.time('cached');
   const cached = await summarizeChapter(workspace, 1, 'short');
   console.timeEnd('cached');
   // 预期: < 100ms（命中缓存）
   ```

---

### 测试 4: 向量搜索（5 分钟）✅

#### 4.1 建立索引

```javascript
// 浏览器控制台
const { indexBook } = await import('$lib/services/ai');
const workspace = '/Users/shichang/Workspace/projects/ai-powered/nothingbut-library';

console.log('开始索引...');
console.time('indexing');
const indexed = await indexBook(workspace, 1);
console.timeEnd('indexing');
console.log('已索引章节:', indexed);
```

**验证**:
- ✅ 显示 "开始索引..."
- ✅ 每章约 2-5 秒
- ✅ 完成后显示索引的章节 ID 列表

#### 4.2 语义搜索

```javascript
const { semanticSearch } = await import('$lib/services/ai');

// 测试 1: 基础搜索
const results1 = await semanticSearch('主角获得力量', 1, 5, 0.6);
console.log('搜索结果:');
results1.forEach((r, i) => {
  console.log(`${i+1}. ${r.chapter_title} (${(r.similarity*100).toFixed(0)}%)`);
  console.log(`   预览: ${r.preview.substring(0, 50)}...`);
});
```

**验证**:
- ✅ < 1 秒返回结果
- ✅ 章节按相似度排序
- ✅ 预览内容正确

```javascript
// 测试 2: 不同查询
await semanticSearch('主角遇到危险', 1, 5, 0.6);
await semanticSearch('感人的场景', 1, 5, 0.6);
```

#### 4.3 增量索引

```javascript
// 重复索引（应该跳过）
console.time('skip');
await indexBook(workspace, 1);
console.timeEnd('skip');
// 预期: 明显快于首次（跳过未变化的章节）
```

---

### 测试 5: 并发控制（2 分钟）✅

```javascript
// 同时发起 4 个请求
const { semanticSearch } = await import('$lib/services/ai');

console.log('发起 4 个并发搜索...');
const promises = [
  semanticSearch('战斗', 1, 5, 0.6),
  semanticSearch('修炼', 1, 5, 0.6),
  semanticSearch('离别', 1, 5, 0.6),
  semanticSearch('相遇', 1, 5, 0.6),
];

console.time('concurrent');
await Promise.all(promises);
console.timeEnd('concurrent');
console.log('✅ 所有搜索完成');
```

**验证**:
- ✅ 前 3 个立即开始
- ✅ 第 4 个等待前面完成
- ✅ 所有请求最终完成
- ✅ 应用无卡顿

---

## 测试结果记录

### ✅ 通过的测试
- [ ] 基础功能（导入、查看）
- [ ] AI 对话（流式响应）
- [ ] 智能摘要（生成、缓存）
- [ ] 向量搜索（索引、搜索）
- [ ] 并发控制（3 个限制）

### ❌ 发现的问题
1.
2.
3.

### 💡 改进建议
1.
2.
3.

---

## 常见问题

### Q: 显示 "服务离线"
```bash
# 检查 Ollama 是否运行
curl http://localhost:11434

# 如果失败，启动服务
ollama serve
```

### Q: 搜索返回空结果
```javascript
// 检查是否已索引
const pool = /* 获取数据库连接 */;
const result = await sqlx.query('SELECT COUNT(*) FROM chapter_embeddings');
console.log('已索引章节数:', result);

// 如果为 0，重新索引
await indexBook(workspace, 1);
```

### Q: 首次响应很慢
正常现象。首次请求需要加载模型（约 10 秒），之后会很快。

---

## 完整测试时间

- 准备工作: 5 分钟
- 测试 1: 2 分钟
- 测试 2: 3 分钟
- 测试 3: 2 分钟
- 测试 4: 5 分钟
- 测试 5: 2 分钟

**总计**: ~15 分钟（不含索引时间）

---

**创建时间**: 2026-03-13
**适用版本**: Tasks 1-13 完成后
