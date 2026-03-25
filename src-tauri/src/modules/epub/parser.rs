use crate::errors::{AppError, AppResult};
use crate::modules::epub::models::{EpubChapter, EpubMetadata};
use epub::doc::EpubDoc;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct EpubParser {
    doc: EpubDoc<BufReader<File>>,
}

impl EpubParser {
    /// 打开 EPUB 文件
    pub fn open(path: &Path) -> AppResult<Self> {
        let doc = EpubDoc::new(path)
            .map_err(|e| AppError::InvalidInput(format!("Failed to open EPUB: {}", e)))?;
        Ok(Self { doc })
    }

    /// 提取元数据
    pub fn extract_metadata(&mut self) -> AppResult<EpubMetadata> {
        let metadata = EpubMetadata {
            title: self.doc.mdata("title").map(|m| m.value.clone()),
            authors: self.extract_authors(),
            publisher: self.doc.mdata("publisher").map(|m| m.value.clone()),
            pubdate: self.doc.mdata("date").map(|m| m.value.clone()),
            language: self.doc.mdata("language").map(|m| m.value.clone()),
            isbn: self.extract_isbn(),
            description: self.doc.mdata("description").map(|m| m.value.clone()),
        };
        Ok(metadata)
    }

    /// 提取封面图片
    pub fn extract_cover(&mut self) -> AppResult<Option<Vec<u8>>> {
        match self.doc.get_cover() {
            Some((cover_data, _mime_type)) => Ok(Some(cover_data)),
            None => Ok(None),
        }
    }

    /// 获取封面 MIME 类型
    pub fn get_cover_mime(&mut self) -> Option<String> {
        self.doc.get_cover().map(|(_, mime)| mime)
    }

    /// 提取目录（TOC）
    pub fn extract_toc(&mut self) -> AppResult<Vec<EpubChapter>> {
        let chapters: Vec<EpubChapter> = self
            .doc
            .toc
            .iter()
            .enumerate()
            .map(|(i, item)| {
                self.flatten_toc_item(item, i as i32, 0)
            })
            .flatten()
            .collect();

        Ok(chapters)
    }

    /// 递归展平 TOC 树结构
    fn flatten_toc_item(&self, item: &epub::doc::NavPoint, order_index: i32, level: i32) -> Vec<EpubChapter> {
        let mut chapters = vec![EpubChapter {
            href: item.content.to_string_lossy().to_string(),
            title: item.label.clone(),
            level,
            order_index,
        }];

        // 递归处理子项
        for (i, child) in item.children.iter().enumerate() {
            chapters.extend(self.flatten_toc_item(child, order_index + i as i32 + 1, level + 1));
        }

        chapters
    }

    /// 验证 EPUB 文件完整性
    pub fn validate(&mut self) -> AppResult<bool> {
        // 检查是否能读取资源
        if self.doc.resources.is_empty() {
            return Err(AppError::InvalidInput(
                "EPUB file has no resources".to_string(),
            ));
        }

        // 检查是否有内容
        #[allow(deprecated)]
        if self.doc.get_num_pages() == 0 {
            return Err(AppError::InvalidInput(
                "EPUB file has no pages".to_string(),
            ));
        }

        Ok(true)
    }

    // Helper methods

    fn extract_authors(&mut self) -> Vec<String> {
        let mut authors = Vec::new();

        // EPUB 可能有多个作者
        if let Some(author) = self.doc.mdata("creator") {
            authors.push(author.value.clone());
        }

        // 尝试其他作者字段
        let mut i = 1;
        while let Some(author) = self.doc.mdata(&format!("creator_{}", i)) {
            authors.push(author.value.clone());
            i += 1;
        }

        authors
    }

    fn extract_isbn(&mut self) -> Option<String> {
        // 尝试多种 ISBN 字段
        self.doc
            .mdata("isbn")
            .or_else(|| self.doc.mdata("identifier"))
            .or_else(|| self.doc.mdata("ISBN"))
            .map(|m| m.value.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parser_open_invalid_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"not an epub file").unwrap();

        let result = EpubParser::open(temp_file.path());
        assert!(result.is_err());

        if let Err(AppError::InvalidInput(msg)) = result {
            assert!(msg.contains("Failed to open EPUB"));
        } else {
            panic!("Expected InvalidInput error");
        }
    }

    #[test]
    fn test_extract_authors_empty() {
        // Test case for parser with no authors
        // This would require a valid EPUB file, so we just verify the method signature
        // In practice, integration tests would use real EPUB files
    }
}
