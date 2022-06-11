use aws_types::credentials::SharedCredentialsProvider;

mod public;
mod cors;

#[rocket::get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to AWS
    let akid = std::env::var("AWS_ACCESS_ID")?;
    let secret = std::env::var("AWS_SECRET")?;
    let region = std::env::var("AWS_REGION")?;
    let credentials = aws_types::Credentials::new(akid, secret, None, None, "System Environment");
    let config = aws_types::SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .region(aws_types::region::Region::new(region))
        .build();

    let client = aws_sdk_dynamodb::Client::new(&config);

    // Start
    let _rocket = rocket::build()
        .mount("/v1", rocket::routes![index])
        .mount("/v1/public/tasker", public::tasker::routes())
        .attach(cors::CORS)
        .manage(client)
        .launch()
        .await?;

    Ok(())
}

#[cfg(test)]
mod main_tests {
    #[test]
    fn test() {
        assert_eq!(1 + 1, 2);
    }
}