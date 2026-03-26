"""
HTTP client with retry logic, connection pooling, and security features.
"""
import random
import time
from typing import Optional, Dict

import requests
from requests.adapters import HTTPAdapter
from urllib3.util.retry import Retry

from config import settings
from utils.logger import get_logger

logger = get_logger(__name__)


class HTTPClient:
    """
    HTTP client with automatic retries, connection pooling, and security features.

    Features:
    - Automatic retry with exponential backoff
    - Connection pooling for better performance
    - Random User-Agent rotation
    - Configurable timeouts
    """

    def __init__(self, config=None):
        """
        Initialize HTTP client.

        Args:
            config: Configuration object (uses settings if None)
        """
        self.config = config or settings
        self.session = self._create_session()

    def _create_session(self) -> requests.Session:
        """
        Create a requests session with retry strategy and connection pooling.

        Returns:
            Configured requests.Session
        """
        session = requests.Session()

        # Configure retry strategy
        retry_strategy = Retry(
            total=self.config.MAX_RETRIES,
            backoff_factor=1,
            status_forcelist=[429, 500, 502, 503, 504],
            allowed_methods=["GET", "HEAD"]
        )

        # Mount adapters with connection pooling
        adapter = HTTPAdapter(
            max_retries=retry_strategy,
            pool_connections=10,
            pool_maxsize=20
        )
        session.mount("http://", adapter)
        session.mount("https://", adapter)

        return session

    def get_random_user_agent(self) -> str:
        """
        Get a random User-Agent from the pool.

        Returns:
            Random User-Agent string
        """
        return random.choice(self.config.USER_AGENTS)

    def get_default_headers(self) -> Dict[str, str]:
        """
        Get default HTTP headers for requests.

        Returns:
            Dictionary of headers
        """
        return {
            'User-Agent': self.get_random_user_agent(),
            'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8',
            'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8',
            # Don't set Accept-Encoding - let requests handle it automatically
            'Connection': 'keep-alive',
            'Upgrade-Insecure-Requests': '1',
            'Cache-Control': 'max-age=0',
        }

    def get(
        self,
        url: str,
        headers: Optional[Dict[str, str]] = None,
        timeout: Optional[int] = None,
        **kwargs
    ) -> Optional[requests.Response]:
        """
        Perform HTTP GET request with automatic retry and error handling.

        Args:
            url: URL to request
            headers: Optional custom headers (merged with defaults)
            timeout: Request timeout in seconds (uses config default if None)
            **kwargs: Additional arguments passed to requests.get

        Returns:
            Response object if successful, None if failed
        """
        try:
            # Merge headers
            request_headers = self.get_default_headers()
            if headers:
                request_headers.update(headers)

            # Use config timeout if not specified
            request_timeout = timeout or self.config.REQUEST_TIMEOUT

            logger.debug(f"GET {url}")

            response = self.session.get(
                url,
                headers=request_headers,
                timeout=request_timeout,
                **kwargs
            )
            response.raise_for_status()

            return response

        except requests.exceptions.Timeout:
            logger.error(f"Request timeout: {url}")
            return None

        except requests.exceptions.HTTPError as e:
            logger.error(f"HTTP error: {e.response.status_code} - {url}")
            return None

        except requests.exceptions.ConnectionError:
            logger.error(f"Connection error: {url}")
            return None

        except requests.exceptions.RequestException as e:
            logger.error(f"Request failed: {e} - {url}")
            return None

    def random_delay(self):
        """
        Apply random delay between requests to avoid overwhelming servers.

        Uses REQUEST_DELAY from config for min/max range.
        """
        min_delay, max_delay = self.config.REQUEST_DELAY
        delay = random.uniform(min_delay, max_delay)
        logger.debug(f"Delaying for {delay:.2f} seconds")
        time.sleep(delay)

    def close(self):
        """
        Close the session and cleanup resources.
        """
        self.session.close()

    def __enter__(self):
        """Context manager entry."""
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Context manager exit."""
        self.close()
