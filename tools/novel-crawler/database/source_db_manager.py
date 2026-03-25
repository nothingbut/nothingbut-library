"""
Database manager for source site detailed data.

Handles storage and retrieval of detailed book information
from source sites (Qidian, Zongheng, etc.)
"""
import sqlite3
import json
from pathlib import Path
from typing import Optional, Dict, List
from contextlib import contextmanager

from database.models import SourceBookDetail
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class SourceDBManager:
    """
    Manages database operations for source site detailed data.

    Each source site has its own database file (e.g., qidian.db, zongheng.db)
    stored in the data directory.
    """

    def __init__(self, site_name: str, config: settings = None):
        """
        Initialize the source database manager.

        Args:
            site_name: Name of the source site (e.g., 'qidian', 'zongheng')
            config: Configuration object (uses global settings if None)
        """
        self.site_name = site_name.lower()
        self.config = config or settings
        self.db_path = Path(str(self.config.SOURCE_DB_PATTERN).format(source=self.site_name))
        self.db_path.parent.mkdir(parents=True, exist_ok=True)

        # Initialize database
        self._init_database()

    @contextmanager
    def get_connection(self):
        """
        Context manager for database connections.

        Yields:
            sqlite3 connection object
        """
        conn = sqlite3.connect(str(self.db_path))
        conn.row_factory = sqlite3.Row  # Enable column access by name
        try:
            yield conn
        finally:
            conn.close()

    def _init_database(self):
        """Initialize database schema if not exists."""
        with self.get_connection() as conn:
            # Check if table exists
            cursor = conn.cursor()
            cursor.execute("""
                SELECT name FROM sqlite_master
                WHERE type='table' AND name='book_details'
            """)

            if not cursor.fetchone():
                logger.info(f"Initializing database for {self.site_name}")
                self._run_migrations(conn)
                conn.commit()

    def _run_migrations(self, conn: sqlite3.Connection):
        """Run database migrations."""
        # Read migration file
        migration_file = Path(__file__).parent / 'migrations' / '002_init_sources.sql'

        if migration_file.exists():
            with open(migration_file, 'r', encoding='utf-8') as f:
                sql = f.read()
                cursor = conn.cursor()
                cursor.executescript(sql)
                logger.info(f"Applied migration: {migration_file.name}")
        else:
            logger.warning(f"Migration file not found: {migration_file}")

    def save_book_detail(self, youshu_id: int, book_detail: Dict) -> bool:
        """
        Save or update book detail information.

        Args:
            youshu_id: The book ID from youshu.me
            book_detail: Dictionary containing book detail fields

        Returns:
            True if successful, False otherwise
        """
        try:
            detail = SourceBookDetail(
                book_id=book_detail.get('book_id', ''),
                youshu_id=youshu_id,
                title=book_detail.get('title', ''),
                author=book_detail.get('author', ''),
                description=book_detail.get('description', ''),
                category=book_detail.get('category', ''),
                sub_category=book_detail.get('sub_category', ''),
                tags=book_detail.get('tags', []),
                cover_url=book_detail.get('cover_url', ''),
                cover_path=book_detail.get('cover_path', ''),
                word_count=book_detail.get('word_count', 0),
                chapter_count=book_detail.get('chapter_count', 0),
                status=book_detail.get('status', ''),
                rating=book_detail.get('rating', 0.0),
                view_count=book_detail.get('view_count', 0),
                favorite_count=book_detail.get('favorite_count', 0),
            )

            with self.get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("""
                    INSERT OR REPLACE INTO book_details (
                        book_id, youshu_id, title, author, description,
                        category, sub_category, tags, cover_url, cover_path,
                        word_count, chapter_count, status, rating,
                        view_count, favorite_count
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """, (
                    detail.book_id, detail.youshu_id, detail.title,
                    detail.author, detail.description,
                    detail.category, detail.sub_category,
                    json.dumps(detail.tags, ensure_ascii=False),
                    detail.cover_url, detail.cover_path,
                    detail.word_count, detail.chapter_count,
                    detail.status, detail.rating,
                    detail.view_count, detail.favorite_count
                ))
                conn.commit()
                logger.debug(f"Saved book detail: {detail.title} (youshu_id={youshu_id})")
                return True

        except Exception as e:
            logger.error(f"Failed to save book detail: {e}")
            return False

    def get_book_detail(self, youshu_id: int) -> Optional[SourceBookDetail]:
        """
        Retrieve book detail by youshu ID.

        Args:
            youshu_id: The book ID from youshu.me

        Returns:
            SourceBookDetail object or None if not found
        """
        try:
            with self.get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("""
                    SELECT * FROM book_details WHERE youshu_id = ?
                """, (youshu_id,))

                row = cursor.fetchone()
                if row:
                    return SourceBookDetail(
                        book_id=row['book_id'],
                        youshu_id=row['youshu_id'],
                        title=row['title'],
                        author=row['author'],
                        description=row['description'],
                        category=row['category'],
                        sub_category=row['sub_category'],
                        tags=json.loads(row['tags']) if row['tags'] else [],
                        cover_url=row['cover_url'],
                        cover_path=row['cover_path'],
                        word_count=row['word_count'],
                        chapter_count=row['chapter_count'],
                        status=row['status'],
                        rating=row['rating'],
                        view_count=row['view_count'],
                        favorite_count=row['favorite_count'],
                    )
                return None

        except Exception as e:
            logger.error(f"Failed to get book detail: {e}")
            return None

    def get_all_books(self) -> List[SourceBookDetail]:
        """
        Retrieve all book details.

        Returns:
            List of SourceBookDetail objects
        """
        try:
            with self.get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("SELECT * FROM book_details ORDER BY youshu_id")

                books = []
                for row in cursor.fetchall():
                    books.append(SourceBookDetail(
                        book_id=row['book_id'],
                        youshu_id=row['youshu_id'],
                        title=row['title'],
                        author=row['author'],
                        description=row['description'],
                        category=row['category'],
                        sub_category=row['sub_category'],
                        tags=json.loads(row['tags']) if row['tags'] else [],
                        cover_url=row['cover_url'],
                        cover_path=row['cover_path'],
                        word_count=row['word_count'],
                        chapter_count=row['chapter_count'],
                        status=row['status'],
                        rating=row['rating'],
                        view_count=row['view_count'],
                        favorite_count=row['favorite_count'],
                    ))
                return books

        except Exception as e:
            logger.error(f"Failed to get all books: {e}")
            return []

    def search_books(self, keyword: str) -> List[SourceBookDetail]:
        """
        Search books by keyword in title or author.

        Args:
            keyword: Search keyword

        Returns:
            List of matching SourceBookDetail objects
        """
        try:
            with self.get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("""
                    SELECT * FROM book_details
                    WHERE title LIKE ? OR author LIKE ?
                    ORDER BY rating DESC
                """, (f'%{keyword}%', f'%{keyword}%'))

                books = []
                for row in cursor.fetchall():
                    books.append(SourceBookDetail(
                        book_id=row['book_id'],
                        youshu_id=row['youshu_id'],
                        title=row['title'],
                        author=row['author'],
                        description=row['description'],
                        category=row['category'],
                        sub_category=row['sub_category'],
                        tags=json.loads(row['tags']) if row['tags'] else [],
                        cover_url=row['cover_url'],
                        cover_path=row['cover_path'],
                        word_count=row['word_count'],
                        chapter_count=row['chapter_count'],
                        status=row['status'],
                        rating=row['rating'],
                        view_count=row['view_count'],
                        favorite_count=row['favorite_count'],
                    ))
                return books

        except Exception as e:
            logger.error(f"Failed to search books: {e}")
            return []

    def get_statistics(self) -> Dict:
        """
        Get database statistics.

        Returns:
            Dictionary with statistics
        """
        try:
            with self.get_connection() as conn:
                cursor = conn.cursor()

                # Total books
                cursor.execute("SELECT COUNT(*) as count FROM book_details")
                total = cursor.fetchone()['count']

                # Books with covers
                cursor.execute("SELECT COUNT(*) as count FROM book_details WHERE cover_path IS NOT NULL AND cover_path != ''")
                with_covers = cursor.fetchone()['count']

                # Average rating
                cursor.execute("SELECT AVG(rating) as avg_rating FROM book_details WHERE rating > 0")
                avg_rating_row = cursor.fetchone()
                avg_rating = avg_rating_row['avg_rating'] if avg_rating_row['avg_rating'] else 0.0

                # Total word count
                cursor.execute("SELECT SUM(word_count) as total_words FROM book_details")
                total_words = cursor.fetchone()['total_words'] or 0

                # Status distribution
                cursor.execute("""
                    SELECT status, COUNT(*) as count
                    FROM book_details
                    GROUP BY status
                """)
                status_dist = {row['status']: row['count'] for row in cursor.fetchall()}

                return {
                    'site_name': self.site_name,
                    'total_books': total,
                    'books_with_covers': with_covers,
                    'average_rating': round(avg_rating, 2),
                    'total_word_count': total_words,
                    'status_distribution': status_dist,
                }

        except Exception as e:
            logger.error(f"Failed to get statistics: {e}")
            return {}

    def clear_all(self, confirm: bool = False) -> bool:
        """
        Clear all data from the database.

        Args:
            confirm: Must be True to confirm deletion

        Returns:
            True if successful, False otherwise
        """
        if not confirm:
            logger.warning("clear_all called without confirm=True")
            return False

        try:
            with self.get_connection() as conn:
                cursor = conn.cursor()
                cursor.execute("DELETE FROM book_details")
                conn.commit()
                logger.info(f"Cleared all data from {self.site_name} database")
                return True

        except Exception as e:
            logger.error(f"Failed to clear database: {e}")
            return False
