use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    #[command(subcommand)]
    Expense(ExpenseCommand),
    #[command(subcommand)]
    Category(CategoryCommand),
    #[command(subcommand)]
    Report(ReportCommand),
}

#[derive(Subcommand)]
pub enum ExpenseCommand {
    List,
    Get {
        id: Uuid,
    },
    Create {
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        category_id: Uuid,
        #[arg(long)]
        description: String,
        #[arg(long)]
        date: NaiveDate,
    },
    Update {
        id: Uuid,
        #[arg(long)]
        amount: f64,
        #[arg(long)]
        category_id: Uuid,
        #[arg(long)]
        description: String,
        #[arg(long)]
        date: NaiveDate,
    },
    Delete {
        id: Uuid,
    },
}

#[derive(Subcommand)]
pub enum CategoryCommand {
    List,
    Create {
        #[arg(long)]
        name: String,
    },
    Delete {
        id: Uuid,
    },
}

#[derive(Subcommand)]
pub enum ReportCommand {
    Total {
        #[arg(long)]
        from: Option<NaiveDate>,
        #[arg(long)]
        to: Option<NaiveDate>,
    },
    ByCategory {
        #[arg(long)]
        from: Option<NaiveDate>,
        #[arg(long)]
        to: Option<NaiveDate>,
    },
    ByMonth {
        #[arg(long)]
        from: Option<NaiveDate>,
        #[arg(long)]
        to: Option<NaiveDate>,
    },
    ByCategoryMonth {
        #[arg(long)]
        from: Option<NaiveDate>,
        #[arg(long)]
        to: Option<NaiveDate>,
    },
}
