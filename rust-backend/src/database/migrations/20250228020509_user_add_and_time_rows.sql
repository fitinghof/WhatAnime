-- Add migration script here
ALTER TABLE animes
  ADD COLUMN date_added TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  ADD COLUMN from_user_name TEXT,
  ADD COLUMN from_user_mail TEXT