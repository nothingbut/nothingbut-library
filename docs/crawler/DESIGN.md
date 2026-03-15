# 小说元数据爬虫系统 - 设计方案

**版本**: v1.0
**创建日期**: 2026-03-15
**设计者**: Development Team

---

## 1. 系统架构

### 1.1 总体架构

```
┌─────────────────────────────────────────────────────────────┐
│                      Crawler Manager                         │
│  - 爬取协调                                                  │
│  - 状态管理                                                  │
│  - 任务调度                                                  │
└─────────────────┬───────────────────────────────────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
    ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│ Youshu  │  │ Qidian  │  │Zongheng │
│ Crawler │  │ Crawler │  │ Crawler │
└────┬────┘  └────┬────┘  └────┬────┘
     │            │             │
     └────────────┼─────────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
    ▼             ▼             ▼
┌─────────┐  ┌─────────┐  ┌─────────┐
│  HTTP   │  │  HTML   │  │  Image  │
│ Client  │  │ Parser  │  │Download │
└─────────┘  └─────────┘  └─────────┘
                  │
                  ▼
         ┌────────────────┐
         │ Database Layer │
         │  - youshu.db   │
         │  - qidian.db   │
         │  - zongheng.db │
         └────────────────┘
```

### 1.2 分层架构

#### Layer 1: 应用层 (Application Layer)
- **CrawlerManager**: 总控制器
- **Scheduler**: 任务调度器
- **CLI Interface**: 命令行界面

#### Layer 2: 业务层 (Business Layer)
- **YoushuCrawler**: 优书网爬虫
- **QidianCrawler**: 起点爬虫
- **ZonghengCrawler**: 纵横爬虫
- **BaseCrawler**: 抽象基类

#### Layer 3: 工具层 (Utility Layer)
- **HTTPClient**: HTTP 请求封装
- **HTMLParser**: HTML 解析器
- **ImageDownloader**: 图片下载器
- **RetryHandler**: 重试处理器

#### Layer 4: 数据层 (Data Layer)
- **DatabaseManager**: 数据库管理
- **DataModel**: 数据模型
- **Migration**: 数据库迁移

---

## 2. 模块设计

### 2.1 项目结构

```
novel-crawler/
├── config/                     # 配置模块
│   ├── __init__.py
│   ├── settings.py             # 全局配置
│   └── site_configs/           # 站点配置
│       ├── youshu.yaml
│       ├── qidian.yaml
│       └── zongheng.yaml
├── crawlers/                   # 爬虫模块
│   ├── __init__.py
│   ├── base_crawler.py         # 基础爬虫类
│   ├── youshu_crawler.py       # 优书网爬虫
│   └── source_crawlers/        # 来源网站爬虫
│       ├── __init__.py
│       ├── qidian.py
│       └── zongheng.py
├── database/                   # 数据库模块
│   ├── __init__.py
│   ├── db_manager.py           # 数据库管理
│   ├── models.py               # 数据模型
│   └── migrations/             # 迁移脚本
│       ├── 001_init_youshu.sql
│       └── 002_init_sources.sql
├── utils/                      # 工具模块
│   ├── __init__.py
│   ├── http_client.py          # HTTP 客户端
│   ├── parser.py               # HTML 解析器
│   ├── image_downloader.py     # 图片下载
│   ├── retry_handler.py        # 重试处理
│   └── logger.py               # 日志工具
├── scheduler/                  # 调度模块
│   ├── __init__.py
│   ├── daily_task.py           # 每日任务
│   └── job_manager.py          # 任务管理
├── tests/                      # 测试模块
│   ├── __init__.py
│   ├── test_crawlers.py
│   ├── test_database.py
│   └── test_utils.py
├── data/                       # 数据目录
│   ├── youshu.db
│   ├── qidian.db
│   ├── zongheng.db
│   └── covers/
├── logs/                       # 日志目录
│   ├── crawler.log
│   └── error.log
├── docs/                       # 文档目录
│   ├── REQUIREMENTS.md
│   ├── DESIGN.md
│   └── API.md
├── main.py                     # 主程序入口
├── requirements.txt            # 依赖列表
├── setup.py                    # 安装脚本
└── README.md                   # 项目说明
```

