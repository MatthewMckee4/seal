use std::fmt::Write as _;
use std::sync::Arc;

use anyhow::Result;
use seal_changelog::DEFAULT_CHANGELOG_PATH;
use seal_fs::FileResolver;
use seal_github::GitHubService;
use seal_project::ProjectWorkspace;

use crate::ExitStatus;
use crate::printer::Printer;

const MAX_PRS: usize = 100;

pub async fn generate_changelog(
    dry_run: bool,
    printer: Printer,
    overwrite: Option<bool>,
    max_prs: Option<usize>,
) -> Result<ExitStatus> {
    let mut stdout = printer.stdout();

    let workspace = ProjectWorkspace::discover()?;
    let config = workspace.config();

    let Some(changelog_config) = config.changelog.as_ref() else {
        return Err(anyhow::anyhow!(
            "No changelog configuration found in discovered workspace at `{}`",
            workspace.root().display()
        ));
    };

    let changelog_path = changelog_config
        .changelog_path
        .clone()
        .unwrap_or(workspace.root().join(DEFAULT_CHANGELOG_PATH));

    #[cfg(feature = "integration-test")]
    let github_client: Arc<dyn GitHubService> = {
        #[cfg(any(test, feature = "integration-test"))]
        use seal_github::MockGithubClient;
        Arc::new(MockGithubClient::new())
    };
    #[cfg(not(feature = "integration-test"))]
    let github_client: Arc<dyn GitHubService> = {
        use seal_github::{GitHubClient, get_git_remote_url, parse_github_repo};

        let repo_url = get_git_remote_url(workspace.root())?;
        let (owner, repo) = parse_github_repo(&repo_url)?;
        Arc::new(GitHubClient::new(owner, repo)?)
    };

    let changelog_content = seal_changelog::generate_full_changelog(
        changelog_config,
        &github_client,
        max_prs.unwrap_or(MAX_PRS),
    )
    .await?;

    let file_resolver = FileResolver::new(workspace.root().clone());

    if !dry_run {
        if changelog_path.exists() && !overwrite.unwrap_or(false) {
            anyhow::bail!(
                "Changelog already exists at `{}`. Remove it first if you want to regenerate it.",
                file_resolver.relative_path(&changelog_path).display()
            );
        }

        fs_err::write(&changelog_path, changelog_content)?;

        writeln!(
            stdout,
            "Changelog generated successfully at `{}`.",
            file_resolver.relative_path(&changelog_path).display()
        )?;
    } else {
        write!(stdout, "{changelog_content}")?;
    }

    Ok(ExitStatus::Success)
}
