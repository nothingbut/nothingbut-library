# 向量搜索功能测试指南

## 前置条件

1. **启动 Ollama 服务**
   ```bash
   ollama serve
   ```

2. **下载嵌入模型**
   ```bash
   ollama pull nomic-embed-text
   ```

3. **导入测试数据**
   - 至少导入 1 本书（包含多个章节）

---

## 测试步骤

### 测试 1: 索引单个章节

#### 使用浏览器控制台

```javascript
// 1. 打开浏览器控制台 (F12)
// 2. 导入 API
const { indexChapter } = await import('$lib/services/ai');

// 3. 获取工作空间路径（根据实际情况修改）
const workspacePath = '/path/to/your/workspace';

// 4. 索引第 1 章
console.time('index_chapter');
await indexChapter(workspacePath, 1);
console.timeEnd('index_chapter');
// 预期: 2-5 秒完成

console.log('✅ 章节 1 已成功索引');
```

#### 预期结果
- ✅ 控制台无错误
- ✅ 耗时 2-5 秒
- ✅ 数据库中创建了 `chapter_embeddings` 记录

#### 验证方法
查看数据库（使用 DB Browser for SQLite）:
```sql
SELECT * FROM chapter_embeddings WHERE chapter_id = 1;
```
应看到一条记录，`embedding` 字段有 3072 字节。

---

### 测试 2: 批量索引整本书

```javascript
const { indexBook } = await import('$lib/services/ai');

const workspacePath = '/path/to/your/workspace';
const bookId = 1; // 第一本书的 ID

console.log('开始批量索引...');
console.time('index_book');
const indexed = await indexBook(workspacePath, bookId);
console.timeEnd('index_book');

console.log(`✅ 成功索引 ${indexed.length} 个章节`);
console.log('章节 ID:', indexed);
```

#### 预期结果
- ✅ 所有章节均成功索引
- ✅ 返回章节 ID 数组
- ✅ 每章节约 2-5 秒
- ✅ 重复索引时跳过未变化的章节（增量索引）

#### 性能基准
| 章节数 | 预期时间 |
|--------|----------|
| 10 章 | 20-50 秒 |
| 50 章 | 2-4 分钟 |
| 100 章 | 3-8 分钟 |

---

### 测试 3: 增量索引（内容哈希）

```javascript
const { indexChapter } = await import('$lib/services/ai');

const workspacePath = '/path/to/your/workspace';
const chapterId = 1;

// 第一次索引
console.time('first_index');
await indexChapter(workspacePath, chapterId);
console.timeEnd('first_index');
// 预期: 2-5 秒

// 第二次索引（内容未变）
console.time('second_index');
await indexChapter(workspacePath, chapterId);
console.timeEnd('second_index');
// 预期: < 100ms（跳过）

console.log('✅ 增量索引生效，跳过未变化的章节');
```

#### 预期结果
- ✅ 第一次索引正常执行
- ✅ 第二次索引立即返回（内容未变）
- ✅ 节省计算资源

---

### 测试 4: 语义搜索

#### 4.1 基础搜索

```javascript
const { semanticSearch } = await import('$lib/services/ai');

// 搜索：主角获得力量的章节
const results = await semanticSearch(
  '主角获得神秘力量',
  1,      // 限制在第 1 本书
  10,     // 返回前 10 个结果
  0.6     // 最小相似度 0.6
);

console.log(`找到 ${results.length} 个相关章节:`);
results.forEach((r, i) => {
  console.log(`\n${i + 1}. ${r.chapter_title} (第 ${r.chapter_number} 章)`);
  console.log(`   相似度: ${(r.similarity * 100).toFixed(1)}%`);
  console.log(`   预览: ${r.preview.substring(0, 50)}...`);
});
```

#### 预期结果
- ✅ 返回相关章节
- ✅ 相似度从高到低排序
- ✅ 相似度 > 0.6
- ✅ 搜索时间 < 1 秒

#### 4.2 不同查询测试

| 查询 | 预期结果 |
|------|----------|
| "主角遇到危险" | 包含战斗、冒险的章节 |
| "感人的离别场景" | 包含情感、分离的章节 |
| "反派角色出现" | 反派登场或活动的章节 |
| "主角实力提升" | 修炼、突破的章节 |
| "初次相遇" | 角色首次见面的章节 |

#### 4.3 跨书搜索

```javascript
// 不限制 book_id，搜索所有书籍
const results = await semanticSearch('主角获得力量', undefined, 10, 0.6);

// 显示每个结果来自哪本书
results.forEach(r => {
  console.log(`《${r.book_title}》 - ${r.chapter_title} (${(r.similarity * 100).toFixed(1)}%)`);
});
```

---

### 测试 5: 边界情况

#### 5.1 查询为空

```javascript
try {
  await semanticSearch('', 1, 10, 0.6);
} catch (e) {
  console.log('✅ 正确处理空查询:', e.message);
}
```

#### 5.2 无相关结果

```javascript
// 使用不太可能匹配的查询
const results = await semanticSearch('火星上的外星生物', 1, 10, 0.9);
console.log(`找到 ${results.length} 个结果`);
// 预期: 0 或很少结果
```

