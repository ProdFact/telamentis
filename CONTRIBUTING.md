# Contributing to Tela Mentis ⚡️

Thank you for your interest in contributing to Tela Mentis! We're excited to build a vibrant community around this project. Whether you're reporting a bug, suggesting a feature, writing documentation, or submitting code, your contributions are valuable.

Please take a moment to review this document to understand how you can contribute effectively.

## 📜 Code of Conduct

All participants are expected to follow our [Code of Conduct](./CODE_OF_CONDUCT.md). Please ensure you are welcoming, inclusive, and professional in all interactions.

## ❓ How Can I Contribute?

There are many ways to contribute to Tela Mentis:

*   **Reporting Bugs**: If you find a bug, please open an issue in our GitHub repository. Include detailed steps to reproduce, expected behavior, and actual behavior.
*   **Suggesting Enhancements**: If you have ideas for new features or improvements, open an issue to discuss them. For significant changes, an RFC (Request for Comments) might be appropriate.
*   **Writing Documentation**: Good documentation is key! If you find areas that are unclear or missing, feel free to submit a PR with improvements.
*   **Submitting Code**: Fixing bugs or implementing new features.

## 🚀 Getting Started with Development

1.  **Fork the Repository**: Create your own fork of the `prodfact/telamentis` repository on GitHub.
2.  **Clone Your Fork**:
    ```bash
    git clone https://github.com/YOUR_USERNAME/telamentis.git
    cd telamentis
    ```
3.  **Set Up Upstream Remote**:
    ```bash
    git remote add upstream https://github.com/prodfact/telamentis.git
    ```
4.  **Create a Branch**: Create a descriptive branch name for your feature or bug fix.
    ```bash
    # For features:
    git checkout -b feat/my-new-feature
    # For bug fixes:
    git checkout -b fix/issue-number-description
    # For documentation:
    git checkout -b docs/update-readme
    ```
5.  **Development Environment**:
    The easiest way to get a full development stack (Tela Mentis core, Neo4j, FastAPI transport) is using Docker Compose:
    ```bash
    make dev-up
    ```
    This command should handle building necessary Docker images and starting the services. Refer to `Makefile` and `docker-compose.yml` for details. For core Rust development, ensure you have a recent Rust toolchain installed.

## ✨ Making Changes

1.  **Write Code**: Make your changes, adhering to the project's coding style and conventions.
2.  **Formatting and Linting**: Before committing, ensure your code is properly formatted and passes linter checks:
    ```bash
    cargo fmt --all
    cargo clippy --all-targets --all-features -- -D warnings
    ```
3.  **Run Tests**: All tests must pass before submitting a PR.
    ```bash
    make test  # Runs all tests, potentially including integration tests
    # Or, for more granular control:
    cargo test --all-features
    ```
4.  **Commit Your Changes**: Follow the commit message convention (see below).
    ```bash
    git add .
    git commit -m "type(scope): concise summary of changes"
    ```
5.  **Keep Your Branch Updated**: Regularly rebase your branch on the latest `main` from `upstream` to avoid conflicts.
    ```bash
    git fetch upstream
    git rebase upstream/main
    ```

### Commit Message Convention

We follow the [Conventional Commits](https://www.conventionalcommits.org/) specification. This helps in automating changelogs and makes commit history more readable.

Format: `type(scope): summary`

*   **type**: `feat` (new feature), `fix` (bug fix), `docs` (documentation), `style` (formatting, linting), `refactor`, `test`, `chore` (build changes, etc.).
*   **scope** (optional): The part of the project affected (e.g., `core`, `neo4j-adapter`, `kgctl`, `llm`).
*   **summary**: A concise description of the change.

Examples:
*   `feat(core): add TimeEdge bitemporal support`
*   `fix(neo4j): correct tenant ID injection in Cypher queries`
*   `docs(temporal): clarify valid_to semantics`
*   `refactor(kgctl): improve error handling for tenant commands`

## ⬆️ Submitting a Pull Request (PR)

1.  **Push Your Branch**:
    ```bash
    git push origin feat/my-new-feature
    ```
2.  **Open a PR**: Go to the `ORG/telamentis` repository on GitHub and open a Pull Request from your branch to the `main` branch.
3.  **Describe Your PR**: Provide a clear title and description for your PR. Explain the "what" and "why" of your changes. If it fixes an issue, link to it (e.g., `Closes #123`).
4.  **Review Process**:
    *   At least one approving review from a maintainer is required for a PR to be merged.
    *   Address any feedback or requested changes from reviewers.
    *   Ensure all CI checks pass.
5.  **Merging**: Once approved and all checks pass, a maintainer will merge your PR.

## 💡 RFC (Request for Comments) Process

For substantial changes, new features, or architectural modifications, we use an RFC process:

1.  **Open an Issue**: Create an issue in the GitHub repository, prefixing the title with `RFC:`.
2.  **Outline the Proposal**: In the issue description, detail the proposed change:
    *   Motivation and problem statement.
    *   Detailed design.
    *   Pros and cons.
    *   Alternatives considered.
    *   Impact on existing functionality.
3.  **Discussion**: The community and maintainers will discuss the proposal in the issue comments.
4.  **Approval**: Once consensus is reached, the RFC will be marked as approved, and implementation can begin.

## ⚙️ Development Workflow Tips

*   **IDE Setup**: For Rust development, VS Code with the `rust-analyzer` extension is highly recommended.
*   **Logging**: Utilize the `tracing` crate for structured logging.
*   **Debugging**: Use `gdb` or `lldb` for debugging Rust code.

Thank you for contributing to Tela Mentis! Your efforts help make this project better. 