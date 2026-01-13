/**
 * Agent Detail Modal
 *
 * A modal that shows detailed information about an agent, including:
 * - System metrics and telemetry
 * - Interactive terminal access
 * - Audit logs for the agent
 */

import { useState, useEffect } from "react";
import { Terminal } from "@/components/Terminal";
import { AuditLogTable } from "@/components/AuditLogTable";
import { getAgentHistory, getAgentTelemetryStats } from "@/lib/commands";
import type { Agent, TelemetryRecord, TelemetryStats } from "@/types";
import {
  X,
  Terminal as TerminalIcon,
  Activity,
  FileText,
  Cpu,
  HardDrive,
  Clock,
  Wifi,
  Server,
} from "lucide-react";
import clsx from "clsx";

type TabId = "overview" | "terminal" | "logs";

interface AgentDetailModalProps {
  agent: Agent;
  isOpen: boolean;
  onClose: () => void;
}

export function AgentDetailModal({
  agent,
  isOpen,
  onClose,
}: AgentDetailModalProps) {
  const [activeTab, setActiveTab] = useState<TabId>("overview");
  const [telemetry, setTelemetry] = useState<TelemetryRecord[]>([]);
  const [stats, setStats] = useState<TelemetryStats | null>(null);
  const [showTerminal, setShowTerminal] = useState(false);

  useEffect(() => {
    if (isOpen) {
      loadTelemetry();
    }
  }, [isOpen, agent.machineId]);

  const loadTelemetry = async () => {
    try {
      const [history, telemetryStats] = await Promise.all([
        getAgentHistory(agent.machineId, 50),
        getAgentTelemetryStats(agent.machineId, 24),
      ]);
      setTelemetry(history);
      setStats(telemetryStats);
    } catch (error) {
      console.error("Failed to load telemetry:", error);
    }
  };

  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB", "TB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
  };

  const formatUptime = (seconds: number): string => {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    if (days > 0) return `${days}d ${hours}h`;
    if (hours > 0) return `${hours}h ${mins}m`;
    return `${mins}m`;
  };

  if (!isOpen) return null;

  const tabs: { id: TabId; label: string; icon: typeof TerminalIcon }[] = [
    { id: "overview", label: "Overview", icon: Activity },
    { id: "terminal", label: "Terminal", icon: TerminalIcon },
    { id: "logs", label: "Audit Logs", icon: FileText },
  ];

  // Assume agent address based on IP - in production this would come from the agent record
  const agentAddress = agent.ipAddresses
    ? `http://${agent.ipAddresses.split(",")[0].trim()}:50052`
    : "";

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 border border-slate-700 rounded-xl w-full max-w-5xl max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <div
              className={clsx(
                "p-2 rounded-lg",
                agent.status === "online"
                  ? "bg-green-500/20"
                  : "bg-slate-600/50"
              )}
            >
              <Server
                className={clsx(
                  "w-5 h-5",
                  agent.status === "online"
                    ? "text-green-400"
                    : "text-slate-500"
                )}
              />
            </div>
            <div>
              <h2 className="text-lg font-semibold text-white">
                {agent.hostname}
              </h2>
              <p className="text-sm text-slate-400 font-mono">
                {agent.machineId.slice(0, 16)}...
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-2 hover:bg-slate-700 rounded-lg transition-colors"
          >
            <X className="w-5 h-5 text-slate-400" />
          </button>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 p-2 border-b border-slate-700">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={clsx(
                "flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-colors",
                activeTab === tab.id
                  ? "bg-optio-600 text-white"
                  : "text-slate-400 hover:text-white hover:bg-slate-700"
              )}
            >
              <tab.icon className="w-4 h-4" />
              {tab.label}
            </button>
          ))}
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {activeTab === "overview" && (
            <div className="space-y-6">
              {/* Quick Stats */}
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <MetricCard
                  icon={Cpu}
                  label="CPU Usage"
                  value={`${agent.cpuUsage.toFixed(1)}%`}
                  subValue={
                    stats
                      ? `Avg: ${stats.avgCpu.toFixed(1)}% | Max: ${stats.maxCpu.toFixed(1)}%`
                      : undefined
                  }
                  color={agent.cpuUsage > 80 ? "red" : "green"}
                />
                <MetricCard
                  icon={HardDrive}
                  label="RAM Usage"
                  value={`${agent.ramUsage.toFixed(1)}%`}
                  subValue={`${formatBytes(
                    (agent.ramUsage / 100) * agent.ramTotal
                  )} / ${formatBytes(agent.ramTotal)}`}
                  color={agent.ramUsage > 80 ? "yellow" : "green"}
                />
                <MetricCard
                  icon={HardDrive}
                  label="Disk Free"
                  value={formatBytes(agent.diskFree)}
                  subValue={`of ${formatBytes(agent.diskTotal)}`}
                  color="blue"
                />
                <MetricCard
                  icon={Clock}
                  label="Uptime"
                  value={formatUptime(agent.uptimeSeconds)}
                  color="purple"
                />
              </div>

              {/* System Info */}
              <div className="bg-slate-700/30 rounded-lg p-4">
                <h3 className="text-sm font-semibold text-white mb-3">
                  System Information
                </h3>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <InfoRow label="OS" value={agent.osInfo || "Unknown"} />
                  <InfoRow
                    label="Agent Version"
                    value={agent.agentVersion || "Unknown"}
                  />
                  <InfoRow
                    label="IP Addresses"
                    value={agent.ipAddresses || "N/A"}
                  />
                  <InfoRow label="Status" value={agent.status} />
                  <InfoRow label="First Seen" value={agent.firstSeen} />
                  <InfoRow label="Last Seen" value={agent.lastSeen} />
                </div>
              </div>

              {/* Quick Actions */}
              <div className="flex gap-3">
                <button
                  onClick={() => setActiveTab("terminal")}
                  disabled={agent.status !== "online" || !agentAddress}
                  className="flex items-center gap-2 px-4 py-2 bg-optio-600 hover:bg-optio-500 disabled:bg-slate-600 disabled:cursor-not-allowed text-white rounded-lg transition-colors"
                >
                  <TerminalIcon className="w-4 h-4" />
                  Open Terminal
                </button>
                <button
                  onClick={() => setActiveTab("logs")}
                  className="flex items-center gap-2 px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg transition-colors"
                >
                  <FileText className="w-4 h-4" />
                  View Audit Logs
                </button>
              </div>
            </div>
          )}

          {activeTab === "terminal" && (
            <div className="h-[500px]">
              {agent.status !== "online" ? (
                <div className="flex flex-col items-center justify-center h-full text-slate-400">
                  <Wifi className="w-12 h-12 mb-4 opacity-50" />
                  <p className="text-lg">Agent is offline</p>
                  <p className="text-sm">
                    Terminal access requires an online agent
                  </p>
                </div>
              ) : !agentAddress ? (
                <div className="flex flex-col items-center justify-center h-full text-slate-400">
                  <Server className="w-12 h-12 mb-4 opacity-50" />
                  <p className="text-lg">No IP address available</p>
                  <p className="text-sm">
                    Cannot establish terminal connection
                  </p>
                </div>
              ) : (
                <Terminal
                  agentId={agent.machineId}
                  agentAddress={agentAddress}
                  hostname={agent.hostname}
                  onClose={() => setActiveTab("overview")}
                />
              )}
            </div>
          )}

          {activeTab === "logs" && (
            <AuditLogTable
              agentId={agent.machineId}
              limit={50}
              showAgentColumn={false}
              title={`Audit Logs - ${agent.hostname}`}
            />
          )}
        </div>
      </div>
    </div>
  );
}

// Helper Components
function MetricCard({
  icon: Icon,
  label,
  value,
  subValue,
  color = "blue",
}: {
  icon: typeof Cpu;
  label: string;
  value: string;
  subValue?: string;
  color?: "green" | "yellow" | "red" | "blue" | "purple";
}) {
  const colorClasses = {
    green: "text-green-400",
    yellow: "text-yellow-400",
    red: "text-red-400",
    blue: "text-blue-400",
    purple: "text-purple-400",
  };

  return (
    <div className="bg-slate-700/30 rounded-lg p-4">
      <div className="flex items-center gap-2 mb-2">
        <Icon className={clsx("w-4 h-4", colorClasses[color])} />
        <span className="text-sm text-slate-400">{label}</span>
      </div>
      <p className={clsx("text-2xl font-bold", colorClasses[color])}>{value}</p>
      {subValue && <p className="text-xs text-slate-500 mt-1">{subValue}</p>}
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <span className="text-slate-400">{label}:</span>
      <span className="text-white ml-2">{value}</span>
    </div>
  );
}

export default AgentDetailModal;
