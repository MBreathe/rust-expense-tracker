use chrono::NaiveDate;
use clap::{Parser, Subcommand};
use uuid::Uuid;

/// CLI client for the expense-tracker HTTP API.
#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create a new user account.
    Register {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    /// Log in and save a bearer token for subsequent commands.
    Login {
        #[arg(long)]
        username: String,
        #[arg(long)]
        password: String,
    },
    /// Manage expenses.
    #[command(subcommand)]
    Expense(ExpenseCommand),
    /// Manage categories.
    #[command(subcommand)]
    Category(CategoryCommand),
    /// View spending reports.
    #[command(subcommand)]
    Report(ReportCommand),
}

#[derive(Subcommand)]
pub enum ExpenseCommand {
    /// List all expenses.
    List,
    /// Get a single expense by id.
    Get {
        id: Uuid,
    },
    /// Create a new expense.
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
    /// Replace an existing expense's fields.
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
    /// Delete an expense by id.
    Delete {
        id: Uuid,
    },
}

#[derive(Subcommand)]
pub enum CategoryCommand {
    /// List all categories.
    List,
    /// Create a new category.
    Create {
        #[arg(long)]
        name: String,
    },
    /// Delete a category by id.
    Delete {
        id: Uuid,
    },
}

#[derive(Subcommand)]
pub enum ReportCommand {
    /// Total spend, optionally filtered by date range.
    Total {
        /// Start of the date range (inclusive). Omit for no lower bound.
        #[arg(long)]
        from: Option<NaiveDate>,
        /// End of the date range (inclusive). Omit for no upper bound.
        #[arg(long)]
        to: Option<NaiveDate>,
    },
    /// Spend grouped by category, optionally filtered by date range.
    ByCategory {
        /// Start of the date range (inclusive). Omit for no lower bound.
        #[arg(long)]
        from: Option<NaiveDate>,
        /// End of the date range (inclusive). Omit for no upper bound.
        #[arg(long)]
        to: Option<NaiveDate>,
    },
    /// Spend grouped by month, optionally filtered by date range.
    ByMonth {
        /// Start of the date range (inclusive). Omit for no lower bound.
        #[arg(long)]
        from: Option<NaiveDate>,
        /// End of the date range (inclusive). Omit for no upper bound.
        #[arg(long)]
        to: Option<NaiveDate>,
    },
    /// Spend grouped by category and month, optionally filtered by date range.
    ByCategoryMonth {
        /// Start of the date range (inclusive). Omit for no lower bound.
        #[arg(long)]
        from: Option<NaiveDate>,
        /// End of the date range (inclusive). Omit for no upper bound.
        #[arg(long)]
        to: Option<NaiveDate>,
    },
}
