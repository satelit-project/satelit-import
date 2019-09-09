use chrono::{Date, DateTime, Duration, NaiveTime, TimeZone, Timelike, Utc};

use crate::db::entity::UpdatedSchedule;
use crate::proto::data::anime::Type as AnimeType;
use crate::proto::data::episode::Type as EpisodeType;
use crate::proto::data::Anime;
use std::ops::Deref;

pub trait Strategy {
    fn accepts(&self, anime: &Anime) -> bool;
    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>>;
}

pub struct UpdateBuilder<'a, S> {
    anime: &'a Anime,
    strategy: S,
}

pub struct UnairedStrategy(State);

pub struct AiringStrategy(State);

pub struct JustAiredStrategy(State);

pub struct AiredStrategy(State);

pub struct NeverStrategy;

struct State {
    interval: Duration,
    now: Date<Utc>,
}

pub fn make_update(anime: &Anime) -> UpdatedSchedule {
    let strategies: Vec<Box<dyn Strategy>> = vec![
        Box::new(UnairedStrategy::new()),
        Box::new(AiringStrategy::new()),
        Box::new(JustAiredStrategy::new()),
        Box::new(AiredStrategy::new()),
    ];

    for strategy in strategies {
        if strategy.accepts(anime) {
            return UpdateBuilder::new(anime, strategy).build();
        }
    }

    // TODO: log as error
    UpdateBuilder::new(anime, NeverStrategy).build()
}

// MARK: impl UpdateBuilder

