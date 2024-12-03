use clap::Parser;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

use qp_compare::diff_plan;
use qp_compare::legacy_planner;
use qp_compare::native_planner;
use qp_compare::plan_matches;
use qp_compare::render_legacy_plan;
use qp_compare::render_native_plan;
use qp_compare::run_legacy_planner;
use qp_compare::run_native_planner;

#[derive(Debug, clap::Parser)]
pub struct PlanArgs {
    /// Specify path to schema file(s) to plan operations against
    #[arg(short, long)]
    pub schema: PathBuf,

    /// Specify path to an operation file to plan.
    /// This can be either a directory of operations or a file.
    #[arg(short, long)]
    pub operation: PathBuf,

    #[arg(long, default_value = "true")]
    pub generate_fragments: bool,

    #[arg(long, default_value = "false")]
    pub type_conditioned_fetching: bool,

    /// Dump both legacy/native query plans in files.
    #[arg(long, default_value = "false")]
    pub dump_plans: bool,
}

impl From<&PlanArgs> for native_planner::QueryPlannerConfig {
    fn from(args: &PlanArgs) -> Self {
        Self {
            generate_query_fragments: args.generate_fragments,
            type_conditioned_fetching: args.type_conditioned_fetching,
            ..Default::default()
        }
    }
}

impl From<&PlanArgs> for legacy_planner::QueryPlannerConfig {
    fn from(args: &PlanArgs) -> Self {
        Self {
            reuse_query_fragments: Some(false),
            generate_query_fragments: Some(args.generate_fragments),
            type_conditioned_fetching: args.type_conditioned_fetching,
            ..Default::default()
        }
    }
}

fn write_file(path: &str, content: &str) {
    let mut file = fs::OpenOptions::new()
        .create(true) // Create the file if it doesn't exist
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Unable to open file");
    file.write_all(content.as_bytes())
        .expect("Unable to write data");
}

pub fn run_both_planners(schema_str: &str, query_str: &str, args: &PlanArgs) -> Result<(), String> {
    let rust_plan = run_native_planner(
        schema_str,
        query_str,
        None,
        &args.operation,
        args.into(),
        Default::default(),
    )
    .map_err(|err| err.to_string())?;
    let js_plan = run_legacy_planner(schema_str, query_str, None, args.into(), Default::default())
        .map_err(|err| err.join("\n"))?;
    if args.dump_plans {
        write_file(
            "./plan_legacy.txt",
            js_plan.formatted_query_plan.as_ref().unwrap(),
        );
        write_file("./plan_legacy.detail.txt", &render_legacy_plan(&js_plan));
        write_file("./plan_native.txt", rust_plan.to_string().as_str());
        write_file("./plan_native.detail.txt", &render_native_plan(&rust_plan));
    }
    match plan_matches(&js_plan, &rust_plan) {
        Ok(_) => Ok(()),
        Err(match_failure) => {
            let diff = diff_plan(&js_plan, &rust_plan);
            Err(format!(
                "Query plan mismatch:\n{match_failure:#?}\n\nDiff:\n{diff}"
            ))
        }
    }
}

fn main() -> ExitCode {
    let args = PlanArgs::parse();
    let schema = fs::read_to_string(&args.schema).unwrap();
    let query = fs::read_to_string(&args.operation).unwrap();
    let result = run_both_planners(&schema, &query, &args);
    match result {
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }

        Ok(_) => {
            println!("qp matched");
            ExitCode::SUCCESS
        }
    }
}
