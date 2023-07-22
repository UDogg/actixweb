use crate::models::task::Task;
use crate::models::task::TaskState;
use crate::repository::ddb::DDBRepository;
use actix_web::{
    get, post, put, error::ResponseError, web,
    web::{Data, Json, Path, ServiceConfig},
    HttpResponse, HttpResponseBuilder, http::header::CONTENT_TYPE, StatusCode,
};
use serde::{Deserialize, Serialize};
use derive_more::{Display, Error};

#[derive(Deserialize, Serialize)]
pub struct TaskIdentifier {
    pub task_global_id: String,
}

#[derive(Deserialize)]
pub struct TaskCompletionRequest {
    result_file: String,
}

#[derive(Deserialize)]
pub struct SubmitTaskRequest {
    user_id: String,
    task_type: String,
    source_file: String,
    title: String,
    description: String,
}

#[derive(Debug, Display, Error)]
pub enum TaskError {
    TaskNotFound,
    TaskUpdateFailure,
    TaskCreationFailure,
    BadTaskRequest,
    TaskAlreadyCompleted,
}

impl ResponseError for TaskError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .header(CONTENT_TYPE, "application/json")
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            TaskError::TaskNotFound => StatusCode::NOT_FOUND,
            TaskError::TaskUpdateFailure => StatusCode::INTERNAL_SERVER_ERROR,
            TaskError::TaskCreationFailure => StatusCode::INTERNAL_SERVER_ERROR,
            TaskError::BadTaskRequest => StatusCode::BAD_REQUEST,
            TaskError::TaskAlreadyCompleted => StatusCode::BAD_REQUEST,
        }
    }
}

#[get("/task/{task_global_id}")]
pub async fn get_task(
    task_identifier: Path<TaskIdentifier>,
    ddb_repo: Data<DDBRepository>,
) -> Result<Json<Task>, TaskError> {
    let task = ddb_repo.get_task(task_identifier.task_global_id.clone()).await;

    match task {
        Some(task) => Ok(Json(task)),
        None => Err(TaskError::TaskNotFound),
    }
}

#[post("/task")]
pub async fn submit_task(
    ddb_repo: Data<DDBRepository>,
    request: Json<SubmitTaskRequest>
) -> Result<Json<TaskIdentifier>, TaskError> {
    let task = Task::new (
        request.user_id.clone(),
        request.task_type.clone(),
        request.source_file.clone(),
        request.title.clone(),
        request.description.clone(),
    );

    let task_identifier = task.get_global_id();
    match ddb_repo.put_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier { task_global_id: task_identifier })),
        Err(_) => Err(TaskError::TaskCreationFailure)
    }
}

async fn state_transition(
    ddb_repo: Data<DDBRepository>,
    task_global_id: String,
    new_state: TaskState,
    result_file: Option<String>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    let mut task = match ddb_repo.get_task(task_global_id.clone()).await {
        Some(task) => task,
        None => return Err(TaskError::TaskNotFound),
    };

    if task.state == TaskState::Completed {
        return Err(TaskError::TaskAlreadyCompleted);
    }

    if !task.can_transition_to(&new_state) {
        return Err(TaskError::BadTaskRequest);
    }

    task.state = new_state;
    task.result_file = result_file;

    let task_identifier: String = task.get_global_id();
    match ddb_repo.update_task(task).await {
        Ok(()) => Ok(Json(TaskIdentifier {
            task_global_id: task_identifier,
        })),
        Err(_) => Err(TaskError::TaskUpdateFailure),
    }
}

#[put("/task/{task_global_id}/start")]
pub async fn start_task(
    task_identifier: Path<TaskIdentifier>,
    ddb_repo: Data<DDBRepository>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo,
        task_identifier.task_global_id.clone(),
        TaskState::InProgress,
        None,
    )
    .await
}

#[put("/task/{task_global_id}/pause")]
pub async fn pause_task(
    ddb_repo: Data<DDBRepository>, 
    task_identifier: Path<TaskIdentifier>
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo, 
        task_identifier.into_inner().task_global_id, 
        TaskState::Paused, 
        None
    ).await
}

#[put("/task/{task_global_id}/fail")]
pub async fn fail_task(
    ddb_repo: Data<DDBRepository>, 
    task_identifier: Path<TaskIdentifier>
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo, 
        task_identifier.into_inner().task_global_id, 
        TaskState::Failed, 
        None
    ).await
}

#[put("/task/{task_global_id}/complete")]
pub async fn complete_task(
    task_completion_request: Json<TaskCompletionRequest>,
    task_identifier: Path<TaskIdentifier>,
    ddb_repo: Data<DDBRepository>,
) -> Result<Json<TaskIdentifier>, TaskError> {
    state_transition(
        ddb_repo,
        task_identifier.task_global_id.clone(),
        TaskState::Completed,
        Some(task_completion_request.result_file.clone()),
    )
    .await
}
