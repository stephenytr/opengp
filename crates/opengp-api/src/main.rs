use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    opengp_api::run().await
}
