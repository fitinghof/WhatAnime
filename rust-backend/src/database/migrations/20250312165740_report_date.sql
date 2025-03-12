-- Add migration script here
ALTER TABLE reports ADD COLUMN date_added TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()