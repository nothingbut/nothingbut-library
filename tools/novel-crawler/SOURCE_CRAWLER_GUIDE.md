# 来源网站爬虫 - 快速使用指南

**Phase 4 完成** - 2026-03-15

---

## 🎉 新功能

现在可以爬取起点中文网和纵横中文网的详细书籍信息！

### 支持的网站

- ✅ 起点中文网 (qidian)
- ✅ 纵横中文网 (zongheng)
- 🔄 更多网站待添加...

---

## 🚀 快速开始

### 1. 查看来源站点统计

```bash
cd tools/novel-crawler
python main.py source-stats
```

输出示例：
```
Source Site Statistics:
  QIDIAN:
    Total Books:     0
    Books with Covers: 0
    Average Rating:  0.0
    Total Words:     0

  ZONGHENG:
    Total Books:     0
    Books with Covers: 0
    Average Rating:  0.0
    Total Words:     0
```

### 2. 爬取所有书籍的来源详情

```bash
# 爬取优书数据库中所有书籍的来源网站详情
python main.py source --mode all

# 限制爬取前100本
python main.py source --mode all --limit 100
```

### 3. 爬取指定书籍

```bash
# 爬取指定ID的书籍
python main.py source --mode batch --ids 1,2,3,4,5

# 逗号分隔多个ID
python main.py source --mode batch --ids 10,20,30
```

---

## 📊 工作流程

```
优书数据库 (youshu.db)
    ↓
提取来源URL
    ↓
┌───────────────┐
│  起点/纵横     │
│  详细信息     │
└───────────────┘
    ↓
保存到独立数据库
    ↓
qidian.db / zongheng.db
```

---

## 📁 数据库结构

### 主数据库 (youshu.db)
- 来自优书网的基本信息
- 书籍索引
- 封面图片

### 来源数据库 (qidian.db, zongheng.db)
- 详细的书籍信息
- 字数、章节数
- 评分、人气数据
- 完整的分类标签

---

## 🔍 使用场景

### 场景 1: 完整爬取流程

```bash
# 1. 先爬取优书网索引
python main.py crawl --mode initial --start 1

# 2. 再爬取来源网站详情
python main.py source --mode all

# 3. 查看统计
python main.py stats           # 优书统计
python main.py source-stats    # 来源统计
```

### 场景 2: 增量更新

```bash
# 1. 优书网增量更新
python main.py crawl --mode incremental

# 2. 来源网站增量更新
python main.py source --mode batch --ids 100,101,102
```

### 场景 3: 单本书籍测试

```bash
# 测试单本书籍的来源爬取
python main.py source --mode batch --ids 1
```

---

## 📖 数据字段

### 起点/纵横详情包含

| 字段 | 说明 | 示例 |
|------|------|------|
| title | 书名 | "诡秘之主" |
| author | 作者 | "爱潜水的乌贼" |
| description | 简介 | "蒸汽与机械..." |
| category | 主分类 | "玄幻" |
| sub_category | 子分类 | "西方玄幻" |
| tags | 标签列表 | ["克苏鲁", "蒸汽朋克"] |
| cover_url | 封面URL | "https://..." |
| word_count | 总字数 | 3250000 |
| chapter_count | 章节数 | 1443 |
| status | 状态 | "连载" / "完结" |
| rating | 评分 | 9.5 |
| view_count | 点击量 | 10000000 |
| favorite_count | 收藏量 | 500000 |

---

## 🛠️ Python API

```python
from crawlers.source_crawler_manager import SourceCrawlerManager

with SourceCrawlerManager() as manager:
    # 单本爬取
    result = manager.crawl_source_for_book(1)
    if result['success']:
        print(f"✓ {result['title']}")

    # 批量爬取
    stats = manager.crawl_batch([1, 2, 3, 4, 5])
    print(f"成功: {stats['success']}/{stats['total']}")

    # 爬取所有（限制数量）
    stats = manager.crawl_all_books(limit=100)
    print(f"成功: {stats['success']}/{stats['total']}")

    # 查看统计
    all_stats = manager.get_source_statistics()
    for site, stats in all_stats.items():
        print(f"{site}: {stats['total_books']}本书")
```

---

## ⚙️ 配置

### 请求延迟
- 默认: 1-3秒随机延迟
- 配置: `config/settings.py` → `REQUEST_DELAY`

### 重试次数
- 默认: 最多重试3次
- 配置: `config/settings.py` → `MAX_RETRIES`

### 超时设置
- 默认: 10秒超时
- 配置: `config/settings.py` → `REQUEST_TIMEOUT`

---

## 🧪 测试

```bash
# 运行单元测试
pytest tests/test_source_crawlers.py -v

# 运行集成测试
pytest tests/test_source_integration.py -v

# 运行所有测试
pytest tests/ -v
```

---

## 📝 添加新网站

### 步骤

1. 创建新爬虫类
```python
# crawlers/source_crawlers/newsite_crawler.py
from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler

class NewSiteCrawler(BaseSourceCrawler):
    def get_site_name(self):
        return 'newsite'

    def get_base_url(self):
        return "https://newsite.com/book/{book_id}"

    def build_book_url(self, book_id):
        return f'https://newsite.com/book/{book_id}'

    def parse_book_detail(self, html, book_id):
        # 实现解析逻辑
        pass
```

2. 注册爬虫
```python
# crawlers/source_crawlers/__init__.py
from crawlers.source_crawlers.newsite_crawler import NewSiteCrawler

CRAWLER_MAP = {
    'qidian': QidianCrawler,
    'zongheng': ZonghengCrawler,
    'newsite': NewSiteCrawler,  # 添加这一行
}
```

3. 更新配置
```python
# config/settings.py
SITE_CONFIGS = {
    # ...
    'newsite': {
        'name': '新网站',
        'base_url': 'https://newsite.com/book/{book_id}',
        'enabled': True
    }
}
```

4. 测试
```bash
python main.py source --mode batch --ids 1
```

---

## ⚠️ 注意事项

1. **爬取频率**: 建议控制爬取频率，避免对目标网站造成压力
2. **数据准确性**: 来源网站可能更新页面结构，需定期验证选择器
3. **存储空间**: 每个来源网站独立数据库，注意磁盘空间
4. **错误处理**: 部分书籍可能没有来源URL或来源不支持

---

## 🎯 下一步

Phase 5 将实现任务调度系统，支持：
- 🕐 定时自动更新
- 🔄 失败自动重试
- 📊 定期统计报告

---

**生成时间**: 2026-03-15
**版本**: Phase 4 完成
**状态**: 可用 ✅
