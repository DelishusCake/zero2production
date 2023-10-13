-- Create a stored procedure to trigger a timestamp update
create or replace function trigger_set_timestamp()
returns trigger as $$
begin
  new.updated_at = now();
  return new;
end;
$$ language plpgsql;

-- Create the subscriptions table
create table if not exists subscriptions(
	id uuid primary key not null default gen_random_uuid(),

	name  text not null,
	email text not null unique,

	confirmed_at timestamptz default null,

	created_at timestamptz not null default now(),
	updated_at timestamptz not null default now()
);

-- Create the update trigger for subscriptions
create trigger set_timestamp_subscriptions 
before update on subscriptions
for each row execute function trigger_set_timestamp();
