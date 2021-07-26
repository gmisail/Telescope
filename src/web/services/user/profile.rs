//! Profile services.

use crate::api::rcos::users::profile::{
    profile::{ProfileTarget, ResponseData},
    Profile,
};
use crate::error::TelescopeError;
use crate::templates::Template;
use crate::web::services::auth::identity::{Identity, AuthenticationCookie};
use actix_web::web::{Query, ServiceConfig, Form};
use actix_web::{HttpRequest, HttpResponse};
use crate::templates::forms::FormTemplate;
use chrono::{Local, Datelike};
use crate::api::rcos::users::UserRole;
use crate::api::rcos::users::edit_profile::EditProfileContext;
use std::collections::HashMap;

/// The path from the template directory to the profile template.
const TEMPLATE_NAME: &'static str = "user/profile";

/// The path from the templates directory to the user settings form template.
const SETTINGS_FORM: &'static str = "user/settings";

/// Wrapper struct for deserializing username.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProfileQuery {
    /// The username of the owner of the profile.
    pub username: String,
}

/// Register services into actix app.
pub fn register(config: &mut ServiceConfig) {
    config
        .service(profile)
        .service(settings)
        .service(save_changes);
}

/// User profile service. The username in the path is url-encoded.
#[get("/user")]
async fn profile(
    req: HttpRequest,
    identity: Identity,
    // TODO: Switch to using Path here when we switch to user ids.
    Query(ProfileQuery { username }): Query<ProfileQuery>,
) -> Result<Template, TelescopeError> {
    // Get the viewer's username.
    let viewer_username: Option<String> = identity.get_rcos_username().await?;

    // Get the user's profile information (and viewer info) from the RCOS API.
    let response: ResponseData = Profile::for_user(username, viewer_username).await?;

    // Throw an error if there is no user.
    if response.target.is_none() {
        return Err(TelescopeError::resource_not_found(
            "User Not Found",
            "Could not find a user by this username.",
        ));
    }

    // Get the target user's info.
    let target_user: &ProfileTarget = response.target.as_ref().unwrap();
    // And use it to make the page title
    let page_title = format!("{} {}", target_user.first_name, target_user.last_name);

    // Get the target user's discord info.

    // Make a profile template
    return Template::new(TEMPLATE_NAME)
        .field("data", response)
        // Render it inside a page (with the user's name as the title)
        .render_into_page(&req, page_title)
        .await;
}

/// Create a form template for the user settings page.
fn make_settings_form() -> FormTemplate {
    // Create the base form.
    let mut form: FormTemplate = FormTemplate::new(SETTINGS_FORM, "Edit Profile");

    // The max entry year should always be the current year.
    form.template = json!({
        "max_entry_year": Local::today().year()
    });

    return form;
}

/// User settings form.
#[get("/edit_profile")]
async fn settings(auth: AuthenticationCookie) -> Result<FormTemplate, TelescopeError> {
    // Get viewers username. You have to be authenticated to edit your own profile.
    let viewer: String = auth.get_rcos_username_or_error().await?;
    // Get the context for the edit form.
    let context = EditProfileContext::get(viewer.clone()).await?;
    // Ensure that the context exists.
    if context.is_none() {
        // Use an ISE since we should be able to get an edit context as long as there is an
        // authenticated username.
        return Err(TelescopeError::ise(format!("Could not get edit context for username {}.", viewer)));
    }

    // Unwrap the context.
    let context = context.unwrap();

    // Create the form to edit the profile.
    let mut form: FormTemplate = make_settings_form();
    // Add the context to the form.
    form.template["context"] = json!(&context);

    // Add the list of roles (and whether the current role can switch to them).
    let role_list = UserRole::ALL_ROLES
        .iter()
        // Add availability data
        .map(|role| (*role, UserRole::can_switch_to(context.role, *role)))
        // Collect into map
        .collect::<HashMap<_, _>>();

    // Add to form.
    form.template["roles"] = json!(role_list);

    // Disable student role if the current role is external and there is no RCS ID in the context.
    if context.role.is_external() && context.rcs_id.first().is_none() {
        form.template["roles"]["student"] = json!(false);
    }

    return Ok(form);
}

/// Edits to the user's profile submitted through the form.
#[derive(Clone, Serialize, Deserialize, Debug)]
struct ProfileEdits {
    first_name: String,
    last_name: String,
    role: UserRole,

    /// Entry year for RPI students.
    #[serde(default)]
    cohort: String,
}

/// Submission endpoint for the user settings form.
#[post("/edit_profile")]
async fn save_changes(
    auth: AuthenticationCookie,
    Form(ProfileEdits { first_name, last_name, role, cohort }): Form<ProfileEdits>
) -> Result<HttpResponse, TelescopeError> {
    // Check that the user is authenticated. Get the username of the target profile (always their own).
    let viewer: String = auth.get_rcos_username_or_error().await?;

    // Convert the cohort to a number or default to no cohort input. This should be checked client side.
    let cohort: Option<i64> = cohort.parse::<i64>().ok();

    Err(TelescopeError::NotImplemented)
}
