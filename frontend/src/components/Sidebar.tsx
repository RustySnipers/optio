import { cn } from "@/lib/utils";
import type { ViewMode } from "@/types";
import {
  LayoutDashboard,
  Factory,
  Users,
  Shield,
  Cloud,
  Network,
  Settings,
  Box,
} from "lucide-react";

interface SidebarProps {
  currentView: ViewMode;
  onViewChange: (view: ViewMode) => void;
}

interface NavItem {
  id: ViewMode;
  label: string;
  icon: React.ComponentType<{ className?: string }>;
  description: string;
}

const navItems: NavItem[] = [
  {
    id: "dashboard",
    label: "Dashboard",
    icon: LayoutDashboard,
    description: "Overview & metrics",
  },
  {
    id: "factory",
    label: "The Factory",
    icon: Factory,
    description: "Script generation",
  },
  {
    id: "clients",
    label: "Clients",
    icon: Users,
    description: "Client management",
  },
  {
    id: "grc",
    label: "GRC Center",
    icon: Shield,
    description: "Compliance & audit",
  },
  {
    id: "infrastructure",
    label: "Infrastructure",
    icon: Cloud,
    description: "Cloud & K8s migration",
  },
  {
    id: "network",
    label: "Network Intel",
    icon: Network,
    description: "Discovery & scanning",
  },
];

export function Sidebar({ currentView, onViewChange }: SidebarProps) {
  return (
    <aside className="w-64 bg-slate-950 border-r border-slate-800 flex flex-col">
      {/* Logo */}
      <div className="p-6 border-b border-slate-800">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-lg bg-gradient-to-br from-optio-500 to-optio-700 flex items-center justify-center">
            <Box className="w-6 h-6 text-white" />
          </div>
          <div>
            <h1 className="text-xl font-bold text-white">Optio</h1>
            <p className="text-xs text-slate-500">Consultant-in-a-Box</p>
          </div>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 p-4 space-y-1">
        {navItems.map((item) => {
          const Icon = item.icon;
          const isActive = currentView === item.id;

          return (
            <button
              key={item.id}
              onClick={() => onViewChange(item.id)}
              className={cn(
                "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all duration-150",
                "hover:bg-slate-800/50 group",
                isActive && "bg-optio-600/20 border border-optio-500/30"
              )}
            >
              <Icon
                className={cn(
                  "w-5 h-5 transition-colors",
                  isActive ? "text-optio-400" : "text-slate-500 group-hover:text-slate-300"
                )}
              />
              <div className="text-left">
                <div
                  className={cn(
                    "text-sm font-medium transition-colors",
                    isActive ? "text-optio-300" : "text-slate-300 group-hover:text-white"
                  )}
                >
                  {item.label}
                </div>
                <div className="text-xs text-slate-600">{item.description}</div>
              </div>
            </button>
          );
        })}
      </nav>

      {/* Settings at bottom */}
      <div className="p-4 border-t border-slate-800">
        <button
          onClick={() => onViewChange("settings")}
          className={cn(
            "w-full flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all",
            "hover:bg-slate-800/50 group",
            currentView === "settings" && "bg-optio-600/20 border border-optio-500/30"
          )}
        >
          <Settings
            className={cn(
              "w-5 h-5",
              currentView === "settings"
                ? "text-optio-400"
                : "text-slate-500 group-hover:text-slate-300"
            )}
          />
          <span
            className={cn(
              "text-sm font-medium",
              currentView === "settings"
                ? "text-optio-300"
                : "text-slate-300 group-hover:text-white"
            )}
          >
            Settings
          </span>
        </button>
      </div>
    </aside>
  );
}
