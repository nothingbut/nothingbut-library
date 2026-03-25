"""
Daily scheduled tasks for automated crawling.

This module defines the daily tasks that run on a schedule:
- Incremental youshu crawling
- Source site detail crawling
- Failed job retry
- Statistics generation
"""
from typing import Dict, List
from datetime import datetime

from crawlers.crawler_manager import CrawlerManager
from crawlers.source_crawler_manager import SourceCrawlerManager
from database.db_manager import DatabaseManager
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class DailyTask:
    """
    Daily scheduled task manager.

    Manages automated daily crawling tasks including:
    - Incremental updates from youshu
    - Source site detail crawling
    - Failed job retry
    - Daily statistics report
    """

    def __init__(self, config: settings = None):
        """
        Initialize the daily task manager.

        Args:
            config: Configuration object (uses global settings if None)
        """
        self.config = config or settings
        self.youshu_db = DatabaseManager(self.config)

    def run_incremental_youshu_crawl(self) -> Dict:
        """
        Run incremental crawl from youshu.me.

        Returns:
            Dictionary with crawl results
        """
        logger.info("=" * 60)
        logger.info("Starting incremental youshu crawl")
        logger.info("=" * 60)

        try:
            with CrawlerManager(self.config) as manager:
                stats = manager.run_incremental_crawl()

                result = {
                    'task': 'incremental_youshu_crawl',
                    'success': True,
                    'timestamp': datetime.now().isoformat(),
                    'stats': stats
                }

                logger.info(f"Incremental youshu crawl completed:")
                logger.info(f"  Success: {stats['success_count']}")
                logger.info(f"  Failed: {stats['failure_count']}")
                logger.info(f"  Duration: {stats['duration_seconds']}s")

                return result

        except Exception as e:
            logger.error(f"Incremental youshu crawl failed: {e}")
            return {
                'task': 'incremental_youshu_crawl',
                'success': False,
                'timestamp': datetime.now().isoformat(),
                'error': str(e)
            }

    def run_source_site_crawl(self, limit: int = 100) -> Dict:
        """
        Run source site detail crawling.

        Args:
            limit: Maximum number of books to crawl

        Returns:
            Dictionary with crawl results
        """
        logger.info("=" * 60)
        logger.info(f"Starting source site crawl (limit={limit})")
        logger.info("=" * 60)

        try:
            with SourceCrawlerManager(self.config) as manager:
                stats = manager.crawl_all_books(limit=limit)

                result = {
                    'task': 'source_site_crawl',
                    'success': True,
                    'timestamp': datetime.now().isoformat(),
                    'stats': stats
                }

                logger.info(f"Source site crawl completed:")
                logger.info(f"  Total: {stats['total']}")
                logger.info(f"  Success: {stats['success']}")
                logger.info(f"  Failed: {stats['failed']}")

                if stats.get('by_site'):
                    logger.info("  By Site:")
                    for site_name, site_stats in stats['by_site'].items():
                        logger.info(f"    {site_name}:")
                        logger.info(f"      Success: {site_stats['success']}")
                        logger.info(f"      Failed: {site_stats['failed']}")

                return result

        except Exception as e:
            logger.error(f"Source site crawl failed: {e}")
            return {
                'task': 'source_site_crawl',
                'success': False,
                'timestamp': datetime.now().isoformat(),
                'error': str(e)
            }

    def retry_failed_jobs(self) -> Dict:
        """
        Retry failed crawling jobs.

        Returns:
            Dictionary with retry results
        """
        logger.info("=" * 60)
        logger.info("Starting failed job retry")
        logger.info("=" * 60)

        try:
            # Get failed IDs from youshu database
            failed_ids = self.youshu_db.get_failed_ids()

            if not failed_ids:
                logger.info("No failed jobs to retry")
                return {
                    'task': 'retry_failed_jobs',
                    'success': True,
                    'timestamp': datetime.now().isoformat(),
                    'retried': 0,
                    'success_count': 0,
                    'failed_count': 0
                }

            logger.info(f"Retrying {len(failed_ids)} failed jobs")

            # Retry crawling
            with CrawlerManager(self.config) as manager:
                success_count = 0
                failed_count = 0

                for book_id in failed_ids:
                    try:
                        book_info = manager.crawl_single_book(book_id)
                        if book_info:
                            success_count += 1
                        else:
                            failed_count += 1
                    except Exception as e:
                        logger.error(f"Failed to retry book {book_id}: {e}")
                        failed_count += 1

                result = {
                    'task': 'retry_failed_jobs',
                    'success': True,
                    'timestamp': datetime.now().isoformat(),
                    'retried': len(failed_ids),
                    'success_count': success_count,
                    'failed_count': failed_count
                }

                logger.info(f"Retry completed:")
                logger.info(f"  Retried: {result['retried']}")
                logger.info(f"  Success: {result['success_count']}")
                logger.info(f"  Failed: {result['failed_count']}")

                return result

        except Exception as e:
            logger.error(f"Failed job retry failed: {e}")
            return {
                'task': 'retry_failed_jobs',
                'success': False,
                'timestamp': datetime.now().isoformat(),
                'error': str(e)
            }

    def generate_daily_report(self) -> Dict:
        """
        Generate daily statistics report.

        Returns:
            Dictionary with daily statistics
        """
        logger.info("=" * 60)
        logger.info("Generating daily statistics report")
        logger.info("=" * 60)

        try:
            # Get youshu statistics
            youshu_stats = self.youshu_db.get_statistics()

            # Get source site statistics
            with SourceCrawlerManager(self.config) as manager:
                source_stats = manager.get_source_statistics()

            report = {
                'task': 'daily_report',
                'success': True,
                'timestamp': datetime.now().isoformat(),
                'youshu': youshu_stats,
                'sources': source_stats
            }

            logger.info("Daily Statistics Report:")
            logger.info("")
            logger.info("Youshu Database:")
            logger.info(f"  Total Books: {youshu_stats.get('total_books', 0)}")
            logger.info(f"  Books with Covers: {youshu_stats.get('books_with_covers', 0)}")

            if 'last_crawl' in youshu_stats:
                last_crawl = youshu_stats['last_crawl']
                logger.info("")
                logger.info("Last Crawl:")
                logger.info(f"  Type: {last_crawl.get('crawl_type')}")
                logger.info(f"  Last ID: {last_crawl.get('last_valid_id')}")
                logger.info(f"  Success: {last_crawl.get('success_count')}")
                logger.info(f"  Failed: {last_crawl.get('failure_count')}")

            logger.info("")
            logger.info("Source Sites:")
            for site_name, stats in source_stats.items():
                logger.info(f"  {site_name.upper()}:")
                logger.info(f"    Total Books: {stats.get('total_books', 0)}")
                logger.info(f"    Books with Covers: {stats.get('books_with_covers', 0)}")
                logger.info(f"    Average Rating: {stats.get('average_rating', 0.0)}")

            return report

        except Exception as e:
            logger.error(f"Failed to generate daily report: {e}")
            return {
                'task': 'daily_report',
                'success': False,
                'timestamp': datetime.now().isoformat(),
                'error': str(e)
            }

    def run_all_daily_tasks(self, source_limit: int = 100) -> List[Dict]:
        """
        Run all daily tasks in sequence.

        Args:
            source_limit: Maximum number of source books to crawl

        Returns:
            List of task results
        """
        logger.info("")
        logger.info("=" * 60)
        logger.info("  DAILY TASK EXECUTION")
        logger.info("=" * 60)
        logger.info("")

        results = []

        # Task 1: Incremental youshu crawl
        result = self.run_incremental_youshu_crawl()
        results.append(result)

        # Task 2: Source site crawl
        result = self.run_source_site_crawl(limit=source_limit)
        results.append(result)

        # Task 3: Retry failed jobs
        result = self.retry_failed_jobs()
        results.append(result)

        # Task 4: Generate daily report
        result = self.generate_daily_report()
        results.append(result)

        # Summary
        logger.info("")
        logger.info("=" * 60)
        logger.info("  DAILY TASK SUMMARY")
        logger.info("=" * 60)
        logger.info("")

        success_count = sum(1 for r in results if r['success'])
        total_count = len(results)

        for result in results:
            status = "✓" if result['success'] else "✗"
            logger.info(f"{status} {result['task']}")

        logger.info("")
        logger.info(f"Total: {success_count}/{total_count} tasks completed successfully")

        return results
