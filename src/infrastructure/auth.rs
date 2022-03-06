use actix_web::{dev::ServiceRequest, Error};
use actix_web_httpauth::extractors::{
    bearer::{BearerAuth, Config},
    AuthenticationError,
};

pub async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    tracing::error!("WTAFASDFA");
    if credentials.token() == "im_a_valid_user" {
        Ok(req)
    } else {
        let config = req
            .app_data::<Config>()
            .map(|data| data.clone())
            .unwrap_or_else(Default::default);

        let error = AuthenticationError::from(config).with_error_description("Wrong Bearer token");

        Err(error.into())
    }
}
