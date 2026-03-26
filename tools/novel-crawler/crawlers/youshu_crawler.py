"""
Youshu.me crawler for fetching book metadata.
"""
from typing import Optional, Dict
from bs4 import BeautifulSoup
import json

from crawlers.base_crawler import BaseCrawler
from utils.logger import get_logger

logger = get_logger(__name__)


class YoushuCrawler(BaseCrawler):
    """
    Crawler for www.youshu.me book index pages.

    Extracts:
    - Title, Author, Description
    - Tags/Categories
    - Cover URL
    - Source site and URL
    - Update status
    """

    def build_url(self, book_id: int) -> str:
        """Build youshu.me URL for the given book ID."""
        return self.config.YOUSHU_BASE_URL.format(book_id=book_id)

    def parse_book_info(self, html: str) -> Optional[Dict]:
        """
        Parse book information from youshu.me HTML page.

        Updated selectors based on actual youshu.me page structure (2026-03-15).

        Args:
            html: HTML content

        Returns:
            Dictionary with book information
        """
        try:
            soup = BeautifulSoup(html, 'html.parser')

            # Extract book information using actual youshu.me selectors
            book_info = {
                'title': self._extract_text(soup, 'span[style*="font-size:20px"], h2, h1'),
                'author': self._extract_author(soup),
                'description': self._extract_text(soup, '.tabvalue'),
                'tags': self._extract_tags(soup),
                'cover_url': self._extract_image(soup, '.book-detail-img img'),
                'source_site': self._extract_source_site(soup),
                'source_url': self._extract_source_url(soup),
                'update_status': self._extract_status(soup),
            }

            # Filter out None values
            book_info = {k: v for k, v in book_info.items() if v is not None and v != ''}

            # Validate that at least title exists
            if not book_info.get('title'):
                self.logger.warning("No title found in page, likely invalid")
                return None

            return book_info

        except Exception as e:
            self.logger.error(f"Error parsing book info: {e}")
            return None

    def _extract_author(self, soup: BeautifulSoup) -> Optional[str]:
        """
        Extract author name from page.

        Args:
            soup: BeautifulSoup object

        Returns:
            Author name or None
        """
        # Try to find the author link
        author_link = soup.select_one('a[href*="/modules/article/authorarticle.php?author="]')
        if author_link:
            return author_link.get_text(strip=True)

        # Fallback: look for text containing "作者："
        for elem in soup.find_all(text=lambda t: t and '作者：' in str(t)):
            parent = elem.parent
            if parent:
                link = parent.find('a')
                if link:
                    return link.get_text(strip=True)

        return None

    def _extract_status(self, soup: BeautifulSoup) -> Optional[str]:
        """
        Extract update status from the author-item-exp div.

        Args:
            soup: BeautifulSoup object

        Returns:
            Status text or None
        """
        elem = soup.select_one('.author-item-exp')
        if elem:
            text = elem.get_text(strip=True)
            # Text format: "起点 玄幻 已完结 2341900字"
            # We want to extract the status part
            parts = text.split()
            for part in parts:
                if '完结' in part or '连载' in part:
                    return part
            return text

        return None

    def _extract_text(self, soup: BeautifulSoup, selector: str) -> Optional[str]:
        """
        Extract text content from first matching element.

        Args:
            soup: BeautifulSoup object
            selector: CSS selector (can contain multiple alternatives)

        Returns:
            Extracted text or None
        """
        # Try each selector separated by comma
        selectors = [s.strip() for s in selector.split(',')]

        for sel in selectors:
            element = soup.select_one(sel)
            if element:
                text = element.get_text(strip=True)
                if text:
                    return text

        return None

    def _extract_tags(self, soup: BeautifulSoup) -> Optional[str]:
        """
        Extract tags and return as JSON string.

        Args:
            soup: BeautifulSoup object

        Returns:
            JSON string of tags, or None
        """
        # Updated selectors for youshu.me
        selectors = [
            '.tag-link',
            '.tag-list .tag',
            '.tags .tag',
            '.book-tags a',
            '.categories a',
        ]

        tags = []
        for selector in selectors:
            elements = soup.select(selector)
            if elements:
                tags = [elem.get_text(strip=True) for elem in elements if elem.get_text(strip=True)]
                if tags:
                    break

        if tags:
            return json.dumps(tags, ensure_ascii=False)

        return None

    def _extract_image(self, soup: BeautifulSoup, selector: str) -> Optional[str]:
        """
        Extract image URL from img element.

        Args:
            soup: BeautifulSoup object
            selector: CSS selector for img element

        Returns:
            Image URL or None
        """
        selectors = [s.strip() for s in selector.split(',')]

        for sel in selectors:
            img = soup.select_one(sel)
            if img:
                # Try src attribute first, then data-src
                url = img.get('src') or img.get('data-src')
                if url:
                    # Convert relative URLs to absolute
                    if url.startswith('//'):
                        return 'https:' + url
                    elif url.startswith('/'):
                        return 'https://www.youshu.me' + url
                    return url

        return None

    def _extract_source_site(self, soup: BeautifulSoup) -> Optional[str]:
        """
        Extract source site name (e.g., qidian, zongheng).

        Args:
            soup: BeautifulSoup object

        Returns:
            Source site name or None
        """
        # Look for source site indicators
        selectors = [
            '.source-site',
            '.book-source',
            '.source-name'
        ]

        for selector in selectors:
            elem = soup.select_one(selector)
            if elem:
                text = elem.get_text(strip=True).lower()
                # Normalize common site names
                if '起点' in text or 'qidian' in text:
                    return 'qidian'
                elif '纵横' in text or 'zongheng' in text:
                    return 'zongheng'
                elif '17k' in text:
                    return '17k'
                elif '晋江' in text or 'jjwxc' in text:
                    return 'jjwxc'
                else:
                    return text

        return None

    def _extract_source_url(self, soup: BeautifulSoup) -> Optional[str]:
        """
        Extract source site URL.

        Args:
            soup: BeautifulSoup object

        Returns:
            Source URL or None
        """
        selectors = [
            '.source-link',
            '.book-source-link',
            'a.source-url',
            '.original-link'
        ]

        for selector in selectors:
            link = soup.select_one(selector)
            if link:
                url = link.get('href')
                if url:
                    return url

        return None
