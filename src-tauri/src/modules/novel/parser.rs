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
    pub preview: String, // First line preview (up to 20 chars)
    pub start_position: usize,
}

/// Metadata extracted from novel file
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovelMetadata {
    pub author: Option<String>,
    pub description: Option<String>,
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
            // Volume patterns (卷)
            r"^第[0-9零一二三四五六七八九十百千万]+卷\s+.+$".to_string(),
            r"^第[0-9]+卷\s+.+$".to_string(),
            r"^第[0-9零一二三四五六七八九十百千万]+卷[:：].+$".to_string(),
            // Chapter patterns (章)
            r"^第[0-9零一二三四五六七八九十百千万]+章\s+.+$".to_string(),
            r"^第[0-9]+章\s+.+$".to_string(),
            r"^第[0-9零一二三四五六七八九十百千万]+章[:：].+$".to_string(),
            // Section patterns (节)
            r"^第[0-9零一二三四五六七八九十百千万]+节\s+.+$".to_string(),
            r"^第[0-9]+节\s+.+$".to_string(),
            r"^第[0-9零一二三四五六七八九十百千万]+节[:：].+$".to_string(),
            // Part patterns (回、部)
            r"^第[0-9零一二三四五六七八九十百千万]+回\s+.+$".to_string(),
            r"^第[0-9]+回\s+.+$".to_string(),
            r"^第[0-9零一二三四五六七八九十百千万]+部\s+.+$".to_string(),
            r"^第[0-9]+部\s+.+$".to_string(),
            // Special patterns (楔子、序章、引子、终幕、后记、番外、完本感言)
            r"^(楔子|序章|序言|序 |引子|终幕|后记|番外|完本感言).*$".to_string(),
            // Simple patterns
            r"^章节\s*[0-9]+\s+.+$".to_string(),
            r"^Chapter\s+\d+\s+.+$".to_string(),
            r"^CHAPTER\s+\d+\s+.+$".to_string(),
            r"^Volume\s+\d+\s+.+$".to_string(),
            r"^Section\s+\d+\s+.+$".to_string(),
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

    /// Extract preview text from content (first line, up to 20 chars)
    fn extract_preview(content: &str) -> String {
        let first_line = content
            .lines()
            .find(|line| !line.trim().is_empty())
            .unwrap_or("");

        let trimmed = first_line.trim();
        if trimmed.chars().count() > 20 {
            trimmed.chars().take(20).collect::<String>() + "..."
        } else {
            trimmed.to_string()
        }
    }

    /// Split content into chapters
    pub fn split_chapters(&self, content: &str) -> Vec<Chapter> {
        if self.chapter_patterns.is_empty() {
            // No patterns, return entire content as single chapter
            return vec![Chapter {
                title: "全文".to_string(),
                preview: Self::extract_preview(content),
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
                    let trimmed_content = content_text.trim().to_string();
                    chapters.push(Chapter {
                        title,
                        preview: Self::extract_preview(&trimmed_content),
                        content: trimmed_content,
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
            let trimmed_content = content_text.trim().to_string();
            chapters.push(Chapter {
                title,
                preview: Self::extract_preview(&trimmed_content),
                content: trimmed_content,
                start_position: start_pos,
            });
        }

        // If no chapters found, return entire content
        if chapters.is_empty() {
            chapters.push(Chapter {
                title: "全文".to_string(),
                preview: Self::extract_preview(content),
                content: content.to_string(),
                start_position: 0,
            });
        }

        chapters
    }

    /// Extract metadata (author, description) from content
    pub fn extract_metadata(&self, content: &str) -> NovelMetadata {
        let mut author = None;
        let mut description = None;

        // Try to extract from first 50 lines
        let lines: Vec<&str> = content.lines().take(50).collect();

        // Enhanced author patterns
        let author_patterns = vec![
            r"(?i)^作者[:：\s]+(.+)$",
            r"(?i)^author[:：\s]+(.+)$",
            r"(?i)^文\s*[:：]\s*(.+)$",
            r"(?i)^by[:：\s]+(.+)$",
        ];

        // Enhanced description patterns
        let desc_patterns = vec![
            r"(?i)^(简介|内容简介|内容介绍|简要介绍|作品简介|小说简介|书籍简介)[:：\s]+(.+)$",
            r"(?i)^(introduction|description)[:：\s]+(.+)$",
        ];

        let mut desc_start_idx = None;
        let mut in_description = false;

        for (idx, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Try to match author patterns
            if author.is_none() {
                for pattern in &author_patterns {
                    if let Some(captures) = Regex::new(pattern)
                        .ok()
                        .and_then(|re| re.captures(trimmed))
                    {
                        let extracted = captures.get(captures.len() - 1).unwrap().as_str().trim();
                        if !extracted.is_empty() && extracted.chars().count() <= 30 {
                            author = Some(extracted.to_string());
                            break;
                        }
                    }
                }
            }

            // Try to match description patterns
            if description.is_none() && !in_description {
                for pattern in &desc_patterns {
                    if let Some(captures) = Regex::new(pattern)
                        .ok()
                        .and_then(|re| re.captures(trimmed))
                    {
                        let extracted = captures.get(captures.len() - 1).unwrap().as_str().trim();
                        if !extracted.is_empty() {
                            desc_start_idx = Some(idx);
                            in_description = true;
                            break;
                        }
                    }
                }
            }
        }

        // Extract multi-line description if found
        if let Some(start_idx) = desc_start_idx {
            let mut desc_lines = Vec::new();

            for line in lines.iter().skip(start_idx) {
                let trimmed = line.trim();

                // Stop at chapter title or next metadata field
                let is_chapter = self.chapter_patterns.iter().any(|re| re.is_match(trimmed));
                let is_next_metadata = Regex::new(r"(?i)^(作者|标签|类型|状态|字数|分类|更新)[:：]")
                    .ok()
                    .map(|re| re.is_match(trimmed))
                    .unwrap_or(false);

                if is_chapter || is_next_metadata {
                    break;
                }

                if !trimmed.is_empty() {
                    desc_lines.push(trimmed);
                } else if !desc_lines.is_empty() {
                    // Allow one empty line, but stop at two consecutive empty lines
                    break;
                }
            }

            if !desc_lines.is_empty() {
                // Extract description text (skip the label line if it only contains label)
                let first_line = desc_lines[0];
                if let Some(captures) = Regex::new(r"(?i)^(简介|内容简介|内容介绍|简要介绍|作品简介|小说简介|书籍简介)[:：\s]+(.+)$")
                    .ok()
                    .and_then(|re| re.captures(first_line))
                {
                    // First line has both label and content
                    let first_content = captures.get(2).unwrap().as_str().trim();
                    if desc_lines.len() > 1 {
                        desc_lines[0] = first_content;
                    } else if !first_content.is_empty() {
                        desc_lines = vec![first_content];
                    } else {
                        desc_lines.clear();
                    }
                }

                let desc_text = desc_lines.join("");
                if desc_text.chars().count() >= 10 && desc_text.chars().count() <= 500 {
                    description = Some(desc_text);
                }
            }
        }

        // If description still not found, try to extract first meaningful paragraph
        if description.is_none() {
            let mut paragraph_lines = Vec::new();
            let mut found_content = false;

            for line in &lines {
                let trimmed = line.trim();

                // Skip metadata lines, chapter titles, and empty lines at start
                let is_chapter = self.chapter_patterns.iter().any(|re| re.is_match(trimmed));
                let is_metadata = Regex::new(r"(?i)^(作者|简介|标签|类型|状态|字数|分类|更新)[:：]")
                    .ok()
                    .map(|re| re.is_match(trimmed))
                    .unwrap_or(false);

                if is_chapter || is_metadata {
                    if found_content {
                        break;
                    }
                    continue;
                }

                if !trimmed.is_empty() {
                    found_content = true;
                    paragraph_lines.push(trimmed);

                    // Stop if we have enough content
                    let current_text = paragraph_lines.join("");
                    if current_text.chars().count() >= 50 {
                        break;
                    }
                } else if found_content {
                    // Stop at first empty line after content
                    break;
                }
            }

            if !paragraph_lines.is_empty() {
                let paragraph = paragraph_lines.join("");
                if paragraph.chars().count() >= 10 && paragraph.chars().count() <= 500 {
                    description = Some(paragraph);
                }
            }
        }

        NovelMetadata {
            author,
            description,
        }
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
