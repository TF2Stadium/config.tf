create table configs (
       id serial primary key,
       name text,
       created_at timestamp with time zone,
       config_type integer,
       config_path text not null,
       user_id bigint not null
);

create table users (
       steam_id bigint primary key,
       user_name text not null,
       created_at timestamp with time zone
);

alter table configs add constraint fk_configs_users_user_id
      foreign key (user_id) references users(steam_id)
