use rocket::serde::json::Json;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct CreateTaskRB<'r> {
    description: &'r str,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Task {
    id: String,
    description: String,
    completed: bool,
}

#[derive(serde::Serialize)]
pub struct TaskList {
    tasks: Vec<Task>,
}

#[rocket::post("/task", data = "<task>")]
pub async fn create_task(task: Json<CreateTaskRB<'_>>, db_client: &rocket::State<aws_sdk_dynamodb::Client>) -> Json<Task> {
    let new_task = Task {
        id: Uuid::new_v4().to_string(),
        description: task.description.to_string(),
        completed: false,
    };

    let item = serde_dynamo::to_item(new_task.clone()).unwrap();

    db_client.put_item()
        .table_name("NYS_tasker")
        .set_item(Some(item))
        .send().await;

    Json(new_task)
}

#[rocket::get("/task/all")]
pub async fn get_all_tasks(db_client: &rocket::State<aws_sdk_dynamodb::Client>) -> Result<Json<TaskList>, ()> {
    let result = db_client.scan()
        .table_name("NYS_tasker")
        .send().await.unwrap();

    // And deserialize them as strongly-typed data structures
    if let Some(items) = result.items {
        let tasks: Vec<Task> = serde_dynamo::from_items(items).unwrap();
        println!("Got {} tasks", tasks.len());

        return Ok(Json(TaskList { tasks }));
    }

    return Err(());
}

/*
#[rocket::get("/task/<id>")]
pub async fn get_task(id: &str, db_client: &rocket::State<aws_sdk_dynamodb::Client>) -> Json<Task<'_>> {

}
*/

pub fn routes() -> Vec<rocket::Route> {
    rocket::routes![create_task, get_all_tasks]
}