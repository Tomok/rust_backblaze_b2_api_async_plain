//! Backblaze API Level V2
mod common_structs;

mod b2_authorize_account;
mod b2_list_buckets;
mod buckets;

pub use b2_authorize_account::{AuthorizeAccountOk, AuthorizeError};
/// Authorize account function see [official documentation](https://www.backblaze.com/b2/docs/b2_authorize_account.html)
pub async fn b2_authorize_account(
    application_key_id: &str,
    application_key: &str,
) -> Result<AuthorizeAccountOk, AuthorizeError> {
    // call the real function with the basic_uri filled in (needes to be changeable for testing)
    b2_authorize_account::b2_authorize_account(
        "https://api.backblazeb2.com",
        application_key_id,
        application_key,
    )
    .await
}

pub use b2_list_buckets::b2_list_buckets;
pub use common_structs::*;

#[cfg(test)]
mod test;
