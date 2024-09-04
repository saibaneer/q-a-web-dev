use crate::error::{CustomError, InvalidPagination};
use crate::types::pagination::extract_pagination;
use crate::types::question::{Question, QuestionId};
use crate::types::store::Store;
use crate::StatusCode;
use std::collections::HashMap;

pub async fn get_single_question(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.read().await.get(&QuestionId(id)) {
        Some(question) => Ok(warp::reply::json(question)),
        None => return Err(warp::reject::custom(CustomError::QuestionNotFound)),
    }
}

pub async fn delete_question(
    id: String,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    //confirm that the id exists
    match store.questions.write().await.remove(&QuestionId(id)) {
        Some(item) => {
            println!("Removed value: {:?}", &item);
            Ok(warp::reply::with_status("Removed value!", StatusCode::OK))
        }
        None => return Err(warp::reject::custom(CustomError::QuestionNotFound)),
    }
}

pub async fn update_question(
    id: String,
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    match store.questions.write().await.get_mut(&QuestionId(id)) {
        Some(q) => *q = question,
        None => return Err(warp::reject::custom(CustomError::QuestionNotFound)),
    }
    Ok(warp::reply::with_status("Question updated", StatusCode::OK))
}

pub async fn add_question(
    store: Store,
    question: Question,
) -> Result<impl warp::Reply, warp::Rejection> {
    store
        .questions
        .write()
        .await
        .insert(question.id.clone(), question);
    Ok(warp::reply::with_status("Question Added!", StatusCode::OK))
}

pub async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        // let response: Vec<Question> = store.questions.values().cloned().collect();
        let response = store
            .questions
            .read()
            .await
            .values()
            // .into_iter()
            .cloned()
            .collect::<Vec<Question>>();
        // Check if start is greater than end
        if pagination.start > pagination.end || pagination.end > response.len() {
            // Return a custom rejection for invalid pagination
            return Err(warp::reject::custom(InvalidPagination));
        }
        let response = &response[pagination.start..pagination.end];
        Ok(warp::reply::json(&response))
    } else {
        let response = store
            .questions
            .read()
            .await
            .values()
            .into_iter()
            .cloned()
            .collect::<Vec<Question>>();
        Ok(warp::reply::json(&response))
    }
}
