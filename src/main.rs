use aws_types::credentials::SharedCredentialsProvider;

mod public;
mod private;
mod cors;
mod db;
mod api_response;

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
    let ddb_config = aws_types::SdkConfig::builder()
        .credentials_provider(SharedCredentialsProvider::new(credentials))
        .region(aws_types::region::Region::new(region))
        .build();

    let ddb_client = aws_sdk_dynamodb::Client::new(&ddb_config);

    // Connect to REDIS
    let redis_url = std::env::var("REDIS")?;
    let redis_client = redis::Client::open(redis_url)?;

    // Start
    let _rocket = rocket::build()
        .mount("/v1", rocket::routes![index])
        .mount("/v1/private/tasker", private::tasker::routes())
        .mount("/v1/private/debug", private::debug::routes())
        .mount("/v1/public/iam", public::iam::routes())
        .attach(cors::CORS)
        .manage(ddb_client)
        .manage(redis_client)
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