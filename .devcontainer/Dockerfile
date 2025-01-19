FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Install cargo tools as the vscode user to prevent file permissions errors
USER vscode

RUN cargo install sqlx-cli && cargo install cargo-shuttle