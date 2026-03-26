"""
Unit tests for utility modules.
"""
import pytest
import time
from unittest.mock import Mock, patch

from utils.retry_handler import retry_on_failure, RetryHandler, RetryException
from utils.logger import setup_logger, get_logger


class TestRetryHandler:
    """Tests for retry handler module."""

    def test_retry_on_failure_success(self):
        """Test successful retry on temporary failure."""
        attempts = [0]

        @retry_on_failure(max_attempts=3, backoff_factor=0.1)
        def failing_function():
            attempts[0] += 1
            if attempts[0] < 2:
                raise ValueError("Temporary error")
            return "success"

        result = failing_function()
        assert result == "success"
        assert attempts[0] == 2

    def test_retry_on_failure_exhausted(self):
        """Test that exception is raised after max attempts."""
        @retry_on_failure(max_attempts=3, backoff_factor=0.1)
        def always_failing_function():
            raise ValueError("Permanent error")

        with pytest.raises(RetryException):
            always_failing_function()

    def test_retry_handler_class(self):
        """Test RetryHandler class."""
        handler = RetryHandler(max_attempts=3, backoff_factor=0.1)
        attempts = [0]

        def failing_func():
            attempts[0] += 1
            if attempts[0] < 2:
                raise ValueError("Temporary error")
            return "success"

        result = handler.execute(failing_func)
        assert result == "success"
        assert attempts[0] == 2

    def test_retry_exponential_backoff(self):
        """Test that exponential backoff delays are correct."""
        @retry_on_failure(max_attempts=3, backoff_factor=0.1)
        def failing_function():
            raise ValueError("Error")

        start_time = time.time()
        with pytest.raises(RetryException):
            failing_function()
        elapsed_time = time.time() - start_time

        # Should have 2 delays: 0.1s and 0.2s (0.1 * 2^0, 0.1 * 2^1)
        # Total delay should be around 0.3s (plus some tolerance)
        assert elapsed_time >= 0.25


class TestLogger:
    """Tests for logger module."""

    def test_setup_logger(self):
        """Test logger setup."""
        logger = setup_logger('test_logger')
        assert logger.name == 'test_logger'
        assert logger.handlers  # Should have handlers

    def test_get_logger(self):
        """Test getting logger instance."""
        logger = get_logger('test_logger_2')
        assert logger is not None
        assert logger.name == 'test_logger_2'

    def test_logger_caches_handlers(self):
        """Test that calling setup_logger twice doesn't duplicate handlers."""
        logger1 = setup_logger('test_logger_cache')
        handler_count_1 = len(logger1.handlers)

        logger2 = setup_logger('test_logger_cache')
        handler_count_2 = len(logger2.handlers)

        assert handler_count_1 == handler_count_2
