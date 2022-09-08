// @generated automatically by Diesel CLI.

diesel::table! {
    random_questions (id) {
        id -> Integer,
        question -> Text,
        answer -> Text,
    }
}
