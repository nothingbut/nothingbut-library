"""
Job manager for scheduling and managing crawling tasks.

This module integrates APScheduler to provide:
- Cron-like scheduling
- Job state monitoring
- Graceful startup/shutdown
- Job history tracking
"""
import signal
import sys
from typing import Dict, Optional, List
from datetime import datetime
from pathlib import Path

from apscheduler.schedulers.background import BackgroundScheduler
from apscheduler.triggers.cron import CronTrigger
from apscheduler.events import EVENT_JOB_EXECUTED, EVENT_JOB_ERROR

from scheduler.daily_task import DailyTask
from utils.logger import get_logger
from config import settings

logger = get_logger(__name__)


class JobManager:
    """
    Job manager for scheduling automated crawling tasks.

    Features:
    - Cron-based scheduling
    - Job state monitoring
    - Event logging
    - Graceful shutdown
    """

    def __init__(self, config: settings = None):
        """
        Initialize the job manager.

        Args:
            config: Configuration object (uses global settings if None)
        """
        self.config = config or settings
        self.scheduler = None
        self.daily_task = DailyTask(self.config)
        self.running = False

        # Job history
        self.job_history: List[Dict] = []
        self.max_history = 100

        # Setup signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

    def _signal_handler(self, signum, frame):
        """
        Handle shutdown signals gracefully.

        Args:
            signum: Signal number
            frame: Current stack frame
        """
        logger.info(f"Received signal {signum}, shutting down gracefully...")
        self.stop()
        sys.exit(0)

    def _job_listener(self, event):
        """
        Listen to job execution events.

        Args:
            event: APScheduler event
        """
        if event.exception:
            logger.error(f"Job {event.job_id} failed: {event.exception}")
            self._add_to_history({
                'job_id': event.job_id,
                'status': 'failed',
                'timestamp': datetime.now().isoformat(),
                'error': str(event.exception)
            })
        else:
            logger.info(f"Job {event.job_id} completed successfully")
            self._add_to_history({
                'job_id': event.job_id,
                'status': 'success',
                'timestamp': datetime.now().isoformat()
            })

    def _add_to_history(self, record: Dict):
        """
        Add record to job history.

        Args:
            record: History record
        """
        self.job_history.append(record)
        # Keep only recent history
        if len(self.job_history) > self.max_history:
            self.job_history = self.job_history[-self.max_history:]

    def initialize(self):
        """Initialize the scheduler and jobs."""
        if self.scheduler:
            logger.warning("Scheduler already initialized")
            return

        logger.info("Initializing job scheduler")

        # Create background scheduler
        self.scheduler = BackgroundScheduler(
            timezone='Asia/Shanghai',
            job_defaults={
                'coalesce': True,  # Combine missed jobs
                'max_instances': 1,  # Only one instance per job
                'misfire_grace_time': 3600  # Allow 1 hour grace time
            }
        )

        # Add job event listener
        self.scheduler.add_listener(
            self._job_listener,
            EVENT_JOB_EXECUTED | EVENT_JOB_ERROR
        )

        # Add scheduled jobs
        self._add_scheduled_jobs()

        logger.info("Job scheduler initialized")

    def _add_scheduled_jobs(self):
        """Add all scheduled jobs."""
        # Daily crawl job - runs at 2:00 AM
        self.scheduler.add_job(
            func=self.daily_task.run_all_daily_tasks,
            trigger=CronTrigger.from_crontab('0 2 * * *'),  # 2:00 AM daily
            id='daily_crawl',
            name='Daily Incremental Crawl',
            kwargs={'source_limit': 100},
            replace_existing=True
        )
        logger.info("Added job: daily_crawl (2:00 AM)")

        # Retry job - runs every Sunday at 3:00 AM
        self.scheduler.add_job(
            func=self.daily_task.retry_failed_jobs,
            trigger=CronTrigger.from_crontab('0 3 * * 0'),  # 3:00 AM Sunday
            id='retry_failed',
            name='Retry Failed Jobs',
            replace_existing=True
        )
        logger.info("Added job: retry_failed (3:00 AM Sunday)")

        # Daily report job - runs at 6:00 AM
        self.scheduler.add_job(
            func=self.daily_task.generate_daily_report,
            trigger=CronTrigger.from_crontab('0 6 * * *'),  # 6:00 AM daily
            id='daily_report',
            name='Daily Statistics Report',
            replace_existing=True
        )
        logger.info("Added job: daily_report (6:00 AM)")

    def start(self):
        """Start the scheduler."""
        if not self.scheduler:
            self.initialize()

        if self.running:
            logger.warning("Scheduler already running")
            return

        logger.info("Starting job scheduler")
        self.scheduler.start()
        self.running = True

        logger.info("Job scheduler started")
        logger.info(f"Next scheduled jobs:")
        for job in self.scheduler.get_jobs():
            logger.info(f"  - {job.name}: {job.next_run_time}")

    def stop(self):
        """Stop the scheduler gracefully."""
        if not self.running:
            logger.warning("Scheduler not running")
            return

        logger.info("Stopping job scheduler")

        if self.scheduler:
            # Wait for jobs to complete
            self.scheduler.shutdown(wait=True)
            self.scheduler = None

        self.running = False
        logger.info("Job scheduler stopped")

    def run_job_now(self, job_id: str, **kwargs) -> Optional[Dict]:
        """
        Run a scheduled job immediately.

        Args:
            job_id: ID of the job to run
            **kwargs: Additional arguments for the job

        Returns:
            Job result or None if job not found
        """
        if not self.scheduler:
            logger.error("Scheduler not initialized")
            return None

        logger.info(f"Running job {job_id} immediately")

        try:
            if job_id == 'daily_crawl':
                return self.daily_task.run_all_daily_tasks(**kwargs)
            elif job_id == 'retry_failed':
                return self.daily_task.retry_failed_jobs()
            elif job_id == 'daily_report':
                return self.daily_task.generate_daily_report()
            elif job_id == 'incremental_youshu':
                return self.daily_task.run_incremental_youshu_crawl()
            elif job_id == 'source_crawl':
                return self.daily_task.run_source_site_crawl(**kwargs)
            else:
                logger.error(f"Unknown job ID: {job_id}")
                return None

        except Exception as e:
            logger.error(f"Failed to run job {job_id}: {e}")
            return None

    def get_scheduled_jobs(self) -> List[Dict]:
        """
        Get information about all scheduled jobs.

        Returns:
            List of job information dictionaries
        """
        if not self.scheduler:
            return []

        jobs = []
        for job in self.scheduler.get_jobs():
            # Get next run time safely
            next_run = None
            if hasattr(job, 'next_run_time') and job.next_run_time:
                next_run = job.next_run_time.isoformat()

            jobs.append({
                'id': job.id,
                'name': job.name,
                'next_run_time': next_run,
                'trigger': str(job.trigger)
            })

        return jobs

    def get_job_history(self, limit: int = 20) -> List[Dict]:
        """
        Get recent job execution history.

        Args:
            limit: Maximum number of history records to return

        Returns:
            List of history records
        """
        return self.job_history[-limit:]

    def get_status(self) -> Dict:
        """
        Get current scheduler status.

        Returns:
            Dictionary with status information
        """
        return {
            'running': self.running,
            'jobs_count': len(self.scheduler.get_jobs()) if self.scheduler else 0,
            'history_count': len(self.job_history),
            'uptime_seconds': (datetime.now() - self.scheduler.start_time).total_seconds() if self.scheduler and self.running else 0
        }

    def modify_job_schedule(self, job_id: str, cron_expression: str) -> bool:
        """
        Modify the schedule of an existing job.

        Args:
            job_id: ID of the job to modify
            cron_expression: New cron expression (e.g., "0 3 * * *")

        Returns:
            True if successful, False otherwise
        """
        if not self.scheduler:
            logger.error("Scheduler not initialized")
            return False

        try:
            job = self.scheduler.get_job(job_id)
            if not job:
                logger.error(f"Job {job_id} not found")
                return False

            # Reschedule with new cron trigger
            job.reschedule(trigger=CronTrigger.from_crontab(cron_expression))
            logger.info(f"Rescheduled job {job_id} to {cron_expression}")
            return True

        except Exception as e:
            logger.error(f"Failed to reschedule job {job_id}: {e}")
            return False

    def pause_job(self, job_id: str) -> bool:
        """
        Pause a scheduled job.

        Args:
            job_id: ID of the job to pause

        Returns:
            True if successful, False otherwise
        """
        if not self.scheduler:
            logger.error("Scheduler not initialized")
            return False

        try:
            self.scheduler.pause_job(job_id)
            logger.info(f"Paused job {job_id}")
            return True

        except Exception as e:
            logger.error(f"Failed to pause job {job_id}: {e}")
            return False

    def resume_job(self, job_id: str) -> bool:
        """
        Resume a paused job.

        Args:
            job_id: ID of the job to resume

        Returns:
            True if successful, False otherwise
        """
        if not self.scheduler:
            logger.error("Scheduler not initialized")
            return False

        try:
            self.scheduler.resume_job(job_id)
            logger.info(f"Resumed job {job_id}")
            return True

        except Exception as e:
            logger.error(f"Failed to resume job {job_id}: {e}")
            return False

    def __enter__(self):
        """Context manager entry."""
        self.initialize()
        self.start()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.stop()
