# 小说元数据爬虫系统 - 开发计划

**项目名称**: Novel Metadata Crawler
**计划版本**: v1.0
**创建日期**: 2026-03-15
**预计工期**: 7 个工作日

---

## 1. 项目概览

### 1.1 项目目标
构建一个自动化的小说元数据采集系统，从优书网和主流小说网站爬取书籍元数据，存储到本地 SQLite 数据库，用于预填充 NothingBut Library 应用。

### 1.2 交付物
- 完整的爬虫系统代码
- 数据库和数据文件
- 完整的技术文档
- 测试用例和测试报告
- 部署脚本

### 1.3 团队角色
- **开发者**: 独立开发（Python 爬虫经验）
- **测试者**: 开发者兼任
- **部署者**: 开发者兼任

---

## 2. 开发阶段划分

### 阶段概览

| 阶段 | 任务 | 工期 | 依赖 | 输出 |
|------|------|------|------|------|
| Phase 1 | 项目初始化 | 0.5 天 | 无 | 项目骨架、文档 |
| Phase 2 | 基础框架开发 | 1.5 天 | Phase 1 | 核心模块、工具类 |
| Phase 3 | 优书网爬虫 | 1 天 | Phase 2 | 主爬虫功能 |
| Phase 4 | 来源网站爬虫 | 1.5 天 | Phase 3 | 起点/纵横爬虫 |
| Phase 5 | 任务调度系统 | 1 天 | Phase 4 | 定时任务 |
| Phase 6 | 测试和优化 | 1 天 | Phase 5 | 测试报告、性能优化 |
| Phase 7 | 文档和部署 | 0.5 天 | Phase 6 | 完整文档、部署脚本 |

**总计**: 7 天

---

## 3. 详细开发计划

### Phase 1: 项目初始化（Day 1 上午，4 小时）

#### 3.1.1 任务清单
- [ ] 创建项目目录结构
- [ ] 初始化 Git 仓库
- [ ] 创建虚拟环境
- [ ] 编写 requirements.txt
- [ ] 配置文件模板
- [ ] 编写 README.md
- [ ] 提交初始代码

#### 3.1.2 项目结构
```bash
mkdir -p novel-crawler/{config,crawlers/source_crawlers,database/migrations,utils,scheduler,tests,data/covers,logs,docs}
cd novel-crawler
python -m venv venv
source venv/bin/activate
git init
```

#### 3.1.3 依赖安装
```bash
# requirements.txt
requests==2.31.0
beautifulsoup4==4.12.3
lxml==4.9.3
APScheduler==3.10.4
python-dotenv==1.0.0
pydantic==2.5.0
pytest==7.4.3
pytest-cov==4.1.0
```

#### 3.1.4 配置文件模板

**config/settings.py**:
```python
import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = BASE_DIR / 'data'
COVER_DIR = DATA_DIR / 'covers'
LOG_DIR = BASE_DIR / 'logs'

# 爬虫配置
YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
REQUEST_TIMEOUT = 10
REQUEST_DELAY = (1, 3)
MAX_RETRIES = 3
MAX_CONCURRENT = 3
MAX_CONSECUTIVE_FAILURES = 50

# 数据库配置
YOUSHU_DB = DATA_DIR / 'youshu.db'

# User-Agent 池
USER_AGENTS = [
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
]

# 日志配置
LOG_LEVEL = 'INFO'
LOG_FILE = LOG_DIR / 'crawler.log'
```

#### 3.1.5 交付物
- [ ] 完整的项目目录结构
- [ ] Git 仓库初始化
- [ ] requirements.txt
- [ ] README.md（项目说明）
- [ ] config/settings.py

---

### Phase 2: 基础框架开发（Day 1 下午 - Day 2，1.5 天）

#### 3.2.1 任务清单

**Day 1 下午（4 小时）**:
- [ ] 开发 HTTPClient（utils/http_client.py）
- [ ] 开发 RetryHandler（utils/retry_handler.py）
- [ ] 开发 Logger 配置（utils/logger.py）
- [ ] 编写单元测试