#### 5.3 相似度阈值

```javascript
// 低阈值（0.3）
const lowResults = await semanticSearch('主角', 1, 10, 0.3);
console.log(`低阈值结果: ${lowResults.length} 个`);

// 高阈值（0.9）
const highResults = await semanticSearch('主角', 1, 10, 0.9);
console.log(`高阈值结果: ${highResults.length} 个`);

// 预期: 低阈值返回更多结果
```

---

### 测试 6: UI 组件测试

#### 集成 SemanticSearch 组件

1. 在页面中添加组件：
   ```svelte
   <script>
     import SemanticSearch from '$lib/components/ai/SemanticSearch.svelte';
     let workspacePath = '/path/to/workspace';
     let currentBookId = 1;
   </script>

   <SemanticSearch bookId={currentBookId} {workspacePath} />
   ```

2. 测试交互：
   - ✅ 输入查询文本
   - ✅ 按 Enter 键搜索
   - ✅ 点击搜索按钮
   - ✅ 查看加载状态
   - ✅ 查看搜索结果
   - ✅ 点击"跳转阅读"

3. 视觉检查：
   - ✅ 相似度显示正确
   - ✅ 章节预览完整
   - ✅ 空状态友好
   - ✅ 错误提示清晰

---

### 测试 7: 性能测试

#### 7.1 搜索性能

```javascript
// 测量多次搜索的平均时间
async function benchmarkSearch(query, iterations = 5) {
  const times = [];
  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await semanticSearch(query, 1, 10, 0.6);
    const end = performance.now();
    times.push(end - start);
  }

  const avg = times.reduce((a, b) => a + b) / times.length;
  console.log(`平均搜索时间: ${avg.toFixed(2)} ms`);
  console.log(`最快: ${Math.min(...times).toFixed(2)} ms`);
  console.log(`最慢: ${Math.max(...times).toFixed(2)} ms`);
}

await benchmarkSearch('主角获得力量');
// 目标: 平均 < 1000ms
```

#### 7.2 索引性能

```javascript
// 测量索引性能
async function benchmarkIndexing(chapterIds) {
  const start = performance.now();

  for (const id of chapterIds) {
    await indexChapter('/path/to/workspace', id);
  }

  const end = performance.now();
  const total = (end - start) / 1000;
  const perChapter = total / chapterIds.length;

  console.log(`总时间: ${total.toFixed(2)} 秒`);
  console.log(`平均每章: ${perChapter.toFixed(2)} 秒`);
}

await benchmarkIndexing([1, 2, 3, 4, 5]);
// 目标: 每章 < 5 秒
```

---

## 常见问题

### Q1: 索引很慢，一直没响应
**原因**: Ollama 服务未启动或模型未下载
**解决**:
```bash
ollama serve
ollama pull nomic-embed-text
```

### Q2: 搜索结果不准确
**原因**:
1. 索引的章节太少
2. 查询文本太短或太抽象
3. 相似度阈值设置不当

**解决**:
- 索引更多章节（建议至少 10 章）
- 使用更具体的查询（"主角第一次战斗"而非"战斗"）
- 降低相似度阈值（0.5-0.6）

### Q3: 重复索引没有跳过
**原因**: 内容哈希计算问题
**解决**: 检查章节内容是否真的未变化

### Q4: 搜索返回空结果
**原因**:
1. 没有索引章节
2. 相似度阈值太高
3. 查询与内容不匹配

**解决**:
- 确认章节已索引：`SELECT COUNT(*) FROM chapter_embeddings;`
- 降低相似度阈值到 0.5
- 尝试不同的查询

### Q5: 向量维度错误
**原因**: 模型不匹配
**解决**: 确保使用 `nomic-embed-text` 模型（768 维）

---

## 性能基准参考

| 指标 | 目标 | 实际 |
|------|------|------|
| 单章节索引 | < 5秒 | _待测_ |
| 批量索引（100章） | < 10分钟 | _待测_ |
| 语义搜索（100章） | < 1秒 | _待测_ |
| 重复索引（命中缓存） | < 100ms | _待测_ |
| 内存占用（100章向量） | < 500KB | _待测_ |

---

## 测试报告模板

```markdown
## 测试环境
- OS: macOS 14.x / Windows 11 / Linux
- Ollama: vX.X.X
- 嵌入模型: nomic-embed-text
- 测试数据: X 本书，共 Y 章

## 测试结果
- [ ] 单章节索引
- [ ] 批量索引
- [ ] 增量索引
- [ ] 语义搜索
- [ ] 边界情况
- [ ] UI 组件
- [ ] 性能测试

## 性能数据
| 操作 | 耗时 | 状态 |
|------|------|------|
| 单章节索引 | _X_ 秒 | ✅/❌ |
| 搜索（10章） | _X_ ms | ✅/❌ |
| 搜索（100章） | _X_ ms | ✅/❌ |

## 发现的问题
1. [描述问题]
2. [描述问题]

## 改进建议
1. [改进建议]
2. [改进建议]
```

---

**创建时间**: 2026-03-13
**适用版本**: Task 8 完成后
