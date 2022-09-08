use diesel::prelude::*;

use crate::models;

type DbError = Box<dyn std::error::Error + Send + Sync>;

pub fn insert_new_random_question(
    question_: &str,
    answer_: &str,
    conn: &mut SqliteConnection,
) -> Result<models::NewRandomQuestion, DbError> {
    use crate::schema::random_questions::dsl::*;

    let new_random_question = models::NewRandomQuestion {
        question: question_.to_owned(),
        answer: answer_.to_owned(),
    };
    diesel::insert_into(random_questions)
        .values(&new_random_question)
        .execute(conn)?;
    Ok(new_random_question)
}
