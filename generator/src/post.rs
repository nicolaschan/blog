use chrono::NaiveDate;
use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use regex::Regex;
use serde::Deserialize;
use syntect::easy::HighlightLines;
use syntect::highlighting::{ThemeSet, Theme};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME: Lazy<Theme> = Lazy::new(|| {
    let theme_set = ThemeSet::load_defaults();
    theme_set.themes["base16-ocean.dark"].clone()
});

// Regex to match markdown links: [text](url)
static LINK_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[([^\]]+)\]\([^)]+\)").unwrap());

#[derive(Debug, Clone)]
pub struct Post {
    pub title: String,
    pub slug: String,
    pub date: Option<NaiveDate>,
    pub excerpt: String,
    pub tags: Vec<String>,
    pub cover_image: Option<String>,
    pub content_html: String,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    slug: Option<String>,
    date: Option<String>,
    excerpt: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
    cover_image: Option<String>,
}

impl Post {
    pub fn from_markdown(content: &str, filename: &str) -> Result<Self, String> {
        // Split frontmatter and content
        let (frontmatter_str, markdown) = Self::split_frontmatter(content)?;

        // Parse frontmatter
        let frontmatter: Frontmatter =
            serde_yaml::from_str(frontmatter_str).map_err(|e| format!("YAML error: {}", e))?;

        // Parse date if provided
        let date = match frontmatter.date {
            Some(date_str) => Some(
                NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
                    .map_err(|e| format!("Date parse error: {}", e))?
            ),
            None => None,
        };

        // Convert markdown to HTML
        let content_html = Self::markdown_to_html(markdown);

        // Generate excerpt if not provided
        let excerpt = frontmatter.excerpt.unwrap_or_else(|| {
            Self::generate_excerpt(markdown, 160)
        });

        // Use slug from frontmatter, or default to filename without .md extension
        let slug = frontmatter.slug.unwrap_or_else(|| {
            filename.strip_suffix(".md").unwrap_or(filename).to_string()
        });

        Ok(Post {
            title: frontmatter.title,
            slug,
            date,
            excerpt,
            tags: frontmatter.tags,
            cover_image: frontmatter.cover_image,
            content_html,
        })
    }

    fn split_frontmatter(content: &str) -> Result<(&str, &str), String> {
        let content = content.trim_start();

        if !content.starts_with("---") {
            return Err("Missing frontmatter delimiter".to_string());
        }

        let after_first = &content[3..];
        let end_pos = after_first
            .find("\n---")
            .ok_or("Missing closing frontmatter delimiter")?;

        let frontmatter = &after_first[..end_pos].trim();
        let markdown = &after_first[end_pos + 4..].trim();

        Ok((frontmatter, markdown))
    }

