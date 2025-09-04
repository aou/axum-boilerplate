use std::io;

use axum_boilerplate::db::models::*;
use axum_boilerplate::db::*;
use diesel::prelude::*;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    NewUser,
    ShowUsers,
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::NewUser => {
            create_new_user_from_prompt();
        }
        Commands::ShowUsers => {
            show_users();
        }
    };

    println!("{cli:#?}");
}

fn show_users() {
    use axum_boilerplate::db::schema::users::dsl::*;

    let connection = &mut establish_connection();
    let results = users
        .limit(5)
        .select(User::as_select())
        .load(connection)
        .expect("Error loading users");

    println!("Displaying {} users", results.len());
    for user in results {
        println!("{:#?}", user);
    }
}

fn create_new_user_from_prompt() {
    use axum_boilerplate::db::schema::users;

    let connection = &mut establish_connection();

    let mut username = String::new();
    let mut password = String::new();

    println!("username: ");
    io::stdin().read_line(&mut username).unwrap();
    println!("password: ");
    io::stdin().read_line(&mut password).unwrap();

    println!("{} : {}", username.trim(), password.trim());

    let hashed_password = bcrypt::hash(password.trim(), bcrypt::DEFAULT_COST).unwrap();

    let new_user = NewUser {
        username: &username,
        hashed_password: &hashed_password,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(connection)
        .expect("error saving user");
}
