use once_cell::sync::Lazy;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
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

#[derive(Debug, Clone)]
pub struct Page {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub cover_image: Option<String>,
    pub content_html: String,
}

#[derive(Debug, Deserialize)]
struct Frontmatter {
    title: String,
    slug: String,
    #[serde(default)]
    description: String,
    cover_image: Option<String>,
}

impl Page {
    pub fn from_markdown(content: &str) -> Result<Self, String> {
        // Split frontmatter and content
        let (frontmatter_str, markdown) = Self::split_frontmatter(content)?;

        // Parse frontmatter
        let frontmatter: Frontmatter =
            serde_yaml::from_str(frontmatter_str).map_err(|e| format!("YAML error: {}", e))?;

        // Convert markdown to HTML
        let content_html = Self::markdown_to_html(markdown);

        Ok(Page {
            title: frontmatter.title,
            slug: frontmatter.slug,
            description: frontmatter.description,
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
        let options = Options::ENABLE_STRIKETHROUGH
            | Options::ENABLE_TABLES
            | Options::ENABLE_FOOTNOTES
            | Options::ENABLE_SMART_PUNCTUATION;

        let parser = Parser::new_ext(markdown, options);
        
        // Process events, adding syntax highlighting to code blocks and wrapping tables
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();
        let mut events: Vec<Event> = Vec::new();

        for event in parser {
            match event {
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
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
