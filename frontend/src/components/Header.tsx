import { Wifi, WifiOff, Globe } from "lucide-react";
import type { SystemInfo } from "@/types";

interface HeaderProps {
  systemInfo: SystemInfo | null;
}

export function Header({ systemInfo }: HeaderProps) {
  return (
    <header className="h-14 bg-slate-950 border-b border-slate-800 flex items-center justify-between px-6">
      <div className="flex items-center gap-4">
        <span className="text-sm text-slate-400">
          Enterprise Architecture & Security Toolkit
        </span>
      </div>

      <div className="flex items-center gap-4">
        {/* Connection Status */}
        <div className="flex items-center gap-2 text-sm">
          {systemInfo?.localIp ? (
            <>
              <Wifi className="w-4 h-4 text-secure" />
              <span className="text-slate-300">{systemInfo.localIp}</span>
            </>
          ) : (
            <>
              <WifiOff className="w-4 h-4 text-slate-500" />
              <span className="text-slate-500">Offline</span>
            </>
          )}
        </div>

        {/* System Info */}
        {systemInfo && (
          <div className="flex items-center gap-2 text-sm text-slate-500 border-l border-slate-700 pl-4">
            <Globe className="w-4 h-4" />
            <span>{systemInfo.hostname}</span>
            <span className="text-slate-700">|</span>
            <span>v{systemInfo.appVersion}</span>
          </div>
        )}
      </div>
    </header>
  );
}