**Day 2 上午（4 小时）**:
- [ ] 开发 DatabaseManager（database/db_manager.py）
- [ ] 开发数据模型（database/models.py）
- [ ] 创建数据库迁移脚本
- [ ] 测试数据库操作

#### 3.2.2 核心代码

**utils/http_client.py**:
```python
import requests
import random
import time
from typing import Optional
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

class HTTPClient:
    def __init__(self, config):
        self.config = config
        self.session = self._create_session()

    def _create_session(self):
        session = requests.Session()
        retry_strategy = Retry(
            total=self.config.MAX_RETRIES,
            backoff_factor=1,
            status_forcelist=[429, 500, 502, 503, 504]
        )
        adapter = HTTPAdapter(max_retries=retry_strategy)
        session.mount("http://", adapter)
        session.mount("https://", adapter)
        return session

    def get(self, url: str) -> Optional[requests.Response]:
        try:
            headers = {
                'User-Agent': random.choice(self.config.USER_AGENTS)
            }
            response = self.session.get(
                url,
                headers=headers,
                timeout=self.config.REQUEST_TIMEOUT
            )
            response.raise_for_status()
            return response
        except requests.RequestException as e:
            print(f"Request failed: {e}")
            return None

    def random_delay(self):
        min_delay, max_delay = self.config.REQUEST_DELAY
        time.sleep(random.uniform(min_delay, max_delay))
```

**database/db_manager.py**:
```python
import sqlite3
import json
from pathlib import Path
from typing import Optional, Dict, List
from datetime import datetime

class DatabaseManager:
    def __init__(self, config):
        self.config = config
        self.db_path = config.YOUSHU_DB
        self._ensure_directories()
        self._init_database()

    def _ensure_directories(self):
        self.config.DATA_DIR.mkdir(parents=True, exist_ok=True)
        self.config.COVER_DIR.mkdir(parents=True, exist_ok=True)

    def _init_database(self):
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        # 创建 books 表
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS books (
                id INTEGER PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT,
                description TEXT,
                tags TEXT,
                cover_path TEXT,
                source_site TEXT,
                source_url TEXT,
                update_status TEXT,
                crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ''')

        # 创建 crawl_status 表
        cursor.execute('''
            CREATE TABLE IF NOT EXISTS crawl_status (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                last_valid_id INTEGER NOT NULL,
                last_crawl_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                total_books INTEGER DEFAULT 0,
                failed_ids TEXT
            )
        ''')

        # 创建索引
        cursor.execute('CREATE INDEX IF NOT EXISTS idx_author ON books(author)')
        cursor.execute('CREATE INDEX IF NOT EXISTS idx_source ON books(source_site)')

        conn.commit()
        conn.close()

    def save_book(self, book_info: Dict) -> bool:
        try:
            conn = sqlite3.connect(self.db_path)
            cursor = conn.cursor()

            cursor.execute('''
                INSERT OR REPLACE INTO books
                (id, title, author, description, tags, cover_path,
                 source_site, source_url, update_status, updated_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (
                book_info.get('id'),
                book_info.get('title'),
                book_info.get('author'),
                book_info.get('description'),
                book_info.get('tags'),
                book_info.get('cover_path'),
                book_info.get('source_site'),
                book_info.get('source_url'),
                book_info.get('update_status'),
                datetime.now()
            ))

            conn.commit()
            conn.close()
            return True
        except Exception as e:
            print(f"Save error: {e}")
            return False

    def get_last_valid_id(self) -> int:
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()
        cursor.execute('SELECT MAX(id) FROM books')
        result = cursor.fetchone()[0]
        conn.close()
        return result or 0
```

#### 3.2.3 单元测试

**tests/test_http_client.py**:
```python
import pytest
from utils.http_client import HTTPClient
from config import settings

def test_http_client_get():
    client = HTTPClient(settings)
    response = client.get('https://httpbin.org/get')
    assert response is not None
    assert response.status_code == 200

def test_http_client_404():
    client = HTTPClient(settings)
    response = client.get('https://httpbin.org/status/404')
    assert response is None
```

#### 3.2.4 交付物
- [ ] HTTPClient 实现和测试
- [ ] RetryHandler 实现
- [ ] Logger 配置
- [ ] DatabaseManager 实现和测试
- [ ] 数据模型定义
- [ ] 数据库迁移脚本
- [ ] 单元测试（覆盖率 > 80%）

