table! {
    queued_jobs (id) {
        id -> Uuid,
        task_id -> Uuid,
        schedule_id -> Int4,
        created_at -> Timestamptz,
    }
}

table! {
    schedules (id) {
        id -> Int4,
        external_id -> Int4,
        source -> Int4,
        priority -> Int4,
        next_update_at -> Nullable<Timestamptz>,
        update_count -> Int4,
        queued_count -> Int4,
        has_poster -> Bool,
        has_start_air_date -> Bool,
        has_end_air_date -> Bool,
        has_type -> Bool,
        has_anidb_id -> Bool,
        has_mal_id -> Bool,
        has_ann_id -> Bool,
        has_tags -> Bool,
        has_ep_count -> Bool,
        has_all_eps -> Bool,
        has_rating -> Bool,
        has_description -> Bool,
        src_created_at -> Nullable<Timestamptz>,
        src_updated_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

table! {
    tasks (id) {
        id -> Uuid,
        source -> Int4,
        schedule_ids -> Array<Int4>,
        finished -> Bool,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

joinable!(queued_jobs -> schedules (schedule_id));
joinable!(queued_jobs -> tasks (task_id));

allow_tables_to_appear_in_same_query!(
    queued_jobs,
    schedules,
    tasks,
);
