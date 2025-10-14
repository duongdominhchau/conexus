create table bookmarks(
    id uuid primary key,
    url varchar not null,
    description text,
    created_at timestamptz not null default now(),
    updated_at timestamptz
);
