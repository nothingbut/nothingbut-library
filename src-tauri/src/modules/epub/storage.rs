use std::fs;
use std::path::{Path, PathBuf};
use image::{imageops::FilterType, ImageFormat};
use crate::errors::{AppError, AppResult};

/// EPUB 存储管理器
pub struct EpubStorage {
    workspace_path: PathBuf,
}

impl EpubStorage {
    /// 创建新的存储管理器
    pub fn new(workspace_path: impl AsRef<Path>) -> Self {
        Self {
            workspace_path: workspace_path.as_ref().to_path_buf(),
        }
    }

    /// 获取 EPUB 根目录
    pub fn epub_root(&self) -> PathBuf {
        self.workspace_path.join("epub")
    }

    /// 获取书籍目录
    pub fn book_dir(&self, book_id: i64) -> PathBuf {
        self.epub_root().join("books").join(book_id.to_string())
    }

    /// 确保 EPUB 根目录存在
    pub fn ensure_epub_root(&self) -> AppResult<()> {
        let epub_root = self.epub_root();
        if !epub_root.exists() {
            fs::create_dir_all(&epub_root).map_err(|e| {
                AppError::Io(format!("Failed to create EPUB root directory: {}", e))
            })?;
        }
        Ok(())
    }

    /// 创建书籍目录
    pub fn create_book_dir(&self, book_id: i64) -> AppResult<PathBuf> {
        let book_dir = self.book_dir(book_id);
        fs::create_dir_all(&book_dir).map_err(|e| {
            AppError::Io(format!("Failed to create book directory: {}", e))
        })?;
        Ok(book_dir)
    }

    /// 复制 EPUB 文件到书库
    pub fn copy_epub_file(&self, source_path: &Path, book_id: i64) -> AppResult<PathBuf> {
        // 确保源文件存在
        if !source_path.exists() {
            return Err(AppError::NotFound(format!(
                "EPUB file not found: {}",
                source_path.display()
            )));
        }

        // 创建书籍目录
        let book_dir = self.create_book_dir(book_id)?;

        // 目标文件路径（始终命名为 book.epub）
        let dest_path = book_dir.join("book.epub");

        // 复制文件
        fs::copy(source_path, &dest_path).map_err(|e| {
            AppError::Io(format!("Failed to copy EPUB file: {}", e))
        })?;

        // 返回目标文件路径
        Ok(dest_path)
    }

    /// 获取 EPUB 文件路径
    pub fn epub_file_path(&self, book_id: i64) -> PathBuf {
        self.book_dir(book_id).join("book.epub")
    }

    /// 保存封面图片（生成两个尺寸）
    pub fn save_cover(&self, cover_data: &[u8], book_id: i64) -> AppResult<()> {
        // 创建书籍目录
        let book_dir = self.create_book_dir(book_id)?;

        // 加载原始图片
        let img = image::load_from_memory(cover_data).map_err(|e| {
            AppError::InvalidInput(format!("Failed to load cover image: {}", e))
        })?;

        // 生成大封面 (600x800, 保持纵横比)
        let large_cover = img.resize(600, 800, FilterType::Lanczos3);
        let large_path = book_dir.join("cover.jpg");
        large_cover
            .save_with_format(&large_path, ImageFormat::Jpeg)
            .map_err(|e| AppError::Io(format!("Failed to save large cover: {}", e)))?;

        // 生成缩略图 (200x267, 保持纵横比)
        let thumb_cover = img.resize(200, 267, FilterType::Lanczos3);
        let thumb_path = book_dir.join("cover_thumb.jpg");
        thumb_cover
            .save_with_format(&thumb_path, ImageFormat::Jpeg)
            .map_err(|e| AppError::Io(format!("Failed to save thumbnail cover: {}", e)))?;

        Ok(())
    }

    /// 获取大封面路径
    pub fn cover_path(&self, book_id: i64) -> PathBuf {
        self.book_dir(book_id).join("cover.jpg")
    }

    /// 获取缩略图路径
    pub fn cover_thumb_path(&self, book_id: i64) -> PathBuf {
        self.book_dir(book_id).join("cover_thumb.jpg")
    }

