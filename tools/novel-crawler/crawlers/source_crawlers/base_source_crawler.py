"""
Base class for source site crawlers (Qidian, Zongheng, etc.)
"""
from abc import ABC, abstractmethod
from typing import Optional, Dict
import time
import random
from bs4 import BeautifulSoup

from utils.http_client import HTTPClient
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class BaseSourceCrawler(ABC):
    """
    Abstract base class for source site crawlers.

    Source site crawlers fetch detailed information from the original
    novel websites (Qidian, Zongheng, etc.) based on URLs found
    in Youshu.me book index pages.
    """

    def __init__(self, config: settings = None):
        """
        Initialize the source crawler.

        Args:
            config: Configuration object (uses global settings if None)
        """
        self.config = config or settings
        self.site_name = self.get_site_name()

    @abstractmethod
    def get_site_name(self) -> str:
        """Return the site name (e.g., 'qidian', 'zongheng')."""
        pass

    @abstractmethod
    def get_base_url(self) -> str:
        """Return the base URL pattern for this site."""
        pass

    @abstractmethod
    def build_book_url(self, book_id: str) -> str:
        """
        Build the URL for a book page.

        Args:
            book_id: The book ID on the source site

        Returns:
            Full URL to the book page
        """
        pass

    @abstractmethod
    def parse_book_detail(self, html: str, book_id: str) -> Optional[Dict]:
        """
        Parse detailed book information from the source site HTML.

        Args:
            html: HTML content of the book page
            book_id: The book ID on the source site

        Returns:
            Dictionary with book detail fields:
            - title (str): Book title
            - author (str): Author name
            - description (str): Book description/synopsis
            - category (str): Main category
            - sub_category (str): Sub-category
            - tags (list): List of tags
            - cover_url (str): URL to cover image
            - word_count (int): Total word count
            - chapter_count (int): Number of chapters
            - status (str): Publication status (连载/完结)
            - rating (float): Rating score
            - view_count (int): View count
            - favorite_count (int): Favorite/collection count
        """
        pass

    def fetch_page(self, url: str) -> Optional[str]:
        """
        Fetch the HTML content of a page.

        Args:
            url: URL to fetch

        Returns:
            HTML content as string, or None if failed
        """
        try:
            with HTTPClient(self.config) as client:
                html = client.get(url)
                if html:
                    logger.debug(f"Successfully fetched: {url}")
                return html
        except Exception as e:
            logger.error(f"Failed to fetch {url}: {e}")
            return None

    def crawl_book_detail(self, book_id: str) -> Optional[Dict]:
        """
        Complete workflow to crawl book details from source site.

        Args:
            book_id: The book ID on the source site

        Returns:
            Dictionary with book detail information, or None if failed
        """
        url = self.build_book_url(book_id)
        logger.info(f"Crawling {self.site_name} book: {url}")

        # Fetch the page
        html = self.fetch_page(url)
        if not html:
            logger.error(f"Failed to fetch page for book_id={book_id}")
            return None

        # Parse the content
        try:
            book_detail = self.parse_book_detail(html, book_id)
            if book_detail:
                book_detail['site_name'] = self.site_name
                book_detail['book_id'] = book_id
                logger.info(f"Successfully parsed {self.site_name} book: {book_detail.get('title', book_id)}")
                return book_detail
            else:
                logger.warning(f"No data extracted from {url}")
                return None
        except Exception as e:
            logger.error(f"Error parsing {url}: {e}")
            return None
        finally:
            # Random delay to be polite
            self.random_delay()

    def random_delay(self):
        """Add a random delay between requests."""
        delay = random.uniform(*self.config.REQUEST_DELAY)
        logger.debug(f"Delaying for {delay:.2f} seconds")
        time.sleep(delay)

    def extract_text(self, soup: BeautifulSoup, selector: str, default: str = "") -> str:
        """
        Extract text from an element using CSS selector.

        Args:
            soup: BeautifulSoup object
            selector: CSS selector
            default: Default value if element not found

        Returns:
            Extracted text or default value
        """
        element = soup.select_one(selector)
        if element:
            return element.get_text(strip=True)
        return default

    def extract_attr(self, soup: BeautifulSoup, selector: str, attr: str, default: str = "") -> str:
        """
        Extract an attribute from an element using CSS selector.

        Args:
            soup: BeautifulSoup object
            selector: CSS selector
            attr: Attribute name to extract
            default: Default value if element not found

        Returns:
            Extracted attribute value or default value
        """
        element = soup.select_one(selector)
        if element:
            return element.get(attr, default)
        return default

    def extract_number(self, text: str) -> int:
        """
        Extract number from text string (e.g., "123.5万" -> 1235000).

        Args:
            text: Text containing a number

        Returns:
            Extracted number as integer
        """
        if not text:
            return 0

        text = text.strip().replace(',', '')

        # Handle "万" (10,000) suffix
        if '万' in text:
            number = text.replace('万', '')
            try:
                return int(float(number) * 10000)
            except ValueError:
                return 0

        # Handle "万字" suffix
        if '万字' in text:
            number = text.replace('万字', '')
            try:
                return int(float(number) * 10000)
            except ValueError:
                return 0

        # Handle "亿" (100,000,000) suffix
        if '亿' in text:
            number = text.replace('亿', '')
            try:
                return int(float(number) * 100000000)
            except ValueError:
                return 0

        # Handle "亿字" suffix
        if '亿字' in text:
            number = text.replace('亿字', '')
            try:
                return int(float(number) * 100000000)
            except ValueError:
                return 0

        # Extract regular numbers
        import re
        match = re.search(r'\d+', text)
        if match:
            return int(match.group())

        return 0