---

### 2.2 核心模块详细设计

#### 2.2.1 BaseCrawler (基础爬虫类)

**职责**: 提供通用爬虫功能

**接口**:
```python
class BaseCrawler(ABC):
    def __init__(self, config: Config)
    def fetch_page(self, url: str) -> Optional[Response]
    def parse_book_info(self, html: str) -> Optional[Dict]
    def crawl_book(self, book_id: int) -> Optional[Dict]
    def random_delay(self)
    def get_random_user_agent(self) -> str
```

**关键方法**:
- `fetch_page()`: 获取网页，带重试和错误处理
- `parse_book_info()`: 抽象方法，子类实现具体解析逻辑
- `random_delay()`: 随机延迟，避免请求过快

**设计模式**: 模板方法模式

---

#### 2.2.2 YoushuCrawler (优书网爬虫)

**职责**: 爬取优书网索引数据

**核心流程**:
```python
def crawl_book(self, book_id: int) -> Optional[Dict]:
    1. 构建 URL: f"https://www.youshu.me/book/{book_id}"
    2. 获取网页: fetch_page()
    3. 解析数据: parse_book_info()
    4. 下载封面: download_cover()
    5. 保存数据库: save_to_db()
    6. 随机延迟: random_delay()
    7. 返回结果
```

**解析策略**:
```python
def parse_book_info(self, html: str) -> Dict:
    soup = BeautifulSoup(html, 'html.parser')

    return {
        'title': soup.select_one('.book-title').text,
        'author': soup.select_one('.author').text,
        'description': soup.select_one('.description').text,
        'tags': [tag.text for tag in soup.select('.tag')],
        'cover_url': soup.select_one('.cover img')['src'],
        'source_site': extract_source_site(soup),
        'source_url': extract_source_url(soup),
        'update_status': soup.select_one('.status').text
    }
```

**容错处理**:
- 缺失字段: 返回 None 而非抛出异常
- 解析失败: 记录日志，返回 None
- 网络错误: 自动重试 3 次

---

#### 2.2.3 SourceCrawlers (来源网站爬虫)

**QidianCrawler 设计**:

```python
class QidianCrawler(BaseCrawler):
    BASE_URL = "https://book.qidian.com/info/{book_id}"

    def parse_book_info(self, html: str) -> Dict:
        soup = BeautifulSoup(html, 'html.parser')

        return {
            'title': soup.select_one('.book-info h1').text,
            'author': soup.select_one('.writer').text,
            'category': soup.select_one('.category').text,
            'word_count': self.parse_number(
                soup.select_one('.word-count').text
            ),
            'chapter_count': self.parse_number(
                soup.select_one('.chapter-count').text
            ),
            'rating': float(soup.select_one('.rating').text),
            'status': soup.select_one('.status').text,
            # ... 其他字段
        }

    def parse_number(self, text: str) -> int:
        """解析数字：'12.3万' -> 123000"""
        if '万' in text:
            return int(float(text.replace('万', '')) * 10000)
        return int(text)
```

**扩展性设计**:
- 每个站点独立的解析器
- 统一的接口规范
- 配置文件驱动（YAML）

---

#### 2.2.4 DatabaseManager (数据库管理)

**职责**: 统一的数据库操作接口

**接口设计**:
```python
class DatabaseManager:
    def __init__(self, config: Config)

    # 书籍操作
    def save_book(self, book_info: Dict) -> bool
    def get_book(self, book_id: int) -> Optional[Dict]
    def search_books(self, query: str) -> List[Dict]
    def update_book(self, book_id: int, updates: Dict) -> bool

    # 状态管理
    def get_last_valid_id(self) -> int
    def update_crawl_status(self, last_id: int, total: int, failed: List)
    def get_failed_ids(self) -> List[int]

    # 来源网站数据
    def save_source_book(self, source: str, book_info: Dict) -> bool
    def get_source_book(self, source: str, book_id: int) -> Optional[Dict]
```

