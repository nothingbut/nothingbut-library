"""
Main crawler manager coordinating all crawling operations.
"""
from typing import Optional, Dict, List
from datetime import datetime

from config import settings
from crawlers.youshu_crawler import YoushuCrawler
from database.db_manager import DatabaseManager
from utils.http_client import HTTPClient
from utils.image_downloader import ImageDownloader
from utils.logger import setup_logger

logger = setup_logger('crawler_manager')


class CrawlerManager:
    """
    Main manager for coordinating crawling operations.

    Responsibilities:
    - Initialize all components
    - Coordinate crawling workflow
    - Manage state and statistics
    - Handle errors and retries
    """

    def __init__(self, config=None):
        """
        Initialize crawler manager and all components.

        Args:
            config: Configuration object (uses settings if None)
        """
        self.config = config or settings
        self.logger = logger

        # Initialize components
        self.http_client = HTTPClient(self.config)
        self.crawler = YoushuCrawler(self.config, self.http_client)
        self.db = DatabaseManager(self.config)
        self.img_downloader = ImageDownloader(self.config)

        # Statistics
        self.start_time = None
        self.success_count = 0
        self.failure_count = 0

    def crawl_single_book(self, book_id: int) -> Optional[Dict]:
        """
        Crawl a single book with full workflow.

        Workflow:
        1. Fetch book metadata from youshu
        2. Download cover image
        3. Save to database

        Args:
            book_id: Book ID to crawl

        Returns:
            Book information dictionary, or None if failed
        """
        self.logger.info(f"Starting crawl for book {book_id}")

        # Step 1: Crawl book metadata
        book_info = self.crawler.crawl_book(book_id)

        if not book_info:
            self.logger.warning(f"Failed to crawl book {book_id}")
            self.failure_count += 1
            return None

        # Step 2: Download cover image
        if book_info.get('cover_url'):
            cover_path = self.img_downloader.download_cover(
                book_info['cover_url'],
                book_id
            )
            if cover_path:
                book_info['cover_path'] = cover_path
                self.logger.info(f"Cover downloaded for book {book_id}")
            else:
                self.logger.warning(f"Failed to download cover for book {book_id}")

        # Step 3: Save to database
        if self.db.save_book(book_info):
            self.logger.info(
                f"[OK] Book {book_id} saved: {book_info.get('title')}"
            )
            self.success_count += 1
            return book_info
        else:
            self.logger.error(f"Failed to save book {book_id} to database")
            self.failure_count += 1
            return None

    def crawl_range(
        self,
        start_id: int,
        end_id: Optional[int] = None,
        stop_on_failure: bool = True,
        max_consecutive_failures: int = 50
    ) -> Dict:
        """
        Crawl a range of book IDs.

        Args:
            start_id: Starting book ID
            end_id: Ending book ID (None means no limit)
            stop_on_failure: Whether to stop after N consecutive failures
            max_consecutive_failures: Maximum consecutive failures before stopping

        Returns:
            Dictionary with crawl statistics
        """
        self.start_time = datetime.now()
        self.success_count = 0
        self.failure_count = 0

        self.logger.info("=" * 60)
        self.logger.info(f"Starting batch crawl from ID {start_id}")
        if end_id:
            self.logger.info(f"Ending at ID {end_id}")
        self.logger.info("=" * 60)

        current_id = start_id
        consecutive_failures = 0
        failed_ids = []

        while True:
            # Check if we've reached the end
            if end_id and current_id > end_id:
                self.logger.info(f"Reached end ID {end_id}")
                break

            # Check stop condition
            if stop_on_failure and consecutive_failures >= max_consecutive_failures:
                self.logger.warning(
                    f"Stopped after {max_consecutive_failures} consecutive failures"
                )
                break

            # Crawl current book
            self.logger.info(f"Progress: ID {current_id} "
                           f"(Success: {self.success_count}, "
                           f"Failed: {self.failure_count})")

            result = self.crawl_single_book(current_id)

            if result:
                consecutive_failures = 0
            else:
                consecutive_failures += 1
                failed_ids.append(current_id)
                self.failure_count += 1

            current_id += 1

            # Save progress every 100 books
            if current_id % 100 == 0:
                self._save_progress(current_id - 1, failed_ids)

        # Final save
        self._save_progress(current_id - 1, failed_ids)

        # Calculate statistics
        elapsed = (datetime.now() - self.start_time).total_seconds()
        stats = {
            'start_id': start_id,
            'end_id': current_id - 1,
            'total_attempted': current_id - start_id,
            'success_count': self.success_count,
            'failure_count': self.failure_count,
            'failed_ids': failed_ids,
            'duration_seconds': int(elapsed),
            'success_rate': self.success_count / (self.success_count + self.failure_count)
                               if (self.success_count + self.failure_count) > 0 else 0
        }

        self._log_summary(stats)
        return stats

    def _save_progress(self, last_id: int, failed_ids: List[int]):
        """Save crawl progress to database."""
        self.db.update_crawl_status(
            last_id=last_id,
            total=self.success_count,
            failed_ids=failed_ids,
            crawl_type='initial',
            duration_seconds=int((datetime.now() - self.start_time).total_seconds())
        )

    def _log_summary(self, stats: Dict):
        """Log crawl summary."""
        self.logger.info("")
        self.logger.info("=" * 60)
        self.logger.info("CRAWL SUMMARY")
        self.logger.info("=" * 60)
        self.logger.info(f"Start ID:        {stats['start_id']}")
        self.logger.info(f"End ID:          {stats['end_id']}")
        self.logger.info(f"Total Attempted: {stats['total_attempted']}")
        self.logger.info(f"Success:         {stats['success_count']}")
        self.logger.info(f"Failed:          {stats['failure_count']}")
        self.logger.info(f"Success Rate:    {stats['success_rate']:.2%}")
        self.logger.info(f"Duration:        {stats['duration_seconds']}s "
                        f"({stats['duration_seconds']/60:.1f}m)")
        if stats['failed_ids']:
            self.logger.info(f"Failed IDs:      {stats['failed_ids'][:10]}...")
        self.logger.info("=" * 60)

    def run_initial_crawl(self, start_id: int = 1) -> Dict:
        """
        Run initial full crawl starting from given ID.

        Args:
            start_id: Starting book ID (default: 1)

        Returns:
            Crawl statistics
        """
        return self.crawl_range(
            start_id=start_id,
            stop_on_failure=True,
            max_consecutive_failures=self.config.MAX_CONSECUTIVE_FAILURES
        )

    def run_incremental_crawl(self) -> Dict:
        """
        Run incremental crawl starting from last valid ID.

        Returns:
            Crawl statistics
        """
        last_id = self.db.get_last_valid_id()
        start_id = last_id + 1

        self.logger.info(f"Starting incremental crawl from ID {start_id}")

        return self.crawl_range(
            start_id=start_id,
            stop_on_failure=True,
            max_consecutive_failures=self.config.MAX_CONSECUTIVE_FAILURES
        )

    def run_refresh_crawl(
        self,
        start_id: int = 1,
        end_id: Optional[int] = None,
        check_existing: bool = True
    ) -> Dict:
        """
        Run refresh crawl - check for missing books and fetch them.

        This iterates through book IDs starting from 1, checks if each ID
        exists in the database, and only attempts to crawl missing ones.

        Args:
            start_id: Starting book ID (default: 1)
            end_id: Ending book ID (None means no limit)
            check_existing: If True, skip IDs that already exist in database

        Returns:
            Crawl statistics
        """
        self.start_time = datetime.now()
        self.success_count = 0
        self.failure_count = 0

        self.logger.info("=" * 60)
        self.logger.info("Starting refresh crawl")
        self.logger.info(f"Starting from ID {start_id}")
        if end_id:
            self.logger.info(f"Ending at ID {end_id}")
        self.logger.info(f"Skip existing: {check_existing}")
        self.logger.info("=" * 60)

        current_id = start_id
        consecutive_failures = 0
        consecutive_skipped = 0
        failed_ids = []
        skipped_ids = []

        while True:
            # Check if we've reached the end
            if end_id and current_id > end_id:
                self.logger.info(f"Reached end ID {end_id}")
                break

            # Check stop condition
            if consecutive_failures >= self.config.MAX_CONSECUTIVE_FAILURES:
                self.logger.warning(
                    f"Stopped after {self.config.MAX_CONSECUTIVE_FAILURES} consecutive failures"
                )
                break

            # Check if book exists in database
            if check_existing:
                existing_book = self.db.get_book(current_id)
                if existing_book:
                    self.logger.info(
                        f"Progress: ID {current_id} - [SKIP] Already exists "
                        f"(Skipped: {len(skipped_ids)}, "
                        f"Success: {self.success_count}, "
                        f"Failed: {self.failure_count})"
                    )
                    skipped_ids.append(current_id)
                    consecutive_skipped += 1
                    consecutive_failures = 0  # Reset on skip
                    current_id += 1
                    continue

            # Attempt to crawl current book
            self.logger.info(
                f"Progress: ID {current_id} - Attempting to fetch "
                f"(Skipped: {len(skipped_ids)}, "
                f"Success: {self.success_count}, "
                f"Failed: {self.failure_count})"
            )

            result = self.crawl_single_book(current_id)

            if result:
                consecutive_failures = 0
                consecutive_skipped = 0
            else:
                consecutive_failures += 1
                consecutive_skipped = 0
                failed_ids.append(current_id)
                self.failure_count += 1

            current_id += 1

            # Save progress every 100 books
            if current_id % 100 == 0:
                self._save_progress(current_id - 1, failed_ids)

        # Final save
        self._save_progress(current_id - 1, failed_ids)

        # Calculate statistics
        elapsed = (datetime.now() - self.start_time).total_seconds()
        stats = {
            'start_id': start_id,
            'end_id': current_id - 1,
            'total_attempted': current_id - start_id,
            'skipped_count': len(skipped_ids),
            'success_count': self.success_count,
            'failure_count': self.failure_count,
            'failed_ids': failed_ids,
            'skipped_ids': skipped_ids,
            'duration_seconds': int(elapsed),
        }

        # Calculate success rate (only on attempted, not skipped)
        total_attempted = self.success_count + self.failure_count
        if total_attempted > 0:
            stats['success_rate'] = self.success_count / total_attempted
        else:
            stats['success_rate'] = 0.0

        self._log_refresh_summary(stats)
        return stats

    def _log_refresh_summary(self, stats: Dict):
        """Log refresh crawl summary."""
        self.logger.info("")
        self.logger.info("=" * 60)
        self.logger.info("REFRESH CRAWL SUMMARY")
        self.logger.info("=" * 60)
        self.logger.info(f"Start ID:        {stats['start_id']}")
        self.logger.info(f"End ID:          {stats['end_id']}")
        self.logger.info(f"Total Checked:   {stats['total_attempted']}")
        self.logger.info(f"Skipped (exist): {stats['skipped_count']}")
        self.logger.info(f"Success:         {stats['success_count']}")
        self.logger.info(f"Failed:          {stats['failure_count']}")
        if stats.get('success_rate'):
            self.logger.info(f"Success Rate:    {stats['success_rate']:.2%} (of attempted)")
        self.logger.info(f"Duration:        {stats['duration_seconds']}s "
                        f"({stats['duration_seconds']/60:.1f}m)")
        if stats['failed_ids']:
            self.logger.info(f"Failed IDs:      {stats['failed_ids'][:10]}...")
        self.logger.info("=" * 60)

    def get_statistics(self) -> Dict:
        """
        Get current database statistics.

        Returns:
            Statistics dictionary
        """
        return self.db.get_statistics()

    def close(self):
        """Cleanup resources."""
        self.http_client.close()

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
