#![forbid(unsafe_code)]

#[tokio::main]
async fn main() -> impl std::process::Termination {
    aoc::main().await
}
