pub mod scraper;

impl actix_web::ResponseError for crate::db::QueryError {}
