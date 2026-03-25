"""
Unit tests for source site crawlers.
"""
import pytest
from unittest.mock import Mock, patch, MagicMock
from bs4 import BeautifulSoup

from crawlers.source_crawlers.qidian_crawler import QidianCrawler
from crawlers.source_crawlers.zongheng_crawler import ZonghengCrawler
from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler
from config import settings


class TestQidianCrawler:
    """Test cases for QidianCrawler."""

    @pytest.fixture
    def crawler(self):
        """Create a QidianCrawler instance."""
        return QidianCrawler()

    def test_site_name(self, crawler):
        """Test site name getter."""
        assert crawler.get_site_name() == 'qidian'

    def test_base_url(self, crawler):
        """Test base URL getter."""
        assert crawler.get_base_url() == "https://book.qidian.com/info/{book_id}"

    def test_build_book_url(self, crawler):
        """Test URL building."""
        url = crawler.build_book_url("123456")
        assert url == "https://book.qidian.com/info/123456"

    def test_parse_book_detail_valid_html(self, crawler):
        """Test parsing valid Qidian HTML."""
        html = """
        <html>
            <body>
                <div class="book-info">
                    <h1>测试书名</h1>
                    <span class="writer">测试作者</span>
                    <div class="book-intro">这是一本测试书籍的简介。</div>
                    <div class="type"><a>玄幻</a></div>
                    <div class="tag"><span>125万</span></div>
                    <div class="state">连载</div>
                </div>
                <div class="book-img">
                    <img src="https://example.com/cover.jpg" />
                </div>
            </body>
        </html>
        """

        result = crawler.parse_book_detail(html, "123456")

        assert result is not None
        assert result['title'] == '测试书名'
        assert result['author'] == '测试作者'
        assert result['description'] == '这是一本测试书籍的简介。'
        assert result['category'] == '玄幻'
        assert result['word_count'] == 1250000
        assert result['status'] == '连载'
        assert result['cover_url'] == 'https://example.com/cover.jpg'

    def test_parse_book_detail_missing_elements(self, crawler):
        """Test parsing HTML with missing elements."""
        html = "<html><body><div></div></body></html>"

        result = crawler.parse_book_detail(html, "123456")

        # Should return None if no title found
        assert result is None

    def test_extract_number_with_wan(self, crawler):
        """Test number extraction with '万' suffix."""
        # Use English encoding for tests
        assert crawler.extract_number("125万") == 1250000
        assert crawler.extract_number("1.5万") == 15000

    def test_extract_number_with_yi(self, crawler):
        """Test number extraction with '亿' suffix."""
        # Use English encoding for tests
        assert crawler.extract_number("1.2亿") == 120000000
        assert crawler.extract_number("2亿") == 200000000

    def test_extract_rating(self, crawler):
        """Test rating extraction."""
        assert crawler._extract_rating("9.5分") == 9.5
        assert crawler._extract_rating("8.0") == 8.0
        assert crawler._extract_rating("") == 0.0


class TestZonghengCrawler:
    """Test cases for ZonghengCrawler."""

    @pytest.fixture
    def crawler(self):
        """Create a ZonghengCrawler instance."""
        return ZonghengCrawler()

    def test_site_name(self, crawler):
        """Test site name getter."""
        assert crawler.get_site_name() == 'zongheng'

    def test_base_url(self, crawler):
        """Test base URL getter."""
        assert crawler.get_base_url() == "http://book.zongheng.com/book/{book_id}.html"

    def test_build_book_url(self, crawler):
        """Test URL building."""
        url = crawler.build_book_url("123456")
        assert url == "http://book.zongheng.com/book/123456.html"

    def test_parse_book_detail_valid_html(self, crawler):
        """Test parsing valid Zongheng HTML."""
        html = """
        <html>
            <body>
                <div class="book-info">
                    <div class="book-name">测试书名</div>
                    <div class="author">测试作者</div>
                    <div class="book-intro">这是一本测试书籍的简介。</div>
                    <div class="category"><a>玄幻</a></div>
                    <div class="word-count">125万</div>
                    <div class="status">连载中</div>
                </div>
                <div class="book-img">
                    <img src="https://example.com/cover.jpg" />
                </div>
            </body>
        </html>
        """

        result = crawler.parse_book_detail(html, "123456")

        assert result is not None
        assert result['title'] == '测试书名'
        assert result['author'] == '测试作者'
        assert result['description'] == '这是一本测试书籍的简介。'
        assert result['category'] == '玄幻'
        assert result['word_count'] == 1250000
        assert result['status'] == '连载'
        assert result['cover_url'] == 'https://example.com/cover.jpg'


class TestBaseSourceCrawler:
    """Test cases for BaseSourceCrawler."""

    def test_base_crawler_is_abstract(self):
        """Test that BaseSourceCrawler cannot be instantiated."""
        with pytest.raises(TypeError):
            BaseSourceCrawler()

    def test_extract_text(self):
        """Test text extraction method."""
        # Create a concrete implementation for testing
        class TestCrawler(BaseSourceCrawler):
            def get_site_name(self):
                return 'test'

            def get_base_url(self):
                return 'https://test.com/{book_id}'

            def build_book_url(self, book_id):
                return f'https://test.com/{book_id}'

            def parse_book_detail(self, html, book_id):
                return {}

        crawler = TestCrawler()
        html = '<div class="test">Hello World</div>'
        soup = BeautifulSoup(html, 'html.parser')

        result = crawler.extract_text(soup, '.test')
        assert result == 'Hello World'

    def test_extract_text_default(self):
        """Test text extraction with default value."""
        class TestCrawler(BaseSourceCrawler):
            def get_site_name(self):
                return 'test'

            def get_base_url(self):
                return 'https://test.com/{book_id}'

            def build_book_url(self, book_id):
                return f'https://test.com/{book_id}'

            def parse_book_detail(self, html, book_id):
                return {}

        crawler = TestCrawler()
        html = '<div>No match</div>'
        soup = BeautifulSoup(html, 'html.parser')

        result = crawler.extract_text(soup, '.nonexistent', default='default_value')
        assert result == 'default_value'

    def test_extract_attr(self):
        """Test attribute extraction method."""
        class TestCrawler(BaseSourceCrawler):
            def get_site_name(self):
                return 'test'

            def get_base_url(self):
                return 'https://test.com/{book_id}'

            def build_book_url(self, book_id):
                return f'https://test.com/{book_id}'

            def parse_book_detail(self, html, book_id):
                return {}

        crawler = TestCrawler()
        html = '<img src="https://example.com/image.jpg" />'
        soup = BeautifulSoup(html, 'html.parser')

        result = crawler.extract_attr(soup, 'img', 'src')
        assert result == 'https://example.com/image.jpg'
