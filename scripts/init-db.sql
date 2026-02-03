-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create a dedicated schema for IAM
CREATE SCHEMA IF NOT EXISTS iam;

-- Set search path
SET search_path TO iam, public;

-- This file runs on initial database creation
-- Actual tables will be created via sqlx migrations
