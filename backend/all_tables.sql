-- Complete D1 database schema for TwoDo app

-- Users table (for auth and references)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    email TEXT UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Groups table
CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Group members table
CREATE TABLE IF NOT EXISTS group_members (
    group_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member', -- 'admin', 'member'
    joined_at TEXT NOT NULL,
    PRIMARY KEY (group_id, user_id)
);

-- Calendar events table
CREATE TABLE IF NOT EXISTS events (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT,
    location TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Event attendees table
CREATE TABLE IF NOT EXISTS event_attendees (
    event_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'accepted', 'declined'
    responded_at TEXT,
    PRIMARY KEY (event_id, user_id)
);

-- Chores table
CREATE TABLE IF NOT EXISTS chores (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    assigned_to TEXT,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'in_progress', 'completed'
    priority TEXT NOT NULL DEFAULT 'medium', -- 'low', 'medium', 'high'
    due_date TEXT,
    completed_at TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Chore comments table
CREATE TABLE IF NOT EXISTS chore_comments (
    id TEXT PRIMARY KEY,
    chore_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    comment TEXT NOT NULL,
    created_at TEXT NOT NULL
);

-- Notifications table
CREATE TABLE IF NOT EXISTS notifications (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    type TEXT NOT NULL, -- 'expense_added', 'chore_assigned', 'event_reminder', etc.
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    related_id TEXT, -- ID of related object (expense, chore, event)
    is_read BOOLEAN DEFAULT FALSE,
    created_at TEXT NOT NULL
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_groups_created_by ON groups(created_by);
CREATE INDEX IF NOT EXISTS idx_group_members_group_id ON group_members(group_id);
CREATE INDEX IF NOT EXISTS idx_group_members_user_id ON group_members(user_id);
CREATE INDEX IF NOT EXISTS idx_events_group_id ON events(group_id);
CREATE INDEX IF NOT EXISTS idx_events_start_time ON events(start_time);
CREATE INDEX IF NOT EXISTS idx_events_created_by ON events(created_by);
CREATE INDEX IF NOT EXISTS idx_event_attendees_event_id ON event_attendees(event_id);
CREATE INDEX IF NOT EXISTS idx_event_attendees_user_id ON event_attendees(user_id);
CREATE INDEX IF NOT EXISTS idx_chores_group_id ON chores(group_id);
CREATE INDEX IF NOT EXISTS idx_chores_assigned_to ON chores(assigned_to);
CREATE INDEX IF NOT EXISTS idx_chores_status ON chores(status);
CREATE INDEX IF NOT EXISTS idx_chores_due_date ON chores(due_date);
CREATE INDEX IF NOT EXISTS idx_chore_comments_chore_id ON chore_comments(chore_id);
CREATE INDEX IF NOT EXISTS idx_notifications_user_id ON notifications(user_id);
CREATE INDEX IF NOT EXISTS idx_notifications_is_read ON notifications(is_read);
