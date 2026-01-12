import { useState, useEffect, useCallback } from "react";
import { listClients, getSystemInfo, getAgents, getAgentStats, refreshAgentStatus } from "@/lib/commands";
import type { Client, SystemInfo, Agent, AgentStats } from "@/types";
import {
  Users,
  FileCode,
  Shield,
  Activity,
  ArrowRight,
  Server,
  Cpu,
  HardDrive,
  Wifi,
  WifiOff,
} from "lucide-react";

interface StatCardProps {
  title: string;
  value: string | number;
  icon: React.ComponentType<{ className?: string }>;
  change?: string;
  trend?: "up" | "down" | "neutral";
}

function StatCard({ title, value, icon: Icon, change, trend }: StatCardProps) {
  return (
    <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
      <div className="flex items-start justify-between">
        <div>
          <p className="text-sm text-slate-400 mb-1">{title}</p>
          <p className="text-3xl font-bold text-white">{value}</p>
          {change && (
            <p
              className={`text-sm mt-2 ${
                trend === "up"
                  ? "text-secure"
                  : trend === "down"
                  ? "text-critical"
                  : "text-slate-400"
              }`}
            >
              {change}
            </p>
          )}
        </div>
        <div className="p-3 bg-optio-600/20 rounded-lg">
          <Icon className="w-6 h-6 text-optio-400" />
        </div>
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`;
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${mins}m`;
  return `${mins}m`;
}

function AgentStatusBadge({ status }: { status: string }) {
  const colors = {
    online: "bg-green-500/20 text-green-400 border-green-500/30",
    offline: "bg-red-500/20 text-red-400 border-red-500/30",
    unknown: "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
    error: "bg-red-500/20 text-red-400 border-red-500/30",
  };
  const color = colors[status as keyof typeof colors] || colors.unknown;

  return (
    <span className={`px-2 py-0.5 text-xs rounded-full border ${color}`}>
      {status}
    </span>
  );
}

function AgentCard({ agent }: { agent: Agent }) {
  return (
    <div className="bg-slate-700/30 rounded-lg p-4 space-y-3">
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3">
          <div className={`p-2 rounded-lg ${agent.status === "online" ? "bg-green-500/20" : "bg-slate-600/50"}`}>
            {agent.status === "online" ? (
              <Wifi className="w-5 h-5 text-green-400" />
            ) : (
              <WifiOff className="w-5 h-5 text-slate-500" />
            )}
          </div>
          <div>
            <p className="text-white font-medium">{agent.hostname}</p>
            <p className="text-xs text-slate-500 font-mono">{agent.machineId.slice(0, 8)}...</p>
          </div>
        </div>
        <AgentStatusBadge status={agent.status} />
      </div>

      <div className="grid grid-cols-2 gap-2 text-sm">
        <div className="flex items-center gap-2 text-slate-400">
          <Cpu className="w-4 h-4" />
          <span>CPU: {agent.cpuUsage.toFixed(1)}%</span>
        </div>
        <div className="flex items-center gap-2 text-slate-400">
          <HardDrive className="w-4 h-4" />
          <span>RAM: {agent.ramUsage.toFixed(1)}%</span>
        </div>
      </div>

      <div className="flex items-center justify-between text-xs text-slate-500">
        <span>{agent.osInfo?.split(" ").slice(0, 2).join(" ") || "Unknown OS"}</span>
        <span>Up: {formatUptime(agent.uptimeSeconds)}</span>
      </div>

      {agent.ipAddresses && (
        <div className="text-xs text-slate-500">
          IP: {agent.ipAddresses}
        </div>
      )}
    </div>
  );
}

export function Dashboard() {
  const [clients, setClients] = useState<Client[]>([]);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [agentStats, setAgentStats] = useState<AgentStats | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const loadAgents = useCallback(async () => {
    try {
      // Refresh stale agents first
      await refreshAgentStatus();
      const [agentData, stats] = await Promise.all([
        getAgents(),
        getAgentStats(),
      ]);
      setAgents(agentData);
      setAgentStats(stats);
    } catch (error) {
      console.error("Failed to load agents:", error);
    }
  }, []);

  useEffect(() => {
    async function loadData() {
      try {
        const [clientData, sysInfo] = await Promise.all([
          listClients(),
          getSystemInfo(),
        ]);
        setClients(clientData);
        setSystemInfo(sysInfo);
        await loadAgents();
      } catch (error) {
        console.error("Failed to load dashboard data:", error);
      } finally {
        setIsLoading(false);
      }
    }
    loadData();
  }, [loadAgents]);

  // Poll for agents every 5 seconds
  useEffect(() => {
    const interval = setInterval(loadAgents, 5000);
    return () => clearInterval(interval);
  }, [loadAgents]);

  if (isLoading) {
    return (
      <div className="p-8 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-optio-500"></div>
      </div>
    );
  }

  return (
    <div className="p-8 space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold text-white mb-2">Dashboard</h1>
        <p className="text-slate-400">
          Welcome back. Here's an overview of your consulting operations.
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <StatCard
          title="Active Clients"
          value={clients.length}
          icon={Users}
          change="+2 this month"
          trend="up"
        />
        <StatCard
          title="Connected Agents"
          value={agentStats?.online || 0}
          icon={Server}
          change={`${agentStats?.total || 0} total`}
          trend={agentStats?.online ? "up" : "neutral"}
        />
        <StatCard
          title="Compliance Score"
          value="--"
          icon={Shield}
          change="Pending audit"
          trend="neutral"
        />
        <StatCard
          title="System Status"
          value="Online"
          icon={Activity}
          change={systemInfo?.osVersion || ""}
          trend="up"
        />
      </div>

      {/* Agents Section */}
      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-white">Connected Agents</h2>
          <div className="flex items-center gap-2 text-sm">
            <span className="flex items-center gap-1 text-green-400">
              <span className="w-2 h-2 bg-green-400 rounded-full"></span>
              {agentStats?.online || 0} online
            </span>
            <span className="text-slate-500">|</span>
            <span className="flex items-center gap-1 text-slate-400">
              <span className="w-2 h-2 bg-slate-500 rounded-full"></span>
              {agentStats?.offline || 0} offline
            </span>
          </div>
        </div>

        {agents.length === 0 ? (
          <div className="text-center py-8">
            <Server className="w-12 h-12 text-slate-600 mx-auto mb-3" />
            <p className="text-slate-400 mb-2">No agents connected</p>
            <p className="text-sm text-slate-500">
              Deploy optio-agent to endpoints to start monitoring
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {agents.slice(0, 6).map((agent) => (
              <AgentCard key={agent.machineId} agent={agent} />
            ))}
          </div>
        )}

        {agents.length > 6 && (
          <div className="mt-4 text-center">
            <button className="text-optio-400 hover:text-optio-300 text-sm">
              View all {agents.length} agents
            </button>
          </div>
        )}
      </div>

      {/* Quick Actions */}
      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
        <h2 className="text-lg font-semibold text-white mb-4">Quick Actions</h2>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <button className="flex items-center justify-between p-4 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg transition-colors group">
            <div>
              <p className="text-white font-medium">New Client</p>
              <p className="text-sm text-slate-400">Add a new client profile</p>
            </div>
            <ArrowRight className="w-5 h-5 text-slate-500 group-hover:text-optio-400 transition-colors" />
          </button>
          <button className="flex items-center justify-between p-4 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg transition-colors group">
            <div>
              <p className="text-white font-medium">Generate Script</p>
              <p className="text-sm text-slate-400">Create a provisioning script</p>
            </div>
            <ArrowRight className="w-5 h-5 text-slate-500 group-hover:text-optio-400 transition-colors" />
          </button>
          <button className="flex items-center justify-between p-4 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg transition-colors group">
            <div>
              <p className="text-white font-medium">Run Audit</p>
              <p className="text-sm text-slate-400">Start a compliance check</p>
            </div>
            <ArrowRight className="w-5 h-5 text-slate-500 group-hover:text-optio-400 transition-colors" />
          </button>
        </div>
      </div>

      {/* Recent Clients */}
      <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
        <h2 className="text-lg font-semibold text-white mb-4">Recent Clients</h2>
        {clients.length === 0 ? (
          <p className="text-slate-400">No clients yet. Create your first client to get started.</p>
        ) : (
          <div className="space-y-2">
            {clients.slice(0, 5).map((client) => (
              <div
                key={client.id}
                className="flex items-center justify-between p-3 bg-slate-700/30 rounded-lg"
              >
                <div>
                  <p className="text-white font-medium">{client.name}</p>
                  <p className="text-sm text-slate-400">{client.targetSubnet || "No subnet configured"}</p>
                </div>
                <span className="text-xs text-slate-500">
                  {new Date(client.createdAt).toLocaleDateString()}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
