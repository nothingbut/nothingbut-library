"""
Manager for coordinating source site crawlers.

Handles the workflow of:
1. Getting source URLs from youshu books
2. Extracting book IDs from source URLs
3. Crawling detailed info from source sites
4. Saving to source databases
"""
from typing import Optional, Dict, List
from urllib.parse import urlparse
import re

from crawlers.source_crawlers import get_crawler, supported_sites
from database.source_db_manager import SourceDBManager
from database.db_manager import DatabaseManager
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class SourceCrawlerManager:
    """
    Manager for coordinating source site crawling operations.

    This manager:
    - Reads books from youshu database
    - Extracts source URLs and book IDs
    - Crawls detailed info from source sites
    - Saves to separate source databases
    """

    def __init__(self, config: settings = None):
        """
        Initialize the source crawler manager.

        Args:
            config: Configuration object (uses global settings if None)
        """
        self.config = config or settings
        self.youshu_db = DatabaseManager(self.config)
        self.source_dbs = {}  # site_name -> SourceDBManager

    def get_source_db(self, site_name: str) -> SourceDBManager:
        """
        Get or create database manager for a source site.

        Args:
            site_name: Name of the source site

        Returns:
            SourceDBManager instance
        """
        if site_name not in self.source_dbs:
            self.source_dbs[site_name] = SourceDBManager(site_name, self.config)
        return self.source_dbs[site_name]

    def extract_book_id_from_url(self, url: str, site_name: str) -> Optional[str]:
        """
        Extract book ID from source site URL.

        Args:
            url: Source site URL
            site_name: Name of the source site

        Returns:
            Book ID as string, or None if not found
        """
        if not url:
            return None

        try:
            # Qidian URLs: https://book.qidian.com/info/123456
            if site_name == 'qidian':
                match = re.search(r'/info/(\d+)', url)
                if match:
                    return match.group(1)

            # Zongheng URLs: http://book.zongheng.com/book/12345.html
            elif site_name == 'zongheng':
                match = re.search(r'/book/(\d+)\.html', url)
                if match:
                    return match.group(1)

            logger.debug(f"Could not extract book ID from {url}")
            return None

        except Exception as e:
            logger.error(f"Error extracting book ID from {url}: {e}")
            return None

    def crawl_source_for_book(self, youshu_id: int) -> Dict:
        """
        Crawl source site details for a single youshu book.

        Args:
            youshu_id: The book ID from youshu.me

        Returns:
            Dictionary with result:
            - success (bool): Whether crawl was successful
            - site_name (str): Name of source site
            - book_id (str): Book ID on source site
            - title (str): Book title
        """
        # Get book info from youshu database
        book = self.youshu_db.get_book(youshu_id)
        if not book:
            return {
                'success': False,
                'error': f'Book {youshu_id} not found in youshu database'
            }

        # Get source site and URL
        source_site = book.source_site
        source_url = book.source_url

        if not source_site or not source_url:
            return {
                'success': False,
                'error': f'Book {youshu_id} has no source information'
            }

        # Check if site is supported
        if source_site.lower() not in supported_sites():
            return {
                'success': False,
                'error': f'Unsupported source site: {source_site}'
            }

        # Extract book ID from URL
        source_book_id = self.extract_book_id_from_url(source_url, source_site.lower())
        if not source_book_id:
            return {
                'success': False,
                'error': f'Could not extract book ID from URL: {source_url}'
            }

        # Get crawler and crawl
        try:
            crawler = get_crawler(source_site)
            book_detail = crawler.crawl_book_detail(source_book_id)

            if book_detail:
                # Save to source database
                source_db = self.get_source_db(source_site)
                source_db.save_book_detail(youshu_id, book_detail)

                logger.info(f"Successfully crawled {source_site} details for youshu_id={youshu_id}")

                return {
                    'success': True,
                    'site_name': source_site,
                    'book_id': source_book_id,
                    'title': book_detail.get('title', ''),
                }
            else:
                return {
                    'success': False,
                    'error': f'Failed to parse book detail from {source_site}'
                }

        except Exception as e:
            logger.error(f"Error crawling {source_site} for youshu_id={youshu_id}: {e}")
            return {
                'success': False,
                'error': str(e)
            }

    def crawl_batch(self, youshu_ids: List[int]) -> Dict:
        """
        Crawl source site details for multiple youshu books.

        Args:
            youshu_ids: List of youshu book IDs

        Returns:
            Dictionary with statistics:
            - total (int): Total number of books
            - success (int): Number of successful crawls
            - failed (int): Number of failed crawls
            - by_site (dict): Success/failure counts by site
        """
        stats = {
            'total': len(youshu_ids),
            'success': 0,
            'failed': 0,
            'by_site': {}
        }

        logger.info(f"Starting source site crawl for {len(youshu_ids)} books")

        for youshu_id in youshu_ids:
            result = self.crawl_source_for_book(youshu_id)

            if result['success']:
                stats['success'] += 1
                site_name = result['site_name']

                if site_name not in stats['by_site']:
                    stats['by_site'][site_name] = {'success': 0, 'failed': 0}

                stats['by_site'][site_name]['success'] += 1
            else:
                stats['failed'] += 1
                logger.warning(f"Failed to crawl youshu_id={youshu_id}: {result.get('error', 'Unknown error')}")

                # Try to determine site for failed attempts
                book = self.youshu_db.get_book(youshu_id)
                if book and book.source_site:
                    site_name = book.source_site
                    if site_name not in stats['by_site']:
                        stats['by_site'][site_name] = {'success': 0, 'failed': 0}
                    stats['by_site'][site_name]['failed'] += 1

        logger.info(f"Source crawl complete: {stats['success']}/{stats['total']} successful")
        return stats

    def crawl_all_books(self, limit: Optional[int] = None) -> Dict:
        """
        Crawl source site details for all books in youshu database.

        Args:
            limit: Maximum number of books to crawl (None for all)

        Returns:
            Dictionary with statistics
        """
        # Get all youshu book IDs
        all_books = self.youshu_db.get_all_books()
        youshu_ids = [book.id for book in all_books]

        if limit:
            youshu_ids = youshu_ids[:limit]

        return self.crawl_batch(youshu_ids)

    def get_source_statistics(self) -> Dict:
        """
        Get statistics from all source databases.

        Returns:
            Dictionary with statistics by site
        """
        stats = {}

        for site_name in supported_sites():
            source_db = self.get_source_db(site_name)
            site_stats = source_db.get_statistics()
            stats[site_name] = site_stats

        return stats

    def close(self):
        """Close all database connections."""
        self.source_dbs.clear()
        # DatabaseManager doesn't have a close method, it uses context managers
        self.youshu_db = None

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
