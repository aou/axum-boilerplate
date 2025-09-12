use diesel::prelude::*;
use dotenvy::dotenv;
use models::User;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn get_user_by_email(email: &str) -> Option<User> {
    let connection = &mut establish_connection();

    let user = schema::users::table
        .filter(schema::users::email.eq(email))
        .first(connection)
        .optional()
        .unwrap();

    user
}

pub fn get_user_by_username(username: &str) -> Option<User> {
    let connection = &mut establish_connection();

    let user = schema::users::table
        .filter(schema::users::username.eq(username))
        .first(connection)
        .optional()
        .unwrap();

    user
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email() {
        let user = get_user_by_email("alexou@gmail.com");
        println!("{user:#?}");
    }
}
