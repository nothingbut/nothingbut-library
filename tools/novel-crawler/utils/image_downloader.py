"""
Image downloader for book cover images.
"""
from pathlib import Path
from typing import Optional
import requests

from config import settings
from utils.logger import get_logger

logger = get_logger(__name__)


class ImageDownloader:
    """
    Downloads and saves book cover images.
    """

    def __init__(self, config=None):
        """
        Initialize image downloader.

        Args:
            config: Configuration object (uses settings if None)
        """
        self.config = config or settings
        self.cover_dir = self.config.COVER_DIR
        self.max_size = self.config.MAX_IMAGE_SIZE
        self.timeout = self.config.IMAGE_TIMEOUT

    def download_cover(self, cover_url: str, book_id: int) -> Optional[str]:
        """
        Download cover image for a book.

        Args:
            cover_url: URL of the cover image
            book_id: Book ID (used for filename)

        Returns:
            Local path to saved image, or None if download fails
        """
        if not cover_url:
            logger.warning(f"No cover URL provided for book {book_id}")
            return None

        try:
            logger.debug(f"Downloading cover for book {book_id}: {cover_url}")

            # Download image
            response = requests.get(
                cover_url,
                timeout=self.timeout,
                stream=True
            )
            response.raise_for_status()

            # Check file size
            content_length = response.headers.get('content-length')
            if content_length and int(content_length) > self.max_size:
                logger.warning(
                    f"Cover image too large for book {book_id}: "
                    f"{content_length} bytes"
                )
                return None

            # Determine file extension from content-type
            content_type = response.headers.get('content-type', '')
            if 'jpeg' in content_type or 'jpg' in content_type:
                ext = '.jpg'
            elif 'png' in content_type:
                ext = '.png'
            elif 'webp' in content_type:
                ext = '.webp'
            else:
                # Default to jpg
                ext = '.jpg'

            # Save to file
            filename = f"{book_id}{ext}"
            filepath = self.cover_dir / filename

            with open(filepath, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    f.write(chunk)

            logger.info(f"Cover saved for book {book_id}: {filepath}")
            return str(filepath)

        except requests.exceptions.Timeout:
            logger.error(f"Timeout downloading cover for book {book_id}")
            return None

        except requests.exceptions.RequestException as e:
            logger.error(f"Error downloading cover for book {book_id}: {e}")
            return None

        except IOError as e:
            logger.error(f"Error saving cover for book {book_id}: {e}")
            return None

    def download_cover_from_html(self, html_content: str, book_id: int) -> Optional[str]:
        """
        Extract cover URL from HTML and download the image.

        Args:
            html_content: HTML content containing cover image
            book_id: Book ID

        Returns:
            Local path to saved image, or None if fails
        """
        from bs4 import BeautifulSoup

        soup = BeautifulSoup(html_content, 'html.parser')

        # Try to find cover image
        selectors = [
            'img.book-cover',
            '.book-cover img',
            'img.cover',
            '.cover img',
            'img[alt*="封面"]',
            'img[alt*="cover"]'
        ]

        for selector in selectors:
            img = soup.select_one(selector)
            if img:
                cover_url = img.get('src') or img.get('data-src')
                if cover_url:
                    # Convert relative URL to absolute if needed
                    if cover_url.startswith('//'):
                        cover_url = 'https:' + cover_url
                    elif cover_url.startswith('/'):
                        from urllib.parse import urljoin
                        cover_url = urljoin('https://www.youshu.me/', cover_url)

                    return self.download_cover(cover_url, book_id)

        logger.warning(f"No cover image found in HTML for book {book_id}")
        return None

    def batch_download(
        self,
        cover_urls: dict,
        max_concurrent: int = 3
    ) -> dict:
        """
        Download multiple cover images.

        Args:
            cover_urls: Dictionary mapping book_id to cover_url
            max_concurrent: Maximum concurrent downloads (not implemented yet)

        Returns:
            Dictionary mapping book_id to local_path (or None if failed)
        """
        results = {}

        for book_id, cover_url in cover_urls.items():
            results[book_id] = self.download_cover(cover_url, book_id)

        # Log summary
        success_count = sum(1 for v in results.values() if v is not None)
        logger.info(
            f"Batch download completed: {success_count}/{len(cover_urls)} successful"
        )

        return results
