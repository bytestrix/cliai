use anyhow::Result;
use clap::{Parser, Subcommand};
use cliai::{Config, TestCategory, TestSuite};
use colored::*;

#[derive(Parser)]
#[command(name = "test_runner")]
#[command(about = "CLIAI Test Suite Runner")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all tests in the comprehensive test suite
    All {
        /// Save results to file
        #[arg(long)]
        save: Option<String>,
    },
    /// Run tests for specific categories
    Category {
        /// Categories to test (file-management, system-info, git-operations, network, programming, process-management, general)
        categories: Vec<String>,
        /// Save results to file
        #[arg(long)]
        save: Option<String>,
    },
    /// List all available test questions
    List,
    /// Show test categories
    Categories,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load();
    let test_suite = TestSuite::new();

    match cli.command {
        Commands::All { save } => {
            println!("{}", "🚀 Running Complete CLIAI Test Suite".bold().green());
            println!();

            let results = test_suite.run_complete_test_suite(config).await?;

            if let Some(filename) = save {
                test_suite.save_test_results(&results, &filename)?;
            }
        }
        Commands::Category { categories, save } => {
            let parsed_categories: Result<Vec<TestCategory>, String> = categories
                .iter()
                .map(|cat| match cat.to_lowercase().as_str() {
                    "file-management" => Ok(TestCategory::FileManagement),
                    "system-info" => Ok(TestCategory::SystemInfo),
                    "git-operations" => Ok(TestCategory::GitOperations),
                    "network" => Ok(TestCategory::Network),
                    "programming" => Ok(TestCategory::Programming),
                    "process-management" => Ok(TestCategory::ProcessManagement),
                    "general" => Ok(TestCategory::General),
                    _ => Err(format!("Unknown category: {}", cat)),
                })
                .collect();

            match parsed_categories {
                Ok(cats) => {
                    let results = test_suite.run_category_tests(config, cats).await?;

                    if let Some(filename) = save {
                        test_suite.save_test_results(&results, &filename)?;
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "❌".red(), e);
                    println!("\nAvailable categories:");
                    println!("  • file-management");
                    println!("  • system-info");
                    println!("  • git-operations");
                    println!("  • network");
                    println!("  • programming");
                    println!("  • process-management");
                    println!("  • general");
                }
            }
        }
        Commands::List => {
            println!("{}", "📋 CLIAI Test Questions".bold().cyan());
            println!();

            for question in test_suite.get_test_questions() {
                let category_color = match question.category {
                    TestCategory::FileManagement => "blue",
                    TestCategory::SystemInfo => "green",
                    TestCategory::GitOperations => "yellow",
                    TestCategory::Network => "magenta",
                    TestCategory::Programming => "cyan",
                    TestCategory::ProcessManagement => "red",
                    TestCategory::General => "white",
                };

                println!(
                    "{:2}. {} [{}] {}",
                    question.id,
                    question.question,
                    format!("{:?}", question.category).color(category_color),
                    if question.should_have_command {
                        "📝"
                    } else {
                        "💬"
                    }
                );
            }

            println!();
            println!("Legend: 📝 = Should generate command, 💬 = Explanation only");
        }
        Commands::Categories => {
            println!("{}", "📂 Test Categories".bold().cyan());
            println!();

            let mut category_counts = std::collections::HashMap::new();
            for question in test_suite.get_test_questions() {
                *category_counts.entry(&question.category).or_insert(0) += 1;
            }

            for (category, count) in category_counts {
                println!("• {:?}: {} tests", category, count);
            }
        }
    }

    Ok(())
}
