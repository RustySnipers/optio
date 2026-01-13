/**
 * Audit Log Table Component
 *
 * Displays action logs for agents with filtering and search capabilities.
 * Used in the Agent Detail view and Reporting Center.
 */

import { useState, useEffect, useCallback } from "react";
import {
  getAuditLogs,
  getAgentAuditLogs,
  getAuditLogsByType,
} from "@/lib/commands";
import type { ActionLog, ActionType, ActionStatus } from "@/types";
import {
  RefreshCw,
  Filter,
  Search,
  Terminal,
  FileCode,
  Package,
  Settings,
  UserPlus,
  UserMinus,
  Command,
  HardDrive,
  CheckCircle,
  XCircle,
  Clock,
  AlertCircle,
  Ban,
} from "lucide-react";
import clsx from "clsx";

interface AuditLogTableProps {
  /** Filter logs by agent ID (optional) */
  agentId?: string;
  /** Maximum number of logs to display */
  limit?: number;
  /** Whether to show the agent column */
  showAgentColumn?: boolean;
  /** Title for the table */
  title?: string;
}

const ACTION_TYPE_CONFIG: Record<
  ActionType,
  { icon: typeof Terminal; label: string; color: string }
> = {
  SCRIPT_EXEC: {
    icon: FileCode,
    label: "Script Execution",
    color: "text-blue-400",
  },
  TERMINAL_SESSION: {
    icon: Terminal,
    label: "Terminal Session",
    color: "text-purple-400",
  },
  PATCH_INSTALL: {
    icon: Package,
    label: "Patch Install",
    color: "text-green-400",
  },
  CONFIG_CHANGE: {
    icon: Settings,
    label: "Config Change",
    color: "text-yellow-400",
  },
  AGENT_REGISTRATION: {
    icon: UserPlus,
    label: "Agent Registration",
    color: "text-cyan-400",
  },
  AGENT_DELETION: {
    icon: UserMinus,
    label: "Agent Deletion",
    color: "text-red-400",
  },
  SYSTEM_COMMAND: {
    icon: Command,
    label: "System Command",
    color: "text-orange-400",
  },
  FILE_TRANSFER: {
    icon: HardDrive,
    label: "File Transfer",
    color: "text-indigo-400",
  },
};

const STATUS_CONFIG: Record<
  ActionStatus,
  { icon: typeof CheckCircle; label: string; color: string; bgColor: string }
> = {
  SUCCESS: {
    icon: CheckCircle,
    label: "Success",
    color: "text-green-400",
    bgColor: "bg-green-400/10",
  },
  FAILURE: {
    icon: XCircle,
    label: "Failure",
    color: "text-red-400",
    bgColor: "bg-red-400/10",
  },
  PENDING: {
    icon: Clock,
    label: "Pending",
    color: "text-yellow-400",
    bgColor: "bg-yellow-400/10",
  },
  IN_PROGRESS: {
    icon: AlertCircle,
    label: "In Progress",
    color: "text-blue-400",
    bgColor: "bg-blue-400/10",
  },
  CANCELLED: {
    icon: Ban,
    label: "Cancelled",
    color: "text-slate-400",
    bgColor: "bg-slate-400/10",
  },
};

