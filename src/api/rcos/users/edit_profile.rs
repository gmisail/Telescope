//! Queries and mutations for editing a user's profile.

use crate::api::rcos::prelude::*;
use crate::api::rcos::send_query;
use crate::api::rcos::users::UserRole;
use crate::error::TelescopeError;

/// Type representing GraphQL query to get context for editing a user profile.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/rcos/schema.json",
    query_path = "graphql/rcos/users/edit_profile.graphql",
    variables_derives = "Debug,Clone",
    response_derives = "Debug,Clone,Serialize"
)]
pub struct EditProfileContext;

/// Type representing GraphQL mutation to save edits to a profile.
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "graphql/rcos/schema.json",
    query_path = "graphql/rcos/users/edit_profile.graphql",
    variables_derives = "Debug,Clone",
    response_derives = "Debug,Clone,Serialize"
)]
pub struct SaveProfileEdits;

impl EditProfileContext {
    /// Get the context to edit a user's profile.
    pub async fn get(
        username: String,
    ) -> Result<Option<edit_profile_context::EditProfileContextUsersByPk>, TelescopeError> {
        send_query::<Self>(edit_profile_context::Variables { username })
            .await
            .map(|response| response.users_by_pk)
    }
}

impl SaveProfileEdits {
    /// Save edits to a user's profile, returning their username if the user was found.
    pub async fn execute(
        username: String,
        first_name: String,
        last_name: String,
        cohort: Option<i64>,
        role: UserRole,
    ) -> Result<Option<String>, TelescopeError> {
        send_query::<Self>(save_profile_edits::Variables {
            username,
            fname: first_name,
            lname: last_name,
            cohort,
            role,
        })
        .await
        // Extract returned username option.
        .map(|response| response.update_users_by_pk.map(|obj| obj.username))
    }
}