**事务处理**:
```python
def save_book_with_transaction(self, book_info: Dict) -> bool:
    conn = sqlite3.connect(self.db_path)
    try:
        cursor = conn.cursor()
        cursor.execute("BEGIN TRANSACTION")

        # 保存主表
        cursor.execute("INSERT INTO books (...) VALUES (...)", data)

        # 更新状态表
        cursor.execute("UPDATE crawl_status SET ...")

        conn.commit()
        return True
    except Exception as e:
        conn.rollback()
        logger.error(f"Transaction failed: {e}")
        return False
    finally:
        conn.close()
```

**连接池**:
```python
class ConnectionPool:
    def __init__(self, db_path: str, pool_size: int = 5):
        self.pool = queue.Queue(maxsize=pool_size)
        for _ in range(pool_size):
            self.pool.put(sqlite3.connect(db_path, check_same_thread=False))

    def get_connection(self) -> sqlite3.Connection:
        return self.pool.get(timeout=5)

    def return_connection(self, conn: sqlite3.Connection):
        self.pool.put(conn)
```

---

#### 2.2.5 HTTPClient (HTTP 客户端)

**职责**: 封装 HTTP 请求，提供重试、代理、限流等功能

**设计**:
```python
class HTTPClient:
    def __init__(self, config: Config):
        self.session = requests.Session()
        self.retry_strategy = Retry(
            total=3,
            backoff_factor=1,
            status_forcelist=[429, 500, 502, 503, 504]
        )
        self.adapter = HTTPAdapter(max_retries=self.retry_strategy)
        self.session.mount("http://", self.adapter)
        self.session.mount("https://", self.adapter)

    def get(self, url: str, **kwargs) -> Optional[Response]:
        try:
            headers = {
                'User-Agent': self.get_random_ua(),
                'Accept': 'text/html,application/xhtml+xml',
                'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
            }
            response = self.session.get(
                url,
                headers=headers,
                timeout=10,
                **kwargs
            )
            response.raise_for_status()
            return response
        except requests.RequestException as e:
            logger.error(f"HTTP request failed: {url}, {e}")
            return None
```

**功能特性**:
- 自动重试（指数退避）
- User-Agent 轮换
- 超时控制
- 会话保持（Cookie）
- 代理支持（可选）

---

#### 2.2.6 RetryHandler (重试处理器)

**职责**: 统一的重试逻辑

**装饰器实现**:
```python
def retry_on_failure(max_attempts=3, backoff_factor=1):
    def decorator(func):
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            for attempt in range(max_attempts):
                try:
                    return func(*args, **kwargs)
                except Exception as e:
                    if attempt == max_attempts - 1:
                        logger.error(f"Failed after {max_attempts} attempts: {e}")
                        raise

                    wait_time = backoff_factor * (2 ** attempt)
                    logger.warning(f"Attempt {attempt + 1} failed, retrying in {wait_time}s...")
                    time.sleep(wait_time)
        return wrapper
    return decorator

# 使用示例
@retry_on_failure(max_attempts=3, backoff_factor=1)
def fetch_page(url: str):
    return requests.get(url)
```

---

#### 2.2.7 Scheduler (任务调度器)

**职责**: 管理定时任务

**实现方案 A: schedule 库**
```python
import schedule
import time

def daily_crawl_job():
    manager = CrawlerManager()
    manager.run_incremental_crawl()

# 每天凌晨 2:00 执行
schedule.every().day.at("02:00").do(daily_crawl_job)

# 每周日凌晨 3:00 补爬失败ID
schedule.every().sunday.at("03:00").do(retry_failed_job)

while True:
    schedule.run_pending()
    time.sleep(60)
```

**实现方案 B: APScheduler**
```python
from apscheduler.schedulers.blocking import BlockingScheduler
from apscheduler.triggers.cron import CronTrigger

scheduler = BlockingScheduler()

# 每天凌晨 2:00
scheduler.add_job(
    daily_crawl_job,
    trigger=CronTrigger(hour=2, minute=0),
    id='daily_crawl',
    name='Daily incremental crawl'
)

# 每周日凌晨 3:00
scheduler.add_job(
    retry_failed_job,
    trigger=CronTrigger(day_of_week='sun', hour=3, minute=0),
    id='weekly_retry',
    name='Weekly failed ID retry'
)

scheduler.start()
```

**推荐**: APScheduler（功能更强大，支持持久化）

---

