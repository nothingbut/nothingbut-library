"""
Novel Metadata Crawler - Main Entry Point

小说元数据爬虫系统主程序入口。
"""
import sys
import argparse
import logging

from config import settings
from crawlers.crawler_manager import CrawlerManager
from crawlers.source_crawler_manager import SourceCrawlerManager
from database.db_manager import DatabaseManager
from scheduler.job_manager import JobManager
from scheduler.daily_task import DailyTask
from utils.logger import setup_logger

# Setup logging
logger = setup_logger('main')


def print_banner():
    """Print application banner."""
    logger.info("")
    logger.info("=" * 60)
    logger.info("  Novel Metadata Crawler v1.0")
    logger.info("  小说元数据爬虫系统")
    logger.info("=" * 60)
    logger.info("")


def handle_crawl(args):
    """Handle crawl command."""
    print_banner()

    with CrawlerManager() as manager:
        if args.mode == 'initial':
            start_id = args.start or 1
            logger.info(f"Starting initial crawl from ID {start_id}")
            stats = manager.run_initial_crawl(start_id=start_id)

        elif args.mode == 'incremental':
            logger.info("Starting incremental crawl")
            stats = manager.run_incremental_crawl()

        else:
            logger.error(f"Unknown mode: {args.mode}")
            return 1

        # Print summary
        logger.info("")
        logger.info("Crawl completed!")
        logger.info(f"Success: {stats['success_count']}")
        logger.info(f"Failed: {stats['failure_count']}")
        logger.info(f"Duration: {stats['duration_seconds']}s")

    return 0


def handle_refresh(args):
    """Handle refresh command."""
    print_banner()

    with CrawlerManager() as manager:
        start_id = args.start or 1
        end_id = args.end if hasattr(args, 'end') and args.end else None

        logger.info(f"Starting refresh crawl from ID {start_id}")
        if end_id:
            logger.info(f"Ending at ID {end_id}")

        stats = manager.run_refresh_crawl(
            start_id=start_id,
            end_id=end_id,
            check_existing=True
        )

        # Print summary
        logger.info("")
        logger.info("Refresh completed!")
        logger.info(f"Total Checked:    {stats['total_attempted']}")
        logger.info(f"Skipped (exist):  {stats['skipped_count']}")
        logger.info(f"Success:          {stats['success_count']}")
        logger.info(f"Failed:           {stats['failure_count']}")
        if stats.get('success_rate'):
            logger.info(f"Success Rate:     {stats['success_rate']:.2%} (of attempted)")
        logger.info(f"Duration:         {stats['duration_seconds']}s")

    return 0


def handle_single(args):
    """Handle single book crawl command."""
    print_banner()

    with CrawlerManager() as manager:
        logger.info(f"Crawling single book: ID {args.book_id}")
        book_info = manager.crawl_single_book(args.book_id)

        if book_info:
            logger.info("✓ Book crawled successfully!")
            logger.info(f"  Title: {book_info.get('title')}")
            logger.info(f"  Author: {book_info.get('author')}")
            if book_info.get('cover_path'):
                logger.info(f"  Cover: {book_info.get('cover_path')}")
            return 0
        else:
            logger.error(f"✗ Failed to crawl book {args.book_id}")
            return 1


def handle_stats(args):
    """Handle stats command."""
    print_banner()

    db = DatabaseManager(settings)
    stats = db.get_statistics()

    logger.info("Database Statistics:")
    logger.info(f"  Total Books:     {stats.get('total_books', 0)}")
    logger.info(f"  Books with Covers: {stats.get('books_with_covers', 0)}")

    if 'by_source' in stats:
        logger.info("  Books by Source:")
        for source, count in stats['by_source'].items():
            logger.info(f"    - {source}: {count}")

    if 'last_crawl' in stats:
        last_crawl = stats['last_crawl']
        logger.info("")
        logger.info("Last Crawl:")
        logger.info(f"  Type:      {last_crawl.get('crawl_type')}")
        logger.info(f"  Last ID:   {last_crawl.get('last_valid_id')}")
        logger.info(f"  Success:   {last_crawl.get('success_count')}")
        logger.info(f"  Failed:    {last_crawl.get('failure_count')}")

    return 0


