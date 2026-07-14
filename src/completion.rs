use clap::{Command, CommandFactory};
use clap_complete::Shell;
use std::fs::{self, File};
use std::io;
use std::path::Path;

fn generate_one(
    shell: Shell,
    app: &mut Command,
    app_name: &str,
    output_dir: &Path,
    relative_path: &str,
) -> io::Result<()> {
    let destination = output_dir.join(relative_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(destination)?;
    clap_complete::generate(shell, app, app_name, &mut file);
    Ok(())
}

/// Generate completion files for every shell supported by clap_complete.
pub fn generate(output_dir: &Path) -> io::Result<()> {
    let app_name = "lls";
    let shells = [
        (Shell::Bash, "bash/lls"),
        (Shell::Zsh, "zsh/_lls"),
        (Shell::Fish, "fish/lls"),
        (Shell::PowerShell, "powershell/lls.ps1"),
        (Shell::Elvish, "elvish/lls"),
    ];

    for (shell, relative_path) in shells {
        let mut app = crate::cli::CliArgs::command();
        app.set_bin_name(app_name);
        generate_one(shell, &mut app, app_name, output_dir, relative_path)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generate;
    use std::fs;

    #[test]
    fn generates_completion_files_for_all_supported_shells() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");

        generate(directory.path()).expect("completion generation should succeed");

        for relative_path in [
            "bash/lls",
            "zsh/_lls",
            "fish/lls",
            "powershell/lls.ps1",
            "elvish/lls",
        ] {
            let path = directory.path().join(relative_path);
            assert!(
                path.is_file(),
                "missing completion file: {}",
                path.display()
            );
            assert!(!fs::read_to_string(path).unwrap_or_default().is_empty());
        }
    }

    #[test]
    fn generated_completions_include_cli_commands_and_sort_values() {
        let directory = tempfile::tempdir().expect("temporary directory should be created");

        generate(directory.path()).expect("completion generation should succeed");

        let zsh = fs::read_to_string(directory.path().join("zsh/_lls"))
            .expect("zsh completion should be readable");
        assert!(zsh.contains("setup"));
        assert!(zsh.contains("completions"));
        assert!(zsh.contains("priority"));
        assert!(zsh.contains("name"));
        assert!(zsh.contains("mtime"));
        assert!(zsh.contains("size"));
    }
}