    fn markdown_to_html(markdown: &str) -> String {
        use std::collections::HashMap;
        
        let options = Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_SMART_PUNCTUATION;

        let parser = Parser::new_ext(markdown, options);
        
        // Process events, adding syntax highlighting to code blocks and wrapping tables
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();
        let mut in_heading = false;
        let mut heading_level = 0;
        let mut heading_text = String::new();
        let mut heading_slugs: HashMap<String, usize> = HashMap::new();
        let mut events: Vec<Event> = Vec::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    in_heading = true;
                    heading_level = match level {
                        pulldown_cmark::HeadingLevel::H1 => 1,
                        pulldown_cmark::HeadingLevel::H2 => 2,
                        pulldown_cmark::HeadingLevel::H3 => 3,
                        pulldown_cmark::HeadingLevel::H4 => 4,
                        pulldown_cmark::HeadingLevel::H5 => 5,
                        pulldown_cmark::HeadingLevel::H6 => 6,
                    };
                    heading_text.clear();
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    in_heading = false;
                    let base_slug = Self::slugify(&heading_text);
                    let count = heading_slugs.entry(base_slug.clone()).or_insert(0);
                    let slug = if *count == 0 {
                        base_slug
                    } else {
                        format!("{}-{}", base_slug, count)
                    };
                    *heading_slugs.get_mut(&Self::slugify(&heading_text)).unwrap() += 1;
                    let heading_html = format!(
                        "<h{level} id=\"{slug}\"><a class=\"heading-link\" href=\"#{slug}\">{text}</a></h{level}>",
                        level = heading_level,
                        slug = slug,
                        text = html_escape(&heading_text)
                    );
                    events.push(Event::Html(heading_html.into()));
                }
                Event::Text(ref text) if in_heading => {
                    heading_text.push_str(text);
                }
                Event::Code(ref code) if in_heading => {
                    heading_text.push_str(code);
                }
                Event::Start(Tag::Table(_)) => {
                    events.push(Event::Html("<div class=\"table-wrapper\">".into()));
                    events.push(event);
                }
                Event::End(Tag::Table(_)) => {
                    events.push(event);
                    events.push(Event::Html("</div>".into()));
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    in_code_block = true;
                    code_lang = match kind {
                        CodeBlockKind::Fenced(lang) => lang.to_string(),
                        CodeBlockKind::Indented => String::new(),
                    };
                    code_content.clear();
                }
                Event::End(Tag::CodeBlock(_)) => {
                    in_code_block = false;
                    
                    // Apply syntax highlighting
                    let highlighted = Self::highlight_code(&code_content, &code_lang);
                    events.push(Event::Html(highlighted.into()));
                    
                    code_lang.clear();
                    code_content.clear();
                }
                Event::Text(text) if in_code_block => {
                    code_content.push_str(&text);
                }
                _ => {
                    events.push(event);
                }
            }
        }

        let mut html_output = String::new();
        pulldown_cmark::html::push_html(&mut html_output, events.into_iter());

        html_output
    }

    fn slugify(text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c
                } else if c.is_whitespace() || c == '-' || c == '_' {
                    '-'
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    fn highlight_code(code: &str, lang_info: &str) -> String {
        // Parse language and optional filename from info string
        let (lang, filename) = Self::parse_code_info(lang_info);
        
        // Find the syntax for the language
        let syntax = SYNTAX_SET
            .find_syntax_by_token(&lang)
            .or_else(|| SYNTAX_SET.find_syntax_by_extension(&lang))
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, &THEME);
        let mut html_output = String::new();
        
        for line in LinesWithEndings::from(code) {
            match highlighter.highlight_line(line, &SYNTAX_SET) {
                Ok(ranges) => {
                    let html_line = syntect::html::styled_line_to_highlighted_html(
                        &ranges[..],
                        syntect::html::IncludeBackground::No,
                    ).unwrap_or_else(|_| html_escape(line));
                    html_output.push_str(&html_line);
                }
                Err(_) => {
                    html_output.push_str(&html_escape(line));
                }
            }
        }

        let lang_class = if lang.is_empty() { 
            String::new() 
        } else { 
            format!(" class=\"language-{}\"", lang) 
        };

        // If we have a language or filename, wrap in a code-block container with header
        if !lang.is_empty() || filename.is_some() {
            let header = Self::build_code_header(&lang, filename.as_deref());
            format!(
                "<div class=\"code-block\">{}<pre{}><code>{}</code></pre></div>",
                header, lang_class, html_output
            )
        } else {
            format!("<div class=\"code-block\">{}<pre{}><code>{}</code></pre></div>", Self::build_code_header("", None), lang_class, html_output)
        }
    }

    fn parse_code_info(info: &str) -> (String, Option<String>) {
        let info = info.trim();
        
        if info.is_empty() {
            return (String::new(), None);
        }

        // Check for "lang:filename" format
        if let Some((lang, filename)) = info.split_once(':') {
            let lang = lang.trim().to_string();
            let filename = filename.trim();
            if !filename.is_empty() {
                return (lang, Some(filename.to_string()));
            }
            return (lang, None);
        }

        // Check for "lang filename=path" format
        if let Some((lang, rest)) = info.split_once(' ') {
            let rest = rest.trim();
            if let Some(filename) = rest.strip_prefix("filename=") {
                return (lang.to_string(), Some(filename.to_string()));
            }
        }

        // Just the language
        (info.split_whitespace().next().unwrap_or("").to_string(), None)
    }

    fn build_code_header(lang: &str, filename: Option<&str>) -> String {
        let copy_button = r#"<button class="code-block-copy" title="Copy code"><svg class="copy-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg><svg class="check-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"></polyline></svg></button>"#;
        
        match (lang.is_empty(), filename) {
            (true, None) => {
                format!(
                    "<div class=\"code-block-header\"><div class=\"code-block-header-content\"></div>{}</div>",
                    copy_button
                )
            }
            (false, None) => {
                format!(
                    "<div class=\"code-block-header\"><div class=\"code-block-header-content\"><span class=\"code-block-lang\">{}</span></div>{}</div>",
                    html_escape(lang),
                    copy_button
                )
            }
            (true, Some(f)) => {
                format!(
                    "<div class=\"code-block-header\"><div class=\"code-block-header-content\"><span class=\"code-block-filename\">{}</span></div>{}</div>",
                    html_escape(f),
                    copy_button
                )
            }
            (false, Some(f)) => {
                format!(
                    "<div class=\"code-block-header\"><div class=\"code-block-header-content\"><span class=\"code-block-lang\">{}</span><span class=\"separator\">·</span><span class=\"code-block-filename\">{}</span></div>{}</div>",
                    html_escape(lang),
                    html_escape(f),
                    copy_button
                )
            }
        }
    }

    fn generate_excerpt(markdown: &str, max_chars: usize) -> String {
        // Find the first paragraph (text before a blank line, heading, or list)
        let first_paragraph: String = markdown
            .lines()
            .take_while(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() 
                    && !trimmed.starts_with('#') 
                    && !trimmed.starts_with("```")
                    && !trimmed.starts_with('-')
                    && !trimmed.starts_with('*')
                    && !trimmed.starts_with("1.")
            })
            .collect::<Vec<_>>()
            .join(" ");
        
        // Replace markdown links [text](url) with just text
        let text = LINK_RE.replace_all(&first_paragraph, "$1").to_string();
        
        // Remove remaining markdown syntax characters
        let text: String = text
            .chars()
            .filter(|c| !['*', '_', '`'].contains(c))
            .collect();

        let text = text.split_whitespace().collect::<Vec<_>>().join(" ");

        if text.len() <= max_chars {
            text
        } else {
            let truncated: String = text.chars().take(max_chars).collect();
            if let Some(last_space) = truncated.rfind(' ') {
                format!("{}…", &truncated[..last_space])
            } else {
                format!("{}…", truncated)
            }
        }
    }

    pub fn formatted_date(&self) -> String {
        self.date.map(|d| d.format("%B %d, %Y").to_string()).unwrap_or_default()
    }

    pub fn has_date(&self) -> bool {
        self.date.is_some()
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_post() {
        let content = r#"---
title: Test Post
date: 2026-01-15
tags:
  - rust
  - testing
---

# Hello World

This is a test post with **bold** text.
"#;

        let post = Post::from_markdown(content, "test-post.md").unwrap();
        assert_eq!(post.title, "Test Post");
        assert_eq!(post.slug, "test-post");
        assert!(post.date.is_some());
        assert_eq!(post.tags, vec!["rust", "testing"]);
        assert!(post.content_html.contains("<strong>bold</strong>"));
    }
}
