use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, AsChangeset)]
#[diesel(table_name = crate::db::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(treat_none_as_null = true)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub hashed_password: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::db::schema::users)]
pub struct NewUser<'a> {
    pub username: &'a str,
    pub email: Option<&'a str>,
    pub hashed_password: Option<&'a str>,
}
