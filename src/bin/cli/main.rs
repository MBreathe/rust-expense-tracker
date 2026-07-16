use clap::Parser;

use crate::args::{CategoryCommand, Cli, Command, ExpenseCommand, ReportCommand};

mod args;
mod commands;
mod http;
mod token;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Command::Register { username, password } => {
            commands::auth::register(username, password).await
        }
        Command::Login { username, password } => commands::auth::login(username, password).await,
        Command::Expense(cmd) => match cmd {
            ExpenseCommand::List => commands::expense::list().await,
            ExpenseCommand::Get { id } => commands::expense::get(id).await,
            ExpenseCommand::Create {
                amount,
                category_id,
                description,
                date,
            } => commands::expense::create(amount, category_id, description, date).await,
            ExpenseCommand::Update {
                id,
                amount,
                category_id,
                description,
                date,
            } => commands::expense::update(id, amount, category_id, description, date).await,
            ExpenseCommand::Delete { id } => commands::expense::delete(id).await,
        },
        Command::Category(cmd) => match cmd {
            CategoryCommand::List => commands::category::list().await,
            CategoryCommand::Create { name } => commands::category::create(name).await,
            CategoryCommand::Delete { id } => commands::category::delete(id).await,
        },
        Command::Report(cmd) => match cmd {
            ReportCommand::Total { from, to } => commands::report::total(from, to).await,
            ReportCommand::ByCategory { from, to } => commands::report::by_category(from, to).await,
            ReportCommand::ByMonth { from, to } => commands::report::by_month(from, to).await,
            ReportCommand::ByCategoryMonth { from, to } => {
                commands::report::by_category_month(from, to).await
            }
        },
    };

    if let Err(err) = result {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
