"""
Unit tests for extended site crawlers (Batch 1 POC).

Tests for:
- YoudubookCrawler
- XRZWWCrawler
- CDDAOYUECrawler
"""
import pytest
from unittest.mock import Mock, patch

from crawlers.source_crawlers.youdubook_crawler import YoudubookCrawler
from crawlers.source_crawlers.xrzww_crawler import XRZWWCrawler
from crawlers.source_crawlers.cddaoyue_crawler import CDDAOYUECrawler
from config import settings


class TestYoudubookCrawler:
    """Test cases for YoudubookCrawler."""

    @pytest.fixture
    def crawler(self):
        """Create a YoudubookCrawler instance."""
        return YoudubookCrawler()

    def test_site_name(self, crawler):
        """Test site name getter."""
        assert crawler.get_site_name() == 'youdubook'

    def test_base_url(self, crawler):
        """Test base URL getter."""
        assert crawler.get_base_url() == "https://www.youdubook.com/book/{book_id}"

    def test_build_book_url(self, crawler):
        """Test URL building."""
        url = crawler.build_book_url("12345")
        assert url == "https://www.youdubook.com/book/12345"

    def test_parse_book_detail_valid_html(self, crawler):
        """Test parsing valid Youdubook HTML."""
        html = """
        <html>
            <body>
                <div class="book-info">
                    <h1 class="book-title">测试书名</h1>
                    <div class="book-author">测试作者</div>
                    <div class="book-intro">这是一本测试书籍的简介。</div>
                    <div class="book-category">玄幻</div>
                    <div class="word-count">125万</div>
                    <div class="book-status">连载中</div>
                </div>
                <div class="book-cover">
                    <img src="https://example.com/cover.jpg" />
                </div>
            </body>
        </html>
        """

        result = crawler.parse_book_detail(html, "12345")

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

        result = crawler.parse_book_detail(html, "12345")

        # Should return None if no title found
        assert result is None


class TestXRZWWCrawler:
    """Test cases for XRZWWCrawler."""

    @pytest.fixture
    def crawler(self):
        """Create a XRZWWCrawler instance."""
        return XRZWWCrawler()

    def test_site_name(self, crawler):
        """Test site name getter."""
        assert crawler.get_site_name() == 'xrzww'

    def test_base_url(self, crawler):
        """Test base URL getter."""
        assert crawler.get_base_url() == "https://www.xrzww.com/book/{book_id}"

    def test_build_book_url(self, crawler):
        """Test URL building."""
        url = crawler.build_book_url("67890")
        assert url == "https://www.xrzww.com/book/67890"

    def test_parse_book_detail_valid_html(self, crawler):
        """Test parsing valid XRZWW HTML."""
        html = """
        <html>
            <body>
                <div class="book-info">
                    <h1 class="book-title">星人网文测试</h1>
                    <div class="book-author">XRZWW作者</div>
                    <div class="book-intro">这是星人网文的测试书籍。</div>
                    <div class="book-category">科幻</div>
                    <div class="word-count">50万</div>
                    <div class="book-status">完结</div>
                </div>
                <div class="book-cover">
                    <img src="https://example.com/xrzww_cover.jpg" />
                </div>
            </body>
        </html>
        """

        result = crawler.parse_book_detail(html, "67890")

        assert result is not None
        assert result['title'] == '星人网文测试'
        assert result['author'] == 'XRZWW作者'
        assert result['description'] == '这是星人网文的测试书籍。'
        assert result['category'] == '科幻'
        assert result['word_count'] == 500000
        assert result['status'] == '完结'
        assert result['cover_url'] == 'https://example.com/xrzww_cover.jpg'


class TestCDDAOYUECrawler:
    """Test cases for CDDAOYUECrawler."""

    @pytest.fixture
    def crawler(self):
        """Create a CDDAOYUECrawler instance."""
        return CDDAOYUECrawler()

    def test_site_name(self, crawler):
        """Test site name getter."""
        assert crawler.get_site_name() == 'cddaoyue'

    def test_base_url(self, crawler):
        """Test base URL getter."""
        assert crawler.get_base_url() == "https://www.cddaoyue.cn/book/{book_id}"

    def test_build_book_url(self, crawler):
        """Test URL building."""
        url = crawler.build_book_url("11111")
        assert url == "https://www.cddaoyue.cn/book/11111"

    def test_parse_book_detail_valid_html(self, crawler):
        """Test parsing valid CDDAOYUE HTML."""
        html = """
        <html>
            <body>
                <div class="book-info">
                    <h1 class="book-title">次元轻小说</h1>
                    <div class="book-author">次元作者</div>
                    <div class="book-intro">这是一部轻小说作品。</div>
                    <div class="book-category">轻小说</div>
                    <div class="word-count">80万</div>
                    <div class="book-status">连载</div>
                </div>
                <div class="book-cover">
                    <img src="https://example.com/cddaoyue_cover.jpg" />
                </div>
            </body>
        </html>
        """

        result = crawler.parse_book_detail(html, "11111")

        assert result is not None
        assert result['title'] == '次元轻小说'
        assert result['author'] == '次元作者'
        assert result['description'] == '这是一部轻小说作品。'
        assert result['category'] == '轻小说'
        assert result['word_count'] == 800000
        assert result['status'] == '连载'
        assert result['cover_url'] == 'https://example.com/cddaoyue_cover.jpg'


class TestExtendedSitesIntegration:
    """Integration tests for extended sites."""

    def test_all_crawlers_inherit_from_base(self):
        """Test that all new crawlers inherit from BaseSourceCrawler."""
        from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler

        assert issubclass(YoudubookCrawler, BaseSourceCrawler)
        assert issubclass(XRZWWCrawler, BaseSourceCrawler)
        assert issubclass(CDDAOYUECrawler, BaseSourceCrawler)

    def test_all_crawlers_registered(self):
        """Test that all new crawlers are registered in CRAWLER_MAP."""
        from crawlers.source_crawlers import CRAWLER_MAP, supported_sites

        sites = supported_sites()
        assert 'youdubook' in sites
        assert 'xrzww' in sites
        assert 'cddaoyue' in sites

        assert 'youdubook' in CRAWLER_MAP
        assert 'xrzww' in CRAWLER_MAP
        assert 'cddaoyue' in CRAWLER_MAP

    def test_all_crawlers_have_required_methods(self):
        """Test that all new crawlers implement required methods."""
        required_methods = [
            'get_site_name',
            'get_base_url',
            'build_book_url',
            'parse_book_detail'
        ]

        for crawler_class in [YoudubookCrawler, XRZWWCrawler, CDDAOYUECrawler]:
            crawler = crawler_class()
            for method in required_methods:
                assert hasattr(crawler, method), f"{crawler_class.__name__} missing {method}"


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
