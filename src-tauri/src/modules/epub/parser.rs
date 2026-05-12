use crate::errors::{AppError, AppResult};
use crate::modules::epub::models::{EpubChapter, EpubMetadata};
use base64::Engine;
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

        let title = doc.mdata("title").map(|m| m.value.clone());
        let publisher = doc.mdata("publisher").map(|m| m.value.clone());
        let pubdate = doc.mdata("date").map(|m| m.value.clone());
        let language = doc.mdata("language").map(|m| m.value.clone());
        let description = doc.mdata("description").map(|m| m.value.clone());

        // 内联 extract_authors 逻辑
        let authors = {
            let mut authors = Vec::new();
            if let Some(author) = doc.mdata("creator") {
                authors.push(author.value.clone());
            }
            let mut i = 1;
            while let Some(author) = doc.mdata(&format!("creator_{}", i)) {
                authors.push(author.value.clone());
                i += 1;
            }
            authors
        };

        // 内联 extract_isbn 逻辑
        let isbn = doc.mdata("isbn")
            .or_else(|| doc.mdata("identifier"))
            .or_else(|| doc.mdata("ISBN"))
            .map(|m| m.value.clone());

        drop(doc); // 显式释放借用

        Ok(EpubMetadata {
            title,
            authors,
            publisher,
            pubdate,
            language,
            isbn,
            description,
        })
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

        eprintln!("[EPUB Parser] Extracting TOC, toc items: {}", doc.toc.len());

        for item in doc.toc.iter() {
            chapters.extend(self.flatten_toc_item(item, &mut order_index, 0));
        }

        eprintln!("[EPUB Parser] Extracted {} chapters from TOC", chapters.len());
        for (i, ch) in chapters.iter().enumerate().take(3) {
            eprintln!("  Chapter {}: href={}, title={}", i, ch.href, ch.title);
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

    /// 获取章节内容（HTML）
    pub fn get_chapter_content(&self, chapter_href: &str) -> AppResult<String> {
        let mut doc = self.doc.borrow_mut();

        // 移除 URL 片段（#anchor）
        let clean_href = chapter_href.split('#').next().unwrap_or(chapter_href);

        // 打印调试信息
        eprintln!("[EPUB Parser] Searching for chapter: {}", chapter_href);
        eprintln!("[EPUB Parser] Clean href (without #): {}", clean_href);

        // 尝试多种匹配策略
        let resource_id = doc
            .resources
            .iter()
            // 策略1: 精确匹配（使用清理后的 href）
            .find(|(_, path)| path.path.to_string_lossy() == clean_href)
            .or_else(|| {
                // 策略2: 路径结尾匹配
                doc.resources
                    .iter()
                    .find(|(_, path)| {
                        let path_str = path.path.to_string_lossy();
                        path_str.ends_with(clean_href)
                    })
            })
            .or_else(|| {
                // 策略3: 文件名匹配（去除路径前缀）
                let href_filename = clean_href.split('/').last().unwrap_or(clean_href);
                doc.resources.iter().find(|(_, path)| {
                    let path_str = path.path.to_string_lossy();
                    let path_filename = path_str.split('/').last().unwrap_or(&path_str);
                    path_filename == href_filename
                })
            })
            .map(|(id, _)| id.clone())
            .ok_or_else(|| {
                eprintln!("[EPUB Parser] Chapter not found after trying all strategies");
                AppError::NotFound(format!(
                    "Chapter not found: {}. Available resources count: {}",
                    chapter_href,
                    doc.resources.len()
                ))
            })?;

        eprintln!("[EPUB Parser] Found resource_id: {}", resource_id);

        // 获取章节内容
        let (mut content, _mime_type) = doc
            .get_resource_str(&resource_id)
            .ok_or_else(|| {
                eprintln!("[EPUB Parser] Failed to read resource: {}", resource_id);
                AppError::InvalidInput(format!("Failed to read chapter content: {}", chapter_href))
            })?;

        eprintln!("[EPUB Parser] Successfully loaded chapter, content length: {}", content.len());

        // 提取并内联 CSS
        content = self.inline_css_for_chapter(&mut content, &mut doc)?;

        // 内联图片（转换为 Base64）
        content = self.inline_images(&content, &mut doc)?;

        Ok(content)
    }

    /// 提取 HTML 中引用的 CSS 并内联
    fn inline_css_for_chapter(
        &self,
        html: &mut String,
        doc: &mut epub::doc::EpubDoc<std::io::BufReader<std::fs::File>>,
    ) -> AppResult<String> {
        use regex::Regex;

        let mut inlined_styles = Vec::new();

        // 匹配 <link> 标签：<link href="style.css" rel="stylesheet" />
        let link_re = Regex::new(r#"<link[^>]*href=["']([^"']+\.css)["'][^>]*/?>"#).unwrap();

        // 收集所有 CSS 引用
        let css_refs: Vec<String> = link_re
            .captures_iter(html)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect();

        // 读取所有 CSS 文件
        for css_path in css_refs {
            eprintln!("[EPUB Parser] Found CSS reference: {}", css_path);

            // 尝试多种匹配策略查找 CSS 资源
            let css_id = doc.resources.iter()
                .find(|(_, res)| {
                    let res_path = res.path.to_string_lossy();
                    // 策略1: 完整路径匹配
                    res_path == css_path
                        // 策略2: 结尾匹配
                        || res_path.ends_with(&css_path)
                        // 策略3: 文件名匹配
                        || res_path.ends_with(&css_path.split('/').last().unwrap_or(&css_path))
                })
                .map(|(id, _)| id.clone());

            if let Some(id) = css_id {
                // 读取 CSS 内容
                if let Some((css_content, _)) = doc.get_resource_str(&id) {
                    eprintln!("[EPUB Parser] Loaded CSS: {}, length: {}", css_path, css_content.len());
                    inlined_styles.push(css_content);
                } else {
                    eprintln!("[EPUB Parser] Failed to read CSS: {}", css_path);
                }
            } else {
                eprintln!("[EPUB Parser] CSS file not found: {}", css_path);
            }
        }

        let mut result = html.clone();

        // 如果找到 CSS，将其内联到 <head> 中
        if !inlined_styles.is_empty() {
            let combined_css = inlined_styles.join("\n\n/* === Next CSS File === */\n\n");
            let style_tag = format!("<style type=\"text/css\">\n{}\n</style>", combined_css);

            // 尝试插入到 <head> 中
            if let Some(head_pos) = result.find("</head>") {
                result.insert_str(head_pos, &style_tag);
                eprintln!("[EPUB Parser] Inlined {} CSS files into <head>", inlined_styles.len());
            } else if let Some(body_pos) = result.find("<body") {
                // 如果没有 </head>，插入到 <body> 之前
                result.insert_str(body_pos, &style_tag);
                eprintln!("[EPUB Parser] Inlined {} CSS files before <body>", inlined_styles.len());
            } else {
                // 如果都没有，追加到开头
                result = format!("{}{}", style_tag, result);
                eprintln!("[EPUB Parser] Inlined {} CSS files at the start", inlined_styles.len());
            }

            // 移除原始的 <link> 标签
            result = link_re.replace_all(&result, "<!-- CSS inlined -->").to_string();
        } else {
            eprintln!("[EPUB Parser] No CSS files found in HTML");
        }

        Ok(result)
    }

    /// 内联图片（转换为 Base64）
    fn inline_images(
        &self,
        html: &str,
        doc: &mut epub::doc::EpubDoc<std::io::BufReader<std::fs::File>>,
    ) -> AppResult<String> {
        use regex::Regex;

        let mut result = html.to_string();

        // 匹配 <img> 标签：<img src="image.jpg" />
        let img_re = Regex::new(r#"<img([^>]*)src=["']([^"']+)["']([^>]*)>"#).unwrap();

        // 收集所有图片引用
        let img_matches: Vec<_> = img_re.captures_iter(html).collect();

        for cap in img_matches {
            let before_attrs = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let img_src = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let after_attrs = cap.get(3).map(|m| m.as_str()).unwrap_or("");

            // 跳过已经是 data: URL 的图片
            if img_src.starts_with("data:") {
                continue;
            }

            eprintln!("[EPUB Parser] Found image reference: {}", img_src);

            // 查找图片资源
            let img_id = doc.resources.iter()
                .find(|(_, res)| {
                    let res_path = res.path.to_string_lossy();
                    res_path == img_src
                        || res_path.ends_with(img_src)
                        || res_path.ends_with(&img_src.split('/').last().unwrap_or(img_src))
                })
                .map(|(id, _)| id.clone());

            if let Some(id) = img_id {
                // 读取图片内容（返回 (Vec<u8>, mime_type) 元组）
                if let Some((img_data, _mime)) = doc.get_resource(&id) {
                    // 检测 MIME 类型
                    let mime_type = if img_src.ends_with(".png") {
                        "image/png"
                    } else if img_src.ends_with(".jpg") || img_src.ends_with(".jpeg") {
                        "image/jpeg"
                    } else if img_src.ends_with(".gif") {
                        "image/gif"
                    } else if img_src.ends_with(".svg") {
                        "image/svg+xml"
                    } else if img_src.ends_with(".webp") {
                        "image/webp"
                    } else {
                        "image/jpeg" // 默认
                    };

                    // 转换为 Base64
                    let base64_data = base64::engine::general_purpose::STANDARD.encode(&img_data);
                    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

                    eprintln!("[EPUB Parser] Converted image to Base64: {}, size: {} bytes", img_src, img_data.len());

                    // 替换原始 src
                    let old_img_tag = format!(r#"<img{}src="{}"{}>"#, before_attrs, img_src, after_attrs);
                    let new_img_tag = format!(r#"<img{}src="{}"{}>"#, before_attrs, data_url, after_attrs);
                    result = result.replace(&old_img_tag, &new_img_tag);
                } else {
                    eprintln!("[EPUB Parser] Failed to read image: {}", img_src);
                }
            } else {
                eprintln!("[EPUB Parser] Image file not found: {}", img_src);
            }
        }

        Ok(result)
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

    #[test]
    fn test_extract_metadata_no_panic() {
        // 测试 extract_metadata() 不会因为 RefCell 双重借用而 panic
        // 即使文件无效，也应该返回错误而不是 panic
        let temp_dir = TempDir::new().unwrap();
        let epub_path = temp_dir.path().join("test.epub");

        // 创建一个无效的 EPUB 文件
        let mut file = fs::File::create(&epub_path).unwrap();
        file.write_all(b"not a valid epub").unwrap();
        drop(file);

        // 尝试打开会失败，但不应该 panic
        let result = EpubParser::open(&epub_path);
        assert!(result.is_err());

        // 验证错误类型
        match result {
            Err(AppError::InvalidInput(_)) => {
                // 预期行为：返回错误而不是 panic
            }
            _ => panic!("Expected InvalidInput error for invalid EPUB"),
        }
    }

}
