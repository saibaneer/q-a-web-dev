use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::{
    f32::consts::E,
    fmt::{self},
    io::ErrorKind,
    num::ParseIntError,
    str::FromStr,
};
use warp::{
    cors::CorsForbidden,
    http::{Method, StatusCode},
    reject::{Reject, Rejection},
    Filter, Reply,
};

// #[derive(Debug)]
// struct InvalidId;
// impl Reject for InvalidId {}

#[derive(Debug)]
struct Pagination {
    start: usize,
    end: usize,
}

fn extract_pagination(params: HashMap<String, String>) -> Result<Pagination, Error> {
    if params.contains_key("start") && params.contains_key("end") {
        return Ok(Pagination {
            start: params
                .get("start")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
            end: params
                .get("end")
                .unwrap()
                .parse::<usize>()
                .map_err(Error::ParseError)?,
        });
    }
    Err(Error::MissingParameters)
}

#[derive(Debug)]
enum Error {
    ParseError(ParseIntError),
    MissingParameters,
}
impl Reject for Error {}

#[derive(Debug)]
struct InvalidPagination;
impl Reject for InvalidPagination {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing paramters"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

// impl Question {
//     fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
//         Question {
//             id,
//             title,
//             content,
//             tags,
//         }
//     }
// }

// impl fmt::Display for Question {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{}, title: {}, content: {}, tags: {:?}",
//             self.id, self.title, self.content, self.tags
//         )
//     }
// }

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
struct QuestionId(String);

// impl fmt::Display for QuestionId {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "id: {}", self.0)
//     }
// }

// impl FromStr for QuestionId {
//     type Err = std::io::Error;
//     fn from_str(id: &str) -> Result<Self, Self::Err> {
//         match id.is_empty() {
//             true => Err(Error::new(ErrorKind::InvalidInput, "No id provided")),
//             false => Ok(QuestionId(id.to_string())),
//         }
//     }
// }

impl From<u64> for QuestionId {
    fn from(value: u64) -> Self {
        QuestionId(value.to_string())
    }
}

#[derive(Clone)]
struct Store {
    questions: HashMap<QuestionId, Question>,
}

impl Store {
    fn new() -> Self {
        Store {
            questions: Self::init(),
        }
    }

    fn add_question(&mut self, new_question: Question) {
        self.questions.insert(new_question.id.clone(), new_question);
    }

    fn init() -> HashMap<QuestionId, Question> {
        let file = include_str!("../questions.json");
        serde_json::from_str(file).expect("can't read questions.json")
    }
}

async fn get_questions(
    params: HashMap<String, String>,
    store: Store,
) -> Result<impl warp::Reply, warp::Rejection> {
    if !params.is_empty() {
        let pagination = extract_pagination(params)?;
        // let response: Vec<Question> = store.questions.values().cloned().collect();
        let response = store
            .questions
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
            .values()
            .into_iter()
            .cloned()
            .collect::<Vec<Question>>();
        Ok(warp::reply::json(&response))
    }
}

async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<Error>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if r.find::<InvalidPagination>().is_some() {
        Ok(warp::reply::with_status(
            "Invalid pagination range: start cannot be greater than end or out of bounds"
                .to_string(),
            StatusCode::BAD_REQUEST,
        ))
    } else if r.find::<CorsForbidden>().is_some() {
        Ok(warp::reply::with_status(
            "CORS request forbidden".to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}

#[tokio::main]
async fn main() {
    // let cors = warp::cors()
    //     .allow_any_origin()
    //     .allow_header("content-type")
    //     .allow_methods(&[
    //         Method::PUT,
    //         Method::PATCH,
    //         Method::DELETE,
    //         Method::POST,
    //         Method::GET,
    //     ]);

    let mut store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let get_items = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter)
        .and_then(get_questions)
        .recover(return_error);
    let routes = get_items;
    // get_items.with(cors);

    // println!("Question is: {:#?}", question)
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
