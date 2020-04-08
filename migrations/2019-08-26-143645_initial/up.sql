/* Schedules table */

create table schedules
(
    id                 serial                    not null,
    external_id        int                       not null,
    source             int                       not null,
    priority           int         default 1000  not null,
    next_update_at     timestamptz default now(),
    update_count       int         default 0     not null,
    queued_count       int         default 0     not null,
    has_poster         boolean     default false not null,
    has_start_air_date boolean     default false not null,
    has_end_air_date   boolean     default false not null,
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
    finished     boolean     default false              not null,
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

-- associate a job with it's task
create function queued_jobs_associate_with_task()
    returns trigger as
$$
begin
    update tasks
    set schedule_ids = array_append(schedule_ids, new.schedule_id)
    where new.task_id = id;

    return null;
end;
$$ language plpgsql;

create trigger queued_jobs_associate_with_task_after_insert
    after insert
    on queued_jobs
    for each row
execute procedure queued_jobs_associate_with_task();

-- increment update_count for a schedule
create function schedules_increment_update_count()
    returns trigger as
$$
begin
    update schedules
    set update_count = update_count + 1,
        priority = 1000
    where new.id = id;

    return null;
end;
$$ language plpgsql;

create trigger schedules_increment_update_count_after_update
    after update of next_update_at
    on schedules
    for each row
execute procedure schedules_increment_update_count();

-- increment queued_count for a schedule

create function queued_jobs_increment_queued_count()
    returns trigger as
$$
begin
    update schedules
    set queued_count = queued_count + 1
    where old.schedule_id = id;

    return null;
end;
$$ language plpgsql;

create trigger queued_jobs_increment_queued_count_after_delete
    after delete
    on queued_jobs
    for each row
execute procedure queued_jobs_increment_queued_count();

-- schedules binding
create function queued_jobs_bind_schedules_for_task(uuid, int)
    returns void as
$$
begin
    perform pg_advisory_xact_lock(42);

    insert into queued_jobs (task_id, schedule_id)
    select $1, id
    from schedules
    where
        next_update_at is not null
        and next_update_at <= now()
        and not exists(
            select true
            from queued_jobs
            where schedule_id = schedules.id
        )
    order by priority desc, queued_count, next_update_at
    limit $2;
end;
$$ language plpgsql;
