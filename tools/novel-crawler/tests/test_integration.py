"""
Integration tests for Phase 3 crawler functionality.
"""
import pytest
import tempfile
from pathlib import Path
from unittest.mock import Mock, patch, MagicMock

from database.models import Book
from database.db_manager import DatabaseManager
from crawlers.youshu_crawler import YoushuCrawler
from utils.http_client import HTTPClient
from utils.image_downloader import ImageDownloader
from crawlers.crawler_manager import CrawlerManager


class TestCrawlerIntegration:
    """Integration tests for crawler components."""

    @pytest.fixture
    def temp_config(self, tmp_path):
        """Create temporary configuration for testing."""
        class MockConfig:
            DATA_DIR = Path(tmp_path)
            COVER_DIR = Path(tmp_path) / 'covers'
            YOUSHU_DB = Path(tmp_path) / 'youshu.db'
            YOUSHU_BASE_URL = "https://www.youshu.me/book/{book_id}"
            REQUEST_DELAY = (0, 0)  # No delay for tests
            MAX_CONSECUTIVE_FAILURES = 3
            REQUEST_TIMEOUT = 10
            MAX_RETRIES = 3
            MAX_IMAGE_SIZE = 1024 * 1024
            IMAGE_TIMEOUT = 10
            USER_AGENTS = ['Test-Agent/1.0']

        return MockConfig()

    @pytest.fixture
    def mock_http_client(self):
        """Create mock HTTP client."""
        client = Mock(spec=HTTPClient)
        return client

    def test_full_crawl_workflow_mocked(self, temp_config, mock_http_client):
        """
        Test full crawl workflow with mocked HTTP responses.

        This test simulates the complete workflow without making real network requests.
        """
        # Mock HTML response
        mock_html = """
        <html>
            <body>
                <h1 class="book-title">测试书籍</h1>
                <div class="author">测试作者</div>
                <div class="book-description">这是一本测试书籍的描述。</div>
                <div class="tag-list">
                    <span class="tag">玄幻</span>
                    <span class="tag">热血</span>
                </div>
                <div class="book-cover">
                    <img src="https://example.com/cover.jpg" />
                </div>
                <div class="book-status">连载中</div>
            </body>
        </html>
        """

        # Mock HTTP response
        mock_response = Mock()
        mock_response.status_code = 200
        mock_response.text = mock_html
        mock_http_client.get.return_value = mock_response

        # Initialize components
        crawler = YoushuCrawler(temp_config, mock_http_client)
        db = DatabaseManager(temp_config)

        # Test 1: Parse book info
        book_info = crawler.parse_book_info(mock_html)
        assert book_info is not None
        assert book_info['title'] == '测试书籍'
        assert book_info['author'] == '测试作者'
        assert book_info['description'] == '这是一本测试书籍的描述。'
        assert book_info['update_status'] == '连载中'

        # Test 2: Save to database
        book_info['id'] = 1
        result = db.save_book(book_info)
        assert result is True

        # Test 3: Retrieve from database
        retrieved = db.get_book(1)
        assert retrieved is not None
        assert retrieved['title'] == '测试书籍'

        print("✓ Full crawl workflow test passed!")

    def test_crawl_batch_workflow(self, temp_config, mock_http_client):
        """Test crawling multiple books in batch."""
        # Mock HTML responses for 3 books
        def mock_get(url):
            book_id = int(url.split('/')[-1])
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.text = f"""
            <html>
                <h1 class="book-title">Book {book_id}</h1>
                <div class="author">Author {book_id}</div>
            </html>
            """
            return mock_response

        mock_http_client.get.side_effect = mock_get

        # Initialize crawler
        crawler = YoushuCrawler(temp_config, mock_http_client)
        db = DatabaseManager(temp_config)

        # Crawl batch
        results = crawler.crawl_batch([1, 2, 3])

        # Verify results
        assert len(results) == 3
        assert results[1] is not None
        assert results[2] is not None
        assert results[3] is not None

        # Verify database
        for book_id in [1, 2, 3]:
            book = db.get_book(book_id)
            assert book is not None
            assert f'Book {book_id}' in book['title']

        print("✓ Batch crawl test passed!")

    def test_image_downloader(self, temp_config):
        """Test image download functionality."""
        # Create a mock image
        mock_image_data = b'\xff\xd8\xff\xe0\x00\x10JFIF'  # JPEG header

        with patch('requests.get') as mock_get:
            mock_response = Mock()
            mock_response.status_code = 200
            mock_response.headers = {'content-type': 'image/jpeg', 'content-length': '100'}
            mock_response.iter_content = lambda chunk_size: [mock_image_data]
            mock_response.raise_for_status = Mock()
            mock_get.return_value = mock_response

            # Test download
            downloader = ImageDownloader(temp_config)
            filepath = downloader.download_cover('https://example.com/cover.jpg', 1)

            # Verify file created
            assert filepath is not None
            assert Path(filepath).exists()

            print("✓ Image download test passed!")

    def test_database_statistics(self, temp_config):
        """Test database statistics functionality."""
        db = DatabaseManager(temp_config)

        # Add test books
        for i in range(1, 6):
            db.save_book({
                'id': i,
                'title': f'Book {i}',
                'author': f'Author {i}',
                'cover_path': f'/covers/{i}.jpg' if i % 2 == 0 else None,
                'source_site': 'qidian' if i % 2 == 0 else None
            })

        # Get statistics
        stats = db.get_statistics()

        assert stats['total_books'] == 5
        assert stats['books_with_covers'] == 2
        assert 'qidian' in stats['by_source']
        assert stats['by_source']['qidian'] == 2

        print("✓ Database statistics test passed!")

    def test_crawl_status_tracking(self, temp_config):
        """Test crawl status update and retrieval."""
        db = DatabaseManager(temp_config)

        # Update crawl status
        result = db.update_crawl_status(
            last_id=100,
            total=95,
            failed_ids=[5, 10, 15],
            crawl_type='initial',
            duration_seconds=3600
        )
        assert result is True

        # Get failed IDs
        failed_ids = db.get_failed_ids()
        assert failed_ids == [5, 10, 15]

        print("✓ Crawl status tracking test passed!")

    def test_search_functionality(self, temp_config):
        """Test book search functionality."""
        db = DatabaseManager(temp_config)

        # Add test books
        db.save_book({'id': 1, 'title': 'Harry Potter', 'author': 'J.K. Rowling'})
        db.save_book({'id': 2, 'title': 'Lord of the Rings', 'author': 'Tolkien'})
        db.save_book({'id': 3, 'title': 'Harry Dresden', 'author': 'Jim Butcher'})

        # Search by title
        results = db.search_books('Harry')
        assert len(results) == 2

        # Search by author
        results = db.search_books('Tolkien')
        assert len(results) == 1
        assert results[0]['title'] == 'Lord of the Rings'

        print("✓ Search functionality test passed!")


def run_integration_tests():
    """Run all integration tests."""
    print("\n" + "=" * 60)
    print("Phase 3 Integration Tests")
    print("=" * 60 + "\n")

    pytest.main([__file__, '-v', '--tb=short'])


if __name__ == '__main__':
    run_integration_tests()
