use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProjectLanguage {
    Rust,
    NodeJs,
    Python,
    Go,
    Php,
    Ruby,
}

impl ProjectLanguage {
    pub fn name(&self) -> &'static str {
        match self {
            ProjectLanguage::Rust => "Rust",
            ProjectLanguage::NodeJs => "Node.js",
            ProjectLanguage::Python => "Python",
            ProjectLanguage::Go => "Go",
            ProjectLanguage::Php => "PHP",
            ProjectLanguage::Ruby => "Ruby",
        }
    }

    pub fn tool(&self) -> &'static str {
        match self {
            ProjectLanguage::Rust => "cargo",
            ProjectLanguage::NodeJs => "npm",
            ProjectLanguage::Python => "pip",
            ProjectLanguage::Go => "go",
            ProjectLanguage::Php => "composer",
            ProjectLanguage::Ruby => "bundle",
        }
    }

    pub fn install_cmd(&self) -> &'static str {
        match self {
            ProjectLanguage::Rust => "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && ~/.cargo/bin/rustup component add rustfmt clippy",
            ProjectLanguage::NodeJs => "curl -fsSL https://deb.nodesource.com/setup_22.x | sudo bash - && sudo apt-get install -y nodejs",
            ProjectLanguage::Python => "sudo apt-get update && sudo apt-get install -y python3 python3-pip",
            ProjectLanguage::Go => "wget https://go.dev/dl/go1.24.5.linux-amd64.tar.gz && sudo tar -C /usr/local -xzf go1.24.5.linux-amd64.tar.gz && rm go1.24.5.linux-amd64.tar.gz",
            ProjectLanguage::Php => "sudo apt-get update && sudo apt-get install -y php-cli unzip && curl -sS https://getcomposer.org/installer | php -- --install-dir=/usr/local/bin --filename=composer",
            ProjectLanguage::Ruby => "sudo apt-get update && sudo apt-get install -y ruby-full && sudo gem install bundler",
        }
    }
}

pub fn detect_project_languages(dir: &Path) -> Vec<ProjectLanguage> {
    let mut langs = Vec::new();
    if dir.join("Cargo.toml").exists() {
        langs.push(ProjectLanguage::Rust);
    }
    if dir.join("package.json").exists() {
        langs.push(ProjectLanguage::NodeJs);
    }
    if dir.join("requirements.txt").exists() || dir.join("pyproject.toml").exists() {
        langs.push(ProjectLanguage::Python);
    }
    if dir.join("go.mod").exists() {
        langs.push(ProjectLanguage::Go);
    }
    if dir.join("composer.json").exists() {
        langs.push(ProjectLanguage::Php);
    }
    if dir.join("Gemfile").exists() {
        langs.push(ProjectLanguage::Ruby);
    }
    langs
}

pub fn ensure_language_tools(container_name: &str, languages: &[ProjectLanguage]) -> Result<()> {
    for lang in languages {
        let tool = lang.tool();
        let check_status = Command::new("docker")
            .args([
                "exec",
                container_name,
                "bash",
                "-lc",
                &format!("command -v {tool}"),
            ])
            .status()
            .with_context(|| format!("Failed to check for {}", tool))?;
        if check_status.success() {
            continue;
        }
        println!("Installing toolchain for {}...", lang.name());
        let install_status = Command::new("docker")
            .args(["exec", container_name, "bash", "-lc", lang.install_cmd()])
            .status()
            .with_context(|| format!("Failed to install {}", tool))?;
        if !install_status.success() {
            anyhow::bail!("Installation for {} failed", tool);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detect_languages() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("Cargo.toml"), "").unwrap();
        fs::write(tmp.path().join("package.json"), "").unwrap();
        let langs = detect_project_languages(tmp.path());
        assert!(langs.contains(&ProjectLanguage::Rust));
        assert!(langs.contains(&ProjectLanguage::NodeJs));
    }
}
