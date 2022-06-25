use crate::api_response::{ApiEmptyReturnValue, ApiResponse, ApiReturnValue};
use crate::public::iam::AuthenticatableEntity;
use rocket::serde::json::Json;

#[rocket::get("/whoami")]
pub fn whoami(ae: AuthenticatableEntity) -> ApiReturnValue<AuthenticatableEntity> {
    Ok(ApiResponse(Json(ae)))
}

pub fn routes() -> Vec<rocket::Route> { rocket::routes![whoami] }