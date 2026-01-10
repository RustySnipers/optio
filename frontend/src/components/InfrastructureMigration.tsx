/**
 * Infrastructure & Migration Component
 *
 * Comprehensive module for cloud migration planning including:
 * - Cloud Readiness Assessment
 * - Kubernetes Hardening Audit
 * - FinOps Cost Calculator
 */

import { useState, useEffect } from "react";
import {
  getCloudReadinessItems,
  getK8sHardeningChecklist,
  getK8sSeverityStats,
  getFinOpsTemplates,
  compareCloudProviders,
} from "@/lib/commands";
import type {
  CloudReadinessItem,
  K8sHardeningCheck,
  K8sSeverityStats,
  FinOpsTemplate,
  ProviderComparison,
  ResourceInput,
} from "@/types";

type InfraTab = "readiness" | "k8s" | "finops";

export function InfrastructureMigration() {
  const [activeTab, setActiveTab] = useState<InfraTab>("readiness");

  return (
    <div className="p-8">
      <div className="mb-6">
        <h2 className="text-2xl font-bold text-white mb-2">
          Infrastructure & Migration
        </h2>
        <p className="text-slate-400">
          Cloud migration planning, Kubernetes security auditing, and cost optimization
        </p>
      </div>

      {/* Tab Navigation */}
      <div className="flex space-x-1 mb-6 bg-slate-800 p-1 rounded-lg w-fit">
        <button
          onClick={() => setActiveTab("readiness")}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === "readiness"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          }`}
        >
          Cloud Readiness
        </button>
        <button
          onClick={() => setActiveTab("k8s")}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === "k8s"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          }`}
        >
          K8s Hardening
        </button>
        <button
          onClick={() => setActiveTab("finops")}
          className={`px-4 py-2 rounded-md text-sm font-medium transition-colors ${
            activeTab === "finops"
              ? "bg-blue-600 text-white"
              : "text-slate-400 hover:text-white hover:bg-slate-700"
          }`}
        >
          FinOps Calculator
        </button>
      </div>

      {/* Tab Content */}
      {activeTab === "readiness" && <CloudReadinessTab />}
      {activeTab === "k8s" && <K8sHardeningTab />}
      {activeTab === "finops" && <FinOpsTab />}
    </div>
  );
}

// ============================================================================
// Cloud Readiness Tab
// ============================================================================

