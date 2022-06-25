use rocket::Request;
use rocket::serde::json::Json;
use rocket::response::Responder;
use rocket::Response;
use rocket::http::Status;

#[derive(rocket::Responder)]
#[response(status = 200)]
pub struct ApiResponse<T>(pub Json<T>);

#[derive(serde::Serialize)]
struct ApiErrorResponse<'r> {
    message: &'r str,
    requested_path: String,
    code: u8,
    additional_information: &'r str,

    #[serde(skip)]
    http_status: Status,
}

#[derive(Debug)]
pub enum ApiError {
    UserNotFound,
    TooManyUsers,
    MeNoLikeyAWS,
    CacheUnavailable,
    AuthenticationFailed,
    MissingSessionKey,
    InvalidSession,
    NoMatchingPrivilege,
    MalformedPermission,
}

impl<'r> Responder<'r, 'r> for ApiError {
    fn respond_to(self, req: &Request) -> rocket::response::Result<'r> {
        let response_body : ApiErrorResponse;

        match self {
            ApiError::UserNotFound => {
                response_body = ApiErrorResponse {
                    message: "UserNotFound",
                    requested_path: req.uri().to_string(),
                    code: 0,
                    additional_information: "The requested user could not be found.",

                    http_status: Status::NotFound,
                };
            }
            ApiError::TooManyUsers => {
                response_body = ApiErrorResponse {
                    message: "TooManyUsers",
                    requested_path: req.uri().to_string(),
                    code: 1,
                    additional_information: "The login request matched with more than one known entity.",

                    http_status: Status::InternalServerError,
                };
            }
            ApiError::MeNoLikeyAWS => {
                response_body = ApiErrorResponse {
                    message: "MeNoLikeyAWS",
                    requested_path: req.uri().to_string(),
                    code: 2,
                    additional_information: "A query to an AWS service has failed. Please contact them and express your disgust.",

                    http_status: Status::ServiceUnavailable,
                };
            }
            ApiError::CacheUnavailable => {
                response_body = ApiErrorResponse {
                    message: "CacheUnavailable",
                    requested_path: req.uri().to_string(),
                    code: 3,
                    additional_information: "Unable to connect or query the server cache (REDIS).",

                    http_status: Status::ServiceUnavailable,
                };
            }
            ApiError::AuthenticationFailed => {
                response_body = ApiErrorResponse {
                    message: "AuthenticationFailed",
                    requested_path: req.uri().to_string(),
                    code: 4,
                    additional_information: "Failed to authenticate entity with the provided credentials.",

                    http_status: Status::Unauthorized,
                };
            }
            ApiError::MissingSessionKey => {
                response_body = ApiErrorResponse {
                    message: "MissingSessionKey",
                    requested_path: req.uri().to_string(),
                    code: 5,
                    additional_information: "No session key passed as 'nys-session' cookie. Please ensure cookies are enabled and authenticate with GET /v1/public/iam/session",

                    http_status: Status::Unauthorized,
                };
            }
            ApiError::InvalidSession => {
                response_body = ApiErrorResponse {
                    message: "InvalidSession",
                    requested_path: req.uri().to_string(),
                    code: 6,
                    additional_information: "The session key passed appears to be invalid.",

                    http_status: Status::Unauthorized,
                };
            }
            ApiError::NoMatchingPrivilege => {
                response_body = ApiErrorResponse {
                    message: "NoMatchingPrivilege",
                    requested_path: req.uri().to_string(),
                    code: 7,
                    additional_information: "The AE doesn't have the requisite permission to perform the solicited action on this resource.",

                    http_status: Status::Unauthorized,
                };
            }
            ApiError::MalformedPermission => {
                response_body = ApiErrorResponse {
                    message: "MalformedPermission",
                    requested_path: req.uri().to_string(),
                    code: 7,
                    additional_information: "The permission string is malformed.",

                    http_status: Status::InternalServerError,
                };
            }
        }

        Response::build_from(Json(&response_body).respond_to(req)?)
            .status(response_body.http_status)
            .ok()
    }
}

pub type ApiReturnValue<T> = Result<ApiResponse<T>, ApiError>;
pub type ApiEmptyReturnValue = Result<(), ApiError>;