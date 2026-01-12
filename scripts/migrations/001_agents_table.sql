-- Migration: 001_agents_table.sql
-- Description: Add agents and pending_commands tables for Alpha release
-- Date: 2026-01-12

-- Agents table for tracking connected optio-agent instances
CREATE TABLE IF NOT EXISTS agents (
    machine_id TEXT PRIMARY KEY,
    hostname TEXT NOT NULL,
    os_info TEXT,
    cpu_usage REAL DEFAULT 0.0,
    ram_usage REAL DEFAULT 0.0,
    ram_total INTEGER DEFAULT 0,
    disk_free INTEGER DEFAULT 0,
    disk_total INTEGER DEFAULT 0,
    uptime_seconds INTEGER DEFAULT 0,
    ip_addresses TEXT,
    agent_version TEXT,
    first_seen TEXT NOT NULL,
    last_seen TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'unknown',
    client_id TEXT,
    tags TEXT,
    notes TEXT,
    FOREIGN KEY (client_id) REFERENCES clients(id) ON DELETE SET NULL
);

-- Pending commands queue for agents
CREATE TABLE IF NOT EXISTS pending_commands (
    command_id TEXT PRIMARY KEY,
    machine_id TEXT NOT NULL,
    command_type TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    priority INTEGER DEFAULT 0,
    timeout_seconds INTEGER DEFAULT 300,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL,
    dispatched_at TEXT,
    completed_at TEXT,
    result_json TEXT,
    error_message TEXT,
    FOREIGN KEY (machine_id) REFERENCES agents(machine_id) ON DELETE CASCADE
);

-- Command execution history
CREATE TABLE IF NOT EXISTS command_history (
    id TEXT PRIMARY KEY,
    command_id TEXT NOT NULL,
    machine_id TEXT NOT NULL,
    command_type TEXT NOT NULL,
    success INTEGER NOT NULL DEFAULT 0,
    exit_code INTEGER,
    stdout TEXT,
    stderr TEXT,
    error_message TEXT,
    started_at TEXT,
    finished_at TEXT,
    duration_ms INTEGER,
    created_at TEXT NOT NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_last_seen ON agents(last_seen);
CREATE INDEX IF NOT EXISTS idx_pending_commands_machine ON pending_commands(machine_id);
CREATE INDEX IF NOT EXISTS idx_pending_commands_status ON pending_commands(status);
CREATE INDEX IF NOT EXISTS idx_command_history_machine ON command_history(machine_id);
