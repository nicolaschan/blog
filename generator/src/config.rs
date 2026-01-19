/// Site-wide configuration and branding
#[derive(Clone)]
pub struct SiteConfig {
    pub name: &'static str,
    pub tagline: &'static str,
    pub description: &'static str,
}

impl Default for SiteConfig {
    fn default() -> Self {
        Self {
            name: "Nicolas Chan",
            tagline: "Software engineering and anything else on my mind 🌁",
            description: "Software engineering and anything else on my mind 🌁",
        }
    }
}
