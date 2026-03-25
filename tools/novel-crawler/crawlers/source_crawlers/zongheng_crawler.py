"""
Zongheng (纵横中文网) crawler for fetching detailed book information.

Zongheng book pages typically have structure like:
- Title in .book-info .book-name or .title
- Author in .book-info .author
- Description in .book-intro or .summary
- Category in .book-info .category
- Cover in .book-img img
- Statistics in .book-info .stat-item
"""
from typing import Optional, Dict
from bs4 import BeautifulSoup
import re

from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class ZonghengCrawler(BaseSourceCrawler):
    """
    Crawler for Zongheng (纵横中文网) book detail pages.

    Extracts comprehensive book information including:
    - Basic metadata (title, author, description)
    - Category and tags
    - Statistics (word count, views, favorites)
    - Publication status
    - Cover image
    """

    def get_site_name(self) -> str:
        return 'zongheng'

    def get_base_url(self) -> str:
        return "http://book.zongheng.com/book/{book_id}.html"

    def build_book_url(self, book_id: str) -> str:
        """Build Zongheng URL for the given book ID."""
        return self.get_base_url().format(book_id=book_id)

    def parse_book_detail(self, html: str, book_id: str) -> Optional[Dict]:
        """
        Parse book information from Zongheng HTML page.

        Uses multiple selector strategies to handle page layout changes.

        Args:
            html: HTML content of the book page
            book_id: The book ID on Zongheng

        Returns:
            Dictionary with book detail information
        """
        try:
            soup = BeautifulSoup(html, 'html.parser')

            book_detail = {}

            # Title - Multiple possible selectors
            book_detail['title'] = (
                self._extract_text_multi(soup, [
                    '.book-info .book-name',
                    '.book-title',
                    'h1.title',
                    '.detail h1'
                ]) or f"Book {book_id}"
            )

            # Author
            book_detail['author'] = (
                self._extract_text_multi(soup, [
                    '.book-info .author',
                    '.book-info .writer a',
                    '.author-name',
                    '.detail .author'
                ])
            )

            # Description
            book_detail['description'] = (
                self._extract_text_multi(soup, [
                    '.book-intro',
                    '.book-info .intro',
                    '.detail .summary',
                    '.book-summary',
                    '.intro-detail'
                ])
            )

            # Category
            book_detail['category'] = (
                self._extract_text_multi(soup, [
                    '.book-info .category a',
                    '.book-info .type',
                    '.detail .category',
                    '.book-category a'
                ])
            )

            # Sub-category
            book_detail['sub_category'] = (
                self._extract_text_multi(soup, [
                    '.book-info .sub-category',
                    '.book-info .category a:nth-child(2)',
                    '.sub-type'
                ])
            )

            # Tags
            book_detail['tags'] = self._extract_tags(soup)

            # Cover URL
            book_detail['cover_url'] = (
                self._extract_attr_multi(soup, [
                    '.book-img img',
                    '.book-cover img',
                    '.detail img',
                    '.book-img-cover img'
                ], 'src')
            )

            # Word count - Zongheng often uses "万字"
            word_count_text = (
                self._extract_text_multi(soup, [
                    '.book-info .word-count',
                    '.book-info .stat-item:contains("字")',
                    '.detail .word-count',
                    '.book-stat span:contains("万字")'
                ])
            )
            book_detail['word_count'] = self.extract_number(word_count_text)

            # Chapter count
            chapter_count_text = (
                self._extract_text_multi(soup, [
                    '.book-info .chapter-count',
                    '.book-info .stat-item:contains("章")',
                    '.detail .chapter-count'
                ])
            )
            book_detail['chapter_count'] = self.extract_number(chapter_count_text)

            # Status (连载/完结)
            book_detail['status'] = (
                self._extract_text_multi(soup, [
                    '.book-info .status',
                    '.book-info .state',
                    '.detail .book-status',
                    '.book-state'
                ])
            )
            # Normalize status
            if book_detail['status']:
                if '完结' in book_detail['status'] or '完成' in book_detail['status']:
                    book_detail['status'] = '完结'
                elif '连载' in book_detail['status']:
                    book_detail['status'] = '连载'

            # Rating
            rating_text = (
                self._extract_text_multi(soup, [
                    '.book-info .score',
                    '.rating-score',
                    '.detail .rating'
                ])
            )
            book_detail['rating'] = self._extract_rating(rating_text)

            # View count
            view_count_text = (
                self._extract_text_multi(soup, [
                    '.book-info .view-count',
                    '.book-info .stat-item:contains("次")',
                    '.detail .views'
                ])
            )
            book_detail['view_count'] = self.extract_number(view_count_text)

            # Favorite/collection count
            favorite_count_text = (
                self._extract_text_multi(soup, [
                    '.book-info .favorite-count',
                    '.book-info .stat-item:contains("票")',
                    '.detail .favorites'
                ])
            )
            book_detail['favorite_count'] = self.extract_number(favorite_count_text)

            # Validate that we got at least some data
            if not book_detail.get('title') or book_detail['title'] == f"Book {book_id}":
                logger.warning(f"Could not extract title from Zongheng book {book_id}")
                return None

            return book_detail

        except Exception as e:
            logger.error(f"Error parsing Zongheng book {book_id}: {e}")
            return None

    def _extract_text_multi(self, soup: BeautifulSoup, selectors: list) -> str:
        """
        Try multiple CSS selectors until one returns content.

        Args:
            soup: BeautifulSoup object
            selectors: List of CSS selectors to try

        Returns:
            Extracted text or empty string
        """
        for selector in selectors:
            try:
                element = soup.select_one(selector)
                if element:
                    text = element.get_text(strip=True)
                    if text:
                        return text
            except Exception:
                continue
        return ""

    def _extract_attr_multi(self, soup: BeautifulSoup, selectors: list, attr: str) -> str:
        """
        Try multiple CSS selectors to extract an attribute.

        Args:
            soup: BeautifulSoup object
            selectors: List of CSS selectors to try
            attr: Attribute name to extract

        Returns:
            Extracted attribute value or empty string
        """
        for selector in selectors:
            try:
                element = soup.select_one(selector)
                if element:
                    value = element.get(attr, '')
                    if value:
                        return value
            except Exception:
                continue
        return ""

    def _extract_tags(self, soup: BeautifulSoup) -> list:
        """
        Extract tags from the page.

        Args:
            soup: BeautifulSoup object

        Returns:
            List of tag strings
        """
        tags = []

        # Try different tag container selectors
        tag_containers = [
            '.book-info .tags span',
            '.tag-list .tag',
            '.detail .tags a',
            '.book-labels a',
            '.book-tags a'
        ]

        for container_selector in tag_containers:
            try:
                tag_elements = soup.select(container_selector)
                if tag_elements:
                    tags = [tag.get_text(strip=True) for tag in tag_elements if tag.get_text(strip=True)]
                    if tags:
                        break
            except Exception:
                continue

        return tags

    def _extract_rating(self, text: str) -> float:
        """
        Extract rating score from text.

        Args:
            text: Text containing rating (e.g., "9.5分", "9.5")

        Returns:
            Rating as float, or 0.0 if not found
        """
        if not text:
            return 0.0

        # Extract decimal number
        match = re.search(r'(\d+\.?\d*)', text)
        if match:
            try:
                return float(match.group(1))
            except ValueError:
                pass

        return 0.0
