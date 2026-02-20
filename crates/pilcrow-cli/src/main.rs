mod commands;
mod util;
mod validate;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "sc", about = "Silcrow Clean Architecture scaffolding")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new service project
    New {
        /// Service name (kebab-case, e.g. order-service)
        name: String,
    },
    /// Generate components
    Gen {
        #[command(subcommand)]
        component: GenCommands,
    },
    /// Validate architectural boundaries
    Validate,
}

#[derive(Subcommand)]
enum GenCommands {
    /// Domain entity + repository trait + infrastructure stub
    Aggregate {
        /// PascalCase entity name (e.g. Order)
        name: String,
    },
    /// Application usecase with constructor injection
    Usecase {
        /// PascalCase usecase name (e.g. CreateOrder)
        name: String,
    },
    /// Presentation handler + route registration
    Endpoint {
        /// PascalCase handler name (e.g. CreateOrder)
        name: String,
        /// HTTP method (GET, POST, PUT, PATCH, DELETE)
        #[arg(long, default_value = "GET")]
        method: String,
        /// Route path (e.g. /orders)
        #[arg(long)]
        path: String,
    },
    /// Maud SSR template in presentation layer
    Template {
        /// PascalCase template name (e.g. OrderList)
        name: String,
    },
    /// sqlx migration file
    Migration {
        /// Migration name in snake_case (e.g. create_orders_table)
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::New { name } => commands::new::run(&name),
        Commands::Gen { component } => match component {
            GenCommands::Aggregate { name } => commands::gen_aggregate::run(&name),
            GenCommands::Usecase { name } => commands::gen_usecase::run(&name),
            GenCommands::Endpoint { name, method, path } => {
                commands::gen_endpoint::run(&name, &method, &path)
            }
            GenCommands::Template { name } => commands::gen_template::run(&name),
            GenCommands::Migration { name } => commands::gen_migration::run(&name),
        },
        Commands::Validate => validate::run(),
    };

    if let Err(e) = result {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
