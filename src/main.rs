use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;

const NUMBER_OF_QUESTIONS: usize = 5;

lazy_static! {
    static ref QUESTION_KV_DATABASE: HashMap<u32, (&'static str, &'static str)> = {
        let mut m = HashMap::new();
        m.insert(0, ("What's my name?", "Acer\n"));
        m.insert(1, ("What's Luiz0s name?", "Luiz\n"));
        m.insert(2, ("What's Jorge's name?", "GOD\n"));
        m.insert(3, ("Are you sentient?", "On Ma I toN\n"));
        m.insert(4, ("Where do you live?", "Stalker!\n"));
        m.insert(5, ("What's the purpose of life?", "FU\n"));
        m.insert(6, ("How to make a good question?", "That's not the way!\n"));
        m
    };
}

#[get("/")]
async fn hello_acer() -> impl Responder {
    HttpResponse::Ok().body("Hello Acer!")
}

#[get("/get_random_answers")]
async fn get_random_answers() -> impl Responder {
    let mut rng = thread_rng();
    let questions_database_size = QUESTION_KV_DATABASE.keys().len();
    let mut array_of_database_keys: Vec<usize> = (1..questions_database_size).collect();
    let array_of_database_keys_slice = &mut array_of_database_keys[..];
    array_of_database_keys_slice.shuffle(&mut rng);
    let answers_index: Vec<u32> = array_of_database_keys
        .into_iter()
        .take(NUMBER_OF_QUESTIONS)
        .map(|i| i as u32)
        .collect();
    let answers: String = answers_index
        .clone()
        .into_iter()
        .map(|i| {
            let question_answer = QUESTION_KV_DATABASE.get(&(i as u32)).unwrap();
            let question = question_answer.0;
            let answer = question_answer.1;
            let v = vec![question, answer];
            let f: String = v.join(" - ");
            f
        })
        .collect::<String>();
    HttpResponse::Ok().body(answers)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello_acer).service(get_random_answers))
        .bind(("127.0.0.1", 8000))?
        .run()
        .await
}
