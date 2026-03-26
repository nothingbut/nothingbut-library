"""
Unit tests for crawler modules.
"""
import pytest
from unittest.mock import Mock, MagicMock, patch

from crawlers.base_crawler import BaseCrawler
from crawlers.youshu_crawler import YoushuCrawler
from crawlers.crawler_manager import CrawlerManager


class TestBaseCrawler:
    """Tests for BaseCrawler."""

    def test_base_crawler_is_abstract(self):
        """Test that BaseCrawler cannot be instantiated directly."""
        config = Mock()
        http_client = Mock()

        with pytest.raises(TypeError):
            BaseCrawler(config, http_client)

    def test_base_crawler_abstract_methods(self):
        """Test that abstract methods must be implemented."""
        config = Mock()
        http_client = Mock()

        class ConcreteCrawler(BaseCrawler):
            def build_url(self, book_id: int) -> str:
                return f"https://example.com/book/{book_id}"

            def parse_book_info(self, html: str):
                return {'title': 'Test'}

        crawler = ConcreteCrawler(config, http_client)
        assert crawler.build_url(1) == "https://example.com/book/1"
        assert crawler.parse_book_info("<html></html>") == {'title': 'Test'}

    def test_random_delay(self):
        """Test random delay functionality."""
        config = Mock()
        config.REQUEST_DELAY = (0.01, 0.02)  # Very short for testing
        http_client = Mock()

        class ConcreteCrawler(BaseCrawler):
            def build_url(self, book_id: int) -> str:
                return f"https://example.com/book/{book_id}"

            def parse_book_info(self, html: str):
                return {'title': 'Test'}

        crawler = ConcreteCrawler(config, http_client)
        import time
        start = time.time()
        crawler.random_delay()
        elapsed = time.time() - start

        assert 0.01 <= elapsed <= 0.05  # Allow some margin


class TestYoushuCrawler:
    """Tests for YoushuCrawler."""

    @pytest.fixture
    def config(self):
        """Create mock config."""
        config = Mock()
        config.YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
        config.REQUEST_DELAY = (0.01, 0.02)
        return config

    @pytest.fixture
    def http_client(self):
        """Create mock HTTP client."""
        return Mock()

    @pytest.fixture
    def crawler(self, config, http_client):
        """Create YoushuCrawler instance."""
        return YoushuCrawler(config, http_client)

    def test_build_url(self, crawler):
        """Test URL building."""
        url = crawler.build_url(123)
        assert url == "https://www.youshu.me/book/123"

    def test_parse_book_info_valid_html(self, crawler):
        """Test parsing valid HTML."""
        html = """
        <html>
            <body>
                <h1 class="book-title">Test Book Title</h1>
                <div class="author">Test Author</div>
                <div class="book-description">Test description</div>
                <div class="book-cover">
                    <img src="https://example.com/cover.jpg" />
                </div>
                <div class="book-status">连载中</div>
            </body>
        </html>
        """

        result = crawler.parse_book_info(html)

        assert result is not None
        assert result['title'] == 'Test Book Title'
        assert result['author'] == 'Test Author'
        assert result['description'] == 'Test description'
        assert result['cover_url'] is not None
        assert result['update_status'] == '连载中'

    def test_parse_book_info_invalid_html(self, crawler):
        """Test parsing invalid HTML."""
        html = "<html><body>No book content here</body></html>"

        result = crawler.parse_book_info(html)
        assert result is None

    def test_extract_text(self, crawler):
        """Test text extraction with multiple selectors."""
        from bs4 import BeautifulSoup

        html = """
        <html>
            <h1 class="book-title">Title 1</h1>
            <div class="author">Author Name</div>
        </html>
        """
        soup = BeautifulSoup(html, 'html.parser')

        # Test single selector
        text = crawler._extract_text(soup, '.book-title')
        assert text == 'Title 1'

        # Test multiple selectors
        text = crawler._extract_text(soup, '.nonexistent, .author')
        assert text == 'Author Name'

    def test_extract_tags(self, crawler):
        """Test tag extraction."""
        from bs4 import BeautifulSoup

        html = """
        <html>
            <div class="tag-list">
                <span class="tag">fantasy</span>
                <span class="tag">action</span>
            </div>
        </html>
        """
        soup = BeautifulSoup(html, 'html.parser')

        tags_json = crawler._extract_tags(soup)
        assert tags_json is not None

        import json
        tags = json.loads(tags_json)
        assert 'fantasy' in tags
        assert 'action' in tags


class TestCrawlerManager:
    """Tests for CrawlerManager."""

    @pytest.fixture
    def manager(self, tmp_path):
        """Create CrawlerManager with temporary database."""
        import tempfile
        from pathlib import Path

        # Create temporary config
        class MockConfig:
            DATA_DIR = Path(tmp_path)
            COVER_DIR = Path(tmp_path) / 'covers'
            YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
            REQUEST_DELAY = (0.01, 0.02)
            MAX_CONSECUTIVE_FAILURES = 5
            REQUEST_TIMEOUT = 10
            MAX_RETRIES = 3
            MAX_IMAGE_SIZE = 10 * 1024 * 1024
            IMAGE_TIMEOUT = 15
            USER_AGENTS = ['Mozilla/5.0']

        import config.settings
        original_settings = config.settings

        # Patch settings
        for attr in dir(MockConfig):
            if not attr.startswith('_'):
                setattr(config.settings, attr, getattr(MockConfig, attr))

        manager = CrawlerManager(MockConfig())

        yield manager

        # Restore settings
        for attr in dir(original_settings):
            if not attr.startswith('_'):
                setattr(config.settings, attr, getattr(original_settings, attr))

    def test_manager_initialization(self, manager):
        """Test manager initialization."""
        assert manager.http_client is not None
        assert manager.crawler is not None
        assert manager.db is not None
        assert manager.img_downloader is not None

    def test_get_statistics(self, manager):
        """Test getting statistics."""
        stats = manager.get_statistics()
        assert 'total_books' in stats
