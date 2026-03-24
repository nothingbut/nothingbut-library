"""
Database manager for handling all database operations.
"""
import sqlite3
import json
from pathlib import Path
from typing import Optional, Dict, List
from datetime import datetime
from contextlib import contextmanager

from config import settings
from database.models import Book, CrawlStatus
from utils.logger import get_logger

logger = get_logger(__name__)


@contextmanager
def get_db_connection(db_path: Path):
    """
    Context manager for database connections.

    Args:
        db_path: Path to database file

    Yields:
        SQLite connection
    """
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    try:
        yield conn
    finally:
        conn.close()


class DatabaseManager:
    """
    Manager for main database (youshu.db) operations.
    """

    def __init__(self, config=None):
        """
        Initialize database manager.

        Args:
            config: Configuration object (uses settings if None)
        """
        self.config = config or settings
        self.db_path = self.config.YOUSHU_DB
        self._ensure_directories()
        self._init_database()

    def _ensure_directories(self):
        """Create necessary directories."""
        self.config.DATA_DIR.mkdir(parents=True, exist_ok=True)
        self.config.COVER_DIR.mkdir(parents=True, exist_ok=True)

    def _init_database(self):
        """Initialize database with tables and indexes."""
        with get_db_connection(self.db_path) as conn:
            cursor = conn.cursor()

            # Books table
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

            # Crawl status table
            cursor.execute('''
                CREATE TABLE IF NOT EXISTS crawl_status (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    last_valid_id INTEGER NOT NULL,
                    last_crawl_date DATETIME DEFAULT CURRENT_TIMESTAMP,
                    total_books INTEGER DEFAULT 0,
                    failed_ids TEXT,
                    crawl_type TEXT,
                    duration_seconds INTEGER,
                    success_count INTEGER DEFAULT 0,
                    failure_count INTEGER DEFAULT 0
                )
            ''')

            # Indexes for better query performance
            cursor.execute('CREATE INDEX IF NOT EXISTS idx_author ON books(author)')
            cursor.execute('CREATE INDEX IF NOT EXISTS idx_source ON books(source_site)')
            cursor.execute('CREATE INDEX IF NOT EXISTS idx_status ON books(update_status)')
            cursor.execute('CREATE INDEX IF NOT EXISTS idx_title ON books(title)')

            conn.commit()

    def save_book(self, book_info: Dict) -> bool:
        """
        Save or update a book in the database.

        Args:
            book_info: Dictionary containing book information

        Returns:
            True if successful, False otherwise
        """
        try:
            with get_db_connection(self.db_path) as conn:
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
                    datetime.now().isoformat()
                ))

                conn.commit()
                logger.debug(f"Saved book {book_info.get('id')}: {book_info.get('title')}")
                return True

        except Exception as e:
            logger.error(f"Error saving book: {e}")
            return False

    def get_book(self, book_id: int) -> Optional[Dict]:
        """
        Retrieve a book by ID.

        Args:
            book_id: Book ID

        Returns:
            Book dictionary or None if not found
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()
                cursor.execute('SELECT * FROM books WHERE id = ?', (book_id,))
                row = cursor.fetchone()

                if row:
                    return dict(row)
                return None

        except Exception as e:
            logger.error(f"Error retrieving book {book_id}: {e}")
            return None

    def get_last_valid_id(self) -> int:
        """
        Get the last valid book ID from the database.

        Returns:
            Last valid ID, or 0 if database is empty
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()
                cursor.execute('SELECT MAX(id) FROM books')
                result = cursor.fetchone()
                return result[0] or 0
        except Exception as e:
            logger.error(f"Error getting last valid ID: {e}")
            return 0

    def get_total_books(self) -> int:
        """
        Get total number of books in database.

        Returns:
            Total count
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()
                cursor.execute('SELECT COUNT(*) FROM books')
                result = cursor.fetchone()
                return result[0]
        except Exception as e:
            logger.error(f"Error getting total books: {e}")
            return 0

    def update_crawl_status(
        self,
        last_id: int,
        total: int,
        failed_ids: List[int],
        crawl_type: str = "initial",
        duration_seconds: int = 0
    ) -> bool:
        """
        Update crawl status in database.

        Args:
            last_id: Last valid book ID
            total: Total books crawled
            failed_ids: List of failed book IDs
            crawl_type: Type of crawl (initial, incremental, retry)
            duration_seconds: Duration of crawl in seconds

        Returns:
            True if successful, False otherwise
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()

                cursor.execute('''
                    INSERT INTO crawl_status
                    (last_valid_id, total_books, failed_ids, crawl_type,
                     duration_seconds, success_count, failure_count)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                ''', (
                    last_id,
                    total,
                    json.dumps(failed_ids),
                    crawl_type,
                    duration_seconds,
                    total - len(failed_ids),
                    len(failed_ids)
                ))

                conn.commit()
                return True

        except Exception as e:
            logger.error(f"Error updating crawl status: {e}")
            return False

    def get_failed_ids(self) -> List[int]:
        """
        Get list of failed book IDs from latest crawl status.

        Returns:
            List of failed IDs
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()
                cursor.execute('''
                    SELECT failed_ids FROM crawl_status
                    ORDER BY id DESC LIMIT 1
                ''')
                row = cursor.fetchone()

                if row and row[0]:
                    return json.loads(row[0])
                return []

        except Exception as e:
            logger.error(f"Error getting failed IDs: {e}")
            return []

    def search_books(self, query: str, limit: int = 10) -> List[Dict]:
        """
        Search books by title or author.

        Args:
            query: Search query
            limit: Maximum results

        Returns:
            List of matching books
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()
                cursor.execute('''
                    SELECT * FROM books
                    WHERE title LIKE ? OR author LIKE ?
                    LIMIT ?
                ''', (f'%{query}%', f'%{query}%', limit))

                rows = cursor.fetchall()
                return [dict(row) for row in rows]

        except Exception as e:
            logger.error(f"Error searching books: {e}")
            return []

    def clear_database(self, confirm: bool = False) -> bool:
        """
        Clear all books and crawl status from database.

        Args:
            confirm: Must be True to confirm the operation

        Returns:
            True if successful, False otherwise
        """
        if not confirm:
            logger.error("Clear operation not confirmed. Set confirm=True to proceed.")
            return False

        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()

                # Get statistics before clearing
                cursor.execute('SELECT COUNT(*) FROM books')
                total_books = cursor.fetchone()[0]

                logger.warning(f"Clearing {total_books} books from database...")

                # Clear tables
                cursor.execute('DELETE FROM books')
                cursor.execute('DELETE FROM crawl_status')

                conn.commit()
                logger.info("Database cleared successfully")

                return True

        except Exception as e:
            logger.error(f"Error clearing database: {e}")
            return False

    def clear_covers(self, confirm: bool = False) -> bool:
        """
        Delete all cover images from cover directory.

        Args:
            confirm: Must be True to confirm the operation

        Returns:
            True if successful, False otherwise
        """
        if not confirm:
            logger.error("Clear operation not confirmed. Set confirm=True to proceed.")
            return False

        try:
            cover_dir = self.config.COVER_DIR
            if not cover_dir.exists():
                logger.info("Cover directory does not exist")
                return True

            # Count files
            cover_files = list(cover_dir.glob('*.jpg')) + list(cover_dir.glob('*.png'))
            logger.warning(f"Deleting {len(cover_files)} cover images...")

            # Delete files
            for cover_file in cover_files:
                cover_file.unlink()

            logger.info("Cover directory cleared successfully")
            return True

        except Exception as e:
            logger.error(f"Error clearing covers: {e}")
            return False

    def clear_all(self, confirm: bool = False) -> bool:
        """
        Clear both database and covers.

        Args:
            confirm: Must be True to confirm the operation

        Returns:
            True if successful, False otherwise
        """
        if not confirm:
            logger.error("Clear operation not confirmed. Set confirm=True to proceed.")
            return False

        logger.warning("=" * 60)
        logger.warning("⚠️  CLEARING ALL DATA")
        logger.warning("=" * 60)

        success = True

        # Clear covers first
        if not self.clear_covers(confirm=True):
            success = False

        # Then clear database
        if not self.clear_database(confirm=True):
            success = False

        if success:
            logger.warning("=" * 60)
            logger.warning("✓ All data cleared successfully")
            logger.warning("=" * 60)
        else:
            logger.error("Some errors occurred during clearing")

        return success

    def get_statistics(self) -> Dict:
        """
        Get database statistics.

        Returns:
            Dictionary with statistics
        """
        try:
            with get_db_connection(self.db_path) as conn:
                cursor = conn.cursor()

                stats = {}

                # Total books
                cursor.execute('SELECT COUNT(*) FROM books')
                stats['total_books'] = cursor.fetchone()[0]

                # Books with covers
                cursor.execute('SELECT COUNT(*) FROM books WHERE cover_path IS NOT NULL')
                stats['books_with_covers'] = cursor.fetchone()[0]

                # Books by source
                cursor.execute('''
                    SELECT source_site, COUNT(*) as count
                    FROM books
                    WHERE source_site IS NOT NULL
                    GROUP BY source_site
                ''')
                stats['by_source'] = {row[0]: row[1] for row in cursor.fetchall()}

                # Last crawl info
                cursor.execute('''
                    SELECT * FROM crawl_status
                    ORDER BY id DESC LIMIT 1
                ''')
                row = cursor.fetchone()
                if row:
                    stats['last_crawl'] = dict(row)

                return stats

        except Exception as e:
            logger.error(f"Error getting statistics: {e}")
            return {}
