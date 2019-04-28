create table schedule
(
    id          integer not null
        constraint schedule_pk
            primary key autoincrement,

    anime_id   integer not null,
    state      integer default 0 not null,
    data_mask1 integer default 0 not null
);

create unique index schedule_anime_id_uindex
    on schedule (anime_id);
