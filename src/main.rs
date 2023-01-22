use program::ProgramBuilder;

mod program;
mod rpc_client;

pub const NEAR_CREDENTIALS_DIR: &str = ".near-credentials";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ProgramBuilder::new()
        .set_network()
        .set_benefit_account()
        .set_master_account()
        .set_working_dir()
        .build()
        .run()
        .await
}