---

### Phase 3: 优书网爬虫开发（Day 2 下午 - Day 3 上午，1 天）

#### 3.3.1 任务清单

**Day 2 下午（4 小时）**:
- [ ] 开发 BaseCrawler（crawlers/base_crawler.py）
- [ ] 开发 YoushuCrawler（crawlers/youshu_crawler.py）
- [ ] 实现 HTML 解析逻辑
- [ ] 编写解析测试

**Day 3 上午（4 小时）**:
- [ ] 开发 ImageDownloader（utils/image_downloader.py）
- [ ] 集成图片下载功能
- [ ] 开发 CrawlerManager（main.py）
- [ ] 端到端测试

#### 3.3.2 核心代码

**crawlers/base_crawler.py**:
```python
from abc import ABC, abstractmethod
from typing import Optional, Dict
import time
import random

class BaseCrawler(ABC):
    def __init__(self, config, http_client):
        self.config = config
        self.http_client = http_client

    def fetch_page(self, url: str) -> Optional[str]:
        response = self.http_client.get(url)
        if response and response.status_code == 200:
            return response.text
        return None

    @abstractmethod
    def parse_book_info(self, html: str) -> Optional[Dict]:
        """子类实现具体解析逻辑"""
        pass

    def crawl_book(self, book_id: int) -> Optional[Dict]:
        url = self.build_url(book_id)
        html = self.fetch_page(url)

        if not html:
            return None

        book_info = self.parse_book_info(html)
        if book_info:
            book_info['id'] = book_id

        self.http_client.random_delay()
        return book_info

    @abstractmethod
    def build_url(self, book_id: int) -> str:
        """构建 URL"""
        pass
```

**crawlers/youshu_crawler.py**:
```python
from bs4 import BeautifulSoup
from .base_crawler import BaseCrawler
import json

class YoushuCrawler(BaseCrawler):
    def build_url(self, book_id: int) -> str:
        return self.config.YOUSHU_BASE_URL.format(book_id=book_id)

    def parse_book_info(self, html: str) -> Optional[Dict]:
        try:
            soup = BeautifulSoup(html, 'html.parser')

            # 根据实际网页结构调整选择器
            book_info = {
                'title': self._extract_text(soup, '.book-title'),
                'author': self._extract_text(soup, '.author-name'),
                'description': self._extract_text(soup, '.description'),
                'tags': self._extract_tags(soup),
                'cover_url': self._extract_image(soup, '.book-cover img'),
                'source_site': self._extract_text(soup, '.source-site'),
                'source_url': self._extract_link(soup, '.source-link'),
                'update_status': self._extract_text(soup, '.status'),
            }

            # 过滤空值
            return {k: v for k, v in book_info.items() if v}

        except Exception as e:
            print(f"Parse error: {e}")
            return None

    def _extract_text(self, soup, selector: str) -> Optional[str]:
        element = soup.select_one(selector)
        return element.get_text(strip=True) if element else None

    def _extract_tags(self, soup) -> Optional[str]:
        tags = soup.select('.tag')
        if tags:
            return json.dumps([tag.get_text(strip=True) for tag in tags])
        return None

    def _extract_image(self, soup, selector: str) -> Optional[str]:
        img = soup.select_one(selector)
        return img.get('src') if img else None

    def _extract_link(self, soup, selector: str) -> Optional[str]:
        link = soup.select_one(selector)
        return link.get('href') if link else None
```

**utils/image_downloader.py**:
```python
import requests
from pathlib import Path
from typing import Optional

class ImageDownloader:
    def __init__(self, config):
        self.config = config
        self.cover_dir = config.COVER_DIR

    def download_cover(self, cover_url: str, book_id: int) -> Optional[str]:
        if not cover_url:
            return None

        try:
            response = requests.get(cover_url, timeout=10)
            if response.status_code == 200:
                filename = f"{book_id}.jpg"
                filepath = self.cover_dir / filename

                with open(filepath, 'wb') as f:
                    f.write(response.content)

                return str(filepath)

        except Exception as e:
            print(f"Download cover failed: {e}")

        return None
```

