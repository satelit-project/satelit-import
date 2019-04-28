create table schedules
(
    id          integer not null
        constraint schedules_pk
            primary key autoincrement,

    anime_id   integer not null,
    state      integer default 0 not null,
    data_mask1 integer default 0 not null
);

create unique index schedules_anime_id_uindex
    on schedules (anime_id);
