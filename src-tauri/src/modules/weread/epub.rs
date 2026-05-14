use std::fs;
use std::path::{Path, PathBuf};

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};

use super::database;

pub async fn build_epub(
    pool: &sqlx::SqlitePool,
    library_id: i64,
    book_id: i64,
    output_dir: &Path,
) -> Result<PathBuf, String> {
    let book = database::get_book_by_id(pool, book_id)
        .await?
        .ok_or("书籍不存在")?;

    let chapters = database::list_chapters(pool, library_id, book_id).await?;

    let filename = sanitize_filename(&format!(
        "{} - {}.epub",
        book.title,
        book.author.as_deref().unwrap_or("未知")
    ));
    let output_path = output_dir.join(&filename);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建输出目录失败: {}", e))?;
    }

    let mut epub = EpubBuilder::new(
        ZipLibrary::new().map_err(|e| format!("初始化 EPUB 失败: {}", e))?,
    )
    .map_err(|e| format!("创建 EPUB Builder 失败: {}", e))?;

    epub.metadata("title", &book.title)
        .map_err(|e| format!("设置标题失败: {}", e))?;

    if let Some(ref author) = book.author {
        epub.metadata("author", author)
            .map_err(|e| format!("设置作者失败: {}", e))?;
    }

    epub.metadata("lang", "zh-CN")
        .map_err(|e| format!("设置语言失败: {}", e))?;

    if let Some(ref intro) = book.intro {
        epub.metadata("description", intro)
            .map_err(|e| format!("设置简介失败: {}", e))?;
    }

    let css = r#"
        body { font-family: sans-serif; line-height: 1.8; padding: 1em; }
        h1, h2, h3, h4 { margin-top: 1.5em; }
        p { text-indent: 2em; margin: 0.5em 0; }
        pre { background: #f5f5f5; padding: 1em; overflow-x: auto; }
        img { max-width: 100%; height: auto; }
    "#;
    epub.stylesheet(css.as_bytes())
        .map_err(|e| format!("设置样式失败: {}", e))?;

    for (i, chapter) in chapters.iter().enumerate() {
        let content_md = chapter.content_md.as_deref().unwrap_or("");
        let html_body = markdown_to_html(content_md);
        let xhtml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>{}</title></head>
<body>
<h1>{}</h1>
{}
</body>
</html>"#,
            escape_xml(&chapter.title),
            escape_xml(&chapter.title),
            html_body
        );

        let chap_filename = format!("chapter_{:04}.xhtml", i);
        let mut content = EpubContent::new(&chap_filename, xhtml.as_bytes())
            .title(&chapter.title);

        if i == 0 {
            content = content.reftype(ReferenceType::Text);
        }

        epub.add_content(content)
            .map_err(|e| format!("添加章节 {} 失败: {}", chapter.title, e))?;
    }

    let mut output = Vec::new();
    epub.generate(&mut output)
        .map_err(|e| format!("生成 EPUB 失败: {}", e))?;

    fs::write(&output_path, &output)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(output_path)
}

fn markdown_to_html(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;

    for line in md.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_xml(line));
            html.push('\n');
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("#### ") {
            html.push_str(&format!("<h4>{}</h4>\n", escape_xml(&trimmed[5..])));
        } else if trimmed.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", escape_xml(&trimmed[4..])));
        } else if trimmed.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", escape_xml(&trimmed[3..])));
        } else if trimmed.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", escape_xml(&trimmed[2..])));
        } else if trimmed.starts_with("![") {
            if let Some(src) = extract_image_src(trimmed) {
                html.push_str(&format!(
                    "<p><img src=\"{}\" alt=\"image\"/></p>\n",
                    escape_xml(&src)
                ));
            }
        } else {
            html.push_str(&format!("<p>{}</p>\n", escape_xml(trimmed)));
        }
    }

    if in_code_block {
        html.push_str("</code></pre>\n");
    }

    html
}

fn extract_image_src(md_img: &str) -> Option<String> {
    let start = md_img.find('(')?;
    let end = md_img.rfind(')')?;
    if start < end {
        Some(md_img[start + 1..end].to_string())
    } else {
        None
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}
