create table schedules
(
    id          integer not null
        constraint schedules_pk
            primary key autoincrement,

    anidb_id   integer not null,
    state      integer default 0 not null,
    data_mask1 integer default 0 not null,
    created_at double default current_timestamp not null,
    updated_at double default current_timestamp not null
);

create unique index schedules_anidb_id_uindex
    on schedules (anidb_id);

create trigger schedules_updated_at_trigger
    after update on schedules
    for each row
    begin
        update schedules set updated_at = current_timestamp
            where id = old.id;
end;
