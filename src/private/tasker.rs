use aws_sdk_dynamodb::model::{AttributeValue, Select};
use rocket::serde::json::Json;
use uuid::Uuid;
use crate::api_response::{ApiEmptyReturnValue, ApiError, ApiResponse, ApiReturnValue};
use crate::db;
use crate::public::iam::AuthenticatableEntity;

#[derive(serde::Deserialize)]
pub struct CreateTaskRB<'r> {
    description: &'r str,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Task {
    owner: String,
    id: String,
    description: String,
    completed: bool,
}

#[derive(serde::Serialize)]
pub struct TaskList {
    tasks: Vec<Task>,
}

#[rocket::post("/<entity>/task", data = "<task>")]
pub async fn create_task(ae: AuthenticatableEntity, task: Json<CreateTaskRB<'_>>, db_client: &rocket::State<aws_sdk_dynamodb::Client>, entity: &str) -> ApiReturnValue<Task> {
    ae.assert_privilege(format!["nys:tasker:{}:TaskList:Write", entity])?;

    let new_task = Task {
        owner: ae.id,
        id: Uuid::new_v4().to_string(),
        description: task.description.to_string(),
        completed: false,
    };

    let item = serde_dynamo::to_item(new_task.clone()).unwrap();

    db_client.put_item()
        .table_name("NYS_tasker")
        .set_item(Some(item))
        .send().await;

    Ok(ApiResponse(Json(new_task)))
}

#[rocket::get("/<entity>/task/all")]
pub async fn get_all_tasks(ae: AuthenticatableEntity, db_client: &rocket::State<aws_sdk_dynamodb::Client>, entity: &str) -> ApiReturnValue<TaskList> {
    ae.assert_privilege(format!["nys:tasker:{}:TaskList:Read", entity])?;

    let query = db_client.query()
        .table_name(db::Table::TASKER.as_str())
        .key_condition_expression("#owner = :owner")
        .expression_attribute_names("#owner", "owner")
        .expression_attribute_values(":owner", AttributeValue::S(ae.id))
        .select(Select::AllAttributes)
        .send().await;

    let query_result = match query {
        Ok(res) => res,
        Err(_) => return Err(ApiError::MeNoLikeyAWS),
    };

    // And deserialize them as strongly-typed data structures
    if let Some(items) = query_result.items {
        let tasks: Vec<Task> = serde_dynamo::from_items(items).unwrap();
        println!("Got {} tasks", tasks.len());

        return Ok(ApiResponse(Json(TaskList { tasks })));
    }

    Err(ApiError::MeNoLikeyAWS)
}

/*
#[rocket::get("/task/<id>")]
pub async fn get_task(id: &str, db_client: &rocket::State<aws_sdk_dynamodb::Client>) -> Json<Task<'_>> {

}
*/

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![create_task, get_all_tasks]
}