use crate::models::*;
use crate::utils;
use crate::{AppData, DbConn};
use actix_web::{delete, get, post, web, Error, HttpRequest, HttpResponse};
use anyhow::Result;

fn db_get_architectures(conn: &DbConn, limit: i64, offset: i64) -> Result<Vec<DbArchitecture>> {
    Ok(DbArchitecture::find_all(conn, limit, offset)?)
}

/// retrieve all architectures
#[get("/architecture")]
pub async fn get_all(req: HttpRequest, data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    // pub async fn get_architectures(req: HttpRequest, json_data: web::Json<utils::Paginate>, data: web::Data<AppData>) -> Result<HttpResponse, Error>{
    // let (q_limit, q_offset) = utils::paginate_qs(req.query_string());
    let (limit, offset) = utils::paginate_qs(req.query_string());
    // let limit = json_data.size.unwrap_or(q_limit);
    // let offset = json_data.page.unwrap_or(q_offset);
    let conn = data.pool.get().expect("couldn't get db connection from pool");
    let response = web::block(move || db_get_architectures(&conn, limit, offset))
        .await
        .map_err(|e| {
            debug!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(response))
}

/// add an architecture
#[post("/architecture")]
pub async fn post(architecture: web::Json<NewArchitecture>, data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let conn = data.pool.get().expect("couldn't get db connection from pool");
    let response = web::block(move || DbArchitecture::create(&conn, architecture.code.clone()))
        .await
        .map_err(|e| {
            debug!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(response))
}

/// delete an architecture by id
#[delete("/architecture")]
pub async fn delete(post_data: web::Json<utils::IdType>, app_data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let conn = app_data.pool.get().expect("couldn't get db connection from pool");
    let response = web::block(move || DbArchitecture::delete(&conn, post_data.id))
        .await
        .map_err(|e| {
            debug!("{}", e);
            HttpResponse::InternalServerError().finish()
        })?;

    Ok(HttpResponse::Ok().json(response))
}
