/**
 * Network Intelligence Component
 *
 * Network discovery, scanning, and asset inventory management.
 * Features Nmap integration for security assessments.
 */

import { useState, useEffect } from "react";
import {
  checkNmap,
  getScanTypeList,
  getDemoAssets,
  getCommonPortList,
  validateScanTarget,
  previewScanCommand,
} from "@/lib/commands";
import type {
  NmapInfo,
  ScanTypeInfo,
  Asset,
  CommonPort,
  TargetValidation,
} from "@/types";

type NetworkTab = "scanner" | "assets" | "ports";

export function NetworkIntelligence() {
  const [activeTab, setActiveTab] = useState<NetworkTab>("assets");
  const [nmapInfo, setNmapInfo] = useState<NmapInfo | null>(null);

  useEffect(() => {
    checkNmap()
      .then(setNmapInfo)
      .catch(() => setNmapInfo({ installed: false, version: null, path: null }));
  }, []);

  return (
    <div className="p-8">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-white mb-2">
          Network Intelligence
        </h2>
        <p className="text-slate-400">
          Network discovery, scanning, and asset inventory management
        </p>
      </div>

      {/* Nmap Status Banner */}
      <div className={`mb-6 p-4 rounded-lg border ${
        nmapInfo?.installed
          ? "bg-green-900/20 border-green-700"
          : "bg-yellow-900/20 border-yellow-700"
      }`}>
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-3">
            <div className={`w-3 h-3 rounded-full ${
              nmapInfo?.installed ? "bg-green-500" : "bg-yellow-500"
            }`} />
            <span className="text-white font-medium">
              {nmapInfo?.installed
                ? `Nmap ${nmapInfo.version || ""} installed`
                : "Nmap not detected"}
            </span>
          </div>
          {nmapInfo?.path && (
            <span className="text-slate-400 text-sm font-mono">
              {nmapInfo.path}
            </span>
          )}
          {!nmapInfo?.installed && (
            <span className="text-yellow-400 text-sm">
              Install Nmap for full scanning capabilities
            </span>
          )}
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-slate-800 p-1 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab("assets")}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === "assets"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          }`}
        >
          Asset Inventory
        </button>
        <button
          onClick={() => setActiveTab("scanner")}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === "scanner"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          }`}
        >
          Network Scanner
        </button>
        <button
          onClick={() => setActiveTab("ports")}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === "ports"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          }`}
        >
          Port Reference
        </button>
      </div>

      {/* Tab Content */}
      {activeTab === "assets" && <AssetInventoryTab />}
      {activeTab === "scanner" && <NetworkScannerTab nmapInstalled={nmapInfo?.installed ?? false} />}
      {activeTab === "ports" && <PortReferenceTab />}
    </div>
  );
}

// ============================================================================
// Asset Inventory Tab
// ============================================================================

function AssetInventoryTab() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [loading, setLoading] = useState(true);
  const [filterCategory, setFilterCategory] = useState<string>("all");
  const [filterCriticality, setFilterCriticality] = useState<string>("all");
  const [searchTerm, setSearchTerm] = useState("");

  useEffect(() => {
    loadAssets();
  }, []);

  const loadAssets = async () => {
    try {
      const data = await getDemoAssets("demo-client");
      setAssets(data);
    } catch (err) {
      console.error("Failed to load assets:", err);
    } finally {
      setLoading(false);
    }
  };

  const categories = [
    { id: "all", name: "All Categories" },
    { id: "server", name: "Servers" },
    { id: "workstation", name: "Workstations" },
    { id: "network_device", name: "Network Devices" },
    { id: "security_device", name: "Security Devices" },
    { id: "printer", name: "Printers" },
    { id: "virtual", name: "Virtual Machines" },
  ];

  const criticalities = [
    { id: "all", name: "All Criticality" },
    { id: "critical", name: "Critical" },
    { id: "high", name: "High" },
    { id: "medium", name: "Medium" },
    { id: "low", name: "Low" },
  ];

  const criticalityColors: Record<string, string> = {
    critical: "bg-red-600",
    high: "bg-orange-600",
    medium: "bg-yellow-600",
    low: "bg-blue-600",
    informational: "bg-slate-600",
  };

  const categoryIcons: Record<string, string> = {
    server: "S",
    workstation: "W",
    network_device: "N",
    security_device: "F",
    printer: "P",
    virtual: "V",
    cloud: "C",
    unknown: "?",
  };

  const filteredAssets = assets.filter((asset) => {
    const categoryMatch = filterCategory === "all" || asset.category === filterCategory;
    const criticalityMatch = filterCriticality === "all" || asset.criticality === filterCriticality;
    const searchMatch = searchTerm === "" ||
      asset.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      asset.ipAddress.includes(searchTerm);
    return categoryMatch && criticalityMatch && searchMatch;
  });

  // Stats
  const totalAssets = assets.length;
  const criticalAssets = assets.filter(a => a.criticality === "critical").length;
  const servers = assets.filter(a => a.category === "server").length;
  const activeAssets = assets.filter(a => a.status === "active").length;

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Stats Overview */}
      <div className="grid grid-cols-4 gap-4">
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
          <div className="text-3xl font-bold text-white">{totalAssets}</div>
          <div className="text-sm text-slate-400">Total Assets</div>
        </div>
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
          <div className="text-3xl font-bold text-green-400">{activeAssets}</div>
          <div className="text-sm text-slate-400">Active</div>
        </div>
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
          <div className="text-3xl font-bold text-red-400">{criticalAssets}</div>
          <div className="text-sm text-slate-400">Critical</div>
        </div>
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
          <div className="text-3xl font-bold text-blue-400">{servers}</div>
          <div className="text-sm text-slate-400">Servers</div>
        </div>
      </div>

      {/* Filters */}
      <div className="flex items-center space-x-4">
        <input
          type="text"
          placeholder="Search by name or IP..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          className="flex-1 bg-slate-800 border border-slate-700 rounded-md px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
        />
        <select
          value={filterCategory}
          onChange={(e) => setFilterCategory(e.target.value)}
          className="bg-slate-800 border border-slate-700 rounded-md px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {categories.map((cat) => (
            <option key={cat.id} value={cat.id}>
              {cat.name}
            </option>
          ))}
        </select>
        <select
          value={filterCriticality}
          onChange={(e) => setFilterCriticality(e.target.value)}
          className="bg-slate-800 border border-slate-700 rounded-md px-3 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          {criticalities.map((crit) => (
            <option key={crit.id} value={crit.id}>
              {crit.name}
            </option>
          ))}
        </select>
      </div>

      {/* Asset List */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg overflow-hidden">
        <table className="w-full">
          <thead className="bg-slate-900">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                Asset
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                IP Address
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                Category
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                OS
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                Criticality
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                Services
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700">
            {filteredAssets.map((asset) => (
              <tr key={asset.id} className="hover:bg-slate-700/50 transition-colors">
                <td className="px-4 py-3">
                  <div className="flex items-center space-x-3">
                    <div className={`w-8 h-8 rounded-lg flex items-center justify-center text-xs font-bold ${
                      asset.status === "active" ? "bg-green-900 text-green-300" : "bg-slate-700 text-slate-400"
                    }`}>
                      {categoryIcons[asset.category] || "?"}
                    </div>
                    <div>
                      <div className="text-white font-medium">{asset.name}</div>
                      {asset.owner && (
                        <div className="text-xs text-slate-500">{asset.owner}</div>
                      )}
                    </div>
                  </div>
                </td>
                <td className="px-4 py-3">
                  <span className="font-mono text-slate-300">{asset.ipAddress}</span>
                </td>
                <td className="px-4 py-3">
                  <span className="text-slate-300 capitalize">
                    {asset.category.replace("_", " ")}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <span className="text-slate-400 text-sm">
                    {asset.operatingSystem || "Unknown"}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <span className={`px-2 py-1 rounded text-xs font-medium text-white ${
                    criticalityColors[asset.criticality] || "bg-slate-600"
                  }`}>
                    {asset.criticality}
                  </span>
                </td>
                <td className="px-4 py-3">
                  <div className="flex flex-wrap gap-1">
                    {asset.services.slice(0, 3).map((service, i) => (
                      <span
                        key={i}
                        className="px-1.5 py-0.5 bg-slate-700 rounded text-xs text-slate-300"
                      >
                        {service.port}/{service.name}
                      </span>
                    ))}
                    {asset.services.length > 3 && (
                      <span className="px-1.5 py-0.5 bg-slate-700 rounded text-xs text-slate-400">
                        +{asset.services.length - 3}
                      </span>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {filteredAssets.length === 0 && (
          <div className="p-8 text-center text-slate-500">
            No assets found matching your filters
          </div>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// Network Scanner Tab
// ============================================================================

function NetworkScannerTab({ nmapInstalled }: { nmapInstalled: boolean }) {
  const [scanTypes, setScanTypes] = useState<ScanTypeInfo[]>([]);
  const [selectedScanType, setSelectedScanType] = useState<string>("quick_scan");
  const [target, setTarget] = useState("");
  const [targetValidation, setTargetValidation] = useState<TargetValidation | null>(null);
  const [commandPreview, setCommandPreview] = useState<string>("");
  const [ports, setPorts] = useState("");
  const [aggressive, setAggressive] = useState(false);

  useEffect(() => {
    getScanTypeList()
      .then(setScanTypes)
      .catch(console.error);
  }, []);

  useEffect(() => {
    if (target) {
      const timeoutId = setTimeout(() => {
        validateScanTarget(target)
          .then(setTargetValidation)
          .catch(console.error);
      }, 300);
      return () => clearTimeout(timeoutId);
    } else {
      setTargetValidation(null);
    }
  }, [target]);

  useEffect(() => {
    if (target && targetValidation?.valid) {
      previewScanCommand([target], selectedScanType, ports || undefined, aggressive)
        .then(setCommandPreview)
        .catch(console.error);
    } else {
      setCommandPreview("");
    }
  }, [target, selectedScanType, ports, aggressive, targetValidation]);

  if (!nmapInstalled) {
    return (
      <div className="bg-slate-800 border border-slate-700 rounded-lg p-8 text-center">
        <div className="text-6xl mb-4">🔍</div>
        <h3 className="text-xl font-semibold text-white mb-2">
          Nmap Required
        </h3>
        <p className="text-slate-400 mb-4">
          Network scanning requires Nmap to be installed on your system.
        </p>
        <div className="bg-slate-900 rounded-lg p-4 inline-block">
          <code className="text-green-400">
            # Ubuntu/Debian<br />
            sudo apt install nmap<br /><br />
            # macOS<br />
            brew install nmap<br /><br />
            # Windows<br />
            choco install nmap
          </code>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Scan Configuration */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Scan Configuration</h3>

        <div className="grid grid-cols-2 gap-6">
          {/* Left Column */}
          <div className="space-y-4">
            {/* Target Input */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Target
              </label>
              <input
                type="text"
                value={target}
                onChange={(e) => setTarget(e.target.value)}
                placeholder="192.168.1.0/24 or hostname"
                className="w-full bg-slate-900 border border-slate-600 rounded-md px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
              {targetValidation && (
                <div className={`mt-1 text-sm ${targetValidation.valid ? "text-green-400" : "text-red-400"}`}>
                  {targetValidation.valid
                    ? `Valid ${targetValidation.targetType}`
                    : targetValidation.error}
                </div>
              )}
            </div>

            {/* Scan Type */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Scan Type
              </label>
              <select
                value={selectedScanType}
                onChange={(e) => setSelectedScanType(e.target.value)}
                className="w-full bg-slate-900 border border-slate-600 rounded-md px-4 py-2 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {scanTypes.map((type) => (
                  <option key={type.scanType} value={type.scanType}>
                    {type.name} - {type.duration}
                  </option>
                ))}
              </select>
            </div>

            {/* Port Specification */}
            <div>
              <label className="block text-sm font-medium text-slate-300 mb-2">
                Ports (optional)
              </label>
              <input
                type="text"
                value={ports}
                onChange={(e) => setPorts(e.target.value)}
                placeholder="22,80,443 or 1-1000"
                className="w-full bg-slate-900 border border-slate-600 rounded-md px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
              />
            </div>

            {/* Options */}
            <div className="flex items-center space-x-4">
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={aggressive}
                  onChange={(e) => setAggressive(e.target.checked)}
                  className="w-4 h-4 rounded border-slate-600 bg-slate-900 text-blue-500 focus:ring-blue-500"
                />
                <span className="text-slate-300 text-sm">Aggressive Timing (faster but noisier)</span>
              </label>
            </div>
          </div>

          {/* Right Column - Scan Type Info */}
          <div className="bg-slate-900 rounded-lg p-4">
            <h4 className="text-white font-medium mb-2">
              {scanTypes.find(t => t.scanType === selectedScanType)?.name || "Scan"}
            </h4>
            <p className="text-slate-400 text-sm mb-3">
              {scanTypes.find(t => t.scanType === selectedScanType)?.description}
            </p>
            <div className="flex items-center space-x-4 text-sm">
              <span className="text-slate-500">
                Duration: {scanTypes.find(t => t.scanType === selectedScanType)?.duration}
              </span>
              {scanTypes.find(t => t.scanType === selectedScanType)?.requiresRoot && (
                <span className="text-yellow-400">Requires sudo/admin</span>
              )}
            </div>
          </div>
        </div>

        {/* Command Preview */}
        {commandPreview && (
          <div className="mt-6">
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Command Preview
            </label>
            <div className="bg-slate-900 rounded-lg p-4 font-mono text-sm text-green-400 overflow-x-auto">
              {commandPreview}
            </div>
          </div>
        )}

        {/* Scan Button */}
        <div className="mt-6 flex justify-end">
          <button
            disabled={!target || !targetValidation?.valid}
            className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-700 disabled:cursor-not-allowed text-white font-medium rounded-md transition-colors"
          >
            Start Scan
          </button>
        </div>
      </div>

      {/* Scan Types Reference */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Available Scan Types</h3>
        <div className="grid grid-cols-2 gap-4">
          {scanTypes.map((type) => (
            <div
              key={type.scanType}
              className="bg-slate-900 border border-slate-700 rounded-lg p-4"
            >
              <div className="flex items-center justify-between mb-2">
                <h4 className="text-white font-medium">{type.name}</h4>
                <span className="text-xs text-slate-400">{type.duration}</span>
              </div>
              <p className="text-slate-400 text-sm">{type.description}</p>
              {type.requiresRoot && (
                <div className="mt-2 text-xs text-yellow-400">Requires elevated privileges</div>
              )}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

// ============================================================================
// Port Reference Tab
// ============================================================================

function PortReferenceTab() {
  const [ports, setPorts] = useState<CommonPort[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState("");

  useEffect(() => {
    getCommonPortList()
      .then(setPorts)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const filteredPorts = ports.filter((port) =>
    searchTerm === "" ||
    port.port.toString().includes(searchTerm) ||
    port.service.toLowerCase().includes(searchTerm.toLowerCase()) ||
    port.description.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Search */}
      <input
        type="text"
        placeholder="Search ports, services, or descriptions..."
        value={searchTerm}
        onChange={(e) => setSearchTerm(e.target.value)}
        className="w-full bg-slate-800 border border-slate-700 rounded-md px-4 py-2 text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />

      {/* Port List */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg overflow-hidden">
        <table className="w-full">
          <thead className="bg-slate-900">
            <tr>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider w-24">
                Port
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider w-32">
                Service
              </th>
              <th className="px-4 py-3 text-left text-xs font-medium text-slate-400 uppercase tracking-wider">
                Description
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-700">
            {filteredPorts.map((port) => (
              <tr key={port.port} className="hover:bg-slate-700/50 transition-colors">
                <td className="px-4 py-3">
                  <span className="font-mono text-blue-400 font-medium">{port.port}</span>
                </td>
                <td className="px-4 py-3">
                  <span className="text-white">{port.service}</span>
                </td>
                <td className="px-4 py-3">
                  <span className="text-slate-400">{port.description}</span>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {filteredPorts.length === 0 && (
          <div className="p-8 text-center text-slate-500">
            No ports found matching "{searchTerm}"
          </div>
        )}
      </div>

      {/* Quick Reference */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Port Ranges</h3>
        <div className="grid grid-cols-3 gap-4">
          <div className="bg-slate-900 rounded-lg p-4">
            <h4 className="text-white font-medium mb-2">Well-Known Ports</h4>
            <p className="text-slate-400 text-sm">0 - 1023</p>
            <p className="text-xs text-slate-500 mt-1">Reserved for privileged services</p>
          </div>
          <div className="bg-slate-900 rounded-lg p-4">
            <h4 className="text-white font-medium mb-2">Registered Ports</h4>
            <p className="text-slate-400 text-sm">1024 - 49151</p>
            <p className="text-xs text-slate-500 mt-1">Assigned by IANA for specific services</p>
          </div>
          <div className="bg-slate-900 rounded-lg p-4">
            <h4 className="text-white font-medium mb-2">Dynamic Ports</h4>
            <p className="text-slate-400 text-sm">49152 - 65535</p>
            <p className="text-xs text-slate-500 mt-1">Available for temporary connections</p>
          </div>
        </div>
      </div>
    </div>
  );
}

export default NetworkIntelligence;
