# 爬虫项目交接文档

**创建日期**: 2026-03-15
**项目**: 小说元数据爬虫系统
**状态**: 文档完成，待另一台电脑实施

---

## 快速开始（在另一台电脑）

### 1. 拉取最新代码

```bash
cd /path/to/nothingbut-library
git pull origin main
```

### 2. 查看爬虫文档

文档位置：`docs/crawler/`

```bash
cd docs/crawler
ls -l
# REQUIREMENTS.md      - 需求文档
# DESIGN.md            - 设计方案
# DEVELOPMENT_PLAN.md  - 开发计划
# HANDOFF.md           - 本文档
```

### 3. 阅读顺序

**推荐阅读顺序**：
1. `REQUIREMENTS.md` - 了解项目需求和目标（15 分钟）
2. `DESIGN.md` - 了解技术架构和实现方案（30 分钟）
3. `DEVELOPMENT_PLAN.md` - 按照开发计划逐步实施（7 天）

---

## 项目概述

### 目标
构建自动化小说元数据采集系统，用于预填充 NothingBut Library 应用。

### 数据源
- **主索引**: www.youshu.me（优书网）
- **来源站点**: 起点中文网、纵横中文网等

### 核心功能
1. 爬取优书网书籍索引（书名、作者、简介、标签、封面等）
2. 下载封面图片到本地
3. 爬取来源网站详细信息
4. 定时增量更新
5. 失败自动重试

---

## 技术栈

```
语言: Python 3.8+
HTTP: requests
解析: BeautifulSoup4
数据库: SQLite
调度: APScheduler
```

---

## 项目结构（待创建）

```
novel-crawler/                    # 新项目（与 nothingbut-library 平级）
├── config/                       # 配置
│   ├── settings.py
│   └── site_configs/
├── crawlers/                     # 爬虫
│   ├── base_crawler.py
│   ├── youshu_crawler.py
│   └── source_crawlers/
│       ├── qidian.py
│       └── zongheng.py
├── database/                     # 数据库
│   ├── db_manager.py
│   └── migrations/
├── utils/                        # 工具
│   ├── http_client.py
│   ├── image_downloader.py
│   └── retry_handler.py
├── scheduler/                    # 调度
│   └── daily_task.py
├── tests/                        # 测试
├── data/                         # 数据（生成）
│   ├── youshu.db
│   ├── qidian.db
│   └── covers/
├── logs/                         # 日志（生成）
├── main.py                       # 入口
├── requirements.txt              # 依赖
└── README.md                     # 说明
```

---

## 开发流程

### Phase 1: 项目初始化（Day 1 上午）
```bash
# 创建新项目
mkdir novel-crawler
cd novel-crawler

# 初始化
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate
git init

# 创建目录结构
mkdir -p config crawlers/source_crawlers database/migrations \
         utils scheduler tests data/covers logs docs

# 安装依赖
pip install requests beautifulsoup4 lxml APScheduler python-dotenv pydantic pytest pytest-cov
pip freeze > requirements.txt
```

### Phase 2-7: 按开发计划实施
参考 `DEVELOPMENT_PLAN.md` 中的详细步骤。

---

## 关键文件示例

### requirements.txt
```txt
requests==2.31.0
beautifulsoup4==4.12.3
lxml==4.9.3
APScheduler==3.10.4
python-dotenv==1.0.0
pydantic==2.5.0
pytest==7.4.3
pytest-cov==4.1.0
```

### config/settings.py（模板）
```python
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = BASE_DIR / 'data'
COVER_DIR = DATA_DIR / 'covers'

# 爬虫配置
YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
REQUEST_TIMEOUT = 10
REQUEST_DELAY = (1, 3)
MAX_RETRIES = 3
MAX_CONSECUTIVE_FAILURES = 50

# 数据库
YOUSHU_DB = DATA_DIR / 'youshu.db'
```

---

## 使用命令

### 开发阶段
```bash
# 首次全量爬取
python main.py crawl --mode initial --start 1

# 测试单本
python main.py crawl --book-id 123

# 运行测试
pytest tests/ -v --cov=.
```

