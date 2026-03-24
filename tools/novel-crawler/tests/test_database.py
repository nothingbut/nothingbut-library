"""
Unit tests for database module.
"""
import pytest
import tempfile
from pathlib import Path
from datetime import datetime

from database.models import Book, SourceBookDetail, CrawlStatus
from database.db_manager import DatabaseManager
from config import settings


class TestBookModel:
    """Tests for Book model."""

    def test_book_to_dict(self):
        """Test converting book to dictionary."""
        book = Book(
            id=1,
            title="Test Book",
            author="Test Author",
            tags=["fantasy", "action"]
        )
        data = book.to_dict()

        assert data['id'] == 1
        assert data['title'] == "Test Book"
        assert '"fantasy"' in data['tags']  # JSON string

    def test_book_from_dict(self):
        """Test creating book from dictionary."""
        data = {
            'id': 1,
            'title': "Test Book",
            'author': "Test Author",
            'tags': '["fantasy", "action"]',  # JSON string
            'crawled_at': None,
            'updated_at': None
        }
        book = Book.from_dict(data)

        assert book.id == 1
        assert book.title == "Test Book"
        assert book.tags == ["fantasy", "action"]

    def test_book_to_tuple(self):
        """Test converting book to tuple."""
        book = Book(
            id=1,
            title="Test Book",
            tags=["fantasy"]
        )
        tup = book.to_tuple()

        assert tup[0] == 1
        assert tup[1] == "Test Book"
        assert '"fantasy"' in tup[4]  # tags as JSON


class TestDatabaseManager:
    """Tests for DatabaseManager."""

    @pytest.fixture
    def temp_db(self):
        """Create a temporary database for testing."""
        with tempfile.TemporaryDirectory() as tmpdir:
            db_path = Path(tmpdir) / 'test.db'
            # Create a mock config
            class MockConfig:
                DATA_DIR = Path(tmpdir)
                COVER_DIR = Path(tmpdir) / 'covers'
                YOUSHU_DB = db_path

            original_settings = settings
            import config.settings
            config.settings.DATA_DIR = MockConfig.DATA_DIR
            config.settings.COVER_DIR = MockConfig.COVER_DIR
            config.settings.YOUSHU_DB = MockConfig.YOUSHU_DB

            db_manager = DatabaseManager(MockConfig())

            yield db_manager

            # Restore settings
            config.settings.DATA_DIR = original_settings.DATA_DIR
            config.settings.COVER_DIR = original_settings.COVER_DIR
            config.settings.YOUSHU_DB = original_settings.YOUSHU_DB

    def test_database_initialization(self, temp_db):
        """Test that database and tables are created."""
        assert temp_db.db_path.exists()

    def test_save_book(self, temp_db):
        """Test saving a book to database."""
        book_info = {
            'id': 1,
            'title': 'Test Book',
            'author': 'Test Author',
            'description': 'Test description',
            'tags': '["fantasy"]'
        }
        result = temp_db.save_book(book_info)
        assert result is True

    def test_get_book(self, temp_db):
        """Test retrieving a book from database."""
        book_info = {
            'id': 1,
            'title': 'Test Book',
            'author': 'Test Author'
        }
        temp_db.save_book(book_info)

        retrieved = temp_db.get_book(1)
        assert retrieved is not None
        assert retrieved['title'] == 'Test Book'

    def test_get_last_valid_id(self, temp_db):
        """Test getting last valid ID."""
        assert temp_db.get_last_valid_id() == 0

        temp_db.save_book({'id': 5, 'title': 'Book 5'})
        temp_db.save_book({'id': 10, 'title': 'Book 10'})

        assert temp_db.get_last_valid_id() == 10

    def test_get_total_books(self, temp_db):
        """Test getting total book count."""
        assert temp_db.get_total_books() == 0

        temp_db.save_book({'id': 1, 'title': 'Book 1'})
        temp_db.save_book({'id': 2, 'title': 'Book 2'})

        assert temp_db.get_total_books() == 2

    def test_update_crawl_status(self, temp_db):
        """Test updating crawl status."""
        result = temp_db.update_crawl_status(
            last_id=100,
            total=95,
            failed_ids=[5, 10, 15],
            crawl_type='initial',
            duration_seconds=3600
        )
        assert result is True

    def test_get_failed_ids(self, temp_db):
        """Test getting failed IDs."""
        temp_db.update_crawl_status(
            last_id=100,
            total=97,
            failed_ids=[5, 10, 20]
        )

        failed = temp_db.get_failed_ids()
        assert failed == [5, 10, 20]

    def test_search_books(self, temp_db):
        """Test searching books."""
        temp_db.save_book({'id': 1, 'title': 'Harry Potter', 'author': 'J.K. Rowling'})
        temp_db.save_book({'id': 2, 'title': 'Lord of the Rings', 'author': 'Tolkien'})

        results = temp_db.search_books('Harry')
        assert len(results) == 1
        assert results[0]['title'] == 'Harry Potter'

        results = temp_db.search_books('Tolkien')
        assert len(results) == 1
        assert results[0]['author'] == 'Tolkien'

    def test_get_statistics(self, temp_db):
        """Test getting database statistics."""
        temp_db.save_book({'id': 1, 'title': 'Book 1', 'cover_path': '/covers/1.jpg'})
        temp_db.save_book({'id': 2, 'title': 'Book 2', 'source_site': 'qidian'})

        stats = temp_db.get_statistics()
        assert stats['total_books'] == 2
        assert stats['books_with_covers'] == 1
