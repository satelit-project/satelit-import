/* Schedules table */

create table schedules
(
    id                 serial                    not null,
    external_id        int                       not null,
    source             int                       not null,
    priority           int         default 1000  not null,
    next_update_at     timestamptz default null,
    update_count       int         default 0     not null,
    has_poster         boolean     default false not null,
    has_start_air_date boolean     default false not null,
    has_end_air_date   bool        default false not null,
    has_type           boolean     default false not null,
    has_anidb_id       boolean     default false not null,
    has_mal_id         boolean     default false not null,
    has_ann_id         boolean     default false not null,
    has_tags           boolean     default false not null,
    has_ep_count       boolean     default false not null,
    has_all_eps        boolean     default false not null,
    has_rating         boolean     default false not null,
    has_description    boolean     default false not null,
    src_created_at     timestamptz default null,
    src_updated_at     timestamptz default null,
    created_at         timestamptz default now() not null,
    updated_at         timestamptz default now() not null
);

create unique index schedules_external_id_source_uindex
    on schedules (external_id, source);

create unique index schedules_id_uindex
    on schedules (id);

alter table schedules
    add constraint schedules_pk
        primary key (id);

select diesel_manage_updated_at('schedules');

/* Tasks table */

create table tasks
(
    id           uuid        default uuid_generate_v4() not null,
    source       int                                    not null,
    schedule_ids int[]       default array []::int[]    not null,
    created_at   timestamptz default now()              not null,
    updated_at   timestamptz default now()              not null
);

create unique index tasks_id_uindex
    on tasks (id);

alter table tasks
    add constraint tasks_pk
        primary key (id);

select diesel_manage_updated_at('tasks');

/* Queued Jobs table */

create table queued_jobs
(
    id          uuid        default uuid_generate_v4() not null,
    task_id     uuid                                   not null
        constraint queued_jobs_tasks_id_fk
            references tasks
            on delete cascade,
    schedule_id serial                                 not null
        constraint queued_jobs_schedules_id_fk
            references schedules
            on delete cascade,
    created_at  timestamptz default now()              not null
);

create unique index queued_jobs_id_uindex
    on queued_jobs (id);

create unique index queued_jobs_schedule_id_uindex
    on queued_jobs (schedule_id);

alter table queued_jobs
    add constraint queued_jobs_pk
        primary key (id);

-- update parent rows
create function queued_jobs_update_parent_rows()
    returns trigger as
$$
begin
    -- put schedule id to task
    update tasks
    set schedule_ids = array_append(schedule_ids, new.schedule_id)
    where new.task_id = id;

    -- increment update_count
    update schedules
    set update_count = update_count + 1
    where new.schedule_id = id;

    return null;
end;
$$ language plpgsql;

create trigger queued_jobs_update_parent_rows_after_insert
    after insert
    on queued_jobs
    for each row
execute procedure queued_jobs_update_parent_rows();

-- schedules binding
create function queued_jobs_bind_schedules_for_task(uuid, int)
    returns void as
$$
begin
    lock queued_jobs in access exclusive mode;

    insert into queued_jobs (task_id, schedule_id)
    select $1, id
    from schedules
    where not exists(
            select true
            from queued_jobs
            where schedule_id = schedules.id
        )
    order by priority desc
    limit $2;
end;
$$ language plpgsql;
