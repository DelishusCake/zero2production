-- Create the users table
create table if not exists users(
  id uuid primary key not null default gen_random_uuid(),

  email         text not null unique,
  password_hash text not null,

  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

-- Create the update trigger for users
create trigger set_timestamp_users 
before update on users
for each row execute function trigger_set_timestamp();