function CloudReadinessTab() {
  const [items, setItems] = useState<CloudReadinessItem[]>([]);
  const [loading, setLoading] = useState(true);
  const [selectedCategory, setSelectedCategory] = useState<string>("all");

  useEffect(() => {
    loadItems();
  }, []);

  const loadItems = async () => {
    try {
      const data = await getCloudReadinessItems();
      setItems(data);
    } catch (err) {
      console.error("Failed to load readiness items:", err);
    } finally {
      setLoading(false);
    }
  };

  const categories = [
    { id: "all", name: "All Categories", color: "bg-slate-600" },
    { id: "BusinessAlignment", name: "Business Alignment", color: "bg-blue-600" },
    { id: "TechnicalReadiness", name: "Technical Readiness", color: "bg-green-600" },
    { id: "SecurityCompliance", name: "Security & Compliance", color: "bg-red-600" },
    { id: "OperationalReadiness", name: "Operational Readiness", color: "bg-yellow-600" },
    { id: "FinancialPlanning", name: "Financial Planning", color: "bg-purple-600" },
    { id: "PeopleProcess", name: "People & Process", color: "bg-pink-600" },
    { id: "DataManagement", name: "Data Management", color: "bg-cyan-600" },
  ];

  const filteredItems = selectedCategory === "all"
    ? items
    : items.filter(item => item.category === selectedCategory);

  const categoryStats = categories.slice(1).map(cat => ({
    ...cat,
    count: items.filter(i => i.category === cat.id).length,
  }));

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Category Overview */}
      <div className="grid grid-cols-7 gap-4">
        {categoryStats.map((cat) => (
          <button
            key={cat.id}
            onClick={() => setSelectedCategory(cat.id)}
            className={`p-4 rounded-lg border transition-all ${
              selectedCategory === cat.id
                ? "border-blue-500 bg-slate-700"
                : "border-slate-700 bg-slate-800 hover:border-slate-600"
            }`}
          >
            <div className={`w-3 h-3 rounded-full ${cat.color} mb-2`} />
            <div className="text-2xl font-bold text-white">{cat.count}</div>
            <div className="text-xs text-slate-400 truncate">{cat.name}</div>
          </button>
        ))}
      </div>

      {/* Filter Bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-2">
          <span className="text-slate-400 text-sm">Filter:</span>
          <select
            value={selectedCategory}
            onChange={(e) => setSelectedCategory(e.target.value)}
            className="bg-slate-800 border border-slate-700 rounded-md px-3 py-1.5 text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            {categories.map((cat) => (
              <option key={cat.id} value={cat.id}>
                {cat.name}
              </option>
            ))}
          </select>
        </div>
        <div className="text-slate-400 text-sm">
          {filteredItems.length} items
        </div>
      </div>

      {/* Checklist Items */}
      <div className="space-y-3">
        {filteredItems.map((item) => (
          <div
            key={item.id}
            className="bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-slate-600 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2 mb-1">
                  <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                    categories.find(c => c.id === item.category)?.color || "bg-slate-600"
                  } text-white`}>
                    {item.category}
                  </span>
                  <span className="text-slate-500 text-xs">
                    Priority: {item.priority}
                  </span>
                  <span className="text-slate-500 text-xs">
                    ~{item.estimatedEffortDays} days
                  </span>
                </div>
                <h4 className="text-white font-medium mb-1">{item.title}</h4>
                <p className="text-slate-400 text-sm mb-2">{item.description}</p>
                <p className="text-slate-500 text-xs">{item.guidance}</p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// K8s Hardening Tab
// ============================================================================

function K8sHardeningTab() {
  const [checks, setChecks] = useState<K8sHardeningCheck[]>([]);
  const [stats, setStats] = useState<K8sSeverityStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [selectedCategory, setSelectedCategory] = useState<string>("all");
  const [selectedSeverity, setSelectedSeverity] = useState<string>("all");

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    try {
      const [checksData, statsData] = await Promise.all([
        getK8sHardeningChecklist(),
        getK8sSeverityStats(),
      ]);
      setChecks(checksData);
      setStats(statsData);
    } catch (err) {
      console.error("Failed to load K8s data:", err);
    } finally {
      setLoading(false);
    }
  };

  const categories = [
    { id: "all", name: "All Categories" },
    { id: "PodSecurity", name: "Pod Security" },
    { id: "NetworkPolicies", name: "Network Policies" },
    { id: "Authentication", name: "Authentication" },
    { id: "Authorization", name: "Authorization" },
    { id: "Logging", name: "Logging & Audit" },
    { id: "ThreatDetection", name: "Threat Detection" },
    { id: "SupplyChain", name: "Supply Chain" },
    { id: "Secrets", name: "Secrets Management" },
  ];

  const severityColors: Record<string, string> = {
    Critical: "bg-red-600",
    High: "bg-orange-600",
    Medium: "bg-yellow-600",
    Low: "bg-blue-600",
  };

  const filteredChecks = checks.filter((check) => {
    const categoryMatch = selectedCategory === "all" || check.category === selectedCategory;
    const severityMatch = selectedSeverity === "all" || check.severity === selectedSeverity;
    return categoryMatch && severityMatch;
  });

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Severity Overview */}
      {stats && (
        <div className="grid grid-cols-5 gap-4">
          <div className="bg-slate-800 border border-slate-700 rounded-lg p-4">
            <div className="text-3xl font-bold text-white">{stats.total}</div>
            <div className="text-sm text-slate-400">Total Checks</div>
          </div>
          <button
            onClick={() => setSelectedSeverity(selectedSeverity === "Critical" ? "all" : "Critical")}
            className={`bg-slate-800 border rounded-lg p-4 transition-colors ${
              selectedSeverity === "Critical" ? "border-red-500" : "border-slate-700 hover:border-red-500/50"
            }`}
          >
            <div className="text-3xl font-bold text-red-500">{stats.critical}</div>
            <div className="text-sm text-slate-400">Critical</div>
          </button>
          <button
            onClick={() => setSelectedSeverity(selectedSeverity === "High" ? "all" : "High")}
            className={`bg-slate-800 border rounded-lg p-4 transition-colors ${
              selectedSeverity === "High" ? "border-orange-500" : "border-slate-700 hover:border-orange-500/50"
            }`}
          >
            <div className="text-3xl font-bold text-orange-500">{stats.high}</div>
            <div className="text-sm text-slate-400">High</div>
          </button>
          <button
            onClick={() => setSelectedSeverity(selectedSeverity === "Medium" ? "all" : "Medium")}
            className={`bg-slate-800 border rounded-lg p-4 transition-colors ${
              selectedSeverity === "Medium" ? "border-yellow-500" : "border-slate-700 hover:border-yellow-500/50"
            }`}
          >
            <div className="text-3xl font-bold text-yellow-500">{stats.medium}</div>
            <div className="text-sm text-slate-400">Medium</div>
          </button>
          <button
            onClick={() => setSelectedSeverity(selectedSeverity === "Low" ? "all" : "Low")}
            className={`bg-slate-800 border rounded-lg p-4 transition-colors ${
              selectedSeverity === "Low" ? "border-blue-500" : "border-slate-700 hover:border-blue-500/50"
            }`}
          >
            <div className="text-3xl font-bold text-blue-500">{stats.low}</div>
            <div className="text-sm text-slate-400">Low</div>
          </button>
        </div>
      )}

      {/* Filter Bar */}
      <div className="flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <span className="text-slate-400 text-sm">Category:</span>
            <select
              value={selectedCategory}
              onChange={(e) => setSelectedCategory(e.target.value)}
              className="bg-slate-800 border border-slate-700 rounded-md px-3 py-1.5 text-white text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {categories.map((cat) => (
                <option key={cat.id} value={cat.id}>
                  {cat.name}
                </option>
              ))}
            </select>
          </div>
        </div>
        <div className="text-slate-400 text-sm">
          {filteredChecks.length} checks
        </div>
      </div>

      {/* NSA/CISA Reference Banner */}
      <div className="bg-slate-800/50 border border-slate-700 rounded-lg p-4">
        <div className="flex items-center space-x-2 mb-2">
          <svg className="w-5 h-5 text-blue-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span className="text-blue-400 font-medium">NSA/CISA Kubernetes Hardening Guide</span>
        </div>
        <p className="text-slate-400 text-sm">
          These security checks are based on the NSA/CISA Kubernetes Hardening Guidance for securing Kubernetes clusters in enterprise environments.
        </p>
      </div>

      {/* Hardening Checks */}
      <div className="space-y-3">
        {filteredChecks.map((check) => (
          <div
            key={check.id}
            className="bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-slate-600 transition-colors"
          >
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center space-x-2 mb-2">
                  <span className={`px-2 py-0.5 rounded text-xs font-medium ${
                    severityColors[check.severity] || "bg-slate-600"
                  } text-white`}>
                    {check.severity}
                  </span>
                  <span className="px-2 py-0.5 rounded text-xs bg-slate-700 text-slate-300">
                    {check.category}
                  </span>
                  {check.automatable && (
                    <span className="px-2 py-0.5 rounded text-xs bg-green-900 text-green-300">
                      Automatable
                    </span>
                  )}
                </div>
                <h4 className="text-white font-medium mb-1">{check.title}</h4>
                <p className="text-slate-400 text-sm mb-3">{check.description}</p>

                <div className="space-y-2">
                  <div>
                    <span className="text-slate-500 text-xs font-medium">Rationale:</span>
                    <p className="text-slate-400 text-xs">{check.rationale}</p>
                  </div>
                  <div>
                    <span className="text-slate-500 text-xs font-medium">Remediation:</span>
                    <p className="text-slate-400 text-xs font-mono bg-slate-900 p-2 rounded mt-1">
                      {check.remediation}
                    </p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// FinOps Tab
// ============================================================================

function FinOpsTab() {
  const [templates, setTemplates] = useState<FinOpsTemplate[]>([]);
  const [comparison, setComparison] = useState<ProviderComparison[] | null>(null);
  const [loading, setLoading] = useState(true);
  const [comparing, setComparing] = useState(false);

  // Simple resource form state
  const [resources, setResources] = useState<ResourceInput[]>([
    { resourceType: "VirtualMachine", name: "App Servers", quantity: 2, vcpus: 4, memoryGb: 16 },
    { resourceType: "Database", name: "PostgreSQL", quantity: 1, vcpus: 2, memoryGb: 8, storageGb: 100 },
  ]);

  useEffect(() => {
    loadTemplates();
  }, []);

  const loadTemplates = async () => {
    try {
      const data = await getFinOpsTemplates();
      setTemplates(data);
    } catch (err) {
      console.error("Failed to load templates:", err);
    } finally {
      setLoading(false);
    }
  };

  const runComparison = async () => {
    setComparing(true);
    try {
      const result = await compareCloudProviders({ resources });
      setComparison(result);
    } catch (err) {
      console.error("Failed to compare providers:", err);
    } finally {
      setComparing(false);
    }
  };

  const addResource = () => {
    setResources([
      ...resources,
      { resourceType: "VirtualMachine", name: "New Resource", quantity: 1, vcpus: 2, memoryGb: 4 },
    ]);
  };

  const removeResource = (index: number) => {
    setResources(resources.filter((_, i) => i !== index));
  };

  const updateResource = (index: number, field: keyof ResourceInput, value: string | number) => {
    const updated = [...resources];
    (updated[index] as any)[field] = value;
    setResources(updated);
  };

  const resourceTypes = [
    { id: "VirtualMachine", name: "Virtual Machine" },
    { id: "Container", name: "Container" },
    { id: "Database", name: "Database" },
    { id: "Storage", name: "Storage" },
    { id: "Kubernetes", name: "Kubernetes Cluster" },
    { id: "Serverless", name: "Serverless Function" },
    { id: "LoadBalancer", name: "Load Balancer" },
  ];

  if (loading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" />
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Templates Overview */}
      <div>
        <h3 className="text-lg font-semibold text-white mb-3">Quick Start Templates</h3>
        <div className="grid grid-cols-3 gap-4">
          {templates.map((template) => (
            <div
              key={template.name}
              className="bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-blue-500 cursor-pointer transition-colors"
            >
              <h4 className="text-white font-medium mb-1">{template.name}</h4>
              <p className="text-slate-400 text-sm mb-2">{template.description}</p>
              <div className="text-slate-500 text-xs">
                {template.resourceCount} resources
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Resource Configuration */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold text-white">Resource Configuration</h3>
          <button
            onClick={addResource}
            className="px-3 py-1.5 bg-blue-600 hover:bg-blue-700 text-white text-sm rounded-md transition-colors"
          >
            + Add Resource
          </button>
        </div>

        <div className="space-y-3">
          {resources.map((resource, index) => (
            <div
              key={index}
              className="bg-slate-900 border border-slate-700 rounded-lg p-4"
            >
              <div className="grid grid-cols-6 gap-3 items-end">
                <div>
                  <label className="block text-xs text-slate-400 mb-1">Type</label>
                  <select
                    value={resource.resourceType}
                    onChange={(e) => updateResource(index, "resourceType", e.target.value)}
                    className="w-full bg-slate-800 border border-slate-600 rounded px-2 py-1.5 text-white text-sm"
                  >
                    {resourceTypes.map((type) => (
                      <option key={type.id} value={type.id}>
                        {type.name}
                      </option>
                    ))}
                  </select>
                </div>
                <div>
                  <label className="block text-xs text-slate-400 mb-1">Name</label>
                  <input
                    type="text"
                    value={resource.name}
                    onChange={(e) => updateResource(index, "name", e.target.value)}
                    className="w-full bg-slate-800 border border-slate-600 rounded px-2 py-1.5 text-white text-sm"
                  />
                </div>
                <div>
                  <label className="block text-xs text-slate-400 mb-1">Quantity</label>
                  <input
                    type="number"
                    value={resource.quantity}
                    onChange={(e) => updateResource(index, "quantity", parseInt(e.target.value) || 1)}
                    className="w-full bg-slate-800 border border-slate-600 rounded px-2 py-1.5 text-white text-sm"
                    min="1"
                  />
                </div>
                <div>
                  <label className="block text-xs text-slate-400 mb-1">vCPUs</label>
                  <input
                    type="number"
                    value={resource.vcpus || ""}
                    onChange={(e) => updateResource(index, "vcpus", parseInt(e.target.value) || 0)}
                    className="w-full bg-slate-800 border border-slate-600 rounded px-2 py-1.5 text-white text-sm"
                  />
                </div>
                <div>
                  <label className="block text-xs text-slate-400 mb-1">Memory (GB)</label>
                  <input
                    type="number"
                    value={resource.memoryGb || ""}
                    onChange={(e) => updateResource(index, "memoryGb", parseFloat(e.target.value) || 0)}
                    className="w-full bg-slate-800 border border-slate-600 rounded px-2 py-1.5 text-white text-sm"
                  />
                </div>
                <div>
                  <button
                    onClick={() => removeResource(index)}
                    className="w-full px-3 py-1.5 bg-red-900 hover:bg-red-800 text-red-300 text-sm rounded transition-colors"
                  >
                    Remove
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>

        <div className="mt-4 flex justify-end">
          <button
            onClick={runComparison}
            disabled={comparing || resources.length === 0}
            className="px-6 py-2 bg-green-600 hover:bg-green-700 disabled:bg-slate-700 disabled:cursor-not-allowed text-white font-medium rounded-md transition-colors"
          >
            {comparing ? "Calculating..." : "Compare Cloud Providers"}
          </button>
        </div>
      </div>

      {/* Cost Comparison Results */}
      {comparison && (
        <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
          <h3 className="text-lg font-semibold text-white mb-4">Cost Comparison</h3>
          <div className="grid grid-cols-3 gap-4">
            {comparison.map((provider) => (
              <div
                key={provider.provider}
                className="bg-slate-900 border border-slate-600 rounded-lg p-4 text-center"
              >
                <div className="text-xl font-bold text-white mb-1">
                  {provider.provider}
                </div>
                <div className="text-3xl font-bold text-green-400 mb-1">
                  ${provider.monthlyCost.toLocaleString()}
                </div>
                <div className="text-slate-400 text-sm">per month</div>
                <div className="text-slate-500 text-xs mt-2">
                  ${provider.annualCost.toLocaleString()}/year
                </div>
              </div>
            ))}
          </div>

          <div className="mt-4 p-4 bg-slate-900 rounded-lg">
            <h4 className="text-sm font-medium text-slate-300 mb-2">Assumptions</h4>
            <ul className="text-xs text-slate-400 space-y-1">
              <li>- Pricing based on on-demand rates; reserved instances can reduce costs by 30-60%</li>
              <li>- Estimates include compute and memory costs only</li>
              <li>- Actual costs may vary based on region and specific instance types</li>
            </ul>
          </div>
        </div>
      )}

      {/* Migration Strategies */}
      <div className="bg-slate-800 border border-slate-700 rounded-lg p-6">
        <h3 className="text-lg font-semibold text-white mb-4">Migration Strategies (6 Rs)</h3>
        <div className="grid grid-cols-3 gap-4">
          {[
            { name: "Rehost", desc: "Lift and shift - move as-is to cloud", effort: "Low", savings: "10-20%" },
            { name: "Replatform", desc: "Lift and optimize - minor modifications", effort: "Medium", savings: "20-35%" },
            { name: "Refactor", desc: "Re-architect for cloud-native", effort: "High", savings: "40-60%" },
            { name: "Repurchase", desc: "Replace with SaaS solution", effort: "Medium", savings: "Variable" },
            { name: "Retire", desc: "Decommission unused applications", effort: "Low", savings: "100%" },
            { name: "Retain", desc: "Keep on-premises for now", effort: "None", savings: "0%" },
          ].map((strategy) => (
            <div
              key={strategy.name}
              className="bg-slate-900 border border-slate-600 rounded-lg p-4"
            >
              <h4 className="text-white font-medium mb-1">{strategy.name}</h4>
              <p className="text-slate-400 text-sm mb-2">{strategy.desc}</p>
              <div className="flex justify-between text-xs">
                <span className="text-slate-500">Effort: {strategy.effort}</span>
                <span className="text-green-400">Savings: {strategy.savings}</span>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}

export default InfrastructureMigration;
