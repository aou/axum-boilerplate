// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        hashed_password -> Nullable<Varchar>,
    }
}
