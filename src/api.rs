pub mod scrape;

impl actix_web::ResponseError for crate::db::QueryError {}