**main.py**（初版）:
```python
import logging
from config import settings
from crawlers.youshu_crawler import YoushuCrawler
from database.db_manager import DatabaseManager
from utils.http_client import HTTPClient
from utils.image_downloader import ImageDownloader

logging.basicConfig(
    level=settings.LOG_LEVEL,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

class CrawlerManager:
    def __init__(self):
        self.config = settings
        self.http_client = HTTPClient(settings)
        self.crawler = YoushuCrawler(settings, self.http_client)
        self.db = DatabaseManager(settings)
        self.img_downloader = ImageDownloader(settings)
        self.logger = logging.getLogger(__name__)

    def crawl_single_book(self, book_id: int) -> Optional[Dict]:
        book_info = self.crawler.crawl_book(book_id)

        if book_info:
            # 下载封面
            if book_info.get('cover_url'):
                cover_path = self.img_downloader.download_cover(
                    book_info['cover_url'],
                    book_id
                )
                book_info['cover_path'] = cover_path

            # 保存到数据库
            if self.db.save_book(book_info):
                self.logger.info(f"Saved book {book_id}: {book_info.get('title')}")
                return book_info

        return None

if __name__ == '__main__':
    manager = CrawlerManager()
    # 测试爬取单本
    book = manager.crawl_single_book(1)
    print(f"Crawled: {book}")
```

#### 3.3.3 测试策略

**手动测试**:
```bash
# 测试单本爬取
python main.py

# 验证数据库
sqlite3 data/youshu.db "SELECT * FROM books LIMIT 1"

# 检查封面文件
ls -lh data/covers/
```

**单元测试**:
```python
def test_youshu_parser():
    html = """
    <html>
        <div class="book-title">测试书名</div>
        <div class="author-name">测试作者</div>
    </html>
    """
    crawler = YoushuCrawler(settings, http_client)
    result = crawler.parse_book_info(html)
    assert result['title'] == '测试书名'
    assert result['author'] == '测试作者'
```

#### 3.3.4 交付物
- [ ] BaseCrawler 基类
- [ ] YoushuCrawler 实现
- [ ] ImageDownloader 实现
- [ ] CrawlerManager 初版
- [ ] 单本爬取功能测试通过
- [ ] 单元测试（覆盖率 > 80%）

---

### Phase 4: 来源网站爬虫开发（Day 3 下午 - Day 4，1.5 天）

#### 3.4.1 任务清单

**Day 3 下午（4 小时）**:
- [ ] 设计来源爬虫架构
- [ ] 开发 QidianCrawler（crawlers/source_crawlers/qidian.py）
- [ ] 创建起点数据库 Schema
- [ ] 测试起点爬虫

**Day 4 上午（4 小时）**:
- [ ] 开发 ZonghengCrawler（crawlers/source_crawlers/zongheng.py）
- [ ] 创建纵横数据库 Schema
- [ ] 集成来源爬虫到主流程
- [ ] 测试完整流程

**Day 4 下午（2 小时）**:
- [ ] 优化解析逻辑
- [ ] 错误处理完善
- [ ] 集成测试

#### 3.4.2 核心代码

**crawlers/source_crawlers/qidian.py**:
```python
from ..base_crawler import BaseCrawler
from bs4 import BeautifulSoup
from typing import Optional, Dict

class QidianCrawler(BaseCrawler):
    BASE_URL = "https://book.qidian.com/info/{book_id}"

    def build_url(self, book_id: int) -> str:
        return self.BASE_URL.format(book_id=book_id)

    def parse_book_info(self, html: str) -> Optional[Dict]:
        try:
            soup = BeautifulSoup(html, 'html.parser')

            return {
                'title': self._extract_text(soup, '.book-info h1'),
                'author': self._extract_text(soup, '.writer'),
                'category': self._extract_text(soup, '.category'),
                'word_count': self._parse_number(
                    self._extract_text(soup, '.word-count')
                ),
                'chapter_count': self._parse_number(
                    self._extract_text(soup, '.chapter-count')
                ),
                'rating': self._parse_float(
                    self._extract_text(soup, '.rating')
                ),
                'status': self._extract_text(soup, '.status'),
            }

        except Exception as e:
            print(f"Parse error: {e}")
            return None

    def _parse_number(self, text: Optional[str]) -> Optional[int]:
        """解析数字：'12.3万' -> 123000"""
        if not text:
            return None
        if '万' in text:
            return int(float(text.replace('万', '')) * 10000)
        return int(text)

    def _parse_float(self, text: Optional[str]) -> Optional[float]:
        if not text:
            return None
        try:
            return float(text)
        except ValueError:
            return None
```

