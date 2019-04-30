table! {
    schedules (id) {
        id -> Integer,
        anidb_id -> Integer,
        state -> Integer,
        priority -> Integer,
        has_poster -> Integer,
        has_air_date -> Integer,
        has_type -> Integer,
        has_mal_id -> Integer,
        has_tags -> Integer,
        has_episode_count -> Integer,
        has_episodes -> Integer,
        has_rating -> Integer,
        has_description -> Integer,
        created_at -> Double,
        updated_at -> Double,
    }
}
