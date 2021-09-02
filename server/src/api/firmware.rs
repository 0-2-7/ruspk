use crate::models::*;
use actix_web::{get, web, Error, HttpResponse};
use crate::{AppData, DbConn};
use anyhow::Result;
use diesel::{self, prelude::*};


fn db_get_firmware(conn: &DbConn) -> Result<Vec<DbFirmware>> {
    use crate::schema::firmware;
    let p = firmware::table
        .load::<DbFirmware>(conn)
        .expect("Error loading firmware from DB");
    Ok(p)
}

/// retrieve all firmware
#[get("/firmware")]
pub async fn get_firmware(data: web::Data<AppData>) -> Result<HttpResponse, Error>{
    let conn = data.pool.get().expect("couldn't get db connection from pool");
    let response = web::block(move || db_get_firmware(&conn)).await.map_err(|e| {
        debug!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    Ok(HttpResponse::Ok().json(response))
}
