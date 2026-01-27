import { useState, useEffect, useCallback } from "react";
import { listClients, getSystemInfo, getAgents, getAgentStats, refreshAgentStatus, generateAgentInstaller, getAgentHistory } from "@/lib/commands";
import type { Client, SystemInfo, Agent, AgentStats, TelemetryRecord, GenerateInstallerRequest } from "@/types";
import { AgentDetailModal } from "@/components/AgentDetailModal";
import {
  Users,
  Shield,
  Activity,
  ArrowRight,
  Server,
  Cpu,
  HardDrive,
  Wifi,
  WifiOff,
  Package,
  X,
  ExternalLink,
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
              className={`text-sm mt-2 ${trend === "up"
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

// Simple sparkline chart for telemetry data
function TelemetrySparkline({ data, color = "optio" }: { data: number[]; color?: string }) {
  if (data.length < 2) return null;

  const max = Math.max(...data, 100);
  const min = Math.min(...data, 0);
  const range = max - min || 1;
  const height = 24;
  const width = 80;
  const points = data.map((v, i) => {
    const x = (i / (data.length - 1)) * width;
    const y = height - ((v - min) / range) * height;
    return `${x},${y}`;
  }).join(" ");

  const colorClasses: Record<string, string> = {
    optio: "stroke-optio-400",
    green: "stroke-green-400",
    yellow: "stroke-yellow-400",
    red: "stroke-red-400",
  };

  return (
    <svg width={width} height={height} className="overflow-visible">
      <polyline
        points={points}
        fill="none"
        className={colorClasses[color] || colorClasses.optio}
        strokeWidth="1.5"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function AgentCard({ agent, telemetry, onClick }: { agent: Agent; telemetry?: TelemetryRecord[]; onClick?: () => void }) {
  const cpuData = telemetry?.slice(0, 20).reverse().map(t => t.cpuPercent) || [];
  const ramData = telemetry?.slice(0, 20).reverse().map(t => t.ramPercent) || [];

  return (
    <div
      onClick={onClick}
      className="bg-slate-700/30 rounded-lg p-4 space-y-3 cursor-pointer hover:bg-slate-700/50 transition-colors group"
    >
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
            <p className="text-white font-medium flex items-center gap-2">
              {agent.hostname}
              <ExternalLink className="w-3 h-3 text-slate-500 opacity-0 group-hover:opacity-100 transition-opacity" />
            </p>
            <p className="text-xs text-slate-500 font-mono">{agent.machineId.slice(0, 8)}...</p>
          </div>
        </div>
        <AgentStatusBadge status={agent.status} />
      </div>

      <div className="grid grid-cols-2 gap-2 text-sm">
        <div className="flex items-center justify-between text-slate-400">
          <div className="flex items-center gap-2">
            <Cpu className="w-4 h-4" />
            <span>{agent.cpuUsage.toFixed(1)}%</span>
          </div>
          {cpuData.length > 1 && <TelemetrySparkline data={cpuData} color={agent.cpuUsage > 80 ? "red" : "optio"} />}
        </div>
        <div className="flex items-center justify-between text-slate-400">
          <div className="flex items-center gap-2">
            <HardDrive className="w-4 h-4" />
            <span>{agent.ramUsage.toFixed(1)}%</span>
          </div>
          {ramData.length > 1 && <TelemetrySparkline data={ramData} color={agent.ramUsage > 80 ? "yellow" : "green"} />}
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

// Installer Modal Component
function InstallerModal({ isOpen, onClose }: { isOpen: boolean; onClose: () => void }) {
  const [clientName, setClientName] = useState("");
  const [hubAddress, setHubAddress] = useState("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [result, setResult] = useState<{ success: boolean; path?: string; agentId?: string; warnings?: string[] } | null>(null);

  const handleGenerate = async () => {
    if (!clientName.trim()) return;

    setIsGenerating(true);
    setResult(null);

    try {
      const request: GenerateInstallerRequest = {
        clientName: clientName.trim(),
        hubAddress: hubAddress.trim() || undefined,
      };

      const response = await generateAgentInstaller(request);
      setResult({
        success: response.success,
        path: response.installerPath,
        agentId: response.agentId,
        warnings: response.warnings,
      });
    } catch (error) {
      setResult({
        success: false,
        warnings: [String(error)],
      });
    } finally {
      setIsGenerating(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 border border-slate-700 rounded-xl p-6 w-full max-w-md">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Generate Agent Installer</h3>
          <button onClick={onClose} className="text-slate-400 hover:text-white">
            <X className="w-5 h-5" />
          </button>
        </div>

        {!result ? (
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-slate-400 mb-1">Client Name *</label>
              <input
                type="text"
                value={clientName}
                onChange={(e) => setClientName(e.target.value)}
                placeholder="e.g., Acme Corp"
                className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-optio-500"
              />
            </div>

            <div>
              <label className="block text-sm text-slate-400 mb-1">Hub Address (optional)</label>
              <input
                type="text"
                value={hubAddress}
                onChange={(e) => setHubAddress(e.target.value)}
                placeholder="e.g., 192.168.1.100:50051"
                className="w-full px-3 py-2 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-optio-500"
              />
              <p className="text-xs text-slate-500 mt-1">Leave empty to auto-detect</p>
            </div>

            <button
              onClick={handleGenerate}
              disabled={!clientName.trim() || isGenerating}
              className="w-full py-2 bg-optio-600 hover:bg-optio-500 disabled:bg-slate-600 disabled:cursor-not-allowed text-white rounded-lg flex items-center justify-center gap-2"
            >
              {isGenerating ? (
                <>
                  <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin"></div>
                  Generating...
                </>
              ) : (
                <>
                  <Package className="w-4 h-4" />
                  Generate Installer
                </>
              )}
            </button>
          </div>
        ) : (
          <div className="space-y-4">
            {result.success ? (
              <>
                <div className="p-3 bg-green-500/20 border border-green-500/30 rounded-lg">
                  <p className="text-green-400 font-medium">Installer Generated Successfully!</p>
                </div>

                <div className="space-y-2 text-sm">
                  <div>
                    <span className="text-slate-400">Agent ID:</span>
                    <span className="text-white ml-2 font-mono">{result.agentId?.slice(0, 8)}...</span>
                  </div>
                  <div>
                    <span className="text-slate-400">Path:</span>
                    <p className="text-white text-xs font-mono mt-1 break-all bg-slate-700/50 p-2 rounded">
                      {result.path}
                    </p>
                  </div>
                </div>

                {result.warnings && result.warnings.length > 0 && (
                  <div className="p-3 bg-yellow-500/20 border border-yellow-500/30 rounded-lg">
                    <p className="text-yellow-400 text-sm font-medium mb-1">Warnings:</p>
                    {result.warnings.map((w, i) => (
                      <p key={i} className="text-yellow-300 text-xs">{w}</p>
                    ))}
                  </div>
                )}
              </>
            ) : (
              <div className="p-3 bg-red-500/20 border border-red-500/30 rounded-lg">
                <p className="text-red-400 font-medium">Generation Failed</p>
                {result.warnings?.map((w, i) => (
                  <p key={i} className="text-red-300 text-sm mt-1">{w}</p>
                ))}
              </div>
            )}

            <button
              onClick={() => {
                setResult(null);
                setClientName("");
                setHubAddress("");
              }}
              className="w-full py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg"
            >
              Generate Another
            </button>
          </div>
        )}
      </div>
    </div>
  );
}

export function Dashboard() {
  const [clients, setClients] = useState<Client[]>([]);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [agentStats, setAgentStats] = useState<AgentStats | null>(null);
  const [agentTelemetry, setAgentTelemetry] = useState<Record<string, TelemetryRecord[]>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [showInstallerModal, setShowInstallerModal] = useState(false);
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);

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

      // Load telemetry for online agents
      const telemetryData: Record<string, TelemetryRecord[]> = {};
      for (const agent of agentData.filter(a => a.status === "online").slice(0, 6)) {
        try {
          const history = await getAgentHistory(agent.machineId, 20);
          telemetryData[agent.machineId] = history;
        } catch {
          // Ignore errors loading individual agent telemetry
        }
      }
      setAgentTelemetry(telemetryData);
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

  // Poll for agents every 2 seconds (near real-time heartbeat tracking)
  // Full Tauri event push would require AppHandle sharing with gRPC thread
  useEffect(() => {
    const interval = setInterval(loadAgents, 2000);
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
              <AgentCard
                key={agent.machineId}
                agent={agent}
                telemetry={agentTelemetry[agent.machineId]}
                onClick={() => setSelectedAgent(agent)}
              />
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
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <button className="flex items-center justify-between p-4 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg transition-colors group">
            <div className="text-left">
              <p className="text-white font-medium">New Client</p>
              <p className="text-sm text-slate-400">Add a new client profile</p>
            </div>
            <ArrowRight className="w-5 h-5 text-slate-500 group-hover:text-optio-400 transition-colors" />
          </button>
          <button
            onClick={() => setShowInstallerModal(true)}
            className="flex items-center justify-between p-4 bg-optio-600/20 hover:bg-optio-600/30 border border-optio-500/30 rounded-lg transition-colors group"
          >
            <div className="text-left">
              <p className="text-white font-medium">Generate Installer</p>
              <p className="text-sm text-slate-400">Create agent deployment package</p>
            </div>
            <Package className="w-5 h-5 text-optio-400 group-hover:text-optio-300 transition-colors" />
          </button>
          <button className="flex items-center justify-between p-4 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg transition-colors group">
            <div className="text-left">
              <p className="text-white font-medium">Generate Script</p>
              <p className="text-sm text-slate-400">Create a provisioning script</p>
            </div>
            <ArrowRight className="w-5 h-5 text-slate-500 group-hover:text-optio-400 transition-colors" />
          </button>
          <button className="flex items-center justify-between p-4 bg-slate-700/30 hover:bg-slate-700/50 rounded-lg transition-colors group">
            <div className="text-left">
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

      {/* Installer Modal */}
      <InstallerModal
        isOpen={showInstallerModal}
        onClose={() => setShowInstallerModal(false)}
      />

      {/* Agent Detail Modal */}
      {selectedAgent && (
        <AgentDetailModal
          agent={selectedAgent}
          isOpen={!!selectedAgent}
          onClose={() => setSelectedAgent(null)}
        />
      )}
    </div>
  );
}
