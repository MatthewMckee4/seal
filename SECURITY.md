# Security Policy

Seal updates project files, runs Git commands, contacts the GitHub API, and can execute commands
listed in `release.pre-commit-commands`.

Treat an untrusted repository and its `seal.toml` as untrusted code. Running configured commands or
applying configured file changes is intended behavior and is not, by itself, a vulnerability in
Seal.

Report vulnerabilities in Seal itself privately by emailing <matthewmckee04@yahoo.co.uk>. Include
the affected version, a minimal reproduction, and the expected impact.

Security fixes target the latest released version and the `main` branch.