### 生产阶段
```bash
# 增量更新
python main.py crawl --mode incremental

# 启动定时任务
python main.py schedule --start

# 查看统计
python main.py stats
```

---

## 注意事项

### 网络限制
- 当前电脑无法访问目标网站
- 需要在另一台可访问的电脑开发

### 合规性
- **仅爬取元数据**，不爬取小说正文
- **遵守 robots.txt**
- **控制请求频率**（1-3 秒延迟）
- **使用真实 User-Agent**

### 反爬虫
- 随机延迟
- User-Agent 轮换
- 连续失败 50 次停止
- 预留代理池接口

---

## 开发建议

### 第一步：环境准备
1. 确认网络可访问目标网站
2. 安装 Python 3.8+
3. 创建项目目录
4. 配置虚拟环境

### 第二步：快速验证
```python
# test_connection.py
import requests

url = "https://www.youshu.me/book/1"
response = requests.get(url)
print(f"Status: {response.status_code}")
print(f"Content: {response.text[:200]}")
```

### 第三步：按计划开发
- 严格按照 `DEVELOPMENT_PLAN.md` 的 7 个阶段
- 每个阶段完成后提交代码
- 遇到问题参考 `DESIGN.md` 的详细设计

---

## 测试策略

### 单元测试
- HTTPClient 测试
- DatabaseManager 测试
- 解析器测试

### 集成测试
- 完整爬取流程
- 增量更新
- 失败重试

### 手动测试
```bash
# 测试爬取 1-10
python main.py crawl --mode initial --start 1

# 验证数据库
sqlite3 data/youshu.db "SELECT * FROM books LIMIT 10"

# 检查封面
ls -lh data/covers/
```

---

## 预期输出

### 数据库
- `youshu.db`: 主索引（10,000+ 本）
- `qidian.db`: 起点详情
- `zongheng.db`: 纵横详情

### 文件
- `data/covers/*.jpg`: 封面图片（~500MB）
- `logs/crawler.log`: 运行日志

### 统计
- 爬取速度: 1000-2000 本/小时
- 成功率: > 95%
- 数据完整性: > 98%

---

## 故障排查

### 网络错误
```python
# 检查网络连接
curl https://www.youshu.me/book/1
```

### 数据库错误
```bash
# 检查数据库
sqlite3 data/youshu.db ".tables"
sqlite3 data/youshu.db ".schema books"
```

### 解析错误
```python
# 测试解析器
from crawlers.youshu_crawler import YoushuCrawler
from config import settings
from utils.http_client import HTTPClient

crawler = YoushuCrawler(settings, HTTPClient(settings))
book = crawler.crawl_book(1)
print(book)
```

---

## 联系方式

**遇到问题**:
1. 查看 `DESIGN.md` 的详细设计
2. 检查 `DEVELOPMENT_PLAN.md` 的代码示例
3. 参考 Python 爬虫最佳实践

**项目仓库**: https://github.com/nothingbut/nothingbut-library

---

## 提交到主项目

爬虫系统完成后，可以考虑：

### 选项 1: 独立项目
```bash
# 保持独立仓库
git remote add origin <crawler-repo-url>
git push origin main
```

### 选项 2: 作为子模块
```bash
# 在 nothingbut-library 中
git submodule add <crawler-repo-url> tools/novel-crawler
```

### 选项 3: 集成工具
```bash
# 将爬虫数据导入主应用
novel-crawler/data/youshu.db -> nothingbut-library/metadata.db
```

---

## 下一步行动

1. ✅ 在另一台电脑拉取代码
2. ✅ 阅读三个核心文档
3. ✅ 创建 `novel-crawler` 项目
4. ✅ 按开发计划实施（7 天）
5. ✅ 测试和优化
6. ✅ 部署到生产环境

---

**祝开发顺利！**

如有疑问，参考文档或查阅相关技术资料。

---

**文档创建**: 2026-03-15
**最后更新**: 2026-03-15
**状态**: 待实施
