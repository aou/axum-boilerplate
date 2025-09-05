use std::io::{StdinLock, StdoutLock, Write, stdin, stdout};

use axum_boilerplate::db::models::*;
use axum_boilerplate::db::*;
use diesel::prelude::*;

use termion::input::TermRead;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(subcommand)]
    User(UserCommands),
}

#[derive(Debug, Subcommand)]
enum UserCommands {
    NewUser,
    ShowUsers,
    ChangePassword { id: i32 },
    DeleteUser { id: i32 },
}

fn main() {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match &cli.command {
        Commands::User(user_command) => match user_command {
            UserCommands::NewUser => {
                create_new_user_from_prompt();
            }
            UserCommands::ShowUsers => {
                show_users();
            }
            UserCommands::ChangePassword { id } => {
                change_password(*id);
            }
            UserCommands::DeleteUser { id } => {
                delete_user_by_id(*id);
            }
        },
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

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    stdout.write_all(b"username: ").unwrap();
    stdout.flush().unwrap();
    let username = stdin
        .read_line()
        .unwrap()
        .expect("Username cannot be blank");

    let hashed_password = prompt_and_hash_password(&mut stdin, &mut stdout);

    let new_user = NewUser {
        username: &username.trim(),
        hashed_password: &hashed_password,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .returning(User::as_returning())
        .get_result(connection)
        .expect("error saving user");
}

fn prompt_and_hash_password(stdin: &mut StdinLock, stdout: &mut StdoutLock) -> String {
    stdout.write_all(b"password: ").unwrap();
    stdout.flush().unwrap();
    let password = stdin
        .read_passwd(stdout)
        .unwrap()
        .expect("Password cannot be blank");

    let hashed_password = bcrypt::hash(password.trim(), bcrypt::DEFAULT_COST).unwrap();
    hashed_password
}

fn change_password(id: i32) {
    use axum_boilerplate::db::schema::users::dsl::{hashed_password, users};

    let connection = &mut establish_connection();

    let stdout = stdout();
    let mut stdout = stdout.lock();
    let stdin = stdin();
    let mut stdin = stdin.lock();

    let new_hashed_password = prompt_and_hash_password(&mut stdin, &mut stdout);

    let user = diesel::update(users.find(id))
        .set(hashed_password.eq(new_hashed_password))
        .returning(User::as_returning())
        .get_result(connection)
        .unwrap();

    println!("{user:#?}");
}

fn delete_user_by_id(id_to_delete: i32) {
    use axum_boilerplate::db::schema::users::dsl::*;

    let connection = &mut establish_connection();

    let num_deleted = diesel::delete(users.filter(id.eq(id_to_delete)))
        .execute(connection)
        .expect("Error while deleting user");

    println!("delected {num_deleted} users");
}
