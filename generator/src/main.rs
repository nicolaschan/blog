mod config;
mod post;
mod template;

use config::SiteConfig;
use post::Post;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    let site = SiteConfig::default();
    
    let content_dir = Path::new("../content/posts");
    let pages_dir = Path::new("../content/pages");
    let resources_dir = Path::new("../content/resources");
    let output_dir = Path::new("../dist");

    // Create output directories
    fs::create_dir_all(output_dir.join("posts")).expect("Failed to create output directory");
    fs::create_dir_all(output_dir.join("resources")).expect("Failed to create resources directory");

    // Copy resources (images, etc.)
    if resources_dir.exists() {
        for entry in WalkDir::new(resources_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            let relative_path = entry.path().strip_prefix(resources_dir).unwrap();
            let dest_path = output_dir.join("resources").join(relative_path);
            
            // Create parent directories if needed
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent).expect("Failed to create resource subdirectory");
            }
            
            fs::copy(entry.path(), &dest_path).expect("Failed to copy resource");
            println!("Copied: {}", dest_path.display());
        }
    }

    // Collect all posts
    let mut posts: Vec<Post> = Vec::new();

    if content_dir.exists() {
        for entry in WalkDir::new(content_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let content = fs::read_to_string(entry.path()).expect("Failed to read file");
            let filename = entry.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
            match Post::from_markdown(&content, filename) {
                Ok(post) => {
                    // Generate individual post HTML in slug/index.html for clean URLs
                    let post_html = template::render_post(&site, &post);
                    let post_dir = output_dir.join("posts").join(&post.slug);
                    fs::create_dir_all(&post_dir).expect("Failed to create post directory");
                    let post_path = post_dir.join("index.html");
                    fs::write(&post_path, post_html).expect("Failed to write post");
                    println!("Generated: {}", post_path.display());
                    posts.push(post);
                }
                Err(e) => eprintln!("Error parsing {}: {}", entry.path().display(), e),
            }
        }
    }

    // Sort posts by date (newest first), only posts with dates
    posts.sort_by(|a, b| b.date.cmp(&a.date));

    // Generate index page
    let index_html = template::render_index(&site, &posts);
    let index_path = output_dir.join("index.html");
    fs::write(&index_path, index_html).expect("Failed to write index");
    println!("Generated: {}", index_path.display());

    // Generate pages from markdown
    if pages_dir.exists() {
        for entry in WalkDir::new(pages_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "md"))
        {
            let content = fs::read_to_string(entry.path()).expect("Failed to read file");
            let filename = entry.path().file_name().and_then(|n| n.to_str()).unwrap_or("");
            match Post::from_markdown(&content, filename) {
                Ok(page) => {
                    // Generate page in slug/index.html for clean URLs
                    let page_html = template::render_page(&site, &page);
                    let page_dir = output_dir.join(&page.slug);
                    fs::create_dir_all(&page_dir).expect("Failed to create page directory");
                    let page_path = page_dir.join("index.html");
                    fs::write(&page_path, page_html).expect("Failed to write page");
                    println!("Generated: {}", page_path.display());
                }
                Err(e) => eprintln!("Error parsing {}: {}", entry.path().display(), e),
            }
        }
    }

    println!("\n✓ Generated {} posts", posts.len());
}
