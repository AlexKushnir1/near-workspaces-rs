
mod rpc;
mod runtime;

pub use runtime::SandboxRuntime;
pub use runner_macros::test;

#[cfg(not(test))] // Work around for rust-lang/rust#62127
pub use runner_macros::main;

pub use rpc::api::*;


#[cfg(test)]
mod tests {
    use super::rpc::api::*;

    #[runner_macros::test(sandbox)]
    async fn test_1() {
        run_().await;
    }
}
