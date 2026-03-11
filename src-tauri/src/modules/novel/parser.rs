use encoding_rs::{Encoding, UTF_8, GBK};
use regex::Regex;
use std::io::{self, Read};

/// Represents a detected encoding for a text file
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    Utf8,
    Gbk,
}

impl DetectedEncoding {
    /// Convert to encoding_rs Encoding
    fn to_encoding(&self) -> &'static Encoding {
        match self {
            DetectedEncoding::Utf8 => UTF_8,
            DetectedEncoding::Gbk => GBK,
        }
    }
}

/// Represents a chapter in a novel
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub title: String,
    pub content: String,
    pub start_position: usize,
}

/// TXT file parser for novels
pub struct TxtParser {
    chapter_patterns: Vec<Regex>,
}

impl TxtParser {
    /// Create a new TxtParser with default chapter patterns
    pub fn new() -> Self {
        Self::with_patterns(Self::default_patterns())
    }

    /// Create a TxtParser with custom patterns
    pub fn with_patterns(patterns: Vec<String>) -> Self {
        let chapter_patterns = patterns
            .into_iter()
            .filter_map(|p| Regex::new(&p).ok())
            .collect();

        Self { chapter_patterns }
    }

    /// Default chapter recognition patterns
    fn default_patterns() -> Vec<String> {
        vec![
            // Chapter 1, Chapter 001, etc.
            r"^第[0-9零一二三四五六七八九十百千万]+章\s+.+$".to_string(),
            r"^第[0-9]+章\s+.+$".to_string(),
            // Chapter 1:, Chapter 001:, etc.
            r"^第[0-9零一二三四五六七八九十百千万]+章[:：].+$".to_string(),
            // Simple patterns
            r"^章节\s*[0-9]+\s+.+$".to_string(),
            r"^Chapter\s+\d+\s+.+$".to_string(),
            r"^CHAPTER\s+\d+\s+.+$".to_string(),
        ]
    }

    /// Detect encoding of the given bytes
    pub fn detect_encoding(&self, bytes: &[u8]) -> DetectedEncoding {
        // Check for UTF-8 BOM
        if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
            return DetectedEncoding::Utf8;
        }

        // Try to decode as UTF-8
        if std::str::from_utf8(bytes).is_ok() {
            return DetectedEncoding::Utf8;
        }

        // Default to GBK for Chinese text
        DetectedEncoding::Gbk
    }

    /// Read and decode file content
    pub fn read_file<R: Read>(&self, mut reader: R) -> io::Result<String> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;

        let encoding = self.detect_encoding(&buffer);
        let (text, _, had_errors) = encoding.to_encoding().decode(&buffer);

        if had_errors {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Decoding error occurred",
            ));
        }

        Ok(text.into_owned())
    }

    /// Split content into chapters
    pub fn split_chapters(&self, content: &str) -> Vec<Chapter> {
        if self.chapter_patterns.is_empty() {
            // No patterns, return entire content as single chapter
            return vec![Chapter {
                title: "全文".to_string(),
                content: content.to_string(),
                start_position: 0,
            }];
        }

        let mut chapters = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut current_chapter: Option<(String, usize, Vec<&str>)> = None;

        for (line_idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Check if this line matches any chapter pattern
            let is_chapter = self.chapter_patterns.iter().any(|re| re.is_match(trimmed));

            if is_chapter && !trimmed.is_empty() {
                // Save previous chapter if exists
                if let Some((title, start_pos, content_lines)) = current_chapter.take() {
                    let content_text = content_lines.join("\n");
                    chapters.push(Chapter {
                        title,
                        content: content_text.trim().to_string(),
                        start_position: start_pos,
                    });
                }

                // Start new chapter
                current_chapter = Some((trimmed.to_string(), line_idx, Vec::new()));
            } else if let Some((_, _, ref mut content_lines)) = current_chapter {
                // Add line to current chapter
                content_lines.push(line);
            }
        }

        // Save last chapter if exists
        if let Some((title, start_pos, content_lines)) = current_chapter {
            let content_text = content_lines.join("\n");
            chapters.push(Chapter {
                title,
                content: content_text.trim().to_string(),
                start_position: start_pos,
            });
        }

        // If no chapters found, return entire content
        if chapters.is_empty() {
            chapters.push(Chapter {
                title: "全文".to_string(),
                content: content.to_string(),
                start_position: 0,
            });
        }

        chapters
    }

    /// Parse a TXT file from a reader
    pub fn parse<R: Read>(&self, reader: R) -> io::Result<Vec<Chapter>> {
        let content = self.read_file(reader)?;
        Ok(self.split_chapters(&content))
    }
}