## 3. 数据库设计

### 3.1 主索引库 (youshu.db)

#### books 表
```sql
CREATE TABLE books (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    tags TEXT,                  -- JSON array: ["玄幻", "热血"]
    cover_path TEXT,            -- 本地路径: data/covers/123.jpg
    source_site TEXT,           -- qidian, zongheng, etc.
    source_url TEXT,            -- 来源详情页链接
    update_status TEXT,         -- 连载中, 已完结
    crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_books_author ON books(author);
CREATE INDEX idx_books_source ON books(source_site);
CREATE INDEX idx_books_status ON books(update_status);
CREATE INDEX idx_books_title ON books(title);  -- 全文搜索
```

#### crawl_status 表
```sql
CREATE TABLE crawl_status (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    last_valid_id INTEGER NOT NULL,
    last_crawl_date DATETIME DEFAULT CURRENT_TIMESTAMP,
    total_books INTEGER DEFAULT 0,
    failed_ids TEXT,            -- JSON array: [10, 25, 38]
    crawl_type TEXT,            -- initial, incremental, retry
    duration_seconds INTEGER,   -- 爬取耗时
    success_count INTEGER,      -- 成功数量
    failure_count INTEGER       -- 失败数量
);
```

#### crawl_logs 表（可选，用于详细日志）
```sql
CREATE TABLE crawl_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    book_id INTEGER,
    status TEXT,                -- success, failure, skipped
    error_message TEXT,
    crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
```

---

### 3.2 来源站点库 (qidian.db, zongheng.db)

#### book_details 表
```sql
CREATE TABLE book_details (
    book_id INTEGER PRIMARY KEY,
    youshu_id INTEGER,          -- 关联优书网ID
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    category TEXT,              -- 主分类：玄幻、都市
    sub_category TEXT,          -- 子分类：东方玄幻
    tags TEXT,                  -- JSON array
    cover_url TEXT,             -- 原站图片链接
    cover_path TEXT,            -- 本地保存路径
    word_count INTEGER,         -- 字数
    chapter_count INTEGER,      -- 章节数
    status TEXT,                -- 连载/完结
    rating REAL,                -- 评分
    view_count INTEGER,         -- 浏览量
    favorite_count INTEGER,     -- 收藏数
    crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (youshu_id) REFERENCES books(id)
);

CREATE INDEX idx_details_youshu ON book_details(youshu_id);
CREATE INDEX idx_details_author ON book_details(author);
CREATE INDEX idx_details_category ON book_details(category);
```

---

### 3.3 数据模型 (Python)

```python
from dataclasses import dataclass, asdict
from typing import Optional, List
from datetime import datetime

@dataclass
class Book:
    id: int
    title: str
    author: Optional[str] = None
    description: Optional[str] = None
    tags: Optional[List[str]] = None
    cover_path: Optional[str] = None
    source_site: Optional[str] = None
    source_url: Optional[str] = None
    update_status: Optional[str] = None
    crawled_at: datetime = None
    updated_at: datetime = None

    def to_dict(self) -> dict:
        data = asdict(self)
        if self.tags:
            data['tags'] = json.dumps(self.tags)
        return data

    @classmethod
    def from_dict(cls, data: dict) -> 'Book':
        if data.get('tags') and isinstance(data['tags'], str):
            data['tags'] = json.loads(data['tags'])
        return cls(**data)

@dataclass
class SourceBookDetail:
    book_id: int
    youshu_id: Optional[int]
    title: str
    author: Optional[str] = None
    description: Optional[str] = None
    category: Optional[str] = None
    sub_category: Optional[str] = None
    tags: Optional[List[str]] = None
    cover_url: Optional[str] = None
    cover_path: Optional[str] = None
    word_count: Optional[int] = None
    chapter_count: Optional[int] = None
    status: Optional[str] = None
    rating: Optional[float] = None
    view_count: Optional[int] = None
    favorite_count: Optional[int] = None
    crawled_at: datetime = None
```

---

## 4. 算法设计

### 4.1 爬取停止算法

