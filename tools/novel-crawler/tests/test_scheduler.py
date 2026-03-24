"""
Unit tests for the scheduler system.
"""
import pytest
import time
from unittest.mock import Mock, patch, MagicMock
from datetime import datetime

from scheduler.daily_task import DailyTask
from scheduler.job_manager import JobManager
from config import settings


@pytest.fixture
def daily_task():
    """Create a DailyTask instance."""
    return DailyTask()


@pytest.fixture
def job_manager():
    """Create a JobManager instance."""
    manager = JobManager()
    yield manager
    # Cleanup
    if manager.running:
        manager.stop()


class TestDailyTask:
    """Test cases for DailyTask."""

    def test_initialization(self, daily_task):
        """Test DailyTask initialization."""
        assert daily_task.config is not None
        assert daily_task.youshu_db is not None

    @patch('scheduler.daily_task.CrawlerManager')
    def test_run_incremental_youshu_crawl(self, mock_crawler_mgr, daily_task):
        """Test incremental youshu crawl."""
        # Mock the manager and its methods
        mock_manager = MagicMock()
        mock_crawler_mgr.return_value.__enter__.return_value = mock_manager
        mock_manager.run_incremental_crawl.return_value = {
            'success_count': 10,
            'failure_count': 2,
            'duration_seconds': 60
        }

        result = daily_task.run_incremental_youshu_crawl()

        assert result['task'] == 'incremental_youshu_crawl'
        assert result['success'] is True
        assert result['stats']['success_count'] == 10

    @patch('scheduler.daily_task.SourceCrawlerManager')
    def test_run_source_site_crawl(self, mock_source_mgr, daily_task):
        """Test source site crawl."""
        # Mock the manager and its methods
        mock_manager = MagicMock()
        mock_source_mgr.return_value.__enter__.return_value = mock_manager
        mock_manager.crawl_all_books.return_value = {
            'total': 50,
            'success': 45,
            'failed': 5,
            'by_site': {
                'qidian': {'success': 30, 'failed': 2},
                'zongheng': {'success': 15, 'failed': 3}
            }
        }

        result = daily_task.run_source_site_crawl(limit=50)

        assert result['task'] == 'source_site_crawl'
        assert result['success'] is True
        assert result['stats']['total'] == 50

    @patch('scheduler.daily_task.CrawlerManager')
    @patch.object(DailyTask, '__init__', return_value=None)
    def test_retry_failed_jobs_empty(self, mock_init, mock_crawler_mgr):
        """Test retry with no failed jobs."""
        # Create daily task without calling __init__
        daily_task = DailyTask.__new__(DailyTask)
        daily_task.youshu_db = MagicMock()
        daily_task.youshu_db.get_failed_ids.return_value = []
        daily_task.config = settings

        result = daily_task.retry_failed_jobs()

        assert result['task'] == 'retry_failed_jobs'
        assert result['success'] is True
        assert result['retried'] == 0

    @patch('scheduler.daily_task.CrawlerManager')
    @patch.object(DailyTask, '__init__', return_value=None)
    def test_retry_failed_jobs_with_failures(self, mock_init, mock_crawler_mgr):
        """Test retry with failed jobs."""
        # Create daily task without calling __init__
        daily_task = DailyTask.__new__(DailyTask)
        daily_task.youshu_db = MagicMock()
        daily_task.youshu_db.get_failed_ids.return_value = [1, 2, 3]
        daily_task.config = settings

        # Mock crawler
        mock_manager = MagicMock()
        mock_crawler_mgr.return_value.__enter__.return_value = mock_manager
        mock_manager.crawl_single_book.side_effect = [
            {'id': 1, 'title': 'Book 1'},  # Success
            None,  # Failed
            {'id': 3, 'title': 'Book 3'}   # Success
        ]

        result = daily_task.retry_failed_jobs()

        assert result['task'] == 'retry_failed_jobs'
        assert result['success'] is True
        assert result['retried'] == 3
        assert result['success_count'] == 2
        assert result['failed_count'] == 1


