use serde::{Deserialize, Serialize};
use std::{
    f32::consts::E,
    fmt::{self},
    io::{Error, ErrorKind},
    str::FromStr,
};
use warp::{
    cors::CorsForbidden,
    http::{Method, StatusCode},
    reject::{Reject, Rejection},
    Filter, Reply,
};

#[derive(Debug)]
struct InvalidId;
impl Reject for InvalidId {}

#[derive(Debug, Serialize, Deserialize)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

impl fmt::Display for Question {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}, title: {}, content: {}, tags: {:?}",
            self.id, self.title, self.content, self.tags
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct QuestionId(String);

impl fmt::Display for QuestionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}", self.0)
    }
}

impl FromStr for QuestionId {
    type Err = std::io::Error;
    fn from_str(id: &str) -> Result<Self, Self::Err> {
        match id.is_empty() {
            true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
            false => Ok(QuestionId(id.to_string())),
        }
    }
}

impl From<u64> for QuestionId {
    fn from(value: u64) -> Self {
        QuestionId(value.to_string())
    }
}

async fn get_questions() -> Result<impl warp::Reply, warp::Rejection> {
    let question = Question::new(
        QuestionId::from(23),
        "When?".to_string(),
        "When does rest come?".to_string(),
        Some(vec!["faq".to_string()]),
    );

    match question.id.0.parse::<u64>() {
        Ok(_) => Ok(warp::reply::json(&question)),
        Err(_) => Err(warp::reject::custom(InvalidId)),
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(_invalid_id) = r.find::<InvalidId>() {
        Ok(warp::reply::with_status(
            "No valid ID presented",
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if r.find::<CorsForbidden>().is_some() {
        Ok(warp::reply::with_status(
            "CORS request forbidden",
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found",
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::POST,
            Method::GET,
        ]);

    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and_then(get_questions)
        .recover(return_error);
    let routes = get_items.with(cors);

    // println!("Question is: {:#?}", question)
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
