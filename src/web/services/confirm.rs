use crate::{
    models::{
        confirmations::{ConfirmNewUserError, Confirmation},
        users::User,
    },
    templates::{
        forms::{common::text_field, confirmation},
        jumbotron, page,
        static_pages::ise::ise,
        Template,
    },
    web::RequestContext,
};
use actix_web::{
    http::header,
    web::{Form, Path},
    HttpResponse,
};
use uuid::Uuid;
use crate::error::TelescopeError;
use crate::util::DbConnection;
use crate::app_data::AppData;

/// The form sent to new users to confirm the account creation.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NewUserConfInput {
    /// The name of the user
    name: String,
    /// The password
    password: String,
    /// The confirmation of the password. This should match the password.
    confirm: String,
}

/// Error header when a confirmation is not found.
const CONF_NOT_FOUND_HEADER: &'static str = "Confirmation Not Found";

/// Error message when a confirmation is not found.
const CONF_NOT_FOUND_MESSAGE: &'static str = "We could not find an email \
    confirmation for that ID. It may have expired, in which case you should \
    try again. If it was recent, and should not have expired, please contact \
    a coordinator.";

/// Make an error for missing confirmations.
fn conf_not_found() -> TelescopeError {
    TelescopeError::resource_not_found(CONF_NOT_FOUND_HEADER, CONF_NOT_FOUND_MESSAGE)
}

/// The page shown to users accepting an invitation to create an account.
#[get("/confirm/{invite_id}")]
pub async fn confirmations_page(ctx: RequestContext, Path(invite_id): Path<Uuid>) -> Result<Template, TelescopeError> {
    // Get database connection.
    let db_conn: DbConnection = AppData::global().get_db_conn().await?;

    // Get confirmation record.
    let confirmation: Confirmation = Confirmation::get_by_id(invite_id)
        .await?
        .ok_or_else(conf_not_found)?;

    if confirmation.creates_user() {
        // Show the new user the form to create their account.
        let form: Template = confirmation::for_conf(&confirmation);
        page::of(&ctx, "Create account", &form).await
    } else {
        let error_message = confirmation.confirm_existing(&ctx).await.err();

        // make page title
        let errored = error_message.is_some();
        let page_title = if errored { "Error" } else { "RCOS" };

        // make confirmation page
        let conf: Template =
            confirmation::for_conf(&confirmation).field(confirmation::ERROR, error_message);
        let rendered: String = ctx.render_in_page(&conf, page_title).await;

        return if errored {
            HttpResponse::InternalServerError().body(rendered)
        } else {
            HttpResponse::Ok().body(rendered)
        };
    }
}

#[post("/confirm/{invite_id}")]
pub async fn confirm(
    ctx: RequestContext,
    Path(invite_id): Path<Uuid>,
    Form(form): Form<NewUserConfInput>,
) -> Result<HttpResponse, TelescopeError> {
    // Get confirmation record from database.
    let confirmation: Confirmation = Confirmation::get_by_id(invite_id)
        .await?
        .ok_or_else(conf_not_found)?;

    // Make sure that the confirmation creates a user. We do not accept post requests for existing
    // users.
    if !confirmation.creates_user() {
        let error_message: String = format!(
            "Confirmation {} is already linked to an existing user",
            invite_id
        );

        let page: String =
            jumbotron::rendered_page(&ctx, "Cannot create user", "Bad request", error_message)
                .await;

        return HttpResponse::BadRequest().body(page);
    }

    // Destructure form.
    let NewUserConfInput {
        name,
        password,
        confirm,
    } = form;

    // Form to return if errors occur.
    let mut form_err: Template = confirmation::for_conf(&confirmation);
    form_err[confirmation::NAME][text_field::PREFILL_FIELD] = name.as_str().into();

    // Check that the password and the confirm password are the same.
    if password != confirm {
        form_err[confirmation::PASSWORD][text_field::ERROR_FIELD] =
            "Password does not match confirm password.".into();
        return HttpResponse::BadRequest().body(ctx.render_in_page(&form_err, "Error").await);
    }

    // Try to confirm the new user.
    let res: Result<User, ConfirmNewUserError> =
        confirmation.confirm_new(&ctx, name.clone(), password).await;

    match res {
        // Success
        Ok(new_user) => {
            // log the user in.
            // in the future we should probably have a better form
            // of user identity than just the uuid.
            ctx.identity().remember(new_user.id_str());

            let profile_url: String = format!("/profile/{}", new_user.id_str());

            return HttpResponse::Found()
                .header(header::LOCATION, profile_url)
                .finish();
        }

        // Handle bad password.
        Err(ConfirmNewUserError::BadPassword(reqs)) => {
            let err_str: String = reqs
                .get_error_string()
                .expect("Could not get error string for password requirements");

            form_err[confirmation::PASSWORD][text_field::ERROR_FIELD] = err_str.into();

            HttpResponse::BadRequest().body(ctx.render_in_page(&form_err, "Error").await)
        }

        // Handle other confirmation error.
        Err(ConfirmNewUserError::Other(msg)) => {
            error!("Could not confirm new user: {}", msg);
            ise(&ctx).await
        }
    }
}
