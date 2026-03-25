use crate::errors::{AppError, AppResult};
use crate::modules::epub::models::{EpubChapter, EpubMetadata};
use epub::doc::EpubDoc;
use std::cell::RefCell;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct EpubParser {
    doc: RefCell<EpubDoc<BufReader<File>>>,
}

impl EpubParser {
    /// 打开 EPUB 文件
    pub fn open(path: &Path) -> AppResult<Self> {
        // 验证文件存在
        if !path.exists() {
            return Err(AppError::NotFound(format!(
                "File not found: {}",
                path.display()
            )));
        }

        if !path.is_file() {
            return Err(AppError::InvalidInput(format!(
                "Path is not a file: {}",
                path.display()
            )));
        }

        // 验证扩展名
        if path.extension().and_then(|s| s.to_str()) != Some("epub") {
            return Err(AppError::InvalidInput(
                "File must have .epub extension".to_string(),
            ));
        }

        let doc = EpubDoc::new(path)
            .map_err(|e| AppError::InvalidInput(format!("Failed to open EPUB: {}", e)))?;
        Ok(Self {
            doc: RefCell::new(doc),
        })
    }

    /// 提取元数据
    pub fn extract_metadata(&self) -> AppResult<EpubMetadata> {
        let doc = self.doc.borrow_mut();
        let metadata = EpubMetadata {
            title: doc.mdata("title").map(|m| m.value.clone()),
            authors: self.extract_authors(),
            publisher: doc.mdata("publisher").map(|m| m.value.clone()),
            pubdate: doc.mdata("date").map(|m| m.value.clone()),
            language: doc.mdata("language").map(|m| m.value.clone()),
            isbn: self.extract_isbn(),
            description: doc.mdata("description").map(|m| m.value.clone()),
        };
        Ok(metadata)
    }

    /// 提取封面图片
    pub fn extract_cover(&self) -> AppResult<Option<Vec<u8>>> {
        let mut doc = self.doc.borrow_mut();
        match doc.get_cover() {
            Some((cover_data, _mime_type)) => Ok(Some(cover_data)),
            None => Ok(None),
        }
    }

    /// 获取封面 MIME 类型
    pub fn get_cover_mime(&self) -> Option<String> {
        let mut doc = self.doc.borrow_mut();
        doc.get_cover().map(|(_, mime)| mime)
    }

    /// 提取目录（TOC）
    pub fn extract_toc(&self) -> AppResult<Vec<EpubChapter>> {
        let doc = self.doc.borrow();
        let mut chapters = Vec::new();
        let mut order_index = 0;

        for item in doc.toc.iter() {
            chapters.extend(self.flatten_toc_item(item, &mut order_index, 0));
        }

        Ok(chapters)
    }

    /// 递归展平 TOC 树结构
    fn flatten_toc_item(
        &self,
        item: &epub::doc::NavPoint,
        next_index: &mut i32,
        level: i32,
    ) -> Vec<EpubChapter> {
        let mut chapters = vec![EpubChapter {
            href: item.content.to_string_lossy().to_string(),
            title: item.label.clone(),
            level,
            order_index: *next_index,
        }];
        *next_index += 1;

        // 递归处理子项
        for child in item.children.iter() {
            chapters.extend(self.flatten_toc_item(child, next_index, level + 1));
        }

        chapters
    }

    /// 验证 EPUB 文件完整性
    pub fn validate(&self) -> AppResult<bool> {
        let doc = self.doc.borrow();

        // 检查是否能读取资源
        if doc.resources.is_empty() {
            return Err(AppError::InvalidInput(
                "EPUB file has no resources".to_string(),
            ));
        }

        // 检查 spine 是否有内容
        if doc.spine.is_empty() {
            return Err(AppError::InvalidInput(
                "EPUB file has no content in spine".to_string(),
            ));
        }

        Ok(true)
    }

    // Helper methods

    fn extract_authors(&self) -> Vec<String> {
        let doc = self.doc.borrow_mut();
        let mut authors = Vec::new();

        // EPUB 可能有多个作者
        if let Some(author) = doc.mdata("creator") {
            authors.push(author.value.clone());
        }

        // 尝试其他作者字段
        let mut i = 1;
        while let Some(author) = doc.mdata(&format!("creator_{}", i)) {
            authors.push(author.value.clone());
            i += 1;
        }

        authors
    }

    fn extract_isbn(&self) -> Option<String> {
        let doc = self.doc.borrow_mut();
        // 尝试多种 ISBN 字段
        doc.mdata("isbn")
            .or_else(|| doc.mdata("identifier"))
            .or_else(|| doc.mdata("ISBN"))
            .map(|m| m.value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_parser_open_invalid_file() {
        let temp_dir = TempDir::new().unwrap();
        let epub_path = temp_dir.path().join("invalid.epub");

        // Create a file with .epub extension but invalid content
        let mut file = fs::File::create(&epub_path).unwrap();
        file.write_all(b"not an epub file").unwrap();
        drop(file);

        let result = EpubParser::open(&epub_path);
        assert!(result.is_err());

        if let Err(AppError::InvalidInput(msg)) = result {
            assert!(msg.contains("Failed to open EPUB"));
        } else {
            panic!("Expected InvalidInput error");
        }
    }

    #[test]
    fn test_parser_open_file_not_found() {
        let result = EpubParser::open(std::path::Path::new("/nonexistent/file.epub"));
        assert!(result.is_err());

        if let Err(AppError::NotFound(msg)) = result {
            assert!(msg.contains("File not found"));
        } else {
            panic!("Expected NotFound error");
        }
    }

    #[test]
    fn test_parser_open_wrong_extension() {
        let temp_dir = TempDir::new().unwrap();
        let txt_path = temp_dir.path().join("file.txt");

        // Create a file with wrong extension
        let mut file = fs::File::create(&txt_path).unwrap();
        file.write_all(b"some content").unwrap();
        drop(file);

        let result = EpubParser::open(&txt_path);
        assert!(result.is_err());

        if let Err(AppError::InvalidInput(msg)) = result {
            assert!(msg.contains(".epub extension"));
        } else {
            panic!("Expected InvalidInput error for wrong extension");
        }
    }

}
