mod error;
mod routes;
mod types;

use crate::error::return_error;
use routes::{
    answer::add_answer,
    question::{
        add_question, delete_question, get_questions, get_single_question, update_question,
    },
};

use types::store::Store;
use warp::{
    http::{Method, StatusCode},
    Filter,
};

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

    let mut store = Store::new();
    let store_filter = warp::any().map(move || store.clone());
    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(update_question);

    let remove_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(delete_question);

    let get_single_question = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(get_single_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_answer);

    let routes = get_questions
        .or(add_question)
        .or(update_question)
        .or(remove_question)
        .or(get_single_question)
        .or(add_answer)
        .with(cors)
        .recover(return_error);
    // get_items.with(cors);

    // println!("Question is: {:#?}", question)
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