**database/source_db_manager.py**:
```python
class SourceDatabaseManager:
    def __init__(self, config, source: str):
        self.config = config
        self.source = source
        self.db_path = config.DATA_DIR / f'{source}.db'
        self._init_database()

    def _init_database(self):
        conn = sqlite3.connect(self.db_path)
        cursor = conn.cursor()

        cursor.execute('''
            CREATE TABLE IF NOT EXISTS book_details (
                book_id INTEGER PRIMARY KEY,
                youshu_id INTEGER,
                title TEXT NOT NULL,
                author TEXT,
                description TEXT,
                category TEXT,
                tags TEXT,
                cover_url TEXT,
                cover_path TEXT,
                word_count INTEGER,
                chapter_count INTEGER,
                status TEXT,
                rating REAL,
                crawled_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
        ''')

        cursor.execute('CREATE INDEX IF NOT EXISTS idx_youshu ON book_details(youshu_id)')

        conn.commit()
        conn.close()

    def save_book_detail(self, book_info: Dict) -> bool:
        # 实现保存逻辑
        pass
```

**集成到 main.py**:
```python
def crawl_book_with_source(self, book_id: int):
    # 1. 爬取优书网索引
    book_info = self.crawler.crawl_book(book_id)

    if not book_info:
        return None

    # 2. 下载封面
    if book_info.get('cover_url'):
        cover_path = self.img_downloader.download_cover(
            book_info['cover_url'],
            book_id
        )
        book_info['cover_path'] = cover_path

    # 3. 保存主索引
    self.db.save_book(book_info)

    # 4. 如果有来源链接，爬取详情
    if book_info.get('source_url') and book_info.get('source_site'):
        source_site = book_info['source_site']
        source_crawler = self.get_source_crawler(source_site)

        if source_crawler:
            source_book_id = self.extract_book_id(book_info['source_url'])
            source_info = source_crawler.crawl_book(source_book_id)

            if source_info:
                source_info['youshu_id'] = book_id
                self.source_dbs[source_site].save_book_detail(source_info)

    return book_info
```

#### 3.4.3 交付物
- [ ] QidianCrawler 实现
- [ ] ZonghengCrawler 实现
- [ ] 来源数据库管理
- [ ] 完整流程集成
- [ ] 测试通过

---

### Phase 5: 任务调度系统（Day 5，1 天）

#### 3.5.1 任务清单

**Day 5 上午（4 小时）**:
- [ ] 实现批量爬取逻辑
- [ ] 实现停止条件判断
- [ ] 实现状态管理
- [ ] 测试批量爬取

**Day 5 下午（4 小时）**:
- [ ] 开发定时任务（scheduler/daily_task.py）
- [ ] 实现增量更新逻辑
- [ ] 实现失败重试逻辑
- [ ] 测试定时任务

#### 3.5.2 核心代码

