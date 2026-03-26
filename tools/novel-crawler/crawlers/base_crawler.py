"""
Base crawler class providing common functionality for all crawlers.
"""
from abc import ABC, abstractmethod
from typing import Optional, Dict
import time
import random

from utils.logger import get_logger

logger = get_logger(__name__)


class BaseCrawler(ABC):
    """
    Abstract base class for all web crawlers.

    Provides common functionality:
    - URL building
    - Page fetching
    - Random delays
    - HTML parsing interface
    """

    def __init__(self, config, http_client):
        """
        Initialize base crawler.

        Args:
            config: Configuration object
            http_client: HTTP client instance
        """
        self.config = config
        self.http_client = http_client
        self.logger = logger

    @abstractmethod
    def build_url(self, book_id: int) -> str:
        """
        Build URL for the given book ID.

        Args:
            book_id: Book identifier

        Returns:
            Full URL to the book page
        """
        pass

    @abstractmethod
    def parse_book_info(self, html: str) -> Optional[Dict]:
        """
        Parse book information from HTML content.

        Args:
            html: HTML content as string

        Returns:
            Dictionary with book information, or None if parsing fails
        """
        pass

    def fetch_page(self, url: str) -> Optional[str]:
        """
        Fetch page content from URL.

        Args:
            url: URL to fetch

        Returns:
            HTML content as string, or None if request fails
        """
        self.logger.debug(f"Fetching page: {url}")

        response = self.http_client.get(url)
        if response and response.status_code == 200:
            return response.text

        return None

    def crawl_book(self, book_id: int) -> Optional[Dict]:
        """
        Crawl a single book by ID.

        This is the main entry point for crawling a book.
        Implements the template method pattern.

        Args:
            book_id: Book identifier

        Returns:
            Dictionary with book information, or None if crawling fails
        """
        try:
            # Build URL
            url = self.build_url(book_id)
            self.logger.debug(f"Crawling book {book_id}: {url}")

            # Fetch page
            html = self.fetch_page(url)
            if not html:
                self.logger.warning(f"Failed to fetch page for book {book_id}")
                return None

            # Parse book information
            book_info = self.parse_book_info(html)
            if not book_info:
                self.logger.warning(f"Failed to parse book {book_id}")
                return None

            # Add book_id to the result
            book_info['id'] = book_id

            self.logger.info(f"Successfully crawled book {book_id}: {book_info.get('title')}")
            return book_info

        except Exception as e:
            self.logger.error(f"Error crawling book {book_id}: {e}")
            return None

        finally:
            # Apply random delay after each request
            self.random_delay()

    def random_delay(self):
        """
        Apply random delay between requests.
        """
        min_delay, max_delay = self.config.REQUEST_DELAY
        delay = random.uniform(min_delay, max_delay)
        self.logger.debug(f"Delaying for {delay:.2f} seconds")
        time.sleep(delay)

    def crawl_batch(self, book_ids: list) -> Dict[int, Optional[Dict]]:
        """
        Crawl multiple books in batch.

        Args:
            book_ids: List of book IDs to crawl

        Returns:
            Dictionary mapping book_id to book_info (or None if failed)
        """
        results = {}

        for book_id in book_ids:
            results[book_id] = self.crawl_book(book_id)

        # Log summary
        success_count = sum(1 for v in results.values() if v is not None)
        self.logger.info(
            f"Batch crawl completed: {success_count}/{len(book_ids)} successful"
        )

        return results