impl<'a, S> UpdateBuilder<'a, S>
where
    S: Strategy,
{
    pub fn new(anime: &'a Anime, strategy: S) -> Self {
        UpdateBuilder { anime, strategy }
    }

    fn next_update_datetime(&self) -> Option<DateTime<Utc>> {
        match self.strategy.next_update_date(self.anime) {
            Some(date) => {
                let mut time = Utc::now().time();
                if time.hour() == 23 {
                    time = NaiveTime::from_hms(22, 59, 0);
                }

                // if scheduled for today add one hour delay
                time += Duration::hours(1);
                Some(date.and_hms(time.hour(), time.minute(), time.second()))
            }
            None => None,
        }
    }

    fn has_type(&self) -> bool {
        let anime_type = AnimeType::from_i32(self.anime.r#type).unwrap_or(AnimeType::Unknown);
        anime_type != AnimeType::Unknown
    }

    fn has_anidb_id(&self) -> bool {
        self.anime
            .source
            .as_ref()
            .map_or(false, |s| !s.anidb_ids.is_empty())
    }

    fn has_mal_id(&self) -> bool {
        self.anime
            .source
            .as_ref()
            .map_or(false, |s| !s.mal_ids.is_empty())
    }

    fn has_ann_id(&self) -> bool {
        self.anime
            .source
            .as_ref()
            .map_or(false, |s| !s.ann_ids.is_empty())
    }

    fn has_all_eps(&self) -> bool {
        let unknown_eps_count = self
            .anime
            .episodes
            .iter()
            .filter(|&e| {
                let ep_type = EpisodeType::from_i32(e.r#type).unwrap_or(EpisodeType::Unknown);
                ep_type == EpisodeType::Unknown
                    || e.air_date == 0
                    || e.duration == 0.0
                    || e.name.is_empty()
            })
            .count();

        unknown_eps_count == 0 && !self.anime.episodes.is_empty()
    }

    fn src_created_at(&self) -> Option<DateTime<Utc>> {
        if self.anime.src_created_at == 0 {
            return None;
        }

        Some(Utc.timestamp(self.anime.src_created_at, 0))
    }

    fn src_updated_at(&self) -> Option<DateTime<Utc>> {
        if self.anime.src_updated_at == 0 {
            return None;
        }

        Some(Utc.timestamp(self.anime.src_updated_at, 0))
    }
}

impl<'a, S> UpdateBuilder<'a, S>
where
    S: Strategy,
{
    pub fn build(&self) -> UpdatedSchedule {
        let mut schedule = UpdatedSchedule::default();
        let anime = self.anime;

        schedule.next_update_at = self.next_update_datetime();
        schedule.has_poster = !anime.poster_url.is_empty();
        schedule.has_start_air_date = anime.start_date != 0;
        schedule.has_end_air_date = anime.end_date != 0;
        schedule.has_type = self.has_type();
        schedule.has_anidb_id = self.has_anidb_id();
        schedule.has_mal_id = self.has_mal_id();
        schedule.has_ann_id = self.has_ann_id();
        schedule.has_tags = !anime.tags.is_empty();
        schedule.has_ep_count = anime.episodes_count != 0;
        schedule.has_all_eps = self.has_all_eps();
        schedule.has_rating = anime.rating != 0.0;
        schedule.has_description = !anime.description.is_empty();
        schedule.src_created_at = self.src_created_at();
        schedule.src_updated_at = self.src_updated_at();

        schedule
    }
}

// MARK: impl UnairedStrategy

impl UnairedStrategy {
    pub fn new() -> Self {
        Self(State {
            interval: Duration::days(5),
            now: Utc::now().date(),
        })
    }
}

impl Strategy for UnairedStrategy {
    fn accepts(&self, anime: &Anime) -> bool {
        // if start date is unknown
        if anime.start_date == 0 {
            return true;
        }

        let start_date = Utc.timestamp(anime.start_date, 0).date();

        // if start date in the future
        if self.0.now < start_date {
            return true;
        }

        // otherwise
        false
    }

    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>> {
        debug_assert!(self.accepts(anime));

        if anime.start_date == 0 {
            return Some(self.0.now + self.0.interval);
        }

        let start_date = Utc.timestamp(anime.start_date, 0).date();
        let diff = start_date - self.0.now;

        // if we too close to airing date just return it
        if diff <= self.0.interval {
            return Some(start_date);
        }

        // find update date relative to start_date
        let util_update = diff.num_days() % self.0.interval.num_days();
        Some(self.0.now + Duration::days(util_update))
    }
}

// MARK: impl AiringStrategy

impl AiringStrategy {
    pub fn new() -> Self {
        Self(State {
            interval: Duration::weeks(1),
            now: Utc::now().date(),
        })
    }

    fn schedule_today(&self, anime: &Anime) -> bool {
        let start_date = Utc.timestamp(anime.start_date, 0).date();
        if start_date <= self.0.now {
            return true;
        }

        let diff = self.0.now - start_date;
        let expected_ep_count = diff.num_weeks() + 1;
        let ep_count = anime.episodes.len() as i64;
        let new_ep_today = (diff.num_days() % self.0.interval.num_days()) == 0;

        // if new episode is today and we don't have it yet
        ep_count < expected_ep_count && new_ep_today
    }

    fn every_week_from_start(&self, anime: &Anime) -> Date<Utc> {
        let start_date = Utc.timestamp(anime.start_date, 0).date();
        let diff = self.0.now - start_date;
        let elapsed_for_week = diff.num_days() % self.0.interval.num_days();
        let until_update = self.0.interval.num_days() - elapsed_for_week;
        self.0.now + Duration::days(until_update)
    }

    fn every_week_before_end(&self, anime: &Anime) -> Date<Utc> {
        let end_date = Utc.timestamp(anime.end_date, 0).date();
        let diff = end_date - self.0.now;

        // if we too close to end air date
        if diff < self.0.interval {
            return end_date;
        }

        let until_update = diff.num_days() % self.0.interval.num_days();
        self.0.now + Duration::days(until_update)
    }
}

impl Strategy for AiringStrategy {
    fn accepts(&self, anime: &Anime) -> bool {
        if anime.start_date == 0 {
            return false;
        }

        let start_date = Utc.timestamp(anime.start_date, 0).date();
        // if start air date is in future
        if self.0.now < start_date {
            return false;
        }

        // if end air date is unknown
        if anime.end_date == 0 {
            return true;
        }

        // if not yet finished airing
        let end_date = Utc.timestamp(anime.end_date, 0).date();
        end_date >= self.0.now
    }

    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>> {
        debug_assert!(self.accepts(anime));

        if self.schedule_today(anime) {
            return Some(self.0.now);
        }

        // if end air date is unknown schedule for every week
        if anime.end_date == 0 {
            return Some(self.every_week_from_start(anime));
        }

        Some(self.every_week_before_end(anime))
    }
}

// MARK: impl JustAiredStrategy

impl JustAiredStrategy {
    pub fn new() -> Self {
        Self(State {
            interval: Duration::days(10),
            now: Utc::now().date(),
        })
    }
}

impl Strategy for JustAiredStrategy {
    fn accepts(&self, anime: &Anime) -> bool {
        // if end air date is unknown
        if anime.end_date == 0 {
            return false;
        }

        // if it's still airing
        let end_date = Utc.timestamp(anime.end_date, 0).date();
        if self.0.now <= end_date {
            return false;
        }

        // if finished airing less than three months ago
        let diff = self.0.now - end_date;
        diff.num_weeks() < 3 * 4
    }

    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>> {
        debug_assert!(self.accepts(anime));

        let end_date = Utc.timestamp(anime.end_date, 0).date();
        let diff = self.0.now - end_date;
        let elapsed_for_interval = diff.num_days() % self.0.interval.num_days();
        let until_update = self.0.interval.num_days() - elapsed_for_interval;
        Some(self.0.now + Duration::days(until_update))
    }
}

// MARK: impl AiredStrategy

impl AiredStrategy {
    pub fn new() -> Self {
        Self(State {
            interval: Duration::zero(),
            now: Utc::now().date(),
        })
    }
}

impl Strategy for AiredStrategy {
    fn accepts(&self, anime: &Anime) -> bool {
        // if end air date is unknown
        if anime.end_date == 0 {
            return false;
        }

        let end_date = Utc.timestamp(anime.end_date, 0).date();

        // if still airing
        if self.0.now <= end_date {
            return false;
        }

        // if aired 3 or more months ago
        let diff = end_date - self.0.now;
        diff.num_weeks() >= 3 * 4
    }

    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>> {
        debug_assert!(self.accepts(anime));
        None
    }
}

// MARK: impl NeverStrategy

impl Strategy for NeverStrategy {
    fn accepts(&self, _anime: &Anime) -> bool {
        true
    }

    fn next_update_date(&self, _anime: &Anime) -> Option<Date<Utc>> {
        // TODO: log as error
        None
    }
}

// MARK: impl Strategy

impl Strategy for Box<dyn Strategy> {
    fn accepts(&self, anime: &Anime) -> bool {
        self.deref().accepts(anime)
    }

    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>> {
        self.deref().next_update_date(anime)
    }
}