class TestJobManager:
    """Test cases for JobManager."""

    def test_initialization(self, job_manager):
        """Test JobManager initialization."""
        assert job_manager.scheduler is None
        assert job_manager.running is False
        assert job_manager.daily_task is not None
        assert len(job_manager.job_history) == 0

    def test_initialize_scheduler(self, job_manager):
        """Test scheduler initialization."""
        job_manager.initialize()

        assert job_manager.scheduler is not None
        assert job_manager.running is False

        # Check that jobs were added
        jobs = job_manager.get_scheduled_jobs()
        assert len(jobs) == 3
        job_ids = [job['id'] for job in jobs]
        assert 'daily_crawl' in job_ids
        assert 'retry_failed' in job_ids
        assert 'daily_report' in job_ids

    def test_start_scheduler(self, job_manager):
        """Test starting the scheduler."""
        job_manager.initialize()
        job_manager.start()

        assert job_manager.running is True

        # Cleanup
        job_manager.stop()

    def test_stop_scheduler(self, job_manager):
        """Test stopping the scheduler."""
        job_manager.initialize()
        job_manager.start()
        job_manager.stop()

        assert job_manager.running is False

    def test_get_scheduled_jobs(self, job_manager):
        """Test getting scheduled jobs."""
        job_manager.initialize()

        jobs = job_manager.get_scheduled_jobs()

        assert len(jobs) == 3
        for job in jobs:
            assert 'id' in job
            assert 'name' in job
            assert 'next_run_time' in job
            assert 'trigger' in job

    @patch('scheduler.job_manager.DailyTask')
    def test_run_job_now(self, mock_daily_task, job_manager):
        """Test running a job immediately."""
        # Mock the daily task method
        mock_task_instance = MagicMock()
        mock_daily_task.return_value = mock_task_instance
        mock_task_instance.generate_daily_report.return_value = {
            'task': 'daily_report',
            'success': True,
            'timestamp': datetime.now().isoformat()
        }

        job_manager.initialize()
        # Don't start scheduler, just use the manager to run jobs

        result = job_manager.run_job_now('daily_report')

        assert result is not None
        assert result['success'] is True

    def test_get_job_history(self, job_manager):
        """Test getting job history."""
        job_manager.initialize()

        # Add some fake history
        job_manager._add_to_history({
            'job_id': 'test_job',
            'status': 'success',
            'timestamp': datetime.now().isoformat()
        })
        job_manager._add_to_history({
            'job_id': 'test_job_2',
            'status': 'failed',
            'timestamp': datetime.now().isoformat()
        })

        history = job_manager.get_job_history(limit=10)

        assert len(history) == 2
        assert history[0]['job_id'] == 'test_job'
        assert history[1]['job_id'] == 'test_job_2'

    def test_get_status(self, job_manager):
        """Test getting scheduler status."""
        job_manager.initialize()

        status = job_manager.get_status()

        assert 'running' in status
        assert 'jobs_count' in status
        assert 'history_count' in status
        assert status['jobs_count'] == 3
        assert status['running'] is False

    @patch('scheduler.job_manager.DailyTask')
    def test_pause_and_resume_job(self, mock_daily_task, job_manager):
        """Test pausing and resuming a job."""
        job_manager.initialize()
        job_manager.start()

        # Pause job
        result = job_manager.pause_job('daily_crawl')
        assert result is True

        # Resume job
        result = job_manager.resume_job('daily_crawl')
        assert result is True

        # Cleanup
        job_manager.stop()

    def test_context_manager(self, job_manager):
        """Test using JobManager as context manager."""
        with JobManager() as manager:
            assert manager.scheduler is not None
            assert manager.running is True

        # After context exit, should be stopped
        assert job_manager.running is False


if __name__ == '__main__':
    pytest.main([__file__, '-v'])
