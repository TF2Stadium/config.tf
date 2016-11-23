create table configs (
       id serial primary key,
       name text not null,
       created_at timestamp with time zone not null,
       config_type smallint not null
);
