use crate::schema::random_questions;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable)]
pub struct RandomQuestion {
    pub id: i32,
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = random_questions)]
pub struct NewRandomQuestion {
    pub question: String,
    pub answer: String,
}
