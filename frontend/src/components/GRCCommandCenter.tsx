import { useState, useEffect } from "react";
import {
  listFrameworks,
  getFrameworkControls,
  listAssessments,
  createAssessment,
  getAssessmentSummary,
  updateControlAssessment,
  getControlAssessments,
  listClients,
  getComplianceStatus,
} from "@/lib/commands";
import type {
  FrameworkInfo,
  Control,
  Assessment,
  AssessmentSummary,
  ControlAssessment,
  Client,
  ComplianceStatus,
  ComplianceStatusReport,
} from "@/types";
import { cn } from "@/lib/utils";
import {
  Shield,
  FileCheck,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  HelpCircle,
  ChevronRight,
  Plus,
  BarChart3,
  List,
  Grid3X3,
  Filter,
  Search,
  RefreshCw,
  TrendingUp,
  Activity,
} from "lucide-react";

type ViewMode = "heatmap" | "list";

export function GRCCommandCenter() {
  // State
  const [frameworks, setFrameworks] = useState<FrameworkInfo[]>([]);
  const [selectedFramework, setSelectedFramework] = useState<string>("NIST_CSF_2");
  const [controls, setControls] = useState<Control[]>([]);
  const [assessments, setAssessments] = useState<Assessment[]>([]);
  const [selectedAssessment, setSelectedAssessment] = useState<Assessment | null>(null);
  const [summary, setSummary] = useState<AssessmentSummary | null>(null);
  const [controlAssessments, setControlAssessments] = useState<Map<string, ControlAssessment>>(new Map());
  const [clients, setClients] = useState<Client[]>([]);
  const [viewMode, setViewMode] = useState<ViewMode>("heatmap");
  const [searchTerm, setSearchTerm] = useState("");
  const [filterStatus, setFilterStatus] = useState<string>("all");
  const [isLoading, setIsLoading] = useState(true);
  const [showNewAssessmentModal, setShowNewAssessmentModal] = useState(false);
  const [complianceStatusReport, setComplianceStatusReport] = useState<ComplianceStatusReport | null>(null);

  // Load initial data
  useEffect(() => {
    async function loadData() {
      try {
        const [frameworkData, assessmentData, clientData] = await Promise.all([
          listFrameworks(),
          listAssessments(),
          listClients(),
        ]);
        setFrameworks(frameworkData);
        setAssessments(assessmentData);
        setClients(clientData);

        // Load controls for default framework
        const controlData = await getFrameworkControls("NIST_CSF_2");
        setControls(controlData);
      } catch (error) {
        console.error("Failed to load GRC data:", error);
      } finally {
        setIsLoading(false);
      }
    }
    loadData();
  }, []);

  // Load controls and compliance status when framework changes
  useEffect(() => {
    async function loadControls() {
      try {
        const [controlData, statusReport] = await Promise.all([
          getFrameworkControls(selectedFramework),
          getComplianceStatus(selectedFramework).catch(() => null),
        ]);
        setControls(controlData);
        setComplianceStatusReport(statusReport);
      } catch (error) {
        console.error("Failed to load controls:", error);
      }
    }
    loadControls();
  }, [selectedFramework]);

  // Load assessment details when selection changes
  useEffect(() => {
    async function loadAssessmentDetails() {
      if (!selectedAssessment) {
        setSummary(null);
        setControlAssessments(new Map());
        return;
      }

      try {
        const [summaryData, caData] = await Promise.all([
          getAssessmentSummary(selectedAssessment.id),
          getControlAssessments(selectedAssessment.id),
        ]);
        setSummary(summaryData);

        const caMap = new Map<string, ControlAssessment>();
        caData.forEach((ca) => caMap.set(ca.controlId, ca));
        setControlAssessments(caMap);
      } catch (error) {
        console.error("Failed to load assessment details:", error);
      }
    }
    loadAssessmentDetails();
  }, [selectedAssessment]);

  // Handle control status update
  const handleControlStatusUpdate = async (controlId: string, status: ComplianceStatus) => {
    if (!selectedAssessment) return;

    try {
      await updateControlAssessment({
        assessmentId: selectedAssessment.id,
        controlId,
        status,
        assessedBy: "Consultant", // TODO: Get from auth
      });

      // Refresh data
      const [summaryData, caData] = await Promise.all([
        getAssessmentSummary(selectedAssessment.id),
        getControlAssessments(selectedAssessment.id),
      ]);
      setSummary(summaryData);

      const caMap = new Map<string, ControlAssessment>();
      caData.forEach((ca) => caMap.set(ca.controlId, ca));
      setControlAssessments(caMap);
    } catch (error) {
      console.error("Failed to update control:", error);
    }
  };

  // Filter controls
  const filteredControls = controls.filter((control) => {
    const matchesSearch =
      searchTerm === "" ||
      control.code.toLowerCase().includes(searchTerm.toLowerCase()) ||
      control.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
      control.description.toLowerCase().includes(searchTerm.toLowerCase());

    const ca = controlAssessments.get(control.id);
    const status = ca?.status || "NOT_ASSESSED";

    const matchesFilter =
      filterStatus === "all" ||
      (filterStatus === "gaps" && (status === "NON_COMPLIANT" || status === "PARTIALLY_COMPLIANT")) ||
      status === filterStatus;

    return matchesSearch && matchesFilter;
  });

  // Group controls by category
  const controlsByCategory = filteredControls.reduce((acc, control) => {
    const category = control.category;
    if (!acc[category]) {
      acc[category] = [];
    }
    acc[category].push(control);
    return acc;
  }, {} as Record<string, Control[]>);

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center">
        <RefreshCw className="w-8 h-8 text-optio-400 animate-spin" />
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-6 border-b border-slate-700">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-optio-600/20 rounded-lg">
              <Shield className="w-6 h-6 text-optio-400" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">GRC Command Center</h1>
              <p className="text-slate-400">Governance, Risk & Compliance Management</p>
            </div>
          </div>
          <button
            onClick={() => setShowNewAssessmentModal(true)}
            className="flex items-center gap-2 px-4 py-2 bg-optio-600 text-white rounded-lg hover:bg-optio-700 transition-colors"
          >
            <Plus className="w-4 h-4" />
            New Assessment
          </button>
        </div>

        {/* Framework Toggle */}
        <div className="flex items-center gap-4">
          <span className="text-sm text-slate-400">Framework:</span>
          <div className="flex gap-2">
            {frameworks.map((fw) => (
              <button
                key={fw.id}
                onClick={() => setSelectedFramework(fw.id)}
                className={cn(
                  "px-4 py-2 rounded-lg text-sm font-medium transition-colors",
                  selectedFramework === fw.id
                    ? "bg-optio-600 text-white"
                    : "bg-slate-800 text-slate-300 hover:bg-slate-700"
                )}
              >
                {fw.name}
              </button>
            ))}
          </div>
        </div>

        {/* Compliance Status Overview */}
        {complianceStatusReport && (
          <div className="mt-4 grid grid-cols-5 gap-4">
            <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div className="flex items-center gap-2 mb-2">
                <TrendingUp className="w-4 h-4 text-blue-400" />
                <span className="text-xs text-slate-400">Completion</span>
              </div>
              <div className="text-2xl font-bold text-white">
                {complianceStatusReport.completionPercentage.toFixed(1)}%
              </div>
              <div className="text-xs text-slate-500 mt-1">
                {complianceStatusReport.assessedControls}/{complianceStatusReport.totalControls} controls
              </div>
            </div>
            <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div className="flex items-center gap-2 mb-2">
                <Activity className="w-4 h-4 text-emerald-400" />
                <span className="text-xs text-slate-400">Compliance</span>
              </div>
              <div className={cn(
                "text-2xl font-bold",
                complianceStatusReport.compliancePercentage >= 80 ? "text-emerald-400" :
                complianceStatusReport.compliancePercentage >= 50 ? "text-amber-400" : "text-red-400"
              )}>
                {complianceStatusReport.compliancePercentage.toFixed(1)}%
              </div>
              <div className="text-xs text-slate-500 mt-1">Overall score</div>
            </div>
            <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle2 className="w-4 h-4 text-secure" />
                <span className="text-xs text-slate-400">Compliant</span>
              </div>
              <div className="text-2xl font-bold text-secure">
                {complianceStatusReport.compliantControls}
              </div>
              <div className="text-xs text-slate-500 mt-1">controls</div>
            </div>
            <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="w-4 h-4 text-warning" />
                <span className="text-xs text-slate-400">Partial</span>
              </div>
              <div className="text-2xl font-bold text-warning">
                {complianceStatusReport.partiallyCompliantControls}
              </div>
              <div className="text-xs text-slate-500 mt-1">controls</div>
            </div>
            <div className="bg-slate-800/50 rounded-xl p-4 border border-slate-700">
              <div className="flex items-center gap-2 mb-2">
                <XCircle className="w-4 h-4 text-critical" />
                <span className="text-xs text-slate-400">Non-Compliant</span>
              </div>
              <div className="text-2xl font-bold text-critical">
                {complianceStatusReport.nonCompliantControls}
              </div>
              <div className="text-xs text-slate-500 mt-1">controls</div>
            </div>
          </div>
        )}
      </div>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Left Sidebar - Assessments */}
        <div className="w-72 border-r border-slate-700 flex flex-col">
          <div className="p-4 border-b border-slate-700">
            <h3 className="text-sm font-semibold text-slate-300">Active Assessments</h3>
          </div>
          <div className="flex-1 overflow-y-auto p-2 space-y-2">
            {assessments.length === 0 ? (
              <p className="text-sm text-slate-500 text-center py-4">
                No assessments yet.
              </p>
            ) : (
              assessments
                .filter((a) => a.framework === selectedFramework || formatFramework(a.framework) === selectedFramework)
                .map((assessment) => (
                  <button
                    key={assessment.id}
                    onClick={() => setSelectedAssessment(assessment)}
                    className={cn(
                      "w-full p-3 rounded-lg text-left transition-colors",
                      selectedAssessment?.id === assessment.id
                        ? "bg-optio-600/20 border border-optio-500"
                        : "bg-slate-800/50 hover:bg-slate-800 border border-transparent"
                    )}
                  >
                    <p className="font-medium text-white truncate">{assessment.name}</p>
                    <p className="text-xs text-slate-400 mt-1">
                      {assessment.status} - {new Date(assessment.startedAt).toLocaleDateString()}
                    </p>
                  </button>
                ))
            )}
          </div>
        </div>

        {/* Center - Controls & Heatmap */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Toolbar */}
          <div className="p-4 border-b border-slate-700 flex items-center gap-4">
            <div className="flex-1 relative">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-slate-500" />
              <input
                type="text"
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                placeholder="Search controls..."
                className="w-full pl-10 pr-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-optio-500"
              />
            </div>
            <select
              value={filterStatus}
              onChange={(e) => setFilterStatus(e.target.value)}
              className="px-4 py-2 bg-slate-800 border border-slate-700 rounded-lg text-white"
            >
              <option value="all">All Status</option>
              <option value="gaps">Gaps Only</option>
              <option value="COMPLIANT">Compliant</option>
              <option value="PARTIALLY_COMPLIANT">Partial</option>
              <option value="NON_COMPLIANT">Non-Compliant</option>
              <option value="NOT_ASSESSED">Not Assessed</option>
            </select>
            <div className="flex gap-1 bg-slate-800 rounded-lg p-1">
              <button
                onClick={() => setViewMode("heatmap")}
                className={cn(
                  "p-2 rounded",
                  viewMode === "heatmap" ? "bg-slate-700" : "hover:bg-slate-700/50"
                )}
              >
                <Grid3X3 className="w-4 h-4 text-slate-300" />
              </button>
              <button
                onClick={() => setViewMode("list")}
                className={cn(
                  "p-2 rounded",
                  viewMode === "list" ? "bg-slate-700" : "hover:bg-slate-700/50"
                )}
              >
                <List className="w-4 h-4 text-slate-300" />
              </button>
            </div>
          </div>

          {/* Content Area */}
          <div className="flex-1 overflow-y-auto p-4">
            {viewMode === "heatmap" ? (
              <ComplianceHeatmap
                summary={summary}
                selectedFramework={selectedFramework}
                frameworks={frameworks}
              />
            ) : (
              <ControlsList
                controlsByCategory={controlsByCategory}
                controlAssessments={controlAssessments}
                selectedAssessment={selectedAssessment}
                onStatusUpdate={handleControlStatusUpdate}
              />
            )}
          </div>
        </div>

        {/* Right Sidebar - Summary */}
        <div className="w-80 border-l border-slate-700 flex flex-col">
          <div className="p-4 border-b border-slate-700">
            <h3 className="text-sm font-semibold text-slate-300">Assessment Summary</h3>
          </div>
          {summary ? (
            <div className="flex-1 overflow-y-auto p-4 space-y-6">
              {/* Overall Score */}
              <div className="text-center">
                <div className="inline-flex items-center justify-center w-24 h-24 rounded-full bg-slate-800 border-4 border-optio-500">
                  <span className="text-2xl font-bold text-white">
                    {summary.overallCompliance.toFixed(0)}%
                  </span>
                </div>
                <p className="text-sm text-slate-400 mt-2">Overall Compliance</p>
              </div>

              {/* Status Breakdown */}
              <div className="space-y-2">
                <StatusBar
                  label="Compliant"
                  count={summary.compliant}
                  total={summary.totalControls}
                  color="bg-secure"
                />
                <StatusBar
                  label="Partial"
                  count={summary.partiallyCompliant}
                  total={summary.totalControls}
                  color="bg-warning"
                />
                <StatusBar
                  label="Non-Compliant"
                  count={summary.nonCompliant}
                  total={summary.totalControls}
                  color="bg-critical"
                />
                <StatusBar
                  label="Not Assessed"
                  count={summary.notAssessed}
                  total={summary.totalControls}
                  color="bg-slate-600"
                />
              </div>

              {/* Metrics */}
              <div className="grid grid-cols-2 gap-3">
                <MetricCard
                  label="High Risk Gaps"
                  value={summary.highRiskGaps}
                  icon={AlertTriangle}
                  color="text-critical"
                />
                <MetricCard
                  label="Evidence Items"
                  value={summary.evidenceCount}
                  icon={FileCheck}
                  color="text-optio-400"
                />
              </div>
            </div>
          ) : (
            <div className="flex-1 flex items-center justify-center">
              <p className="text-sm text-slate-500 text-center px-4">
                Select an assessment to view summary
              </p>
            </div>
          )}
        </div>
      </div>

      {/* New Assessment Modal */}
      {showNewAssessmentModal && (
        <NewAssessmentModal
          clients={clients}
          frameworks={frameworks}
          selectedFramework={selectedFramework}
          onClose={() => setShowNewAssessmentModal(false)}
          onCreated={(assessment) => {
            setAssessments((prev) => [assessment, ...prev]);
            setSelectedAssessment(assessment);
            setShowNewAssessmentModal(false);
          }}
        />
      )}
    </div>
  );
}

