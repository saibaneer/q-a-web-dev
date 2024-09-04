// use warp::reject::Reject;

use std::{
    fmt::{self},
    num::ParseIntError,
};
use warp::{
    filters::{body::BodyDeserializeError, cors::CorsForbidden},
    http::StatusCode,
    reject::{Reject, Rejection},
    Reply,
};

#[derive(Debug)]
pub enum CustomError {
    ParseError(ParseIntError),
    MissingParameters,
    QuestionNotFound,
}
impl Reject for CustomError {}
impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CustomError::ParseError(ref err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            CustomError::MissingParameters => write!(f, "Missing paramters"),
            CustomError::QuestionNotFound => write!(f, "Question not found"),
        }
    }
}

#[derive(Debug)]
pub struct InvalidPagination;
impl Reject for InvalidPagination {}

pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    if let Some(error) = r.find::<CustomError>() {
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
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else {
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND,
        ))
    }
}
