use actix_web::web::Data;
use actix_web::{get, put, web, HttpResponse, Responder};
use std::sync::Mutex;
use crate::{DbPool};
use crate::mutex::LockResultExt;
use crate::service::notification_service::NotificationService;

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Gets all unread notifications.",body= Vec<Notification>)),
tag="notifications"
)]
#[get("/notifications/unread")]
pub async fn get_unread_notifications(notification_service: Data<Mutex<NotificationService>>, conn: Data<DbPool> ) ->
                                                                                             impl Responder {
    let notifications = notification_service
        .lock().ignore_poison()
        .get_unread_notifications(&mut conn.get().unwrap());
    HttpResponse::Ok().json(notifications.unwrap())
}

#[derive(Deserialize)]
pub struct NotificationId {
    id: i32,
}

#[utoipa::path(
context_path="/api/v1",
responses(
(status = 200, description = "Dismisses a notification")),
tag="notifications"
)]
#[put("/notifications/dismiss")]
pub async fn dismiss_notifications(
    id: web::Json<NotificationId>,
    notification_service: Data<Mutex<NotificationService>>, conn: Data<DbPool>
) -> impl Responder {
    notification_service.lock()
        .ignore_poison()
        .update_status_of_notification(id.id, "dismissed",
                                       &mut conn.get().unwrap())
        .expect("Error dismissing notification");
    HttpResponse::Ok()
}