```python
def should_stop_crawling(consecutive_failures: int,
                         max_failures: int = 50) -> bool:
    """
    停止条件：连续失败次数达到阈值

    Args:
        consecutive_failures: 当前连续失败次数
        max_failures: 最大允许连续失败次数

    Returns:
        是否应该停止爬取
    """
    return consecutive_failures >= max_failures
```

**优化方案**（自适应阈值）:
```python
def adaptive_stop_condition(consecutive_failures: int,
                            total_crawled: int) -> bool:
    """
    自适应停止条件：
    - 前 1000 本：容忍 50 次失败
    - 1000-5000 本：容忍 100 次失败
    - 5000+ 本：容忍 200 次失败
    """
    if total_crawled < 1000:
        threshold = 50
    elif total_crawled < 5000:
        threshold = 100
    else:
        threshold = 200

    return consecutive_failures >= threshold
```

---

### 4.2 并发控制算法

```python
import asyncio
from asyncio import Semaphore

class ConcurrentCrawler:
    def __init__(self, max_concurrent: int = 3):
        self.semaphore = Semaphore(max_concurrent)

    async def crawl_book_async(self, book_id: int) -> Optional[Dict]:
        async with self.semaphore:
            # 获取许可后才能执行
            return await self.fetch_and_parse(book_id)

    async def crawl_range(self, start_id: int, end_id: int):
        tasks = [
            self.crawl_book_async(book_id)
            for book_id in range(start_id, end_id + 1)
        ]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        return [r for r in results if r is not None]
```

**同步版本（简化）**:
```python
from concurrent.futures import ThreadPoolExecutor
import threading

class ConcurrentCrawler:
    def __init__(self, max_workers: int = 3):
        self.executor = ThreadPoolExecutor(max_workers=max_workers)
        self.semaphore = threading.Semaphore(max_workers)

    def crawl_range(self, start_id: int, end_id: int):
        futures = []
        for book_id in range(start_id, end_id + 1):
            future = self.executor.submit(self.crawl_book, book_id)
            futures.append(future)

        results = [f.result() for f in futures if f.result()]
        return results
```

---

### 4.3 增量索引算法

```python
def incremental_crawl(db: DatabaseManager,
                     crawler: YoushuCrawler) -> dict:
    """
    增量爬取算法

    流程：
    1. 获取 last_valid_id
    2. 从 last_valid_id + 1 开始
    3. 爬取直到停止条件
    4. 更新状态
    """
    last_id = db.get_last_valid_id()
    start_id = last_id + 1

    current_id = start_id
    consecutive_failures = 0
    new_books = []

    while consecutive_failures < MAX_FAILURES:
        book_info = crawler.crawl_book(current_id)

        if book_info:
            db.save_book(book_info)
            new_books.append(book_info)
            consecutive_failures = 0
        else:
            consecutive_failures += 1

        current_id += 1

    db.update_crawl_status(
        last_id=current_id - 1,
        total=len(new_books),
        failed_ids=[]
    )

    return {
        'start_id': start_id,
        'end_id': current_id - 1,
        'new_books': len(new_books)
    }
```

---

### 4.4 失败重试算法

```python
def retry_failed_ids(db: DatabaseManager,
                    crawler: YoushuCrawler) -> dict:
    """
    补爬失败ID

    策略：
    1. 读取 failed_ids
    2. 按顺序重试
    3. 仍失败的保留
    4. 成功的从列表移除
    """
    failed_ids = db.get_failed_ids()
    still_failed = []
    recovered = []

    for book_id in failed_ids:
        book_info = crawler.crawl_book(book_id)

        if book_info:
            db.save_book(book_info)
            recovered.append(book_id)
        else:
            still_failed.append(book_id)

        time.sleep(random.uniform(2, 4))  # 更长延迟

    # 更新失败列表
    db.update_failed_ids(still_failed)

    return {
        'total': len(failed_ids),
        'recovered': len(recovered),
        'still_failed': len(still_failed)
    }
```

---

## 5. 接口设计

### 5.1 命令行接口 (CLI)

```bash
# 首次全量爬取
python main.py crawl --mode initial --start 1

# 增量爬取
python main.py crawl --mode incremental

# 补爬失败ID
python main.py retry --failed-ids 10,25,38

# 爬取指定范围
python main.py crawl --range 1000-2000

# 查询统计
python main.py stats

# 启动定时任务
python main.py schedule --start

# 导出数据
python main.py export --format json --output books.json
```

