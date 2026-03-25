"""
Data models for the novel crawler system.
"""
from dataclasses import dataclass, asdict
from typing import Optional, List
from datetime import datetime
import json


@dataclass
class Book:
    """
    Book model representing a novel entry from the main index (youshu.db).
    """
    id: int
    title: str
    author: Optional[str] = None
    description: Optional[str] = None
    tags: Optional[List[str]] = None
    cover_path: Optional[str] = None
    source_site: Optional[str] = None
    source_url: Optional[str] = None
    update_status: Optional[str] = None
    crawled_at: Optional[datetime] = None
    updated_at: Optional[datetime] = None

    def to_dict(self) -> dict:
        """
        Convert book to dictionary.

        Returns:
            Dictionary representation of the book
        """
        data = asdict(self)
        # Convert tags list to JSON string
        if self.tags:
            data['tags'] = json.dumps(self.tags, ensure_ascii=False)
        # Convert datetime objects to ISO format strings
        if self.crawled_at:
            data['crawled_at'] = self.crawled_at.isoformat()
        if self.updated_at:
            data['updated_at'] = self.updated_at.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: dict) -> 'Book':
        """
        Create Book instance from dictionary.

        Args:
            data: Dictionary containing book data

        Returns:
            Book instance
        """
        # Parse tags from JSON string
        if data.get('tags') and isinstance(data['tags'], str):
            try:
                data['tags'] = json.loads(data['tags'])
            except json.JSONDecodeError:
                data['tags'] = None

        # Convert ISO format strings to datetime
        if data.get('crawled_at'):
            if isinstance(data['crawled_at'], str):
                data['crawled_at'] = datetime.fromisoformat(data['crawled_at'])
        if data.get('updated_at'):
            if isinstance(data['updated_at'], str):
                data['updated_at'] = datetime.fromisoformat(data['updated_at'])

        return cls(**data)

    def to_tuple(self) -> tuple:
        """
        Convert book to tuple for database insertion.

        Returns:
            Tuple of book fields in order
        """
        tags_json = json.dumps(self.tags, ensure_ascii=False) if self.tags else None
        crawled_at_iso = self.crawled_at.isoformat() if self.crawled_at else None
        updated_at_iso = self.updated_at.isoformat() if self.updated_at else None

        return (
            self.id,
            self.title,
            self.author,
            self.description,
            tags_json,
            self.cover_path,
            self.source_site,
            self.source_url,
            self.update_status,
            crawled_at_iso,
            updated_at_iso
        )


@dataclass
class SourceBookDetail:
    """
    Detailed book model from source sites (qidian.db, zongheng.db, etc.).
    """
    book_id: int
    youshu_id: Optional[int] = None
    title: str = ""
    author: Optional[str] = None
    description: Optional[str] = None
    category: Optional[str] = None
    sub_category: Optional[str] = None
    tags: Optional[List[str]] = None
    cover_url: Optional[str] = None
    cover_path: Optional[str] = None
    word_count: Optional[int] = None
    chapter_count: Optional[int] = None
    status: Optional[str] = None
    rating: Optional[float] = None
    view_count: Optional[int] = None
    favorite_count: Optional[int] = None
    crawled_at: Optional[datetime] = None

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        data = asdict(self)
        if self.tags:
            data['tags'] = json.dumps(self.tags, ensure_ascii=False)
        if self.crawled_at:
            data['crawled_at'] = self.crawled_at.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: dict) -> 'SourceBookDetail':
        """Create instance from dictionary."""
        if data.get('tags') and isinstance(data['tags'], str):
            try:
                data['tags'] = json.loads(data['tags'])
            except json.JSONDecodeError:
                data['tags'] = None
        if data.get('crawled_at') and isinstance(data['crawled_at'], str):
            data['crawled_at'] = datetime.fromisoformat(data['crawled_at'])
        return cls(**data)


@dataclass
class CrawlStatus:
    """
    Model representing crawl status and statistics.
    """
    id: Optional[int] = None
    last_valid_id: int = 0
    last_crawl_date: Optional[datetime] = None
    total_books: int = 0
    failed_ids: Optional[List[int]] = None
    crawl_type: Optional[str] = None  # initial, incremental, retry
    duration_seconds: Optional[int] = None
    success_count: int = 0
    failure_count: int = 0

    def to_dict(self) -> dict:
        """Convert to dictionary."""
        data = asdict(self)
        if self.failed_ids:
            data['failed_ids'] = json.dumps(self.failed_ids)
        if self.last_crawl_date:
            data['last_crawl_date'] = self.last_crawl_date.isoformat()
        return data

    @classmethod
    def from_dict(cls, data: dict) -> 'CrawlStatus':
        """Create instance from dictionary."""
        if data.get('failed_ids') and isinstance(data['failed_ids'], str):
            try:
                data['failed_ids'] = json.loads(data['failed_ids'])
            except json.JSONDecodeError:
                data['failed_ids'] = []
        if data.get('last_crawl_date') and isinstance(data['last_crawl_date'], str):
            data['last_crawl_date'] = datetime.fromisoformat(data['last_crawl_date'])
        return cls(**data)
