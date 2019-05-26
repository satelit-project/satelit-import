/* Schedules table */

create table schedules
(
    id                integer not null
        constraint schedules_pk
            primary key autoincrement,

    anidb_id          integer not null,
    state             integer default 0 not null,
    priority          integer default 1000 not null,
    has_poster        integer default 0 not null,
    has_air_date      integer default 0 not null,
    has_type          integer default 0 not null,
    has_mal_id        integer default 0 not null,
    has_ann_id        integer default 0 not null,
    has_tags          integer default 0 not null,
    has_episode_count integer default 0 not null,
    has_episodes      integer default 0 not null,
    has_rating        integer default 0 not null,
    has_description   integer default 0 not null,
    created_at        double  default current_timestamp not null,
    updated_at        double  default current_timestamp not null
);

create unique index schedules_anidb_id_uindex
    on schedules (anidb_id);

-- update `update_at` field every time when row is changed
create trigger schedules_updated_at_trigger
    after update
    on schedules
    for each row
begin
    update schedules
    set updated_at = current_timestamp
    where schedules.id = old.id;
end;

/* Tasks table */

create table tasks
(
    id     text    not null
        constraint tasks_pk
            primary key,

    source integer not null
);

create unique index tasks_id_uindex
    on tasks (id);

/* Scheduled tasks */

create table scheduled_tasks
(
    id          integer not null
        constraint scheduled_tasks_pk
            primary key autoincrement,

    task_id     text    not null
        constraint scheduled_tasks_tasks_id_fk
            references tasks (id)
            on delete cascade,

    schedule_id integer not null
        constraint scheduled_tasks_schedules_id_fk
            references schedules (id)
            on delete cascade
);

create unique index scheduled_tasks_id_uindex
    on scheduled_tasks (id);

create unique index scheduled_tasks_schedule_id_uindex
    on scheduled_tasks (schedule_id);

-- set `state` to 'Processing' when task is created
create trigger scheduled_tasks_anime_set_processing_state
    after insert
    on scheduled_tasks
begin
    update schedules
    set state = 1
    where schedules.id = new.schedule_id;
end;

-- set `state` to 'Finished' when task is deleted
create trigger scheduled_tasks_anime_set_finished_state
    after delete
    on scheduled_tasks
begin
    update schedules
    set state = 2
    where schedules.id = old.schedule_id;
end;