def handle_source(args):
    """Handle source site crawl command."""
    print_banner()

    with SourceCrawlerManager() as manager:
        if args.mode == 'all':
            limit = args.limit if args.limit > 0 else None
            logger.info("Crawling source sites for all books")
            stats = manager.crawl_all_books(limit=limit)

        elif args.mode == 'batch':
            if not args.ids:
                logger.error("--ids required for batch mode")
                return 1

            youshu_ids = [int(id_str) for id_str in args.ids.split(',')]
            logger.info(f"Crawling source sites for {len(youshu_ids)} books")
            stats = manager.crawl_batch(youshu_ids)

        else:
            logger.error(f"Unknown mode: {args.mode}")
            return 1

        # Print summary
        logger.info("")
        logger.info("Source crawl completed!")
        logger.info(f"Total:   {stats['total']}")
        logger.info(f"Success: {stats['success']}")
        logger.info(f"Failed:  {stats['failed']}")

        if stats.get('by_site'):
            logger.info("")
            logger.info("By Site:")
            for site_name, site_stats in stats['by_site'].items():
                logger.info(f"  {site_name}:")
                logger.info(f"    Success: {site_stats['success']}")
                logger.info(f"    Failed:  {site_stats['failed']}")

    return 0


def handle_source_stats(args):
    """Handle source stats command."""
    print_banner()

    with SourceCrawlerManager() as manager:
        all_stats = manager.get_source_statistics()

        logger.info("Source Site Statistics:")
        for site_name, stats in all_stats.items():
            logger.info("")
            logger.info(f"  {site_name.upper()}:")
            logger.info(f"    Total Books:     {stats.get('total_books', 0)}")
            logger.info(f"    Books with Covers: {stats.get('books_with_covers', 0)}")
            logger.info(f"    Average Rating:  {stats.get('average_rating', 0.0)}")

            if stats.get('total_word_count'):
                word_count = stats['total_word_count']
                if word_count > 100000000:
                    logger.info(f"    Total Words:     {word_count / 100000000:.2f}亿")
                elif word_count > 10000:
                    logger.info(f"    Total Words:     {word_count / 10000:.2f}万")
                else:
                    logger.info(f"    Total Words:     {word_count}")

            if stats.get('status_distribution'):
                logger.info(f"    Status:")
                for status, count in stats['status_distribution'].items():
                    logger.info(f"      - {status}: {count}")

    return 0


def handle_schedule_start(args):
    """Handle schedule start command."""
    print_banner()

    logger.info("Starting job scheduler...")
    logger.info("Press Ctrl+C to stop")

    try:
        with JobManager() as manager:
            # Keep running until interrupted
            import time
            while True:
                time.sleep(1)
    except KeyboardInterrupt:
        logger.info("Scheduler stopped by user")
        return 0


def handle_schedule_run(args):
    """Handle schedule run command."""
    print_banner()

    with JobManager() as manager:
        # Run job immediately without starting scheduler
        result = manager.run_job_now(args.job, **(args.kwargs or {}))

        if result:
            if isinstance(result, list):
                # Multiple tasks (like run_all_daily_tasks)
                logger.info("")
                logger.info("Job Results:")
                for r in result:
                    status = "[OK]" if r['success'] else "[FAIL]"
                    logger.info(f"{status} {r['task']}")
                return 0 if all(r['success'] for r in result) else 1
            elif result.get('success'):
                logger.info(f"[OK] Job {args.job} completed successfully")
                return 0
            else:
                logger.error(f"[FAIL] Job {args.job} failed: {result.get('error', 'Unknown error')}")
                return 1
        else:
            logger.error(f"[FAIL] Job {args.job} not found or failed to run")
            return 1


def handle_schedule_list(args):
    """Handle schedule list command."""
    print_banner()

    with JobManager() as manager:
        manager.initialize()  # Don't start, just initialize

        jobs = manager.get_scheduled_jobs()

        if not jobs:
            logger.info("No scheduled jobs")
            return 0

        logger.info("Scheduled Jobs:")
        logger.info("")

        for job in jobs:
            logger.info(f"  ID: {job['id']}")
            logger.info(f"  Name: {job['name']}")
            logger.info(f"  Trigger: {job['trigger']}")
            logger.info(f"  Next Run: {job['next_run_time']}")
            logger.info("")

    return 0


def handle_schedule_status(args):
    """Handle schedule status command."""
    print_banner()

    # Note: This only shows if scheduler is running from another process
    # For a real implementation, you'd use a persistent state or pid file
    logger.info("Scheduler Status:")
    logger.info("  The scheduler runs as a separate process.")
    logger.info("  Use 'python main.py schedule start' to run it.")
    logger.info("")
    logger.info("To check if it's running, look for the process:")
    logger.info("  Windows: tasklist | find python")
    logger.info("  Linux/Mac: ps aux | grep python")

    return 0


