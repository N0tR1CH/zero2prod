-- Add migration script here
create table if not exists subscriptions
(
    id            uuid        not null,
    email         text        not null unique,
    name          text        not null,
    subscribed_at timestamptz not null,

    primary key (id)
);
