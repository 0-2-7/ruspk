use crate::DbId;
use crate::{
    models::{DbRole, UserRole},
    Connection,
};
use crate::{schema::*, utils};
use chrono::NaiveDateTime;
use diesel::prelude::*;

use bcrypt::verify;
extern crate bcrypt;
use anyhow::Result;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

#[derive(Serialize, Deserialize, Identifiable, Queryable, Debug, Clone)]
#[diesel(table_name = user)]
pub struct DbUser {
    pub id: DbId,
    pub username: String,
    pub email: String,
    pub password: String,
    pub api_key: Option<String>,
    pub github_access_token: Option<String>,
    pub active: bool,
    pub confirmed_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Identifiable, Queryable, Debug, Clone)]
#[diesel(table_name = user)]
pub struct UserWithKey {
    pub id: DbId,
    pub username: String,
    pub password: String,
    pub email: String,
    pub active: bool,
    pub api_key: Option<String>,
    pub confirmed_at: Option<NaiveDateTime>,
}
impl UserWithKey {
    fn remove_password_from_output(&mut self) {
        // FixMe
        self.password = "".to_string();
    }
}

#[derive(Serialize, Deserialize, Identifiable, Queryable, Debug, Clone)]
#[diesel(table_name = user)]
pub struct User {
    pub id: DbId,
    pub username: String,
    pub email: String,
    pub active: bool,
    pub confirmed_at: Option<NaiveDateTime>,
}

impl User {
    pub fn find_all(conn: &mut Connection, limit: i64, offset: i64, search_term: String) -> QueryResult<Vec<User>> {
        user::table
            .order(user::id.desc())
            .filter(user::username.like(utils::fuzzy_search(&search_term)))
            .limit(limit)
            .offset(offset)
            .select((user::id, user::username, user::email, user::active, user::confirmed_at))
            .load::<User>(conn)
    }

    pub fn delete(conn: &mut Connection, id: DbId) -> QueryResult<usize> {
        let result = diesel::delete(user::table.filter(user::id.eq(id))).execute(conn)?;
        Ok(result)
    }

    // Auth
    pub fn validate_api_key(conn: &mut Connection, key: String) -> QueryResult<User> {
        user::table
            .filter(user::api_key.eq(key).and(user::active.eq(true)))
            .select((user::id, user::username, user::email, user::active, user::confirmed_at))
            .first::<User>(conn)
    }

    fn find_user(
        conn: &mut Connection,
        username: &Option<String>,
        email: &Option<String>,
    ) -> Result<UserWithKey> {
        if let Some(email) = email {
            debug!("{:?}", email);
            return Ok(user::table
                .filter(user::email.eq(email).and(user::active.eq(true)))
                .select((
                    user::id,
                    user::username,
                    user::password,
                    user::email,
                    user::active,
                    user::api_key,
                    user::confirmed_at,
                ))
                .first::<UserWithKey>(conn)?);
        } else if let Some(username) = username {
            debug!("{:?}", username);
            return Ok(user::table
                .filter(user::username.eq(username).and(user::active.eq(true)))
                .select((
                    user::id,
                    user::username,
                    user::password,
                    user::email,
                    user::active,
                    user::api_key,
                    user::confirmed_at,
                ))
                .first::<UserWithKey>(conn)?);
        }
        Err(diesel::result::Error::NotFound.into())
    }

    pub fn login(
        conn: &mut Connection,
        username: &Option<String>,
        email: &Option<String>,
        password: &str,
    ) -> Result<(UserWithKey, Vec<DbRole>)> {
        // let hashed_password = bcrypt::hash(password, 12)?;
        // debug!("{:?}", hashed_password);
        let mut user = Self::find_user(conn, username, email)?;
        debug!("{:?}", user);
        debug!(
            "{:?} | {:?}  | {:?}",
            password,
            &user.password,
            bcrypt::hash(password, 12)
        );
        let valid = verify(password, &user.password)?;
        debug!("{:?} | {:?} | {:?}", password, &user.password, valid);
        if valid {
            user.remove_password_from_output(); //todo fix me
            let roles = UserRole::belonging_to(&user)
                .inner_join(role::table)
                .select((role::id, role::name, role::description))
                .load::<DbRole>(conn)?;
            if !roles.is_empty() {
                // user has at least one role
                return Ok((user, roles));
            }
        }
        Err(diesel::result::Error::NotFound.into())
    }

    pub fn send_reset_link(
        conn: &mut Connection,
        email: &String,
    ) -> Result<(UserWithKey, Vec<DbRole>)> {
        // let hashed_password = bcrypt::hash(password, 12)?;
        // debug!("{:?}", hashed_password);

        let user = user::table
            .filter(user::email.eq(email).and(user::active.eq(true)))
            .select((
                user::id,
                user::username,
                user::password,
                user::email,
                user::active,
                user::api_key,
                user::confirmed_at,
            ))
            .first::<UserWithKey>(conn)?;

        debug!("{:?}", user);

        let token: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        debug!("Password Reset Token: {:?}", token);

        Err(diesel::result::Error::NotFound.into())
    }

    pub fn reset(
        conn: &mut Connection,
        username: &Option<String>,
        email: &Option<String>,
        new_password: &str,
        reset_token: &str,
    ) -> Result<(UserWithKey, Vec<DbRole>)> {
        // let hashed_password = bcrypt::hash(password, 12)?;
        // debug!("{:?}", hashed_password);
        let mut user = Self::find_user(conn, username, email)?;

        debug!("{:?}", user);
        debug!(
            "{:?} | {:?}  | {:?}",
            new_password,
            &user.password,
            bcrypt::hash(new_password, 12)
        );
        let valid = verify(new_password, &user.password)?;
        debug!("{:?} | {:?} | {:?}", new_password, &user.password, valid);
        if valid {
            user.remove_password_from_output(); //todo fix me
            let roles = UserRole::belonging_to(&user)
                .inner_join(role::table)
                .select((role::id, role::name, role::description))
                .load::<DbRole>(conn)?;
            return Ok((user, roles));
        }
        Err(diesel::result::Error::NotFound.into())
    }
}
