create table users (
       steam_id bigint primary key,
       user_name text not null,
       created_at timestamp with time zone not null
);

create table configs (
       id serial primary key,
       name text not null,
       created_at timestamp with time zone not null,
       config_type smallint not null,
       user_id bigint not null references users(steam_id)
);

alter table configs add constraint fk_configs_users_user_id
      foreign key (user_id) references users(steam_id)
