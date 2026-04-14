"""
Logging configuration for the novel crawler system.
"""
import logging
import sys
from pathlib import Path
from logging.handlers import RotatingFileHandler

from config import settings


def setup_logger(name: str = 'novel_crawler') -> logging.Logger:
    """
    Setup and configure logger with file and console handlers.

    Args:
        name: Logger name

    Returns:
        Configured logger instance
    """
    logger = logging.getLogger(name)
    logger.setLevel(getattr(logging, settings.LOG_LEVEL))

    # Avoid duplicate handlers
    if logger.handlers:
        return logger

    # Formatter
    formatter = logging.Formatter(settings.LOG_FORMAT)

    # Console handler - force UTF-8 encoding for Windows
    console_handler = logging.StreamHandler(sys.stdout)
    console_handler.setLevel(logging.INFO)
    # Set UTF-8 encoding for console to handle Chinese characters and Unicode symbols
    if hasattr(console_handler.stream, 'reconfigure'):
        try:
            console_handler.stream.reconfigure(encoding='utf-8')
        except (ValueError, OSError):
            # If UTF-8 fails, fall back to default encoding
            pass
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)

    # File handler - general log
    file_handler = RotatingFileHandler(
        settings.LOG_FILE,
        maxBytes=50 * 1024 * 1024,  # 50MB - reduces rotation frequency
        backupCount=5,
        encoding='utf-8'
    )
    file_handler.setLevel(getattr(logging, settings.LOG_LEVEL))
    file_handler.setFormatter(formatter)
    logger.addHandler(file_handler)

    # Error handler - separate error log
    error_handler = RotatingFileHandler(
        settings.ERROR_LOG_FILE,
        maxBytes=10 * 1024 * 1024,  # 10MB
        backupCount=5,
        encoding='utf-8'
    )
    error_handler.setLevel(logging.ERROR)
    error_handler.setFormatter(formatter)
    logger.addHandler(error_handler)

    return logger


def get_logger(name: str) -> logging.Logger:
    """
    Get or create a logger with the given name.

    Args:
        name: Logger name

    Returns:
        Logger instance
    """
    return logging.getLogger(name)


def shutdown_loggers():
    """
    Shutdown all loggers and close file handlers.

    This should be called before program exit to ensure all file handles
    are properly closed, especially important on Windows which locks files.
    """
    # Close all handlers for all loggers
    for logger_name in list(logging.Logger.manager.loggerDict.keys()):
        logger = logging.getLogger(logger_name)
        for handler in logger.handlers[:]:
            handler.close()
            logger.removeHandler(handler)

    # Also shutdown the root logger
    logging.shutdown()
