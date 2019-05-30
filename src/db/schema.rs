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
        source_id -> Integer,
        source -> Integer,
        state -> Integer,
        priority -> Integer,
        has_poster -> Bool,
        has_air_date -> Bool,
        has_type -> Bool,
        has_anidb_id -> Bool,
        has_mal_id -> Bool,
        has_ann_id -> Bool,
        has_tags -> Bool,
        has_episode_count -> Bool,
        has_all_episodes -> Bool,
        has_rating -> Bool,
        has_description -> Bool,
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
