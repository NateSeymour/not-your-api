use aws_sdk_dynamodb::model::AttributeValue;

#[rocket::post("/create")]
pub async fn create(db_client: &rocket::State<aws_sdk_dynamodb::Client>) -> Result<(), ()> {
    db_client.put_item()
        .table_name("NYS_tasker")
        .item("TaskID", AttributeValue::N(1.to_string()))
        .item("Penis", AttributeValue::S("Large".into()))
        .send().await;

    Ok(())
}

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![create]
}