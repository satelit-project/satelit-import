use crate::import;
use crate::anidb;
use crate::db::{schedules, Table, ConnectionPool, QueryError, entity::NewSchedule};

struct DumpImport<P: ConnectionPool> {
    schedules: schedules::Schedules<P>,
}

impl<P: ConnectionPool> import::ImportScheduler for DumpImport<P> {
    type Error = QueryError;

    fn add_title(&mut self, anime: &anidb::Anime) -> Result<(), Self::Error> {
        use crate::schedules_insert;

        let schedule = NewSchedule::new(anime.id);
        schedules_insert!(self.schedules, &schedule)
    }

    fn remove_title(&mut self, anime: &anidb::Anime) -> Result<(), Self::Error> {
        use crate::schedules_delete;

        schedules_delete!(self.schedules, anidb_id(anime.id))
    }
}
