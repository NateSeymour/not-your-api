use aws_sdk_dynamodb::model::{AttributeValue, Select};
use redis::Commands;
use rocket::serde::json::Json;
use rocket::http::{CookieJar, Cookie, Status};
use rocket::Request;
use rocket::request::{FromRequest, Outcome};
use uuid::Uuid;
use crate::db;
use crate::api_response::{ApiEmptyReturnValue, ApiError, ApiResponse, ApiReturnValue};

/*
universe:service:entity:resource:action
 */

pub struct AuthenticatedSession {
    id: String,
    entity_id: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct PermissionsDefinition {
    permissions: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AuthenticatableEntity {
    pub id: String,
    password_hash: String,
    enabled: bool,
    permissions: PermissionsDefinition,
}

impl AuthenticatableEntity {
    pub fn new(id: String, password: String) -> AuthenticatableEntity {
        let base_permission_string = format!["nys:*:{}:*:*", id];

        AuthenticatableEntity {
            id,
            password_hash: bcrypt::hash(password, 10).unwrap(),
            enabled: true,
            permissions: PermissionsDefinition {
                permissions: Vec::from([base_permission_string]),
            },
        }
    }

    pub fn assert_privilege(&self, privilege: String) -> Result<(), ApiError> {
        let mut req_privilege_components = privilege.split_terminator(':');

        if req_privilege_components.clone().count() != 5 {
            return Err(ApiError::MalformedPermission);
        }

        'new_permission: for real_privilege in &self.permissions.permissions {
            let mut real_privilege_components = real_privilege.split_terminator(':');

            if real_privilege_components.clone().count() != 5 {
                continue;
            }

            'component: for i in 0..5 {
                let pcomp = req_privilege_components.next().unwrap();
                let rcomp = real_privilege_components.next().unwrap();

                if rcomp.len() > pcomp.len() {
                    continue 'new_permission;
                }

                if rcomp.eq(pcomp) {
                    continue 'component;
                }

                'char: for y in 0..pcomp.len() {
                    if rcomp.chars().nth(y).unwrap() == '*' {
                        continue 'component;
                    }

                    if rcomp.chars().nth(y).unwrap() != pcomp.chars().nth(y).unwrap() {
                        continue 'char;
                    }

                    continue 'new_permission;
                }
            }

            return Ok(());
        }

