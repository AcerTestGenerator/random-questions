use crate::schema::random_questions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct RandomQuestion {
    pub id: i32,
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Insertable)]
#[diesel(table_name = random_questions)]
pub struct NewRandomQuestion {
    pub question: String,
    pub answer: String,
}
impl fmt::Display for NewRandomQuestion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.question, self.answer)
    }
}
