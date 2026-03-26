"""
Retry handler with exponential backoff for resilient operations.
"""
import time
import functools
import logging
from typing import Callable, Optional, Type, Tuple

logger = logging.getLogger(__name__)


class RetryException(Exception):
    """Exception raised when all retry attempts are exhausted."""
    pass


def retry_on_failure(
    max_attempts: int = 3,
    backoff_factor: float = 1.0,
    exceptions: Tuple[Type[Exception], ...] = (Exception,),
    logger_instance: Optional[logging.Logger] = None
) -> Callable:
    """
    Decorator for retrying functions with exponential backoff.

    Args:
        max_attempts: Maximum number of retry attempts
        backoff_factor: Multiplier for exponential backoff delay
        exceptions: Tuple of exception types to catch and retry
        logger_instance: Optional logger instance for logging retries

    Returns:
        Decorated function with retry logic

    Example:
        @retry_on_failure(max_attempts=3, backoff_factor=1.0)
        def fetch_data(url):
            return requests.get(url)
    """
    def decorator(func: Callable) -> Callable:
        @functools.wraps(func)
        def wrapper(*args, **kwargs):
            last_exception = None
            log = logger_instance or logger

            for attempt in range(max_attempts):
                try:
                    return func(*args, **kwargs)
                except exceptions as e:
                    last_exception = e

                    if attempt == max_attempts - 1:
                        # Last attempt failed
                        log.error(
                            f"Function '{func.__name__}' failed after "
                            f"{max_attempts} attempts: {e}"
                        )
                        raise RetryException(
                            f"Failed after {max_attempts} attempts"
                        ) from e

                    # Calculate backoff delay
                    wait_time = backoff_factor * (2 ** attempt)
                    log.warning(
                        f"Attempt {attempt + 1}/{max_attempts} for "
                        f"'{func.__name__}' failed: {e}. "
                        f"Retrying in {wait_time:.1f}s..."
                    )
                    time.sleep(wait_time)

            # Should not reach here, but just in case
            raise last_exception

        return wrapper
    return decorator


class RetryHandler:
    """
    Handler class for retry logic without decorators.

    Useful for retrying operations in a more imperative style.
    """

    def __init__(
        self,
        max_attempts: int = 3,
        backoff_factor: float = 1.0,
        logger_instance: Optional[logging.Logger] = None
    ):
        self.max_attempts = max_attempts
        self.backoff_factor = backoff_factor
        self.log = logger_instance or logger

    def execute(
        self,
        func: Callable,
        *args,
        exceptions: Tuple[Type[Exception], ...] = (Exception,),
        **kwargs
    ):
        """
        Execute a function with retry logic.

        Args:
            func: Function to execute
            *args: Positional arguments for the function
            exceptions: Exception types to catch and retry
            **kwargs: Keyword arguments for the function

        Returns:
            Function return value

        Raises:
            RetryException: If all attempts fail
        """
        last_exception = None

        for attempt in range(self.max_attempts):
            try:
                return func(*args, **kwargs)
            except exceptions as e:
                last_exception = e

                if attempt == self.max_attempts - 1:
                    self.log.error(
                        f"Operation failed after {self.max_attempts} attempts: {e}"
                    )
                    raise RetryException(
                        f"Failed after {self.max_attempts} attempts"
                    ) from e

                wait_time = self.backoff_factor * (2 ** attempt)
                self.log.warning(
                    f"Attempt {attempt + 1}/{self.max_attempts} failed. "
                    f"Retrying in {wait_time:.1f}s..."
                )
                time.sleep(wait_time)

        raise last_exception
