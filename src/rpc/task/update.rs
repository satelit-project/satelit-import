use chrono::{Date, DateTime, Duration, NaiveTime, TimeZone, Timelike, Utc};

use crate::db::entity::UpdatedSchedule;
use crate::proto::data::anime::Type as AnimeType;
use crate::proto::data::episode::Type as EpisodeType;
use crate::proto::data::Anime;
use std::ops::Deref;
use std::cmp::min;

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
        if !self.accepts(anime) {
            return None;
        }

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
        let mut until_update = diff.num_days() % self.0.interval.num_days();
        if until_update == 0 {
            until_update = self.0.interval.num_days();
        }

        Some(self.0.now + Duration::days(until_update))
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

    fn schedule_asap(&self, anime: &Anime) -> bool {
        let start_date = Utc.timestamp(anime.start_date, 0).date();
        let end_date = Utc.timestamp(anime.end_date, 0).date();

        if self.0.now <= start_date || (self.0.now >= end_date && anime.end_date != 0) {
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
        let days = self.0.interval.num_days();

        let diff = self.0.now - start_date;
        let elapsed_for_week = diff.num_days() % days;
        let mut until_update = days - elapsed_for_week;

        if until_update == 0 {
            until_update = days;
        }

        self.0.now + Duration::days(until_update)
    }

    fn every_week_before_end(&self, anime: &Anime) -> Date<Utc> {
        let end_date = Utc.timestamp(anime.end_date, 0).date();
        let diff = end_date - self.0.now;
        let days = self.0.interval.num_days();

        // if we too close to end air date
        if diff < self.0.interval {
            return end_date;
        }

        let mut until_update = diff.num_days() % days;
        if until_update == 0 {
            until_update = days;
        }

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
        if !self.accepts(anime) {
            return None;
        }

        if self.schedule_asap(anime) {
            return Some(self.0.now + Duration::days(1));
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
    const WEEKS_AFTER_AIRING: i64 = 3 * 4;

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
        diff.num_weeks() < Self::WEEKS_AFTER_AIRING
    }

    fn next_update_date(&self, anime: &Anime) -> Option<Date<Utc>> {
        if !self.accepts(anime) {
            return None;
        }

        let end_date = Utc.timestamp(anime.end_date, 0).date();
        let diff = dbg!(self.0.now - end_date);
        let elapsed_for_interval = diff.num_days() % self.0.interval.num_days();
        let until_update = self.0.interval.num_days() - elapsed_for_interval;

        let proposed = self.0.now + Duration::days(until_update);
        let latest_date = end_date + Duration::weeks(Self::WEEKS_AFTER_AIRING);
        Some(min(proposed, latest_date))
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
        let diff = self.0.now - end_date;
        diff.num_weeks() >= JustAiredStrategy::WEEKS_AFTER_AIRING
    }

    fn next_update_date(&self, _anime: &Anime) -> Option<Date<Utc>> {
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

#[cfg(test)]
mod tests_strategies {
    use super::*;
    use crate::proto::data::Episode;

    #[test]
    fn test_unaired_accepts() {
        let strategy = UnairedStrategy::new();
        let mut anime = Anime::default();
        anime.end_date = Utc::now().timestamp();

        // no start date
        assert!(strategy.accepts(&anime));

        // start date is in future
        anime.start_date = (Utc::now() + Duration::days(1)).timestamp();
        assert!(strategy.accepts(&anime));

        // start date is today
        anime.start_date = Utc::now().timestamp();
        assert!(!strategy.accepts(&anime));

        // start date is in past
        anime.start_date = (Utc::now() - Duration::days(1)).timestamp();
        assert!(!strategy.accepts(&anime));
    }

    #[test]
    fn test_unaired() {
        let strategy = UnairedStrategy::new();
        let mut anime = Anime::default();
        anime.end_date = Utc::now().timestamp();

        // no start date
        assert_eq!(strategy.next_update_date(&anime), Some(strategy.0.now + strategy.0.interval));

        // very soon, before next update interval
        let start_date = Utc::now() + strategy.0.interval / 2;
        anime.start_date = start_date.timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(start_date.date()));

        // start date aligned with update date
        let expected = Utc::now() + strategy.0.interval;
        let start_date = Utc::now() + strategy.0.interval * 2;
        anime.start_date = start_date.timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected.date()));

        // start date is in future
        let offset = Duration::days(2);
        let start_date = Utc::now() + strategy.0.interval + offset;
        anime.start_date = start_date.timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(strategy.0.now + offset));
    }

    #[test]
    fn test_airing_accepts() {
        let strategy = AiringStrategy::new();
        let mut anime = Anime::default();

        // no start date
        assert!(!strategy.accepts(&anime));

        // start date in future
        anime.start_date = (Utc::now() + Duration::days(1)).timestamp();
        assert!(!strategy.accepts(&anime));

        // started and finished today
        anime.start_date = Utc::now().timestamp();
        anime.end_date = anime.start_date;
        assert!(strategy.accepts(&anime));

        // started in past without finish date
        anime.start_date = (Utc::now() - Duration::days(1)).timestamp();
        anime.end_date = 0;
        assert!(strategy.accepts(&anime));

        // started in past and will finish in future
        anime.start_date = (Utc::now() - Duration::days(1)).timestamp();
        anime.end_date = (Utc::now() + Duration::days(1)).timestamp();
        assert!(strategy.accepts(&anime));

        // started and already finished
        anime.start_date = (Utc::now() - Duration::days(2)).timestamp();
        anime.end_date = (Utc::now() - Duration::days(1)).timestamp();
        assert!(!strategy.accepts(&anime));
    }

    #[test]
    fn test_airing_asap() {
        let strategy = AiringStrategy::new();
        let mut anime = Anime::default();
        let tomorrow = (Utc::now() + Duration::days(1)).date();

        // start date is today and end date is in future
        anime.start_date = Utc::now().timestamp();
        anime.end_date = (Utc::now() + Duration::days(2)).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(tomorrow));

        // start date is in past and end date is today
        anime.start_date = (Utc::now() - Duration::days(2)).timestamp();
        anime.end_date = Utc::now().timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(tomorrow));

        // episode is missing but wasn't airing today
        anime.start_date = (Utc::now() - Duration::days(1)).timestamp();
        anime.end_date = (Utc::now() + Duration::weeks(2)).timestamp();
        assert_ne!(strategy.next_update_date(&anime), Some(tomorrow));
        anime.episodes.push(Episode::default());

        // episode is missing and is airing today
        anime.start_date = (Utc::now() - Duration::weeks(1)).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(tomorrow));
        anime.episodes.push(Episode::default());

        anime.start_date = (Utc::now() - Duration::weeks(2)).timestamp();
        anime.end_date = (Utc::now() + Duration::weeks(2)).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(tomorrow));
        anime.episodes.push(Episode::default());

        // no missing episodes
        assert_ne!(strategy.next_update_date(&anime), Some(tomorrow));

        anime.start_date = (Utc::now() - Duration::days(1)).timestamp();
        anime.episodes.clear();
        anime.episodes.push(Episode::default());
        assert_ne!(strategy.next_update_date(&anime), Some(tomorrow));
    }

    #[test]
    fn test_airing_no_end() {
        let strategy = AiringStrategy::new();
        let mut anime = Anime::default();
        anime.episodes = vec![Episode::default(); 24];

        // started in past aligned to start date
        anime.start_date = (Utc::now() - strategy.0.interval).timestamp();
        let expected = Utc::now() + strategy.0.interval;
        assert_eq!(strategy.next_update_date(&anime), Some(expected.date()));

        let offset = Duration::days(1);
        let expected = Utc::now().date() + strategy.0.interval - offset;

        anime.start_date = (Utc::now() - offset).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));
        assert_ne!(strategy.next_update_date(&anime), Some(Utc::now().date()));

        anime.start_date = (Utc::now() - strategy.0.interval * 2 - offset).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));
        assert_ne!(strategy.next_update_date(&anime), Some(Utc::now().date()));
    }

    #[test]
    fn test_airing_has_end() {
        let strategy = AiringStrategy::new();
        let mut anime = Anime::default();
        anime.episodes = vec![Episode::default(); 24];
        anime.start_date = (Utc::now() - strategy.0.interval).timestamp();

        // started in past and is aligned to end date
        anime.end_date = (Utc::now() + strategy.0.interval).timestamp();
        let expected = Utc.timestamp(anime.end_date, 0).date();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));

        anime.end_date = (Utc::now() + strategy.0.interval * 2).timestamp();
        let expected = Utc::now() + strategy.0.interval;
        assert_eq!(strategy.next_update_date(&anime), Some(expected.date()));

        anime.end_date = (Utc::now() + strategy.0.interval / 2).timestamp();
        let expected = Utc.timestamp(anime.end_date, 0).date();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));

        anime.end_date = (Utc::now() + strategy.0.interval + strategy.0.interval / 2).timestamp();
        let expected = Utc::now() + strategy.0.interval / 2;
        assert_eq!(strategy.next_update_date(&anime), Some(expected.date()));
    }

    #[test]
    fn test_just_aired_accept() {
        let strategy = JustAiredStrategy::new();
        let mut anime = Anime::default();
        let threshold = JustAiredStrategy::WEEKS_AFTER_AIRING;

        // unknown end airing date
        assert!(!strategy.accepts(&anime));

        // not finished airing yet
        anime.end_date = (Utc::now() + Duration::days(1)).timestamp();
        assert!(!strategy.accepts(&anime));

        // finished airing today
        anime.end_date = Utc::now().timestamp();
        assert!(!strategy.accepts(&anime));

        // finished airing recently
        anime.end_date = (Utc::now() - Duration::weeks(threshold / 2)).timestamp();
        assert!(strategy.accepts(&anime));

        // finished airing long ago
        anime.end_date = (Utc::now() - Duration::weeks(threshold)).timestamp();
        assert!(!strategy.accepts(&anime));
    }

    #[test]
    fn test_just_aired() {
        let strategy = JustAiredStrategy::new();
        let mut anime = Anime::default();
        let threshold = JustAiredStrategy::WEEKS_AFTER_AIRING;

        // recently aired and not aligned
        let offset = Duration::days(1);
        let expected = (Utc::now() + strategy.0.interval - offset).date();
        anime.end_date = (Utc::now() - offset).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));

        anime.end_date = (Utc::now() - strategy.0.interval - offset).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));

        // recently aired and aligned
        let expected = (Utc::now() + strategy.0.interval).date();
        anime.end_date = (Utc::now() - strategy.0.interval).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));

        anime.end_date = (Utc::now() - strategy.0.interval * 2).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));

        // aired long ago but needs last update
        let offset = Duration::days(1);
        let expected = (Utc::now() + offset).date();
        anime.end_date = (Utc::now() - Duration::weeks(threshold) + offset).timestamp();
        assert_eq!(strategy.next_update_date(&anime), Some(expected));
    }

    #[test]
    fn test_aired_accept() {
        let strategy = AiredStrategy::new();
        let mut anime = Anime::default();

        // no end air date
        assert!(!strategy.accepts(&anime));

        // still airing
        anime.end_date = (Utc::now() + Duration::days(1)).timestamp();
        assert!(!strategy.accepts(&anime));

        // recently finished airing
        anime.end_date = (Utc::now() - Duration::days(1)).timestamp();
        assert!(!strategy.accepts(&anime));

        // finished airing long ago
        let end_date = Utc::now() - Duration::weeks(JustAiredStrategy::WEEKS_AFTER_AIRING);
        anime.end_date = end_date.timestamp();
        assert!(strategy.accepts(&anime));
    }
}