**批量爬取（main.py）**:
```python
def run_initial_crawl(self, start_id: int = 1):
    """首次全量爬取"""
    self.logger.info(f"Starting initial crawl from ID {start_id}")

    current_id = start_id
    consecutive_failures = 0
    total_success = 0
    failed_ids = []

    while consecutive_failures < self.config.MAX_CONSECUTIVE_FAILURES:
        try:
            book_info = self.crawl_book_with_source(current_id)

            if book_info:
                total_success += 1
                consecutive_failures = 0
                self.logger.info(f"✓ Book {current_id}: {book_info.get('title')}")
            else:
                consecutive_failures += 1
                failed_ids.append(current_id)
                self.logger.warning(f"✗ Book {current_id} failed ({consecutive_failures})")

        except Exception as e:
            self.logger.error(f"Error crawling {current_id}: {e}")
            failed_ids.append(current_id)
            consecutive_failures += 1

        current_id += 1

        # 每100本保存状态
        if current_id % 100 == 0:
            self.db.update_crawl_status(current_id - 1, total_success, failed_ids)
            self.logger.info(f"Progress: {total_success} books, {len(failed_ids)} failed")

    # 最终状态
    self.db.update_crawl_status(current_id - 1, total_success, failed_ids)
    self.logger.info(f"Completed: {total_success} books, {len(failed_ids)} failed")

    return {
        'total_success': total_success,
        'total_failed': len(failed_ids),
        'last_id': current_id - 1
    }

def run_incremental_crawl(self):
    """增量爬取"""
    last_id = self.db.get_last_valid_id()
    self.logger.info(f"Starting incremental crawl from ID {last_id + 1}")
    return self.run_initial_crawl(start_id=last_id + 1)
```

**定时任务（scheduler/daily_task.py）**:
```python
from apscheduler.schedulers.blocking import BlockingScheduler
from apscheduler.triggers.cron import CronTrigger
import logging

logger = logging.getLogger(__name__)

def daily_crawl_job():
    """每日增量爬取"""
    from main import CrawlerManager
    manager = CrawlerManager()
    result = manager.run_incremental_crawl()
    logger.info(f"Daily crawl completed: {result}")

def weekly_retry_job():
    """每周补爬失败ID"""
    from main import CrawlerManager
    manager = CrawlerManager()
    failed_ids = manager.db.get_failed_ids()
    logger.info(f"Retrying {len(failed_ids)} failed IDs")
    # 实现重试逻辑

def start_scheduler():
    scheduler = BlockingScheduler()

    # 每天凌晨2点
    scheduler.add_job(
        daily_crawl_job,
        trigger=CronTrigger(hour=2, minute=0),
        id='daily_crawl',
        name='Daily incremental crawl'
    )

    # 每周日凌晨3点
    scheduler.add_job(
        weekly_retry_job,
        trigger=CronTrigger(day_of_week='sun', hour=3, minute=0),
        id='weekly_retry',
        name='Weekly failed ID retry'
    )

    logger.info("Scheduler started")
    scheduler.start()

if __name__ == '__main__':
    start_scheduler()
```

**命令行接口**:
```python
import argparse

def main():
    parser = argparse.ArgumentParser(description='Novel Metadata Crawler')
    subparsers = parser.add_subparsers(dest='command')

    # crawl 命令
    crawl_parser = subparsers.add_parser('crawl')
    crawl_parser.add_argument('--mode', choices=['initial', 'incremental'], required=True)
    crawl_parser.add_argument('--start', type=int, default=1)

    # schedule 命令
    schedule_parser = subparsers.add_parser('schedule')
    schedule_parser.add_argument('--start', action='store_true')

    # stats 命令
    stats_parser = subparsers.add_parser('stats')

    args = parser.parse_args()

    if args.command == 'crawl':
        manager = CrawlerManager()
        if args.mode == 'initial':
            manager.run_initial_crawl(start_id=args.start)
        else:
            manager.run_incremental_crawl()

    elif args.command == 'schedule':
        from scheduler.daily_task import start_scheduler
        start_scheduler()

    elif args.command == 'stats':
        db = DatabaseManager(settings)
        print(f"Total books: {db.get_total_books()}")
        print(f"Last ID: {db.get_last_valid_id()}")
```

#### 3.5.3 交付物
- [ ] 批量爬取功能
- [ ] 增量更新功能
- [ ] 定时任务系统
- [ ] 命令行接口
- [ ] 测试通过

---

### Phase 6: 测试和优化（Day 6，1 天）

#### 3.6.1 任务清单

**Day 6 上午（4 小时）**:
- [ ] 编写集成测试
- [ ] 性能测试
- [ ] 错误场景测试
- [ ] 修复发现的 Bug

**Day 6 下午（4 小时）**:
- [ ] 代码优化
- [ ] 性能优化
- [ ] 日志完善
- [ ] 文档更新

#### 3.6.2 测试用例

