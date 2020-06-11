use handlebars::{Handlebars, RenderError};
use crate::templates::navbar::Navbar;
use crate::web::PageContext;
use crate::web::context::Template;


/// A page on the RCOS website.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Page {
    /// The page title.
    page_title: String,
    /// The navbar at the top of the page.
    navbar: String,
    /// The inner html for this webpage. This is rendered unescaped. Do not let the user get stuff
    /// Ensure that no user input gets rendered into this unescaped (as it will create an XSS vulnerability).
    page_body: String,
}

impl Page {
    /// Create a new web page.
    pub fn new(title: impl Into<String>, body: impl Into<String>, pc: &PageContext) -> Self {
        Self {
            page_title: title.into(),
            page_body: body.into(),
            navbar: pc.render(&Navbar::from_context(pc)).unwrap()
        }
    }
}

impl Template for Page {
    const TEMPLATE_NAME: &'static str = "page";
}