-- Create the users table
create table if not exists users(
  id uuid primary key not null default gen_random_uuid(),

  oauth2_provider    text not null,
  oauth2_provider_id text not null,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create unique index idx_users_oauth2
on users(oauth2_provider, oauth2_provider_id);

-- Create the update trigger for users
create trigger set_timestamp_users 
before update on users
for each row execute function trigger_set_timestamp();