**集成测试（tests/test_integration.py）**:
```python
def test_full_crawl_flow():
    """测试完整爬取流程"""
    manager = CrawlerManager()

    # 爬取 1-10
    manager.run_initial_crawl(start_id=1)

    # 验证数据库
    db = DatabaseManager(settings)
    count = db.get_total_books()
    assert count > 0

    # 验证封面
    covers = list(settings.COVER_DIR.glob('*.jpg'))
    assert len(covers) > 0

def test_incremental_crawl():
    """测试增量爬取"""
    manager = CrawlerManager()

    # 首次爬取
    manager.run_initial_crawl(start_id=1)
    last_id_1 = manager.db.get_last_valid_id()

    # 增量爬取
    manager.run_incremental_crawl()
    last_id_2 = manager.db.get_last_valid_id()

    assert last_id_2 > last_id_1
```

**性能测试**:
```python
import time

def test_crawl_speed():
    """测试爬取速度"""
    manager = CrawlerManager()

    start_time = time.time()
    result = manager.run_initial_crawl(start_id=1)
    elapsed = time.time() - start_time

    success_count = result['total_success']
    speed = success_count / (elapsed / 3600)  # 本/小时

    print(f"Speed: {speed:.0f} books/hour")
    assert speed > 500  # 至少 500 本/小时
```

#### 3.6.3 性能优化

**批量数据库插入**:
```python
def batch_save_books(self, books: List[Dict], batch_size: int = 50):
    conn = sqlite3.connect(self.db_path)
    cursor = conn.cursor()

    for i in range(0, len(books), batch_size):
        batch = books[i:i + batch_size]
        cursor.executemany(
            "INSERT OR REPLACE INTO books (...) VALUES (?, ?, ...)",
            [(b['id'], b['title'], ...) for b in batch]
        )
        conn.commit()

    conn.close()
```

**异步爬取（可选）**:
```python
import asyncio
import aiohttp

async def crawl_batch_async(self, book_ids: List[int]):
    async with aiohttp.ClientSession() as session:
        tasks = [self.crawl_book_async(session, id) for id in book_ids]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        return [r for r in results if r is not None]
```

#### 3.6.4 交付物
- [ ] 完整的测试套件
- [ ] 性能测试报告
- [ ] 代码优化完成
- [ ] 测试覆盖率 > 80%

---

### Phase 7: 文档和部署（Day 7，0.5 天）

#### 3.7.1 任务清单

**Day 7 上午（4 小时）**:
- [ ] 完善 README.md
- [ ] 编写 API 文档
- [ ] 编写部署文档
- [ ] 创建部署脚本
- [ ] 最终测试

#### 3.7.2 文档清单

**README.md**:
```markdown
# Novel Metadata Crawler

小说元数据爬虫系统

## 快速开始

### 安装
```bash
git clone <repo-url>
cd novel-crawler
python -m venv venv
source venv/bin/activate
pip install -r requirements.txt
```

### 使用
```bash
# 首次爬取
python main.py crawl --mode initial

# 增量更新
python main.py crawl --mode incremental

# 启动定时任务
python main.py schedule --start
```

## 功能特性
- 自动爬取优书网书籍索引
- 支持起点、纵横等来源网站
- 定时增量更新
- 失败自动重试

## 配置
编辑 `config/settings.py` 修改配置。
```

**部署脚本（deploy.sh）**:
```bash
#!/bin/bash

# 部署脚本

echo "Deploying Novel Crawler..."

# 1. 创建虚拟环境
python3 -m venv venv
source venv/bin/activate

# 2. 安装依赖
pip install -r requirements.txt

# 3. 初始化数据库
python -c "from database.db_manager import DatabaseManager; from config import settings; DatabaseManager(settings)"

# 4. 创建 systemd 服务
sudo cp novel-crawler.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable novel-crawler
sudo systemctl start novel-crawler

echo "Deployment completed!"
```

#### 3.7.3 交付物
- [ ] 完整的 README.md
- [ ] API 文档
- [ ] 部署文档
- [ ] 部署脚本
- [ ] 最终测试通过

---

## 4. 里程碑和验收标准

### 4.1 里程碑