    /// 保存元数据 JSON 备份
    pub fn save_metadata_json(&self, book_id: i64, metadata: &serde_json::Value) -> AppResult<()> {
        let book_dir = self.book_dir(book_id);
        let metadata_path = book_dir.join("metadata.json");

        let json_str = serde_json::to_string_pretty(metadata)
            .map_err(|e| AppError::Json(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_path, json_str)
            .map_err(|e| AppError::Io(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// 删除书籍所有文件
    pub fn delete_book(&self, book_id: i64) -> AppResult<()> {
        let book_dir = self.book_dir(book_id);

        if book_dir.exists() {
            fs::remove_dir_all(&book_dir).map_err(|e| {
                AppError::Io(format!("Failed to delete book directory: {}", e))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_epub_root() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let epub_root = manager.epub_root();
        assert_eq!(epub_root, temp_dir.path().join("epub"));
    }

    #[test]
    fn test_book_dir() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let book_dir = manager.book_dir(123);
        assert_eq!(book_dir, temp_dir.path().join("epub/books/123"));
    }

    #[test]
    fn test_ensure_epub_root() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        manager.ensure_epub_root().unwrap();
        assert!(manager.epub_root().exists());
    }

    #[test]
    fn test_create_book_dir() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let book_dir = manager.create_book_dir(456).unwrap();
        assert!(book_dir.exists());
        assert_eq!(book_dir, temp_dir.path().join("epub/books/456"));
    }

    #[test]
    fn test_copy_epub_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        // 创建临时 EPUB 文件
        let source_file = temp_dir.path().join("test.epub");
        fs::write(&source_file, b"fake epub content").unwrap();

        // 复制文件
        let dest_path = manager.copy_epub_file(&source_file, 789).unwrap();

        // 验证返回的是目标文件路径
        assert_eq!(dest_path, manager.book_dir(789).join("book.epub"));

        // 验证文件存在且内容正确
        assert!(dest_path.exists());
        let content = fs::read_to_string(&dest_path).unwrap();
        assert_eq!(content, "fake epub content");
    }

    #[test]
    fn test_copy_nonexistent_file() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let result = manager.copy_epub_file(Path::new("/nonexistent/file.epub"), 999);
        assert!(result.is_err());

        match result {
            Err(AppError::NotFound(msg)) => {
                assert!(msg.contains("EPUB file not found"));
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn test_save_cover() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        // 创建一个简单的 1x1 像素图片
        let img = image::RgbImage::from_pixel(1, 1, image::Rgb([255, 0, 0]));
        let mut cover_data = Vec::new();
        img.write_to(&mut std::io::Cursor::new(&mut cover_data), ImageFormat::Jpeg)
            .unwrap();

        // 保存封面
        manager.save_cover(&cover_data, 101).unwrap();

        // 验证文件存在
        assert!(manager.cover_path(101).exists());
        assert!(manager.cover_thumb_path(101).exists());
    }

    #[test]
    fn test_save_invalid_cover() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let invalid_data = b"not an image";
        let result = manager.save_cover(invalid_data, 102);

        assert!(result.is_err());
        match result {
            Err(AppError::InvalidInput(msg)) => {
                assert!(msg.contains("Failed to load cover image"));
            }
            _ => panic!("Expected InvalidInput error"),
        }
    }

    #[test]
    fn test_save_metadata_json() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        manager.create_book_dir(201).unwrap();

        let metadata = serde_json::json!({
            "title": "Test Book",
            "authors": ["Author One", "Author Two"],
            "publisher": "Test Publisher",
            "pubdate": "2025-01-01",
            "language": "en",
            "isbn": "1234567890",
            "description": "A test book",
        });

        manager.save_metadata_json(201, &metadata).unwrap();

        // 验证文件存在
        let metadata_path = manager.book_dir(201).join("metadata.json");
        assert!(metadata_path.exists());

        // 验证内容
        let json_str = fs::read_to_string(&metadata_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

        assert_eq!(json["title"], "Test Book");
        assert_eq!(json["authors"][0], "Author One");
        assert_eq!(json["publisher"], "Test Publisher");
    }

    #[test]
    fn test_delete_book() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        // 创建书籍目录和文件
        manager.create_book_dir(301).unwrap();
        let test_file = manager.book_dir(301).join("test.txt");
        fs::write(&test_file, b"test content").unwrap();

        assert!(manager.book_dir(301).exists());

        // 删除书籍
        manager.delete_book(301).unwrap();

        // 验证已删除
        assert!(!manager.book_dir(301).exists());
    }

    #[test]
    fn test_delete_nonexistent_book() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        // 删除不存在的书籍应该成功（幂等）
        let result = manager.delete_book(999);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cover_paths() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let cover_path = manager.cover_path(401);
        let thumb_path = manager.cover_thumb_path(401);

        assert_eq!(cover_path, temp_dir.path().join("epub/books/401/cover.jpg"));
        assert_eq!(
            thumb_path,
            temp_dir.path().join("epub/books/401/cover_thumb.jpg")
        );
    }

    #[test]
    fn test_epub_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let manager = EpubStorage::new(temp_dir.path());

        let epub_path = manager.epub_file_path(501);
        assert_eq!(epub_path, temp_dir.path().join("epub/books/501/book.epub"));
    }
}