**实现**:
```python
import argparse

def main():
    parser = argparse.ArgumentParser(description='Novel Metadata Crawler')
    subparsers = parser.add_subparsers(dest='command')

    # crawl 子命令
    crawl_parser = subparsers.add_parser('crawl')
    crawl_parser.add_argument('--mode', choices=['initial', 'incremental'])
    crawl_parser.add_argument('--start', type=int, default=1)
    crawl_parser.add_argument('--range', type=str)

    # retry 子命令
    retry_parser = subparsers.add_parser('retry')
    retry_parser.add_argument('--failed-ids', type=str)

    # stats 子命令
    stats_parser = subparsers.add_parser('stats')

    # schedule 子命令
    schedule_parser = subparsers.add_parser('schedule')
    schedule_parser.add_argument('--start', action='store_true')

    args = parser.parse_args()

    # 路由到具体处理函数
    if args.command == 'crawl':
        handle_crawl(args)
    elif args.command == 'retry':
        handle_retry(args)
    # ...
```

---

### 5.2 Python API

```python
from novel_crawler import CrawlerManager, DatabaseManager

# 初始化
manager = CrawlerManager()
db = DatabaseManager()

# 爬取单本书
book = manager.crawl_single_book(book_id=123)

# 爬取范围
books = manager.crawl_range(start=1, end=100)

# 查询书籍
book = db.get_book(book_id=123)
books = db.search_books(query="斗破苍穹")

# 获取统计
stats = db.get_statistics()
# {
#   'total_books': 5000,
#   'last_crawl_date': '2026-03-15',
#   'failed_count': 25
# }
```

---

## 6. 配置设计

### 6.1 全局配置 (settings.py)

```python
import os
from pathlib import Path

# 项目路径
BASE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = BASE_DIR / 'data'
COVER_DIR = DATA_DIR / 'covers'
LOG_DIR = BASE_DIR / 'logs'

# 数据库配置
YOUSHU_DB = DATA_DIR / 'youshu.db'
SOURCE_DB_PATTERN = DATA_DIR / '{source}.db'

# 爬虫配置
REQUEST_TIMEOUT = 10
REQUEST_DELAY = (1, 3)  # 随机延迟范围(秒)
MAX_RETRIES = 3
MAX_CONCURRENT = 3
MAX_CONSECUTIVE_FAILURES = 50

# 调度配置
DAILY_CRAWL_TIME = "02:00"
RETRY_SCHEDULE = "sunday 03:00"

# 日志配置
LOG_LEVEL = 'INFO'
LOG_FORMAT = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
LOG_FILE = LOG_DIR / 'crawler.log'

# User-Agent 池
USER_AGENTS = [
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
    # ... 更多
]

# 网站配置
SITE_CONFIGS = {
    'youshu': {
        'base_url': 'https://www.youshu.me/book/{book_id}',
        'enabled': True
    },
    'qidian': {
        'base_url': 'https://book.qidian.com/info/{book_id}',
        'enabled': True
    },
    'zongheng': {
        'base_url': 'http://book.zongheng.com/book/{book_id}.html',
        'enabled': False  # 初期不启用
    }
}
```

---

### 6.2 站点配置 (YAML)

**youshu.yaml**:
```yaml
name: 优书网
base_url: https://www.youshu.me/book/{book_id}
enabled: true

selectors:
  title: .book-title
  author: .author-name
  description: .book-description
  tags: .tag-list .tag
  cover: .book-cover img
  source_site: .source-site-name
  source_url: .source-link
  update_status: .update-status

headers:
  User-Agent: Mozilla/5.0
  Accept: text/html,application/xhtml+xml
  Accept-Language: zh-CN,zh;q=0.9

retry:
  max_attempts: 3
  backoff_factor: 1
```

**qidian.yaml**:
```yaml
name: 起点中文网
base_url: https://book.qidian.com/info/{book_id}
enabled: true

selectors:
  title: .book-info h1
  author: .writer
  category: .category
  word_count: .word-count
  chapter_count: .chapter-count
  rating: .rating-score

transformers:
  word_count: parse_chinese_number  # "12.3万" -> 123000
  chapter_count: parse_integer
  rating: parse_float
```

