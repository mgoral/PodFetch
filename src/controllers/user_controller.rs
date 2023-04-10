use actix_web::{HttpRequest, HttpResponse, post, get, put, Responder, web, delete};
use actix_web::web::Data;
use crate::constants::constants::{Role, USERNAME};
use crate::DbPool;
use crate::exception::exceptions::PodFetchErrorTrait;
use crate::models::user::User;
use crate::service::user_management_service::UserManagementService;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOnboardingModel{
    invite_id: String,
    username: String,
    password: String
}

#[derive(Deserialize, Debug)]
pub struct InvitePostModel{
    role: Role
}

#[derive(Deserialize)]
pub struct UserRoleUpdateModel{
    role: Role
}

#[post("/users/")]
pub async fn onboard_user(user_onboarding: web::Json<UserOnboardingModel>, conn: Data<DbPool>)->impl Responder{
    let user_to_onboard = user_onboarding.into_inner();

    let res = UserManagementService::onboard_user(user_to_onboard.username, user_to_onboard
        .password,
                                        user_to_onboard.invite_id, &mut *conn.get().unwrap());

    return match res {
        Ok(user) => HttpResponse::Ok().json(User::map_to_dto(user)),
        Err(e) => HttpResponse::BadRequest()
            .body(e.name().clone())
    };
}

#[get("")]
pub async fn get_users(req: HttpRequest, conn: Data<DbPool>)->impl Responder{
    let username = get_user_from_request(req);
    let user = User::find_by_username(&username, &mut *conn.get().unwrap());
    if user.is_none() {
        return HttpResponse::NotFound()
            .body("User not found")
    }
    UserManagementService::get_users(user.unwrap(),&mut *conn.get().unwrap()).map_err(|e| {
        HttpResponse::Forbidden()
            .body(e.name().clone())
    }).map(|users| {
        HttpResponse::Ok().json(users)
    }).unwrap()
}

#[get("/users/{username}")]
pub async fn get_user(req: HttpRequest,mut conn: Data<DbPool>)->impl Responder{
    let username = get_user_from_request(req);
    let user = User::find_by_username(&username, &mut *conn.get().unwrap());
    return match user {
        Some(user) => HttpResponse::Ok().json(User::map_to_dto(user)),
        None => HttpResponse::NotFound()
            .body("User not found")
    };
}

#[put("/{username}/role")]
pub async fn update_role(req: HttpRequest, role: web::Json<UserRoleUpdateModel>, conn: Data<DbPool>, username: web::Path<String>)
    ->impl Responder{

    let requester_username = get_user_from_request(req);
    let requester = User::find_by_username(&requester_username, &mut *conn.get().unwrap());
    if requester.is_none() {
        return HttpResponse::NotFound()
            .body("User not found")
    }
    let user_to_update = User::find_by_username(&username, &mut *conn.get().unwrap());

    if user_to_update.is_none() {
        return HttpResponse::NotFound()
            .body("User not found")
    }

    // Update to his/her designated role
    let mut found_user = user_to_update.unwrap();
    found_user.role = role.role.to_string();

    let res = UserManagementService::update_role(found_user, requester.unwrap(), &mut
                                              *conn.get()
        .unwrap());

    match res {
        Ok(_) =>{
            HttpResponse::Ok().into()
        },
        Err(e) => HttpResponse::BadRequest()
            .body(e.name().clone())
    }
}

#[post("/invites")]
pub async fn create_invite(req: HttpRequest, invite: web::Json<InvitePostModel>, conn: Data<DbPool>)
    ->impl
Responder{
    let invite = invite.into_inner();
    let username  = req.headers().get(USERNAME).unwrap()
        .to_str().unwrap();
    let user = User::find_by_username(username, &mut *conn.get().unwrap()).unwrap();
    UserManagementService::create_invite(invite.role, &mut *conn.get().unwrap(), user).expect
    ("Error creating invite");
    HttpResponse::Ok()
}

#[get("/invites")]
pub async fn get_invites(req: HttpRequest, conn: Data<DbPool>)->impl Responder{
    let username  = get_user_from_request(req);
    let user = User::find_by_username(&username, &mut *conn.get().unwrap()).unwrap();
    let invites = UserManagementService::get_invites(user, &mut *conn.get().unwrap()).expect
    ("Error getting invites");
    HttpResponse::Ok().json(invites)
}

#[get("/users/invites/{invite_id}")]
pub async fn get_invite(conn: Data<DbPool>, invite_id: web::Path<String>)->
    impl Responder{
    match UserManagementService::get_invite(invite_id.into_inner(), &mut *conn.get().unwrap()){
        Ok(invite) => HttpResponse::Ok().json(invite),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    }
}

#[delete("/{username}")]
pub async fn delete_user(conn:Data<DbPool>, username: web::Path<String>, req: HttpRequest)->impl Responder{
    let username_req  = get_user_from_request(req);
    let user = User::find_by_username(&username_req, &mut *conn.get().unwrap()).unwrap();

    let user_to_delete = User::find_by_username(&username, &mut *conn.get().unwrap()).unwrap();
    return match UserManagementService::delete_user(user_to_delete,user, &mut *conn.get().unwrap())
    {
        Ok(_) => HttpResponse::Ok().into(),
        Err(e) => HttpResponse::BadRequest().body(e.to_string())
    };
}


fn get_user_from_request(req: HttpRequest)->String{
    req.clone().headers().get(USERNAME).unwrap().to_str().unwrap().to_string()
}