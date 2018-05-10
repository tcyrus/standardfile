use db::{UpdateUser, UserUpdateChange};
use bcrypt::{DEFAULT_COST, hash};
use pwdetails::{PasswordDetails};
use actix_web::{
    HttpRequest, HttpResponse, HttpMessage,
    FutureResponse, AsyncResponder,
    Json, State, Either, ResponseError,
};
use actix_web::middleware::identity::RequestIdentity;
use futures::{Future};

use api::{
    errors::SFError,
    ServiceState
};
use super::{
    encode_user_jwt, JwtMsg
};

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRequest {
    #[serde(flatten)]
    pw_info: PasswordDetails
}

pub fn update(req: HttpRequest<ServiceState>, info: Json<UpdateRequest>, state: State<ServiceState>) -> Either<FutureResponse<HttpResponse>, HttpResponse> {
    let user_uuid = match req.identity() {
        Some(user_uuid) => user_uuid.to_string(),
        None => return Either::B(SFError::InvalidCredentials.error_response()),
    };
    let user_change = UserUpdateChange {
        pw_func: info.pw_info.pw_func.clone(),
        pw_alg: info.pw_info.pw_alg.clone(),
        pw_cost: info.pw_info.pw_cost.clone(),
        pw_key_size: info.pw_info.pw_key_size.clone(),
        pw_nonce: info.pw_info.pw_nonce.clone(),
        pw_salt: info.pw_info.pw_salt.clone(),
        version: info.pw_info.version.clone(),
        ..UserUpdateChange::default()
    };
    Either::A(
        state.db.send(
            UpdateUser {
                uuid: user_uuid.clone(),
                user: user_change
            })
            .from_err()
            .and_then(|res| match res {
                Err(_) => Ok(SFError::InvalidCredentials.error_response()),
                Ok(new_user) => Ok(HttpResponse::Ok().json(encode_user_jwt(&new_user)))
            })
            .responder())
}

