mod cli;
mod classifier;
mod error;
mod output;
mod priority;
mod project_type;
mod recommendation;
mod scanner;

use clap::Parser;
use std::path::Path;

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(e.exit_code());
    }
}

fn run() -> Result<(), error::LlsError> {
    let args = cli::Cli::parse();
    let target = Path::new(&args.path);

    // Step 1-2: Scan filesystem
    let (entries, scanner_warnings) = scanner::scan(target, args.depth)?;

    // Step 3-4: Classify each entry
    let mut classified: Vec<classifier::ClassifiedEntry> = entries
        .iter()
        .map(classifier::classify)
        .collect();

    // Step 5: Assign priority
    for entry in &mut classified {
        priority::assign(entry);
    }

    // Step 6: Sort by priority, role, name
    let priority_order = |p: &str| -> usize {
        match p {
            "critical" => 0,
            "high" => 1,
            "medium" => 2,
            "low" => 3,
            "ignore" => 4,
            _ => 5,
        }
    };
    classified.sort_by(|a, b| {
        let pa = priority_order(&a.priority);
        let pb = priority_order(&b.priority);
        pa.cmp(&pb).then_with(|| {
            // For same priority, sort by role roughly
            a.role.cmp(&b.role).then_with(|| a.name.cmp(&b.name))
        })
    });

    // Step 7: Detect project type
    let project_type = project_type::detect(&classified);

    // Step 8: Generate recommendations
    let recommended = recommendation::generate(&classified);

    // Step 9: Output
    if args.json {
        let json = output::build_output(
            &args.path,
            project_type,
            classified,
            recommended,
            scanner_warnings,
            args.compact,
        );
        println!("{json}");
    } else {
        // Build human-readable output with warnings
        let mut warnings: Vec<output::Warning> = scanner_warnings
            .into_iter()
            .map(|(path, msg)| output::Warning { path, message: msg })
            .collect();
        for entry in &classified {
            if entry.sensitive {
                warnings.push(output::Warning {
                    path: entry.path.clone(),
                    message: "秘密情報候補を検出したため、明示的に必要な場合を除き内容を読まないこと".into(),
                });
            }
        }
        let human = output::format_human(
            &args.path,
            &project_type,
            &classified,
            &recommended,
            &warnings,
        );
        println!("{human}");
    }

    Ok(())
}