export function AuditLogTable({
  agentId,
  limit = 100,
  showAgentColumn = true,
  title = "Audit Logs",
}: AuditLogTableProps) {
  const [logs, setLogs] = useState<ActionLog[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState("");
  const [filterType, setFilterType] = useState<ActionType | "all">("all");
  const [filterStatus, setFilterStatus] = useState<ActionStatus | "all">("all");

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    try {
      let fetchedLogs: ActionLog[];

      if (filterType !== "all") {
        fetchedLogs = await getAuditLogsByType(filterType, limit);
        // Filter by agent if needed
        if (agentId) {
          fetchedLogs = fetchedLogs.filter((log) => log.agentId === agentId);
        }
      } else if (agentId) {
        fetchedLogs = await getAgentAuditLogs(agentId, limit);
      } else {
        fetchedLogs = await getAuditLogs(undefined, limit);
      }

      setLogs(fetchedLogs);
    } catch (error) {
      console.error("Failed to fetch audit logs:", error);
    } finally {
      setLoading(false);
    }
  }, [agentId, limit, filterType]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  // Filter logs based on search and status
  const filteredLogs = logs.filter((log) => {
    // Search filter
    if (searchTerm) {
      const search = searchTerm.toLowerCase();
      const matchesSearch =
        log.agentId.toLowerCase().includes(search) ||
        log.actionType.toLowerCase().includes(search) ||
        log.commandContent?.toLowerCase().includes(search) ||
        log.sessionId?.toLowerCase().includes(search);
      if (!matchesSearch) return false;
    }

    // Status filter
    if (filterStatus !== "all" && log.status !== filterStatus) {
      return false;
    }

    return true;
  });

  // Format timestamp
  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString();
  };

  // Format relative time
  const formatRelativeTime = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSecs = Math.floor(diffMs / 1000);
    const diffMins = Math.floor(diffSecs / 60);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffSecs < 60) return `${diffSecs}s ago`;
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${diffDays}d ago`;
  };

  return (
    <div className="bg-slate-800 rounded-lg border border-slate-700">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-slate-700">
        <h3 className="text-lg font-semibold text-white">{title}</h3>
        <button
          onClick={fetchLogs}
          disabled={loading}
          className="flex items-center gap-2 px-3 py-1.5 text-sm bg-slate-700 hover:bg-slate-600 rounded-md transition-colors disabled:opacity-50"
        >
          <RefreshCw
            className={clsx("w-4 h-4", loading && "animate-spin")}
          />
          Refresh
        </button>
      </div>

      {/* Filters */}
      <div className="flex flex-wrap items-center gap-4 p-4 border-b border-slate-700 bg-slate-800/50">
        {/* Search */}
        <div className="relative flex-1 min-w-[200px]">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-400" />
          <input
            type="text"
            placeholder="Search logs..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full pl-10 pr-4 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Action Type Filter */}
        <div className="flex items-center gap-2">
          <Filter className="w-4 h-4 text-slate-400" />
          <select
            value={filterType}
            onChange={(e) =>
              setFilterType(e.target.value as ActionType | "all")
            }
            className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="all">All Types</option>
            {Object.entries(ACTION_TYPE_CONFIG).map(([type, config]) => (
              <option key={type} value={type}>
                {config.label}
              </option>
            ))}
          </select>
        </div>

        {/* Status Filter */}
        <select
          value={filterStatus}
          onChange={(e) =>
            setFilterStatus(e.target.value as ActionStatus | "all")
          }
          className="px-3 py-2 bg-slate-700 border border-slate-600 rounded-md text-sm text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          <option value="all">All Statuses</option>
          {Object.entries(STATUS_CONFIG).map(([status, config]) => (
            <option key={status} value={status}>
              {config.label}
            </option>
          ))}
        </select>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead>
            <tr className="text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
              <th className="px-4 py-3">Type</th>
              {showAgentColumn && <th className="px-4 py-3">Agent</th>}
              <th className="px-4 py-3">Status</th>
              <th className="px-4 py-3">Details</th>
              <th className="px-4 py-3">Time</th>
              <th className="px-4 py-3">Source</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700">
            {loading ? (
              <tr>
                <td
                  colSpan={showAgentColumn ? 6 : 5}
                  className="px-4 py-8 text-center text-slate-400"
                >
                  <RefreshCw className="w-6 h-6 animate-spin mx-auto mb-2" />
                  Loading logs...
                </td>
              </tr>
            ) : filteredLogs.length === 0 ? (
              <tr>
                <td
                  colSpan={showAgentColumn ? 6 : 5}
                  className="px-4 py-8 text-center text-slate-400"
                >
                  No logs found
                </td>
              </tr>
            ) : (
              filteredLogs.map((log) => {
                const typeConfig = ACTION_TYPE_CONFIG[log.actionType];
                const statusConfig = STATUS_CONFIG[log.status];
                const TypeIcon = typeConfig?.icon || Command;
                const StatusIcon = statusConfig?.icon || Clock;

                return (
                  <tr
                    key={log.id}
                    className="hover:bg-slate-700/50 transition-colors"
                  >
                    {/* Type */}
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-2">
                        <TypeIcon
                          className={clsx(
                            "w-4 h-4",
                            typeConfig?.color || "text-slate-400"
                          )}
                        />
                        <span className="text-sm text-slate-200">
                          {typeConfig?.label || log.actionType}
                        </span>
                      </div>
                    </td>

                    {/* Agent */}
                    {showAgentColumn && (
                      <td className="px-4 py-3">
                        <span className="text-sm text-slate-300 font-mono">
                          {log.agentId.slice(0, 8)}...
                        </span>
                      </td>
                    )}

                    {/* Status */}
                    <td className="px-4 py-3">
                      <span
                        className={clsx(
                          "inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium",
                          statusConfig?.bgColor || "bg-slate-600/50",
                          statusConfig?.color || "text-slate-300"
                        )}
                      >
                        <StatusIcon className="w-3 h-3" />
                        {statusConfig?.label || log.status}
                      </span>
                    </td>

                    {/* Details */}
                    <td className="px-4 py-3">
                      <div className="max-w-xs">
                        {log.commandContent ? (
                          <span
                            className="text-sm text-slate-400 truncate block"
                            title={log.commandContent}
                          >
                            {log.commandContent.length > 50
                              ? `${log.commandContent.slice(0, 50)}...`
                              : log.commandContent}
                          </span>
                        ) : log.sessionId ? (
                          <span className="text-sm text-slate-400 font-mono">
                            Session: {log.sessionId.slice(0, 8)}...
                          </span>
                        ) : (
                          <span className="text-sm text-slate-500">-</span>
                        )}
                      </div>
                    </td>

                    {/* Time */}
                    <td className="px-4 py-3">
                      <div className="flex flex-col">
                        <span className="text-sm text-slate-300">
                          {formatRelativeTime(log.timestamp)}
                        </span>
                        <span className="text-xs text-slate-500">
                          {formatTimestamp(log.timestamp)}
                        </span>
                      </div>
                    </td>

                    {/* Source */}
                    <td className="px-4 py-3">
                      <span
                        className={clsx(
                          "text-xs px-2 py-0.5 rounded",
                          log.userInitiated
                            ? "bg-blue-400/10 text-blue-400"
                            : "bg-slate-600/50 text-slate-400"
                        )}
                      >
                        {log.userInitiated ? "Manual" : "Automated"}
                      </span>
                    </td>
                  </tr>
                );
              })
            )}
          </tbody>
        </table>
      </div>

      {/* Footer */}
      <div className="flex items-center justify-between px-4 py-3 border-t border-slate-700 text-sm text-slate-400">
        <span>
          Showing {filteredLogs.length} of {logs.length} logs
        </span>
        <span>Last updated: {new Date().toLocaleTimeString()}</span>
      </div>
    </div>
  );
}

export default AuditLogTable;