---

## 7. 安全设计

### 7.1 反爬虫应对

**措施**:
1. **随机延迟**: 1-3 秒，模拟人类行为
2. **User-Agent 轮换**: 使用多个真实浏览器 UA
3. **Cookie 管理**: 保持会话状态
4. **请求头伪装**: 完整的浏览器请求头
5. **代理池**（可选）: 轮换 IP 地址

**示例**:
```python
def get_safe_headers(self) -> dict:
    return {
        'User-Agent': random.choice(USER_AGENTS),
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
        'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
        'Accept-Encoding': 'gzip, deflate, br',
        'Connection': 'keep-alive',
        'Upgrade-Insecure-Requests': '1',
        'Cache-Control': 'max-age=0',
        'Referer': self.get_referer()  # 伪造来源页
    }
```

---

### 7.2 错误处理

**异常层次**:
```python
class CrawlerException(Exception):
    """基础异常类"""
    pass

class NetworkException(CrawlerException):
    """网络相关异常"""
    pass

class ParseException(CrawlerException):
    """解析相关异常"""
    pass

class DatabaseException(CrawlerException):
    """数据库相关异常"""
    pass
```

**全局异常处理**:
```python
def safe_crawl(func):
    @functools.wraps(func)
    def wrapper(*args, **kwargs):
        try:
            return func(*args, **kwargs)
        except NetworkException as e:
            logger.error(f"Network error: {e}")
            # 记录失败ID，后续重试
        except ParseException as e:
            logger.error(f"Parse error: {e}")
            # 记录错误页面，人工检查
        except DatabaseException as e:
            logger.critical(f"Database error: {e}")
            # 发送告警
        except Exception as e:
            logger.exception(f"Unexpected error: {e}")
            raise
    return wrapper
```

---

### 7.3 数据验证

```python
from pydantic import BaseModel, validator

class BookSchema(BaseModel):
    id: int
    title: str
    author: Optional[str]
    description: Optional[str]

    @validator('id')
    def id_must_be_positive(cls, v):
        if v <= 0:
            raise ValueError('ID must be positive')
        return v

    @validator('title')
    def title_not_empty(cls, v):
        if not v or not v.strip():
            raise ValueError('Title cannot be empty')
        return v.strip()
```

---

## 8. 性能优化

### 8.1 数据库优化

**批量插入**:
```python
def batch_insert_books(books: List[Dict], batch_size: int = 100):
    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    for i in range(0, len(books), batch_size):
        batch = books[i:i + batch_size]
        cursor.executemany(
            "INSERT INTO books (...) VALUES (?, ?, ...)",
            [book.to_tuple() for book in batch]
        )
        conn.commit()

    conn.close()
```

**索引优化**:
- 在查询频繁的字段上建立索引
- 避免过多索引影响写入性能
- 定期分析和优化（ANALYZE 命令）

---

### 8.2 网络优化

**连接池复用**:
```python
session = requests.Session()
adapter = HTTPAdapter(
    pool_connections=10,
    pool_maxsize=20,
    max_retries=3
)
session.mount('http://', adapter)
session.mount('https://', adapter)
```

**异步爬虫**（性能提升 5-10 倍）:
```python
import aiohttp
import asyncio

async def fetch_async(session, url):
    async with session.get(url) as response:
        return await response.text()

async def crawl_batch(book_ids: List[int]):
    async with aiohttp.ClientSession() as session:
        tasks = [fetch_async(session, build_url(id)) for id in book_ids]
        results = await asyncio.gather(*tasks)
        return results
```

---

### 8.3 内存优化

**生成器模式**:
```python
def iter_books(start_id: int, end_id: int):
    """使用生成器避免一次性加载"""
    for book_id in range(start_id, end_id + 1):
        book_info = crawl_book(book_id)
        if book_info:
            yield book_info

# 使用
for book in iter_books(1, 10000):
    db.save_book(book)
```

---

## 9. 监控和日志

### 9.1 日志设计