def handle_test(args):
    """Handle test command."""
    logger.info("Running tests...")

    import subprocess
    result = subprocess.run(
        ["pytest", "-v", "tests/"],
        cwd=settings.BASE_DIR
    )

    return result.returncode


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Novel Metadata Crawler - 小说元数据爬虫系统',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Initial crawl starting from ID 1
  python main.py crawl --mode initial --start 1

  # Incremental crawl
  python main.py crawl --mode incremental

  # Crawl single book
  python main.py single --book-id 123

  # Crawl source sites for all books
  python main.py source --mode all

  # Crawl source sites for specific books
  python main.py source --mode batch --ids 1,2,3

  # Show youshu statistics
  python main.py stats

  # Show source site statistics
  python main.py source-stats

  # Start job scheduler (runs continuously)
  python main.py schedule start

  # Run a scheduled job immediately
  python main.py schedule run --job daily_crawl

  # List scheduled jobs
  python main.py schedule list

  # Run tests
  python main.py test
        """
    )

    subparsers = parser.add_subparsers(dest='command', help='Available commands')

    # Crawl command
    crawl_parser = subparsers.add_parser('crawl', help='Crawl books')
    crawl_parser.add_argument(
        '--mode',
        choices=['initial', 'incremental'],
        required=True,
        help='Crawl mode'
    )
    crawl_parser.add_argument(
        '--start',
        type=int,
        default=1,
        help='Starting book ID (for initial mode)'
    )

    # Refresh command
    refresh_parser = subparsers.add_parser('refresh', help='Refresh crawl - check for missing books')
    refresh_parser.add_argument(
        '--start',
        type=int,
        default=1,
        help='Starting book ID (default: 1)'
    )
    refresh_parser.add_argument(
        '--end',
        type=int,
        help='Ending book ID (optional, no limit if not specified)'
    )

    # Single book command
    single_parser = subparsers.add_parser('single', help='Crawl a single book')
    single_parser.add_argument(
        '--book-id',
        type=int,
        required=True,
        help='Book ID to crawl'
    )

    # Stats command
    stats_parser = subparsers.add_parser('stats', help='Show database statistics')

    # Source command
    source_parser = subparsers.add_parser('source', help='Crawl source sites')
    source_parser.add_argument(
        '--mode',
        choices=['all', 'batch'],
        required=True,
        help='Source crawl mode'
    )
    source_parser.add_argument(
        '--limit',
        type=int,
        default=0,
        help='Maximum number of books to crawl (0 for all, only for "all" mode)'
    )
    source_parser.add_argument(
        '--ids',
        type=str,
        help='Comma-separated list of youshu book IDs (for "batch" mode)'
    )

    # Source stats command
    source_stats_parser = subparsers.add_parser('source-stats', help='Show source site statistics')

    # Schedule command
    schedule_parser = subparsers.add_parser('schedule', help='Manage job scheduler')
    schedule_subparsers = schedule_parser.add_subparsers(dest='schedule_command', help='Schedule commands')

    # Schedule start
    schedule_start_parser = schedule_subparsers.add_parser('start', help='Start job scheduler')

    # Schedule run
    schedule_run_parser = schedule_subparsers.add_parser('run', help='Run a scheduled job immediately')
    schedule_run_parser.add_argument(
        '--job',
        type=str,
        required=True,
        choices=['daily_crawl', 'retry_failed', 'daily_report', 'incremental_youshu', 'source_crawl'],
        help='Job ID to run'
    )
    schedule_run_parser.add_argument(
        '--kwargs',
        type=str,
        help='Optional kwargs as JSON string (e.g., \'{"source_limit": 50}\')'
    )

    # Schedule list
    schedule_list_parser = schedule_subparsers.add_parser('list', help='List scheduled jobs')

    # Schedule status
    schedule_status_parser = schedule_subparsers.add_parser('status', help='Show scheduler status')

    # Test command
    test_parser = subparsers.add_parser('test', help='Run tests')

    # Parse arguments
    args = parser.parse_args()

    # Show help if no command
    if not args.command:
        parser.print_help()
        return 1

    # Route to handler
    handlers = {
        'crawl': handle_crawl,
        'refresh': handle_refresh,
        'single': handle_single,
        'stats': handle_stats,
        'source': handle_source,
        'source-stats': handle_source_stats,
        'schedule': handle_schedule_command,
        'test': handle_test
    }

    handler = handlers.get(args.command)
    if handler:
        return handler(args)
    else:
        logger.error(f"Unknown command: {args.command}")
        return 1


def handle_schedule_command(args):
    """Route schedule subcommands."""
    if not args.schedule_command:
        # Show help if no subcommand
        return 1

    schedule_handlers = {
        'start': handle_schedule_start,
        'run': handle_schedule_run,
        'list': handle_schedule_list,
        'status': handle_schedule_status
    }

    handler = schedule_handlers.get(args.schedule_command)
    if handler:
        return handler(args)
    else:
        logger.error(f"Unknown schedule command: {args.schedule_command}")
        return 1


if __name__ == "__main__":
    sys.exit(main())
