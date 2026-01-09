import { useState, useEffect } from "react";
import { listClients, getSystemInfo } from "@/lib/commands";
import type { Client, SystemInfo } from "@/types";
import {
  Users,
  FileCode,
  Shield,
  Activity,
  ArrowRight,
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

export function Dashboard() {
  const [clients, setClients] = useState<Client[]>([]);
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function loadData() {
      try {
        const [clientData, sysInfo] = await Promise.all([
          listClients(),
          getSystemInfo(),
        ]);
        setClients(clientData);
        setSystemInfo(sysInfo);
      } catch (error) {
        console.error("Failed to load dashboard data:", error);
      } finally {
        setIsLoading(false);
      }
    }
    loadData();
  }, []);

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
          title="Scripts Generated"
          value={0}
          icon={FileCode}
          change="0 today"
          trend="neutral"
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