// Compliance Heatmap Component
interface HeatmapProps {
  summary: AssessmentSummary | null;
  selectedFramework: string;
  frameworks: FrameworkInfo[];
}

function ComplianceHeatmap({ summary, selectedFramework, frameworks }: HeatmapProps) {
  const framework = frameworks.find((f) => f.id === selectedFramework);

  if (!summary) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="text-center">
          <BarChart3 className="w-12 h-12 text-slate-600 mx-auto mb-4" />
          <p className="text-slate-400">Select an assessment to view the compliance heatmap</p>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <h3 className="text-lg font-semibold text-white">Compliance by Category</h3>
      <div className="grid grid-cols-2 lg:grid-cols-3 gap-4">
        {summary.categoryScores.map((cat) => (
          <div
            key={cat.category}
            className="bg-slate-800/50 border border-slate-700 rounded-xl p-4"
          >
            <div className="flex items-center justify-between mb-3">
              <div className="flex items-center gap-2">
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: cat.color }}
                />
                <span className="font-medium text-white">{cat.displayName}</span>
              </div>
              <span
                className={cn(
                  "text-lg font-bold",
                  cat.compliancePercentage >= 80
                    ? "text-secure"
                    : cat.compliancePercentage >= 50
                    ? "text-warning"
                    : "text-critical"
                )}
              >
                {cat.compliancePercentage.toFixed(0)}%
              </span>
            </div>

            {/* Mini heatmap bar */}
            <div className="h-3 bg-slate-900 rounded-full overflow-hidden flex">
              {cat.compliant > 0 && (
                <div
                  className="bg-secure"
                  style={{ width: `${(cat.compliant / cat.totalControls) * 100}%` }}
                />
              )}
              {cat.partiallyCompliant > 0 && (
                <div
                  className="bg-warning"
                  style={{ width: `${(cat.partiallyCompliant / cat.totalControls) * 100}%` }}
                />
              )}
              {cat.nonCompliant > 0 && (
                <div
                  className="bg-critical"
                  style={{ width: `${(cat.nonCompliant / cat.totalControls) * 100}%` }}
                />
              )}
              {cat.notAssessed > 0 && (
                <div
                  className="bg-slate-600"
                  style={{ width: `${(cat.notAssessed / cat.totalControls) * 100}%` }}
                />
              )}
            </div>

            <div className="flex justify-between mt-2 text-xs text-slate-500">
              <span>{cat.compliant} compliant</span>
              <span>{cat.totalControls} total</span>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}

// Controls List Component
interface ControlsListProps {
  controlsByCategory: Record<string, Control[]>;
  controlAssessments: Map<string, ControlAssessment>;
  selectedAssessment: Assessment | null;
  onStatusUpdate: (controlId: string, status: ComplianceStatus) => void;
}

function ControlsList({
  controlsByCategory,
  controlAssessments,
  selectedAssessment,
  onStatusUpdate,
}: ControlsListProps) {
  const [expandedCategories, setExpandedCategories] = useState<Set<string>>(new Set());

  const toggleCategory = (category: string) => {
    setExpandedCategories((prev) => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };

  return (
    <div className="space-y-4">
      {Object.entries(controlsByCategory).map(([category, categoryControls]) => (
        <div key={category} className="bg-slate-800/50 border border-slate-700 rounded-xl overflow-hidden">
          <button
            onClick={() => toggleCategory(category)}
            className="w-full flex items-center justify-between p-4 hover:bg-slate-800 transition-colors"
          >
            <div className="flex items-center gap-3">
              <ChevronRight
                className={cn(
                  "w-5 h-5 text-slate-400 transition-transform",
                  expandedCategories.has(category) && "rotate-90"
                )}
              />
              <span className="font-medium text-white">{category}</span>
              <span className="text-sm text-slate-500">({categoryControls.length} controls)</span>
            </div>
          </button>

          {expandedCategories.has(category) && (
            <div className="border-t border-slate-700">
              {categoryControls.map((control) => {
                const ca = controlAssessments.get(control.id);
                const status = ca?.status || "NOT_ASSESSED";

                return (
                  <div
                    key={control.id}
                    className="p-4 border-b border-slate-700/50 last:border-0 hover:bg-slate-800/50"
                  >
                    <div className="flex items-start justify-between">
                      <div className="flex-1">
                        <div className="flex items-center gap-2">
                          <code className="text-xs bg-slate-900 px-2 py-0.5 rounded text-optio-400">
                            {control.code}
                          </code>
                          <span className="font-medium text-white">{control.title}</span>
                        </div>
                        <p className="text-sm text-slate-400 mt-1">{control.description}</p>
                      </div>

                      {selectedAssessment && (
                        <div className="flex items-center gap-2 ml-4">
                          <StatusButton
                            status="COMPLIANT"
                            currentStatus={status}
                            onClick={() => onStatusUpdate(control.id, "COMPLIANT")}
                          />
                          <StatusButton
                            status="PARTIALLY_COMPLIANT"
                            currentStatus={status}
                            onClick={() => onStatusUpdate(control.id, "PARTIALLY_COMPLIANT")}
                          />
                          <StatusButton
                            status="NON_COMPLIANT"
                            currentStatus={status}
                            onClick={() => onStatusUpdate(control.id, "NON_COMPLIANT")}
                          />
                          <StatusButton
                            status="NOT_APPLICABLE"
                            currentStatus={status}
                            onClick={() => onStatusUpdate(control.id, "NOT_APPLICABLE")}
                          />
                        </div>
                      )}
                    </div>
                  </div>
                );
              })}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

// Status Button Component
interface StatusButtonProps {
  status: ComplianceStatus;
  currentStatus: string;
  onClick: () => void;
}

function StatusButton({ status, currentStatus, onClick }: StatusButtonProps) {
  const isActive = currentStatus === status;

  const config = {
    COMPLIANT: { icon: CheckCircle2, color: "text-secure", bg: "bg-secure/20" },
    PARTIALLY_COMPLIANT: { icon: AlertTriangle, color: "text-warning", bg: "bg-warning/20" },
    NON_COMPLIANT: { icon: XCircle, color: "text-critical", bg: "bg-critical/20" },
    NOT_APPLICABLE: { icon: HelpCircle, color: "text-slate-400", bg: "bg-slate-600/20" },
  }[status] || { icon: HelpCircle, color: "text-slate-400", bg: "bg-slate-600/20" };

  const Icon = config.icon;

  return (
    <button
      onClick={onClick}
      className={cn(
        "p-1.5 rounded transition-colors",
        isActive ? config.bg : "hover:bg-slate-700"
      )}
      title={status.replace("_", " ")}
    >
      <Icon className={cn("w-4 h-4", isActive ? config.color : "text-slate-500")} />
    </button>
  );
}

// Status Bar Component
interface StatusBarProps {
  label: string;
  count: number;
  total: number;
  color: string;
}

function StatusBar({ label, count, total, color }: StatusBarProps) {
  const percentage = total > 0 ? (count / total) * 100 : 0;

  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span className="text-slate-400">{label}</span>
        <span className="text-white">{count}</span>
      </div>
      <div className="h-2 bg-slate-800 rounded-full overflow-hidden">
        <div className={cn("h-full rounded-full", color)} style={{ width: `${percentage}%` }} />
      </div>
    </div>
  );
}

// Metric Card Component
interface MetricCardProps {
  label: string;
  value: number;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
}

function MetricCard({ label, value, icon: Icon, color }: MetricCardProps) {
  return (
    <div className="bg-slate-800/50 rounded-lg p-3">
      <div className="flex items-center gap-2">
        <Icon className={cn("w-4 h-4", color)} />
        <span className="text-2xl font-bold text-white">{value}</span>
      </div>
      <p className="text-xs text-slate-500 mt-1">{label}</p>
    </div>
  );
}

// New Assessment Modal
interface NewAssessmentModalProps {
  clients: Client[];
  frameworks: FrameworkInfo[];
  selectedFramework: string;
  onClose: () => void;
  onCreated: (assessment: Assessment) => void;
}

function NewAssessmentModal({
  clients,
  frameworks,
  selectedFramework,
  onClose,
  onCreated,
}: NewAssessmentModalProps) {
  const [name, setName] = useState("");
  const [clientId, setClientId] = useState(clients[0]?.id || "");
  const [framework, setFramework] = useState(selectedFramework);
  const [scope, setScope] = useState("");
  const [isCreating, setIsCreating] = useState(false);

  const handleCreate = async () => {
    if (!name || !clientId) return;

    setIsCreating(true);
    try {
      const assessment = await createAssessment({
        clientId,
        name,
        description: scope || undefined,
        framework,
        scope: scope || undefined,
        leadAssessor: "Consultant", // TODO: Get from auth
      });
      onCreated(assessment);
    } catch (error) {
      console.error("Failed to create assessment:", error);
    } finally {
      setIsCreating(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 border border-slate-700 rounded-xl w-full max-w-md p-6">
        <h2 className="text-xl font-bold text-white mb-4">New Assessment</h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Assessment Name *
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Q1 2024 Compliance Audit"
              className="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Client *
            </label>
            <select
              value={clientId}
              onChange={(e) => setClientId(e.target.value)}
              className="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
            >
              {clients.length === 0 && <option value="">No clients available</option>}
              {clients.map((client) => (
                <option key={client.id} value={client.id}>
                  {client.name}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Framework
            </label>
            <select
              value={framework}
              onChange={(e) => setFramework(e.target.value)}
              className="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white"
            >
              {frameworks.map((fw) => (
                <option key={fw.id} value={fw.id}>
                  {fw.name}
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-slate-300 mb-2">
              Scope (Optional)
            </label>
            <textarea
              value={scope}
              onChange={(e) => setScope(e.target.value)}
              placeholder="Define the scope of this assessment..."
              rows={3}
              className="w-full px-4 py-2 bg-slate-900 border border-slate-700 rounded-lg text-white resize-none"
            />
          </div>
        </div>

        <div className="flex justify-end gap-3 mt-6">
          <button
            onClick={onClose}
            className="px-4 py-2 bg-slate-700 text-white rounded-lg hover:bg-slate-600"
          >
            Cancel
          </button>
          <button
            onClick={handleCreate}
            disabled={isCreating || !name || !clientId}
            className="px-4 py-2 bg-optio-600 text-white rounded-lg hover:bg-optio-700 disabled:opacity-50"
          >
            {isCreating ? "Creating..." : "Create Assessment"}
          </button>
        </div>
      </div>
    </div>
  );
}

// Helper function
function formatFramework(fw: string): string {
  if (fw === "NistCsf2") return "NIST_CSF_2";
  if (fw === "Soc2TypeII") return "SOC_2_TYPE_II";
  if (fw === "Gdpr") return "GDPR";
  return fw;
}
