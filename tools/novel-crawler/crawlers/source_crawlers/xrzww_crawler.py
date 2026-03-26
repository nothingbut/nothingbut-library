"""
XRZWW (星人网文) crawler for fetching detailed book information.

XRZWW is a small-scale novel platform with simple structure.
"""
from typing import Optional, Dict
from bs4 import BeautifulSoup
import re

from crawlers.source_crawlers.base_source_crawler import BaseSourceCrawler
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class XRZWWCrawler(BaseSourceCrawler):
    """
    Crawler for XRZWW (星人网文) book detail pages.

    Extracts comprehensive book information including:
    - Basic metadata (title, author, description)
    - Category and tags
    - Statistics (word count, views, favorites)
    - Publication status
    - Cover image
    """

    def get_site_name(self) -> str:
        return 'xrzww'

    def get_base_url(self) -> str:
        return "https://www.xrzww.com/book/{book_id}"

    def build_book_url(self, book_id: str) -> str:
        """Build XRZWW URL for the given book ID."""
        return self.get_base_url().format(book_id=book_id)

    def parse_book_detail(self, html: str, book_id: str) -> Optional[Dict]:
        """
        Parse book information from XRZWW HTML page.

        Args:
            html: HTML content of the book page
            book_id: The book ID on XRZWW

        Returns:
            Dictionary with book detail information
        """
        try:
            soup = BeautifulSoup(html, 'html.parser')

            book_detail = {}

            # Title - Multiple possible selectors
            book_detail['title'] = (
                self._extract_text_multi(soup, [
                    'h1.book-title',
                    '.book-info h1',
                    '.detail h1',
                    'h1.title'
                ]) or f"Book {book_id}"
            )

            # Author
            book_detail['author'] = (
                self._extract_text_multi(soup, [
                    '.book-author',
                    '.author-name',
                    '.writer',
                    '.detail .author'
                ])
            )

            # Description
            book_detail['description'] = (
                self._extract_text_multi(soup, [
                    '.book-intro',
                    '.description',
                    '.book-info .intro',
                    '.detail .summary'
                ])
            )

            # Category
            book_detail['category'] = (
                self._extract_text_multi(soup, [
                    '.book-category',
                    '.category a',
                    '.book-info .type',
                    '.detail .category'
                ])
            )

            # Sub-category
            book_detail['sub_category'] = (
                self._extract_text_multi(soup, [
                    '.book-sub-category',
                    '.sub-category',
                    '.book-info .sub-type'
                ])
            )

            # Tags
            book_detail['tags'] = self._extract_tags(soup)

            # Cover URL
            book_detail['cover_url'] = (
                self._extract_attr_multi(soup, [
                    '.book-cover img',
                    '.cover-img',
                    '.detail img',
                    '.book-img img'
                ], 'src')
            )

            # Word count
            word_count_text = (
                self._extract_text_multi(soup, [
                    '.word-count',
                    '.book-info .word-count',
                    '.detail .words'
                ])
            )
            book_detail['word_count'] = self.extract_number(word_count_text)

            # Chapter count
            chapter_count_text = (
                self._extract_text_multi(soup, [
                    '.chapter-count',
                    '.book-info .chapter-count',
                    '.detail .chapters'
                ])
            )
            book_detail['chapter_count'] = self.extract_number(chapter_count_text)

            # Status
            book_detail['status'] = (
                self._extract_text_multi(soup, [
                    '.book-status',
                    '.status',
                    '.book-info .state',
                    '.detail .book-status'
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
                    '.book-rating',
                    '.rating-score',
                    '.detail .rating'
                ])
            )
            book_detail['rating'] = self._extract_rating(rating_text)

            # View count
            view_count_text = (
                self._extract_text_multi(soup, [
                    '.view-count',
                    '.book-info .views',
                    '.detail .view-count'
                ])
            )
            book_detail['view_count'] = self.extract_number(view_count_text)

            # Favorite/collection count
            favorite_count_text = (
                self._extract_text_multi(soup, [
                    '.favorite-count',
                    '.book-info .favorites',
                    '.detail .favorite-count'
                ])
            )
            book_detail['favorite_count'] = self.extract_number(favorite_count_text)

            # Validate that we got at least some data
            if not book_detail.get('title') or book_detail['title'] == f"Book {book_id}":
                logger.warning(f"Could not extract title from XRZWW book {book_id}")
                return None

            return book_detail

        except Exception as e:
            logger.error(f"Error parsing XRZWW book {book_id}: {e}")
            return None

    def _extract_text_multi(self, soup: BeautifulSoup, selectors: list) -> str:
        """Try multiple CSS selectors until one returns content."""
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
        """Try multiple CSS selectors to extract an attribute."""
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
        """Extract tags from the page."""
        tags = []

        tag_containers = [
            '.book-tags span',
            '.tag-list .tag',
            '.detail .tags a',
            '.book-labels a',
            '.tags a'
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
        """Extract rating score from text."""
        if not text:
            return 0.0

        match = re.search(r'(\d+\.?\d*)', text)
        if match:
            try:
                return float(match.group(1))
            except ValueError:
                pass

        return 0.0
