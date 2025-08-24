-- TwoDo Optimized Database Schema for Cloudflare D1
-- Designed for performance and minimal storage

-- Users table (minimal for couples app)
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    -- Index for fast login lookups
    UNIQUE(username)
);

-- Groups table (couples/families)
CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_by TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Group memberships (optimized for small groups)
CREATE TABLE group_members (
    group_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('admin', 'member')),
    joined_at INTEGER NOT NULL,
    PRIMARY KEY (group_id, user_id),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Expenses table (optimized for Splitwise functionality)
CREATE TABLE expenses (
    id TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    amount REAL NOT NULL,
    date INTEGER NOT NULL,
    paid_by TEXT NOT NULL,
    group_id TEXT NOT NULL,
    category_name TEXT,
    category_color TEXT,
    split_type TEXT NOT NULL CHECK (split_type IN ('equal', 'exact', 'percentage', 'full')),
    notes TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (paid_by) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

-- Expense splits (normalized for efficiency)
CREATE TABLE expense_splits (
    expense_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    amount REAL NOT NULL,
    is_settled BOOLEAN DEFAULT FALSE,
    settled_at INTEGER,
    PRIMARY KEY (expense_id, user_id),
    FOREIGN KEY (expense_id) REFERENCES expenses(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Settlements (debt payments between users)
CREATE TABLE settlements (
    id TEXT PRIMARY KEY,
    group_id TEXT NOT NULL,
    payer TEXT NOT NULL,
    payee TEXT NOT NULL,
    amount REAL NOT NULL,
    description TEXT,
    settled_at INTEGER NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (payer) REFERENCES users(id),
    FOREIGN KEY (payee) REFERENCES users(id)
);

-- Chores table
CREATE TABLE chores (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    assigned_to TEXT,
    deadline INTEGER,
    status TEXT NOT NULL CHECK (status IN ('pending', 'in_progress', 'completed')),
    category_name TEXT,
    category_color TEXT,
    recurring TEXT CHECK (recurring IN ('none', 'daily', 'weekly', 'monthly')),
    group_id TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (assigned_to) REFERENCES users(id),
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE
);

-- Calendar events
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    group_id TEXT NOT NULL,
    created_by TEXT NOT NULL,
    linked_expense_id TEXT,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (group_id) REFERENCES groups(id) ON DELETE CASCADE,
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (linked_expense_id) REFERENCES expenses(id)
);

-- Event participants (many-to-many)
CREATE TABLE event_participants (
    event_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    PRIMARY KEY (event_id, user_id),
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Push notification tokens
CREATE TABLE push_tokens (
    user_id TEXT NOT NULL,
    token TEXT NOT NULL,
    platform TEXT NOT NULL CHECK (platform IN ('ios', 'android', 'web')),
    created_at INTEGER NOT NULL,
    PRIMARY KEY (user_id, token),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- Performance indexes for common queries
CREATE INDEX idx_expenses_group_date ON expenses(group_id, date DESC);
CREATE INDEX idx_expense_splits_user ON expense_splits(user_id);
CREATE INDEX idx_chores_assigned_deadline ON chores(assigned_to, deadline);
CREATE INDEX idx_events_group_time ON events(group_id, start_time);
CREATE INDEX idx_settlements_group ON settlements(group_id, settled_at DESC);

-- View for expense balances (cached computation)
CREATE VIEW expense_balances AS
SELECT 
    group_id,
    user_id,
    SUM(CASE WHEN transaction_type = 'paid' THEN amount ELSE 0 END) as total_paid,
    SUM(CASE WHEN transaction_type = 'owed' THEN amount ELSE 0 END) as total_owed,
    SUM(CASE WHEN transaction_type = 'paid' THEN amount ELSE -amount END) as net_balance
FROM (
    -- Money paid by user
    SELECT group_id, paid_by as user_id, amount, 'paid' as transaction_type
    FROM expenses
    
    UNION ALL
    
    -- Money owed by user (unsettled splits)
    SELECT e.group_id, es.user_id, es.amount, 'owed' as transaction_type
    FROM expenses e
    JOIN expense_splits es ON e.id = es.expense_id
    WHERE es.is_settled = FALSE
    
    UNION ALL
    
    -- Settlements (money paid back)
    SELECT group_id, payer as user_id, amount, 'paid' as transaction_type
    FROM settlements
) transactions
GROUP BY group_id, user_id;
