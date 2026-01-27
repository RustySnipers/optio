import { useState, useEffect } from "react";
import {
    Settings as SettingsIcon,
    Monitor,
    Shield,
    Database,
    Info,
    RefreshCw,
    HardDrive,
    Cpu,
    User,
    Globe,
} from "lucide-react";
import { getSystemInfo } from "@/lib/commands";
import type { SystemInfo } from "@/types";

export function Settings() {
    const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        loadSystemInfo();
    }, []);

    const loadSystemInfo = async () => {
        setLoading(true);
        try {
            const info = await getSystemInfo();
            setSystemInfo(info);
        } catch (error) {
            console.error("Failed to load system info:", error);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="h-full flex flex-col p-6">
            <div className="flex items-center gap-3 mb-8">
                <div className="p-2 bg-slate-700/50 rounded-lg">
                    <SettingsIcon className="w-6 h-6 text-slate-300" />
                </div>
                <div>
                    <h1 className="text-2xl font-bold text-white">Settings</h1>
                    <p className="text-slate-400">Application configuration and system status</p>
                </div>
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
                {/* System Information Card */}
                <div className="bg-slate-800 border border-slate-700 rounded-xl overflow-hidden">
                    <div className="p-4 border-b border-slate-700 bg-slate-900/50 flex items-center justify-between">
                        <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                            <Monitor className="w-5 h-5 text-blue-400" />
                            System Information
                        </h2>
                        <button
                            onClick={loadSystemInfo}
                            className="p-1.5 hover:bg-slate-700 rounded text-slate-400 hover:text-white transition-colors"
                            title="Refresh"
                        >
                            <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                        </button>
                    </div>

                    <div className="p-6 space-y-4">
                        <div className="grid grid-cols-2 gap-4">
                            <div className="p-4 bg-slate-900/50 rounded-lg">
                                <div className="flex items-center gap-2 text-slate-400 mb-1">
                                    <User className="w-4 h-4" />
                                    <span className="text-xs uppercase tracking-wider">User</span>
                                </div>
                                <div className="font-mono text-white truncate">
                                    {systemInfo?.username || "—"}
                                </div>
                            </div>

                            <div className="p-4 bg-slate-900/50 rounded-lg">
                                <div className="flex items-center gap-2 text-slate-400 mb-1">
                                    <Monitor className="w-4 h-4" />
                                    <span className="text-xs uppercase tracking-wider">Hostname</span>
                                </div>
                                <div className="font-mono text-white truncate">
                                    {systemInfo?.hostname || "—"}
                                </div>
                            </div>

                            <div className="p-4 bg-slate-900/50 rounded-lg">
                                <div className="flex items-center gap-2 text-slate-400 mb-1">
                                    <HardDrive className="w-4 h-4" />
                                    <span className="text-xs uppercase tracking-wider">OS Name</span>
                                </div>
                                <div className="font-medium text-white truncate">
                                    {systemInfo?.osName || "—"}
                                </div>
                            </div>

                            <div className="p-4 bg-slate-900/50 rounded-lg">
                                <div className="flex items-center gap-2 text-slate-400 mb-1">
                                    <Cpu className="w-4 h-4" />
                                    <span className="text-xs uppercase tracking-wider">OS Version</span>
                                </div>
                                <div className="font-medium text-white truncate">
                                    {systemInfo?.osVersion || "—"}
                                </div>
                            </div>
                        </div>

                        <div className="p-4 bg-slate-900/50 rounded-lg">
                            <div className="flex items-center gap-2 text-slate-400 mb-1">
                                <Globe className="w-4 h-4" />
                                <span className="text-xs uppercase tracking-wider">Network Address</span>
                            </div>
                            <div className="flex items-center gap-2">
                                <div className={`w-2 h-2 rounded-full ${systemInfo?.localIp ? 'bg-green-500' : 'bg-red-500'}`} />
                                <code className="text-white font-mono">
                                    {systemInfo?.localIp || "Not Connected"}
                                </code>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Application Info */}
                <div className="space-y-6">
                    <div className="bg-slate-800 border border-slate-700 rounded-xl overflow-hidden lead-snug">
                        <div className="p-4 border-b border-slate-700 bg-slate-900/50">
                            <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                                <Shield className="w-5 h-5 text-purple-400" />
                                Application Details
                            </h2>
                        </div>
                        <div className="p-6">
                            <div className="flex items-center justify-between mb-4">
                                <span className="text-slate-400">Version</span>
                                <span className="text-white font-mono bg-slate-700 px-2 py-0.5 rounded text-sm">
                                    {systemInfo?.appVersion || "v0.0.0"}
                                </span>
                            </div>
                            <div className="flex items-center justify-between mb-4">
                                <span className="text-slate-400">Environment</span>
                                <span className="text-green-400 font-medium text-sm flex items-center gap-1">
                                    <span className="w-1.5 h-1.5 rounded-full bg-green-500"></span>
                                    Development
                                </span>
                            </div>
                            <div className="flex items-center justify-between">
                                <span className="text-slate-400">Database Status</span>
                                <span className="text-white text-sm">Connected (optio.db)</span>
                            </div>
                        </div>
                    </div>

                    <div className="bg-slate-800 border border-slate-700 rounded-xl overflow-hidden">
                        <div className="p-4 border-b border-slate-700 bg-slate-900/50">
                            <h2 className="text-lg font-semibold text-white flex items-center gap-2">
                                <Database className="w-5 h-5 text-orange-400" />
                                Data Management
                            </h2>
                        </div>
                        <div className="p-6">
                            <div className="bg-blue-900/20 border border-blue-900/50 rounded-lg p-4 flex gap-3">
                                <Info className="w-5 h-5 text-blue-400 shrink-0" />
                                <p className="text-sm text-blue-200">
                                    Data management features such as database backup, restore, and
                                    reset will be available in the next release.
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div className="mt-auto text-center pt-8 text-slate-600 text-sm">
                <p>Optio - Enterprise Architecture & Security Toolkit</p>
                <p>© 2026 RustySnipers. All rights reserved.</p>
            </div>
        </div>
    );
}
