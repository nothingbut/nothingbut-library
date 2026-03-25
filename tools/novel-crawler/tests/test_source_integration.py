"""
Integration tests for source crawler functionality.

These tests verify the complete workflow of:
1. Getting books from youshu database
2. Extracting source URLs
3. Crawling source sites
4. Saving to source databases
"""
import pytest
import tempfile
import shutil
from pathlib import Path

from crawlers.source_crawler_manager import SourceCrawlerManager
from database.db_manager import DatabaseManager
from database.source_db_manager import SourceDBManager
from database.models import Book
from config import settings


@pytest.fixture
def temp_config():
    """Create a temporary configuration for testing."""
    # Create temp directory
    temp_dir = tempfile.mkdtemp()

    # Create temporary config
    class TestConfig:
        BASE_DIR = Path(temp_dir)
        DATA_DIR = Path(temp_dir) / 'data'
        COVER_DIR = Path(temp_dir) / 'data' / 'covers'
        LOG_DIR = Path(temp_dir) / 'logs'
        YOUSHU_DB = Path(temp_dir) / 'data' / 'youshu.db'
        SOURCE_DB_PATTERN = Path(temp_dir) / 'data' / '{source}.db'

        YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
        REQUEST_TIMEOUT = 10
        REQUEST_DELAY = (0, 0.1)  # Short delay for tests
        MAX_RETRIES = 2
        MAX_CONCURRENT = 2
        MAX_CONSECUTIVE_FAILURES = 10

        USER_AGENTS = ['Test-Agent']

        SITE_CONFIGS = {
            'youshu': {'name': '优书网', 'base_url': 'https://www.youshu.me/book/{book_id}', 'enabled': True},
            'qidian': {'name': '起点中文网', 'base_url': 'https://book.qidian.com/info/{book_id}', 'enabled': True},
            'zongheng': {'name': '纵横中文网', 'base_url': 'http://book.zongheng.com/book/{book_id}.html', 'enabled': True}
        }

        MAX_IMAGE_SIZE = 10 * 1024 * 1024
        IMAGE_TIMEOUT = 15

        BATCH_SIZE = 10
        SAVE_INTERVAL = 50

    yield TestConfig

    # Cleanup
    shutil.rmtree(temp_dir, ignore_errors=True)


@pytest.fixture
def sample_youshu_book(temp_config):
    """Create a sample youshu book for testing."""
    return Book(
        id=1,
        title='测试书籍',
        author='测试作者',
        description='测试描述',
        tags=['玄幻', '魔法'],
        cover_path='/data/covers/1.jpg',
        source_site='qidian',
        source_url='https://book.qidian.com/info/123456',
        update_status='连载中'
    )


class TestSourceDBManager:
    """Test cases for SourceDBManager."""

    def test_init_database(self, temp_config):
        """Test database initialization."""
        db = SourceDBManager('qidian', temp_config)
        assert db.db_path.exists()

    def test_save_and_get_book_detail(self, temp_config, sample_youshu_book):
        """Test saving and retrieving book details."""
        db = SourceDBManager('qidian', temp_config)

        # Save book detail
        book_detail = {
            'book_id': '123456',
            'title': '起点测试书',
            'author': '起点作者',
            'description': '起点描述',
            'category': '玄幻',
            'sub_category': '东方玄幻',
            'tags': ['标签1', '标签2'],
            'cover_url': 'https://example.com/qidian_cover.jpg',
            'cover_path': '',
            'word_count': 1000000,
            'chapter_count': 500,
            'status': '连载',
            'rating': 8.5,
            'view_count': 10000,
            'favorite_count': 500
        }

        result = db.save_book_detail(sample_youshu_book.id, book_detail)
        assert result is True

        # Retrieve book detail
        retrieved = db.get_book_detail(sample_youshu_book.id)
        assert retrieved is not None
        assert retrieved.title == '起点测试书'
        assert retrieved.author == '起点作者'
        assert retrieved.youshu_id == sample_youshu_book.id

    def test_get_all_books(self, temp_config):
        """Test retrieving all books."""
        db = SourceDBManager('qidian', temp_config)

        # Save multiple books
        for i in range(3):
            book_detail = {
                'book_id': f'{i}',
                'title': f'Book {i}',
                'author': f'Author {i}',
                'description': f'Description {i}',
                'category': '测试',
                'tags': [],
                'word_count': 100000,
                'chapter_count': 100,
                'status': '完结',
                'rating': 8.0,
                'view_count': 1000,
                'favorite_count': 100
            }
            db.save_book_detail(i, book_detail)

        # Retrieve all
        all_books = db.get_all_books()
        assert len(all_books) == 3

    def test_search_books(self, temp_config):
        """Test book search functionality."""
        db = SourceDBManager('qidian', temp_config)

        # Save books
        book_detail = {
            'book_id': '1',
            'title': '搜索测试书',
            'author': '搜索作者',
            'description': '用于搜索的书籍',
            'category': '玄幻',
            'tags': ['搜索'],
            'word_count': 100000,
            'chapter_count': 100,
            'status': '完结',
            'rating': 8.0,
            'view_count': 1000,
            'favorite_count': 100
        }
        db.save_book_detail(1, book_detail)

        # Search by title
        results = db.search_books('搜索')
        assert len(results) >= 1
        assert '搜索测试书' in [b.title for b in results]

    def test_get_statistics(self, temp_config):
        """Test statistics retrieval."""
        db = SourceDBManager('qidian', temp_config)

        # Save some books
        for i in range(5):
            book_detail = {
                'book_id': f'{i}',
                'title': f'Book {i}',
                'author': f'Author {i}',
                'description': f'Desc {i}',
                'category': '玄幻' if i % 2 == 0 else '都市',
                'tags': [],
                'word_count': 100000 * (i + 1),
                'chapter_count': 100 * (i + 1),
                'status': '连载' if i % 2 == 0 else '完结',
                'rating': 7.0 + i * 0.5,
                'view_count': 1000 * (i + 1),
                'favorite_count': 100 * (i + 1)
            }
            db.save_book_detail(i, book_detail)

        # Get statistics
        stats = db.get_statistics()
        assert stats['total_books'] == 5
        assert stats['site_name'] == 'qidian'
        assert 'status_distribution' in stats


class TestSourceCrawlerManager:
    """Test cases for SourceCrawlerManager."""

    def test_extract_qidian_book_id(self, temp_config):
        """Test extracting book ID from Qidian URL."""
        manager = SourceCrawlerManager(temp_config)

        url = "https://book.qidian.com/info/123456"
        book_id = manager.extract_book_id_from_url(url, 'qidian')

        assert book_id == '123456'

    def test_extract_zongheng_book_id(self, temp_config):
        """Test extracting book ID from Zongheng URL."""
        manager = SourceCrawlerManager(temp_config)

        url = "http://book.zongheng.com/book/654321.html"
        book_id = manager.extract_book_id_from_url(url, 'zongheng')

        assert book_id == '654321'

    def test_extract_invalid_url(self, temp_config):
        """Test extracting book ID from invalid URL."""
        manager = SourceCrawlerManager(temp_config)

        url = "https://example.com/book/123"
        book_id = manager.extract_book_id_from_url(url, 'qidian')

        assert book_id is None

    def test_get_source_db(self, temp_config):
        """Test getting source database manager."""
        manager = SourceCrawlerManager(temp_config)

        db = manager.get_source_db('qidian')
        assert db is not None
        assert db.site_name == 'qidian'

        # Should return the same instance on second call
        db2 = manager.get_source_db('qidian')
        assert db is db2


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
