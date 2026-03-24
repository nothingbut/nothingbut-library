"""
Global configuration settings for the novel crawler system.
"""
import os
from pathlib import Path

# Base paths
BASE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = BASE_DIR / 'data'
COVER_DIR = DATA_DIR / 'covers'
LOG_DIR = BASE_DIR / 'logs'

# Database configuration
YOUSHU_DB = DATA_DIR / 'youshu.db'
SOURCE_DB_PATTERN = DATA_DIR / '{source}.db'

# Crawler configuration
YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
REQUEST_TIMEOUT = 10
REQUEST_DELAY = (1, 3)  # Random delay range in seconds
MAX_RETRIES = 3
MAX_CONCURRENT = 3
MAX_CONSECUTIVE_FAILURES = 50

# Scheduler configuration
DAILY_CRAWL_TIME = "02:00"
RETRY_SCHEDULE = "sunday 03:00"

# Logging configuration
LOG_LEVEL = 'INFO'
LOG_FORMAT = '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
LOG_FILE = LOG_DIR / 'crawler.log'
ERROR_LOG_FILE = LOG_DIR / 'error.log'

# User-Agent pool
USER_AGENTS = [
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Safari/605.1.15',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0',
    'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
]

# Site configurations
SITE_CONFIGS = {
    'youshu': {
        'name': '优书网',
        'base_url': 'https://www.youshu.me/book/{book_id}',
        'enabled': True
    },
    'qidian': {
        'name': '起点中文网',
        'base_url': 'https://book.qidian.com/info/{book_id}',
        'enabled': True
    },
    'zongheng': {
        'name': '纵横中文网',
        'base_url': 'http://book.zongheng.com/book/{book_id}.html',
        'enabled': False  # Initial phase only
    }
}

# Image download settings
MAX_IMAGE_SIZE = 10 * 1024 * 1024  # 10MB
IMAGE_TIMEOUT = 15

# Batch processing
BATCH_SIZE = 50
SAVE_INTERVAL = 100  # Save status every 100 books

# Ensure directories exist
def ensure_directories():
    """Create necessary directories if they don't exist."""
    DATA_DIR.mkdir(parents=True, exist_ok=True)
    COVER_DIR.mkdir(parents=True, exist_ok=True)
    LOG_DIR.mkdir(parents=True, exist_ok=True)

# Auto-initialize on import
ensure_directories()