        Err(ApiError::NoMatchingPrivilege)
    }

    pub async fn retrieve(db_client: &aws_sdk_dynamodb::Client, redis_client: &redis::Client, id: String, force_reload: bool) -> Result<AuthenticatableEntity, ApiError> {
        let ae_cache_key = format!("cache:ae:{}", id);

        // Start by checking the cache for the user
        if !force_reload {
            match redis_client.get_connection() {
                Ok(mut conn) => {
                    let ae_exists : bool = conn.exists(&ae_cache_key).unwrap();

                    if ae_exists {
                        // Get value string
                        let ae_json : String = conn.get(&ae_cache_key).unwrap();
                        return Ok(serde_json::from_str(&ae_json).unwrap());
                    }
                },
                Err(_) => return Err(ApiError::CacheUnavailable),
            }
        }

        // Query entity from DB
        let query = db_client.query()
            .table_name(db::Table::IAM.as_str())
            .key_condition_expression("id = :id")
            .expression_attribute_values(":id", AttributeValue::S(id))
            .select(Select::AllAttributes)
            .send().await;

        let query_result = match query {
            Ok(res) => res,
            Err(_) => return Err(ApiError::MeNoLikeyAWS),
        };

        if query_result.count() > 1 {
            return Err(ApiError::TooManyUsers);
        }

        let query_items = match query_result.items {
            Some(items) => items,
            None => return Err(ApiError::UserNotFound),
        };

        let authenticatable_entities : Vec<AuthenticatableEntity> = serde_dynamo::from_items(query_items).unwrap();
        let authenticatable_entity = match authenticatable_entities.first() {
            Some(user) => user.clone(),
            None => return Err(ApiError::UserNotFound),
        };

        // Cache AE
        match redis_client.get_connection() {
            Ok(mut conn) => {
                let _ : () = conn.set(ae_cache_key, serde_json::to_string(&authenticatable_entity).unwrap()).unwrap();
            },
            Err(_) => return Err(ApiError::CacheUnavailable),
        }

        Ok(authenticatable_entity)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatableEntity {
    type Error = ApiError;
    
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        // Get session key from cookies
        let session_cookie = req.cookies().get("nys-session");

        let session_key = match session_cookie {
            Some(cookie) => cookie.value().to_string(),
            None => return Outcome::Failure((Status::BadRequest, ApiError::MissingSessionKey)),
        };

        // Grab database and cache clients
        let ddb_client = match req.rocket().state::<aws_sdk_dynamodb::Client>() {
            Some(client) => client,
            None => return Outcome::Failure((Status::InternalServerError, ApiError::MeNoLikeyAWS)),
        };

        let redis_client = match req.rocket().state::<redis::Client>() {
            Some(client) => client,
            None => return Outcome::Failure((Status::InternalServerError, ApiError::CacheUnavailable)),
        };

        // Find AE associated with session
        let session_cache_key = format!("session:{}", session_key);

        let ae_id : String;

        match redis_client.get_connection() {
            Ok(mut conn) => {
                let session_exists : bool = conn.exists(&session_cache_key).unwrap();

                if session_exists {
                    // Get value string
                    ae_id = conn.get(&session_cache_key).unwrap();
                } else {
                    return Outcome::Failure((Status::Unauthorized, ApiError::CacheUnavailable));
                }
            },
            Err(_) => return Outcome::Failure((Status::InternalServerError, ApiError::CacheUnavailable)),
        };

        match AuthenticatableEntity::retrieve(ddb_client, redis_client, ae_id, false).await {
            Ok(ae) => Outcome::Success(ae),
            Err(err) => Outcome::Failure((Status::InternalServerError, err)),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateAuthenticatableEntityRB<'r> {
    id: &'r str,
    password: &'r str,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GetSessionResponse {
    session_id: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct GetSessionRB<'r> {
    id: &'r str,
    password: &'r str,
}

#[rocket::get("/session", data="<login_info>")]
pub async fn get_session(cookies: &CookieJar<'_>, db_client: &rocket::State<aws_sdk_dynamodb::Client>, redis_client: &rocket::State<redis::Client>, login_info: Json<GetSessionRB<'_>>) -> ApiReturnValue<GetSessionResponse> {
    let authenticatable_entity = AuthenticatableEntity::retrieve(db_client, redis_client, login_info.id.to_string(), false).await?;

    // Check password validity
    match bcrypt::verify(login_info.password, authenticatable_entity.password_hash.as_str()) {
        Ok(success) => {
            if success {
                // Generate new session token
                let session_id = Uuid::new_v4();

                // Generate session info
                let session = AuthenticatedSession {
                    id: session_id.to_string(),
                    entity_id: login_info.id.to_string(),
                };

                // Insert session into cache
                match redis_client.get_connection() {
                    Ok(mut conn) => {
                        match conn.set(format!("session:{}", session.id), session.entity_id) {
                            Ok(()) => {},
                            Err(_) => return Err(ApiError::CacheUnavailable),
                        }
                    },
                    Err(_) => return Err(ApiError::CacheUnavailable),
                }

                // Set cookie
                let session_cookie = Cookie::build("nys-session", session.id.clone())
                    //.domain("api.notyoursoftware.com")
                    .secure(true)
                    .http_only(true)
                    .path("/v1")
                    .finish();

                cookies.add(session_cookie);

                // Send response
                let response = GetSessionResponse {
                    session_id: session_id.to_string(),
                };

                return Ok(ApiResponse(Json(response)));
            } else {
                return Err(ApiError::AuthenticationFailed);
            }
        },
        Err(_) => Err(ApiError::AuthenticationFailed),
    }
}

#[rocket::post("/authenticatable_entity", data="<entity_info>")]
pub async fn create_authenticatable_entity(db_client: &rocket::State<aws_sdk_dynamodb::Client>, entity_info: Json<CreateAuthenticatableEntityRB<'_>>) -> ApiEmptyReturnValue {
    let new_entity = AuthenticatableEntity::new(entity_info.id.to_string(), entity_info.password.to_string());

    let item = serde_dynamo::to_item(new_entity).unwrap();

    let result = db_client.put_item()
        .table_name(db::Table::IAM.as_str())
        .set_item(Some(item))
        .send().await;

    match result {
        Ok(_) => Ok(()),
        Err(_) => Err(ApiError::MeNoLikeyAWS),
    }
}

pub fn routes() -> Vec<rocket::Route> { rocket::routes![get_session, create_authenticatable_entity] }