```python
import logging
from logging.handlers import RotatingFileHandler

def setup_logger():
    logger = logging.getLogger('novel_crawler')
    logger.setLevel(logging.INFO)

    # 文件处理器（自动轮转）
    file_handler = RotatingFileHandler(
        'logs/crawler.log',
        maxBytes=10*1024*1024,  # 10MB
        backupCount=5
    )
    file_handler.setLevel(logging.INFO)

    # 错误日志单独文件
    error_handler = RotatingFileHandler(
        'logs/error.log',
        maxBytes=10*1024*1024,
        backupCount=5
    )
    error_handler.setLevel(logging.ERROR)

    # 格式化
    formatter = logging.Formatter(
        '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    )
    file_handler.setFormatter(formatter)
    error_handler.setFormatter(formatter)

    logger.addHandler(file_handler)
    logger.addHandler(error_handler)

    return logger
```

---

### 9.2 监控指标

```python
class CrawlerMetrics:
    def __init__(self):
        self.start_time = time.time()
        self.success_count = 0
        self.failure_count = 0
        self.total_requests = 0

    def record_success(self):
        self.success_count += 1
        self.total_requests += 1

    def record_failure(self):
        self.failure_count += 1
        self.total_requests += 1

    def get_stats(self) -> dict:
        elapsed = time.time() - self.start_time
        return {
            'elapsed_seconds': elapsed,
            'total_requests': self.total_requests,
            'success_count': self.success_count,
            'failure_count': self.failure_count,
            'success_rate': self.success_count / self.total_requests,
            'requests_per_second': self.total_requests / elapsed
        }
```

---

## 10. 部署方案

### 10.1 开发环境
```bash
# 1. 创建虚拟环境
python -m venv venv
source venv/bin/activate  # Windows: venv\Scripts\activate

# 2. 安装依赖
pip install -r requirements.txt

# 3. 初始化配置
cp config/settings.example.py config/settings.py

# 4. 运行测试
python -m pytest tests/

# 5. 启动爬虫
python main.py crawl --mode initial
```

---

### 10.2 生产环境

**使用 systemd 守护进程**:

`/etc/systemd/system/novel-crawler.service`:
```ini
[Unit]
Description=Novel Metadata Crawler
After=network.target

[Service]
Type=simple
User=crawler
WorkingDirectory=/opt/novel-crawler
ExecStart=/opt/novel-crawler/venv/bin/python main.py schedule --start
Restart=on-failure
RestartSec=10

[Install]
WantedBy=multi-user.target
```

**启动服务**:
```bash
sudo systemctl enable novel-crawler
sudo systemctl start novel-crawler
sudo systemctl status novel-crawler
```

---

### 10.3 Docker 部署（可选）

**Dockerfile**:
```dockerfile
FROM python:3.9-slim

WORKDIR /app

COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

COPY . .

CMD ["python", "main.py", "schedule", "--start"]
```

**docker-compose.yml**:
```yaml
version: '3'
services:
  crawler:
    build: .
    volumes:
      - ./data:/app/data
      - ./logs:/app/logs
    environment:
      - LOG_LEVEL=INFO
    restart: unless-stopped
```

---

## 11. 总结

### 11.1 核心技术选型

| 模块 | 技术选型 | 理由 |
|------|----------|------|
| HTTP 请求 | requests | 成熟稳定，文档完善 |
| HTML 解析 | BeautifulSoup4 | 简单易用，容错性好 |
| 数据库 | SQLite | 轻量级，无需额外服务 |
| 定时任务 | APScheduler | 功能强大，支持持久化 |
| 日志 | logging | Python 标准库 |

---

### 11.2 设计原则

1. **模块化**: 每个模块职责单一，易于测试和维护
2. **可扩展**: 新站点支持通过继承基类实现
3. **容错性**: 完善的异常处理和重试机制
4. **可配置**: 外部配置文件，无需修改代码
5. **可监控**: 完整的日志和指标

---

### 11.3 后续优化方向

1. **异步爬虫**: 使用 aiohttp 提升性能
2. **分布式部署**: 支持多机协同爬取
3. **代理池**: 应对 IP 封禁
4. **增量更新优化**: 智能判断更新频率
5. **数据清洗**: 自动修正错误数据

---

**文档状态**: 已完成
**最后更新**: 2026-03-15