impl Default for TxtParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_detect_encoding_utf8() {
        let parser = TxtParser::new();
        let utf8_text = "Hello, World!".as_bytes();
        assert_eq!(parser.detect_encoding(utf8_text), DetectedEncoding::Utf8);
    }

    #[test]
    fn test_detect_encoding_utf8_bom() {
        let parser = TxtParser::new();
        let mut bom_text = vec![0xEF, 0xBB, 0xBF];
        bom_text.extend_from_slice("Hello".as_bytes());
        assert_eq!(parser.detect_encoding(&bom_text), DetectedEncoding::Utf8);
    }

    #[test]
    fn test_detect_encoding_gbk() {
        let parser = TxtParser::new();
        // GBK encoded "你好" (invalid UTF-8)
        let gbk_text = vec![0xC4, 0xE3, 0xBA, 0xC3];
        assert_eq!(parser.detect_encoding(&gbk_text), DetectedEncoding::Gbk);
    }

    #[test]
    fn test_read_file_utf8() {
        let parser = TxtParser::new();
        let content = "Hello, 世界!";
        let reader = Cursor::new(content.as_bytes());
        let result = parser.read_file(reader).unwrap();
        assert_eq!(result, content);
    }

    #[test]
    fn test_read_file_gbk() {
        let parser = TxtParser::new();
        // GBK encoded "你好"
        let gbk_bytes = vec![0xC4, 0xE3, 0xBA, 0xC3];
        let reader = Cursor::new(gbk_bytes);
        let result = parser.read_file(reader).unwrap();
        assert_eq!(result, "你好");
    }

    #[test]
    fn test_split_chapters_simple() {
        let parser = TxtParser::new();
        let content = r#"第一章 开始
这是第一章的内容。
第二章 继续
这是第二章的内容。"#;

        let chapters = parser.split_chapters(content);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章 开始");
        assert_eq!(chapters[0].content, "这是第一章的内容。");
        assert_eq!(chapters[0].start_position, 0);
        assert_eq!(chapters[1].title, "第二章 继续");
        assert_eq!(chapters[1].content, "这是第二章的内容。");
    }

    #[test]
    fn test_split_chapters_with_numbers() {
        let parser = TxtParser::new();
        let content = r#"第1章 开始
内容1
第2章 继续
内容2"#;

        let chapters = parser.split_chapters(content);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第1章 开始");
        assert_eq!(chapters[1].title, "第2章 继续");
    }

    #[test]
    fn test_split_chapters_no_match() {
        let parser = TxtParser::new();
        let content = "这是一段没有章节标记的文本。\n只有普通内容。";

        let chapters = parser.split_chapters(content);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "全文");
        assert_eq!(chapters[0].content, content);
    }

    #[test]
    fn test_split_chapters_empty_content() {
        let parser = TxtParser::new();
        let content = "";

        let chapters = parser.split_chapters(content);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "全文");
        assert_eq!(chapters[0].content, "");
    }

    #[test]
    fn test_custom_patterns() {
        let parser = TxtParser::with_patterns(vec![
            r"^Chapter\s+\d+".to_string(),
        ]);

        let content = r#"Chapter 1
Content of chapter 1
Chapter 2
Content of chapter 2"#;

        let chapters = parser.split_chapters(content);
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "Chapter 1");
        assert_eq!(chapters[1].title, "Chapter 2");
    }

    #[test]
    fn test_parse_complete() {
        let parser = TxtParser::new();
        let content = r#"第一章 测试
这是测试内容。
第二章 结束
这是结束内容。"#;

        let reader = Cursor::new(content.as_bytes());
        let chapters = parser.parse(reader).unwrap();

        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第一章 测试");
        assert_eq!(chapters[0].content, "这是测试内容。");
        assert_eq!(chapters[1].title, "第二章 结束");
        assert_eq!(chapters[1].content, "这是结束内容。");
    }

    #[test]
    fn test_chapter_with_colon() {
        let parser = TxtParser::new();
        let content = "第一章：开始\n内容";

        let chapters = parser.split_chapters(content);
        assert_eq!(chapters.len(), 1);
        assert_eq!(chapters[0].title, "第一章：开始");
    }
}
