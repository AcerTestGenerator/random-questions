#[macro_use]
extern crate diesel;
extern crate core;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use actix_web::{
    error, get,
    http::{header::ContentType, StatusCode},
    middleware, post, web, App, Error as err, HttpResponse, HttpServer, Responder, Result,
};
use derive_more::{Display, Error};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use std::collections::HashMap;

mod actions;
mod models;
mod schema;

#[derive(Debug, Display, Error)]
enum AcerError {
    #[display(fmt = "Internal Error")]
    InternalError,
    #[display(fmt = "Bad Request")]
    BadClientData,
    #[display(fmt = "Timeout")]
    Timeout,
}

impl error::ResponseError for AcerError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            AcerError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AcerError::BadClientData => StatusCode::BAD_REQUEST,
            AcerError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[derive(Serialize, Clone)]
struct QuestionAnswer {
    question: String,
    answer: String,
}

lazy_static! {
    static ref QUESTION_KV_DATABASE: HashMap<u32, QuestionAnswer> = {
        let db = HashMap::from([
            (
                0,
                QuestionAnswer {
                    question: "What's my name?".to_string(),
                    answer: "Acer".to_string(),
                },
            ),
            (
                1,
                QuestionAnswer {
                    question: "What's Luiz's name?".to_string(),
                    answer: "Luiz".to_string(),
                },
            ),
            (
                2,
                QuestionAnswer {
                    question: "What's Jorge's name?".to_string(),
                    answer: "GOD".to_string(),
                },
            ),
            (
                3,
                QuestionAnswer {
                    question: "Are you sentient?".to_string(),
                    answer: "On Ma I toN".to_string(),
                },
            ),
            (
                4,
                QuestionAnswer {
                    question: "Where do you live?".to_string(),
                    answer: "Stalker!".to_string(),
                },
            ),
            (
                5,
                QuestionAnswer {
                    question: "What's the purpose of life?".to_string(),
                    answer: "FU".to_string(),
                },
            ),
            (
                6,
                QuestionAnswer {
                    question: "How to make a good question?".to_string(),
                    answer: "That's not the way!".to_string(),
                },
            ),
        ]);
        db
    };
}

#[get("/")]
async fn hello_acer() -> impl Responder {
    HttpResponse::Ok().body("Hello Acer!")
}

#[get("/random_questions/{number_of_questions}")]
async fn random_questions(
    number_of_questions: web::Path<usize>,
) -> Result<impl Responder, AcerError> {
    let mut rng = thread_rng();
    let questions_database_size = QUESTION_KV_DATABASE.keys().len();
    let number_of_questions = *number_of_questions;
    if number_of_questions > questions_database_size {
        return Err(AcerError::BadClientData);
    }

    let mut array_of_database_keys: Vec<usize> = (1..questions_database_size).collect();
    let array_of_database_keys_slice = &mut array_of_database_keys[..];
    array_of_database_keys_slice.shuffle(&mut rng);
    let answers_index: Vec<u32> = array_of_database_keys
        .into_iter()
        .take(number_of_questions)
        .map(|i| i as u32)
        .collect();
    let question_answers: Vec<QuestionAnswer> = answers_index
        .clone()
        .into_iter()
        .map(|i| {
            let question_answer = QUESTION_KV_DATABASE.get(&(i as u32)).unwrap();
            question_answer.clone()
        })
        .collect();
    Ok(web::Json(question_answers))
}

type DbPool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

#[post("/add_question")]
async fn add_question(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewRandomQuestion>,
) -> Result<impl Responder, err> {
    let question = web::block(move || {
        let mut conn = pool.get()?;
        actions::insert_new_random_question(&form.question, &form.answer, &mut conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;
    Ok(web::Json(question))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let conn_spec = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let manager = ConnectionManager::<SqliteConnection>::new(conn_spec);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(hello_acer)
            .service(random_questions)
            .service(add_question)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}
