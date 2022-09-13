extern crate core;
extern crate diesel;

use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};

use actix_cors::Cors;
use actix_web::error::BlockingError;
use actix_web::ResponseError;
use actix_web::{
    error, get, http::StatusCode, middleware, post, web, App, Error as err, HttpResponse,
    HttpServer, Responder, Result,
};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::Serialize;
use std::fmt::{Display, Formatter, Result as FmtResult};

mod actions;
mod models;
mod schema;

#[derive(Debug, Serialize)]
struct AcerError {
    msg: String,
    status: u16,
}

impl Display for AcerError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let err_json = serde_json::to_string(self).unwrap();
        write!(f, "{}", err_json)
    }
}

impl From<BlockingError> for AcerError {
    fn from(e: BlockingError) -> Self {
        let code = e.status_code();
        AcerError {
            msg: e.to_string(),
            status: u16::from(code),
        }
    }
}

impl From<actix_web::Error> for AcerError {
    fn from(e: actix_web::Error) -> Self {
        let code = e.as_response_error().status_code();
        let msg = e.as_response_error().to_string();
        AcerError {
            msg,
            status: u16::from(code),
        }
    }
}

impl ResponseError for AcerError {
    fn error_response(&self) -> HttpResponse {
        let response = HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json(self);

        dbg!(&response);

        response
    }
}

#[derive(Serialize, Clone)]
struct QuestionAnswer {
    question: String,
    answer: String,
}

#[get("/random_questions/{number_of_questions}")]
async fn random_questions(
    number_of_questions: web::Path<usize>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder, AcerError> {
    let mut rng = thread_rng();
    // HACK: so we can have 2 block points
    let pool1 = pool.clone();

    let database_size = web::block(move || {
        let mut conn = pool1.get()?;
        actions::get_questions_database_size(&mut *conn)
    })
    .await?
    .map_err(error::ErrorInternalServerError)?;

    let number_of_questions = *number_of_questions;
    if number_of_questions as i64 > database_size {
        return Err(AcerError {
            msg: "Not enough questions on the database.".to_string(),
            status: 400,
        });
    }

    let mut array_of_database_keys: Vec<i64> = (1..database_size).collect();
    let array_of_database_keys_slice = &mut array_of_database_keys[..];
    array_of_database_keys_slice.shuffle(&mut rng);

    let answers_index: Vec<i32> = array_of_database_keys
        .into_iter()
        .take(number_of_questions)
        .map(|i| i as i32)
        .collect();

    let res = web::block(move || {
        let mut conn = pool.get()?;
        actions::get_questions_randomly(&mut *conn, answers_index)
    })
    .await?
    .map_err(|_e| AcerError {
        msg: "Could not get the questions from the database!".to_string(),
        status: 500,
    })?;

    Ok(web::Json(res))
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
        let cors = Cors::permissive();
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .service(random_questions)
            .service(add_question)
    })
    .bind(("127.0.0.1", 8000))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{
        body::to_bytes,
        dev::{Service, ServiceResponse},
        http::StatusCode,
        test::{self},
        web::Bytes,
    };

    use super::*;

    trait BodyTest {
        fn as_str(&self) -> &str;
    }

    impl BodyTest for Bytes {
        fn as_str(&self) -> &str {
            std::str::from_utf8(self).unwrap()
        }
    }

    #[actix_web::test]
    async fn integration_test_endpoints() {
        std::env::set_var("RUST_LOG", "actix_web=debug");
        env_logger::init();
        dotenv::dotenv().ok();

        let conn = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL");
        let manager = ConnectionManager::<SqliteConnection>::new(conn);
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create connection pool!");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .service(add_question)
                .service(random_questions),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/add_question")
            .set_json(&models::NewRandomQuestion {
                question: "Who is god?".to_owned(),
                answer: "REDOUANE".to_owned(),
            })
            .to_request();
        // this will be fixed after the change to Option<i32> id
        let reponse: models::NewRandomQuestion = test::call_and_read_body_json(&app, req).await;
        assert_eq!(reponse.answer, "REDOUANE");

        let req = test::TestRequest::get()
            .uri(&format!("/random_questions/{}", 6))
            .to_request();
        let resp: ServiceResponse = app.call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(resp.into_body()).await.unwrap();
        assert_eq!(body.as_str(), "Not enough questions on the database!");
    }
}
