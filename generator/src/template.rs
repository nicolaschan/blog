use askama::Template;
use crate::config::SiteConfig;
use crate::post::Post;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate<'a> {
    pub site: &'a SiteConfig,
    pub posts: &'a [Post],
    pub path_prefix: &'static str,
}

#[derive(Template)]
#[template(path = "post.html")]
pub struct PostTemplate<'a> {
    pub site: &'a SiteConfig,
    pub post: &'a Post,
    pub path_prefix: &'static str,
}

#[derive(Template)]
#[template(path = "page.html")]
pub struct PageTemplate<'a> {
    pub site: &'a SiteConfig,
    pub page: &'a Post,
    pub path_prefix: &'static str,
}

/// Renders the index page with a list of posts
pub fn render_index(site: &SiteConfig, posts: &[Post]) -> String {
    let template = IndexTemplate {
        site,
        posts,
        path_prefix: "",
    };
    template.render().expect("Failed to render index template")
}

/// Renders a single post page
pub fn render_post(site: &SiteConfig, post: &Post) -> String {
    let template = PostTemplate {
        site,
        post,
        path_prefix: "../../",
    };
    template.render().expect("Failed to render post template")
}

/// Renders a static page (uses Post type with optional date)
pub fn render_page(site: &SiteConfig, page: &Post) -> String {
    let template = PageTemplate {
        site,
        page,
        path_prefix: "../",
    };
    template.render().expect("Failed to render page template")
}