| 里程碑 | 完成标准 | 预计日期 |
|--------|----------|----------|
| M1: 项目初始化完成 | 项目结构、依赖安装 | Day 1 |
| M2: 基础框架完成 | HTTP、DB、Logger 就绪 | Day 2 |
| M3: 主爬虫完成 | 优书网爬虫可用 | Day 3 |
| M4: 来源爬虫完成 | 起点/纵横爬虫可用 | Day 4 |
| M5: 调度系统完成 | 定时任务、增量更新 | Day 5 |
| M6: 测试完成 | 所有测试通过 | Day 6 |
| M7: 部署就绪 | 文档、脚本完成 | Day 7 |

---

### 4.2 验收标准

#### 功能验收
- [ ] 成功爬取 >= 1000 本书籍
- [ ] 封面下载成功率 >= 95%
- [ ] 来源网站解析成功率 >= 90%
- [ ] 定时任务正常运行
- [ ] 增量更新正常工作
- [ ] 失败重试机制有效

#### 质量验收
- [ ] 单元测试覆盖率 >= 80%
- [ ] 集成测试全部通过
- [ ] 代码通过 pylint 检查（评分 >= 8.0）
- [ ] 无严重 Bug

#### 性能验收
- [ ] 爬取速度 >= 1000 本/小时
- [ ] 单个请求响应时间 < 5 秒
- [ ] 数据库查询 < 100ms
- [ ] 内存占用 < 500MB

#### 文档验收
- [ ] README 完整
- [ ] API 文档完整
- [ ] 部署文档可用
- [ ] 代码注释充分

---

## 5. 风险管理

### 5.1 技术风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|----------|
| 网站反爬虫 | 高 | 中 | 预留 User-Agent 轮换、代理池接口 |
| 网页结构变化 | 高 | 中 | 模块化设计，易于更新 |
| 网络不稳定 | 中 | 低 | 完善重试机制 |
| 性能不达标 | 中 | 低 | 异步爬虫方案备选 |

### 5.2 进度风险

| 风险 | 应对措施 |
|------|----------|
| 开发延期 | 优先保证核心功能，来源爬虫可延后 |
| 测试不充分 | 预留 1 天 buffer |
| 文档不完整 | 开发过程同步更新文档 |

---

## 6. 资源需求

### 6.1 开发环境
- Python 3.8+
- SQLite 3
- Git
- 代码编辑器（VS Code 推荐）

### 6.2 测试环境
- 稳定的网络连接
- 至少 2GB 磁盘空间
- 测试用书籍 ID 列表

### 6.3 生产环境
- Linux 服务器（推荐）
- Python 3.8+
- systemd（用于守护进程）
- 至少 5GB 磁盘空间

---

## 7. 交付清单

### 7.1 代码交付
- [ ] 完整的源代码
- [ ] requirements.txt
- [ ] .gitignore
- [ ] setup.py

### 7.2 数据交付
- [ ] 初始数据库文件（示例）
- [ ] 封面图片（示例）
- [ ] 数据库 Schema 文件

### 7.3 文档交付
- [ ] README.md
- [ ] REQUIREMENTS.md（需求文档）
- [ ] DESIGN.md（设计文档）
- [ ] API.md（API 文档）
- [ ] DEPLOYMENT.md（部署文档）

### 7.4 脚本交付
- [ ] deploy.sh（部署脚本）
- [ ] systemd 服务文件
- [ ] 测试脚本

---

## 8. 后续计划（可选）

### Phase 8: 增强功能（1-2 周）
- [ ] 异步爬虫（aiohttp）
- [ ] 代理池集成
- [ ] 数据清洗和校验
- [ ] Web 管理界面
- [ ] 监控和告警

### Phase 9: 分布式部署（2-3 周）
- [ ] Redis 任务队列
- [ ] 多机协同爬取
- [ ] 负载均衡
- [ ] 分布式数据库

---

## 9. 联系和支持

**项目负责人**: [Your Name]
**邮箱**: [Your Email]
**Git 仓库**: [Repository URL]

---

**文档状态**: 已批准
**最后更新**: 2026-03-15
**预计开始日期**: [To be determined]
**预计完成日期**: [Start Date + 7 days]
