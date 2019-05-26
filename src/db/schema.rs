table! {
    scheduled_tasks (id) {
        id -> Integer,
        task_id -> Text,
        schedule_id -> Integer,
    }
}

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
        has_ann_id -> Integer,
        has_tags -> Integer,
        has_episode_count -> Integer,
        has_episodes -> Integer,
        has_rating -> Integer,
        has_description -> Integer,
        created_at -> Double,
        updated_at -> Double,
    }
}

table! {
    tasks (id) {
        id -> Text,
        source -> Integer,
    }
}

joinable!(scheduled_tasks -> schedules (schedule_id));
joinable!(scheduled_tasks -> tasks (task_id));

allow_tables_to_appear_in_same_query!(
    scheduled_tasks,
    schedules,
    tasks,
);
