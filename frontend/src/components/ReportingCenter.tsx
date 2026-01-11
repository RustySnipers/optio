/**
 * Reporting Center Component
 *
 * Intelligent report generation module for creating professional
 * security assessment documentation.
 */

import { useState, useEffect } from "react";
import {
  FileText,
  Plus,
  Download,
  Eye,
  Trash2,
  RefreshCw,
  FileJson,
  FileCode,
  CheckCircle,
  Clock,
  AlertTriangle,
  Archive,
  BarChart3,
  Shield,
  Network,
  Cloud,
  Bug,
  BookOpen,
  Loader2,
  ChevronRight,
  FolderOpen,
  FileOutput,
} from "lucide-react";
import {
  getReportTypes,
  getExportFormatList,
  generateReport,
  generateDemoReports,
  exportReportHtml,
  exportReportMarkdown,
  exportReportJson,
  generateExecutivePdf,
  generateDemoPdf,
  openPdfLocation,
} from "@/lib/commands";
import type {
  ReportTypeInfo,
  ExportFormatInfo,
  ReportSummary,
  GenerateReportRequest,
  ReportType,
  ReportStatus,
  PdfGenerationResult,
} from "@/types";

// Report type icons mapping
const reportTypeIcons: Record<ReportType, React.ReactNode> = {
  ExecutiveSummary: <BookOpen className="w-5 h-5" />,
  TechnicalAssessment: <Bug className="w-5 h-5" />,
  ComplianceReport: <Shield className="w-5 h-5" />,
  NetworkAssessment: <Network className="w-5 h-5" />,
  CloudReadiness: <Cloud className="w-5 h-5" />,
  SecurityFindings: <AlertTriangle className="w-5 h-5" />,
  FullEngagement: <FileText className="w-5 h-5" />,
};

// Status icons and colors
const statusConfig: Record<ReportStatus, { icon: React.ReactNode; color: string; bg: string }> = {
  Draft: { icon: <Clock className="w-4 h-4" />, color: "text-slate-400", bg: "bg-slate-500/20" },
  Generating: { icon: <Loader2 className="w-4 h-4 animate-spin" />, color: "text-blue-400", bg: "bg-blue-500/20" },
  Ready: { icon: <CheckCircle className="w-4 h-4" />, color: "text-emerald-400", bg: "bg-emerald-500/20" },
  Error: { icon: <AlertTriangle className="w-4 h-4" />, color: "text-red-400", bg: "bg-red-500/20" },
  Archived: { icon: <Archive className="w-4 h-4" />, color: "text-amber-400", bg: "bg-amber-500/20" },
};

// Format file size
function formatFileSize(bytes: number | null): string {
  if (!bytes) return "-";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function ReportingCenter() {
  const [activeTab, setActiveTab] = useState<"reports" | "generate" | "pdf">("reports");
  const [reportTypes, setReportTypes] = useState<ReportTypeInfo[]>([]);
  const [exportFormats, setExportFormats] = useState<ExportFormatInfo[]>([]);
  const [reports, setReports] = useState<ReportSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isGenerating, setIsGenerating] = useState(false);
  const [selectedReport, setSelectedReport] = useState<ReportSummary | null>(null);
  const [exportContent, setExportContent] = useState<string | null>(null);

  // Form state for new report
  const [formData, setFormData] = useState<GenerateReportRequest>({
    reportType: "executive_summary",
    clientId: "demo-client",
    clientName: "Demo Client",
    title: "",
    author: "Security Consultant",
    format: "html",
    includeToc: true,
    includeExecutiveSummary: true,
    includeAppendices: true,
    includeCharts: true,
  });

  // PDF generation state
  const [pdfResult, setPdfResult] = useState<PdfGenerationResult | null>(null);
  const [isGeneratingPdf, setIsGeneratingPdf] = useState(false);

  useEffect(() => {
    loadData();
  }, []);

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [types, formats, demoReports] = await Promise.all([
        getReportTypes(),
        getExportFormatList(),
        generateDemoReports("demo-client", "Demo Client"),
      ]);
      setReportTypes(types);
      setExportFormats(formats);
      setReports(demoReports);
    } catch (error) {
      console.error("Failed to load reporting data:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleGenerateReport = async () => {
    if (!formData.title) {
      alert("Please enter a report title");
      return;
    }

    setIsGenerating(true);
    try {
      const report = await generateReport(formData);
      // Add to reports list
      setReports((prev) => [
        {
          id: report.id,
          title: report.config.title,
          reportType: report.config.reportType,
          clientName: report.config.clientName,
          status: report.status,
          format: report.config.format,
          createdAt: report.createdAt,
          fileSize: report.fileSize,
        },
        ...prev,
      ]);
      setActiveTab("reports");
      // Reset form
      setFormData((prev) => ({ ...prev, title: "" }));
    } catch (error) {
      console.error("Failed to generate report:", error);
      alert(`Failed to generate report: ${error}`);
    } finally {
      setIsGenerating(false);
    }
  };

  const handleExport = async (report: ReportSummary, format: "html" | "markdown" | "json") => {
    try {
      let content: string;
      switch (format) {
        case "html":
          content = await exportReportHtml(report.id);
          break;
        case "markdown":
          content = await exportReportMarkdown(report.id);
          break;
        case "json":
          content = await exportReportJson(report.id);
          break;
      }
      setSelectedReport(report);
      setExportContent(content);
    } catch (error) {
      console.error("Failed to export report:", error);
      alert(`Failed to export: ${error}`);
    }
  };

  const handleGenerateExecutivePdf = async () => {
    setIsGeneratingPdf(true);
    setPdfResult(null);
    try {
      const result = await generateExecutivePdf({
        clientId: formData.clientId,
        clientName: formData.clientName || "Demo Client",
        title: formData.title || "Executive Security Assessment",
        framework: "NIST_CSF_2",
        includeNetworkData: true,
        includeComplianceData: true,
      });
      setPdfResult(result);
    } catch (error) {
      console.error("Failed to generate PDF:", error);
      alert(`Failed to generate PDF: ${error}`);
    } finally {
      setIsGeneratingPdf(false);
    }
  };

  const handleGenerateDemoPdf = async () => {
    setIsGeneratingPdf(true);
    setPdfResult(null);
    try {
      const result = await generateDemoPdf(formData.clientName || "Demo Client");
      setPdfResult(result);
    } catch (error) {
      console.error("Failed to generate demo PDF:", error);
      alert(`Failed to generate demo PDF: ${error}`);
    } finally {
      setIsGeneratingPdf(false);
    }
  };

  const handleOpenPdfLocation = async () => {
    if (pdfResult?.filePath) {
      try {
        await openPdfLocation(pdfResult.filePath);
      } catch (error) {
        console.error("Failed to open PDF location:", error);
        alert(`Failed to open folder: ${error}`);
      }
    }
  };

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center">
        <div className="flex flex-col items-center gap-4">
          <Loader2 className="w-8 h-8 text-blue-500 animate-spin" />
          <p className="text-slate-400">Loading Reporting Center...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-6 border-b border-slate-700">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-bold text-white flex items-center gap-3">
              <FileText className="w-7 h-7 text-blue-500" />
              Reporting Center
            </h1>
            <p className="text-slate-400 mt-1">
              Generate professional security assessment reports
            </p>
          </div>
          <div className="flex items-center gap-3">
            <button
              onClick={loadData}
              className="px-4 py-2 bg-slate-700 hover:bg-slate-600 text-white rounded-lg flex items-center gap-2 transition-colors"
            >
              <RefreshCw className="w-4 h-4" />
              Refresh
            </button>
            <button
              onClick={() => setActiveTab("generate")}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg flex items-center gap-2 transition-colors"
            >
              <Plus className="w-4 h-4" />
              New Report
            </button>
          </div>
        </div>

        {/* Tab Navigation */}
        <div className="flex gap-4 mt-6">
          <button
            onClick={() => setActiveTab("reports")}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              activeTab === "reports"
                ? "bg-blue-600 text-white"
                : "bg-slate-700 text-slate-300 hover:bg-slate-600"
            }`}
          >
            Reports
          </button>
          <button
            onClick={() => setActiveTab("generate")}
            className={`px-4 py-2 rounded-lg font-medium transition-colors ${
              activeTab === "generate"
                ? "bg-blue-600 text-white"
                : "bg-slate-700 text-slate-300 hover:bg-slate-600"
            }`}
          >
            Generate New
          </button>
          <button
            onClick={() => setActiveTab("pdf")}
            className={`px-4 py-2 rounded-lg font-medium transition-colors flex items-center gap-2 ${
              activeTab === "pdf"
                ? "bg-red-600 text-white"
                : "bg-slate-700 text-slate-300 hover:bg-slate-600"
            }`}
          >
            <FileOutput className="w-4 h-4" />
            Executive PDF
          </button>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        {activeTab === "reports" ? (
          <div className="space-y-6">
            {/* Stats Cards */}
            <div className="grid grid-cols-4 gap-4">
              <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-blue-500/20 rounded-lg">
                    <FileText className="w-5 h-5 text-blue-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-white">{reports.length}</p>
                    <p className="text-sm text-slate-400">Total Reports</p>
                  </div>
                </div>
              </div>
              <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-emerald-500/20 rounded-lg">
                    <CheckCircle className="w-5 h-5 text-emerald-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-white">
                      {reports.filter((r) => r.status === "Ready").length}
                    </p>
                    <p className="text-sm text-slate-400">Ready</p>
                  </div>
                </div>
              </div>
              <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-amber-500/20 rounded-lg">
                    <Clock className="w-5 h-5 text-amber-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-white">
                      {reports.filter((r) => r.status === "Draft").length}
                    </p>
                    <p className="text-sm text-slate-400">Drafts</p>
                  </div>
                </div>
              </div>
              <div className="bg-slate-800 rounded-xl p-4 border border-slate-700">
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-purple-500/20 rounded-lg">
                    <BarChart3 className="w-5 h-5 text-purple-400" />
                  </div>
                  <div>
                    <p className="text-2xl font-bold text-white">7</p>
                    <p className="text-sm text-slate-400">Report Types</p>
                  </div>
                </div>
              </div>
            </div>

            {/* Reports Table */}
            <div className="bg-slate-800 rounded-xl border border-slate-700 overflow-hidden">
              <div className="p-4 border-b border-slate-700">
                <h2 className="text-lg font-semibold text-white">Recent Reports</h2>
              </div>
              <table className="w-full">
                <thead className="bg-slate-900/50">
                  <tr>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Report
                    </th>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Type
                    </th>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Client
                    </th>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Status
                    </th>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Format
                    </th>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Size
                    </th>
                    <th className="text-left text-sm font-medium text-slate-400 px-4 py-3">
                      Actions
                    </th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-700">
                  {reports.map((report) => (
                    <tr key={report.id} className="hover:bg-slate-700/50 transition-colors">
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-3">
                          <div className="p-2 bg-slate-700 rounded-lg">
                            {reportTypeIcons[report.reportType]}
                          </div>
                          <div>
                            <p className="font-medium text-white">{report.title}</p>
                            <p className="text-xs text-slate-400">
                              {new Date(report.createdAt).toLocaleDateString()}
                            </p>
                          </div>
                        </div>
                      </td>
                      <td className="px-4 py-3">
                        <span className="text-sm text-slate-300">
                          {report.reportType.replace(/([A-Z])/g, " $1").trim()}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <span className="text-sm text-slate-300">{report.clientName}</span>
                      </td>
                      <td className="px-4 py-3">
                        <span
                          className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium ${statusConfig[report.status].bg} ${statusConfig[report.status].color}`}
                        >
                          {statusConfig[report.status].icon}
                          {report.status}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <span className="text-sm text-slate-300 uppercase">{report.format}</span>
                      </td>
                      <td className="px-4 py-3">
                        <span className="text-sm text-slate-400">
                          {formatFileSize(report.fileSize)}
                        </span>
                      </td>
                      <td className="px-4 py-3">
                        <div className="flex items-center gap-2">
                          <button
                            onClick={() => handleExport(report, "html")}
                            className="p-1.5 hover:bg-slate-600 rounded-lg transition-colors"
                            title="View HTML"
                          >
                            <Eye className="w-4 h-4 text-slate-400" />
                          </button>
                          <button
                            onClick={() => handleExport(report, "markdown")}
                            className="p-1.5 hover:bg-slate-600 rounded-lg transition-colors"
                            title="Export Markdown"
                          >
                            <FileCode className="w-4 h-4 text-slate-400" />
                          </button>
                          <button
                            onClick={() => handleExport(report, "json")}
                            className="p-1.5 hover:bg-slate-600 rounded-lg transition-colors"
                            title="Export JSON"
                          >
                            <FileJson className="w-4 h-4 text-slate-400" />
                          </button>
                          <button
                            className="p-1.5 hover:bg-red-500/20 rounded-lg transition-colors"
                            title="Delete"
                          >
                            <Trash2 className="w-4 h-4 text-red-400" />
                          </button>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        ) : activeTab === "generate" ? (
          <div className="max-w-4xl mx-auto space-y-6">
            {/* Report Type Selection */}
            <div className="bg-slate-800 rounded-xl border border-slate-700 p-6">
              <h2 className="text-lg font-semibold text-white mb-4">Select Report Type</h2>
              <div className="grid grid-cols-2 gap-4">
                {reportTypes.map((type) => (
                  <button
                    key={type.reportType}
                    onClick={() =>
                      setFormData((prev) => ({
                        ...prev,
                        reportType: type.reportType.toLowerCase().replace(/ /g, "_"),
                      }))
                    }
                    className={`flex items-start gap-4 p-4 rounded-xl border transition-all ${
                      formData.reportType.toLowerCase().includes(type.reportType.toLowerCase().replace(/ /g, ""))
                        ? "bg-blue-500/20 border-blue-500"
                        : "bg-slate-700/50 border-slate-600 hover:border-slate-500"
                    }`}
                  >
                    <div className="p-2 bg-slate-600 rounded-lg">
                      {reportTypeIcons[type.reportType]}
                    </div>
                    <div className="text-left">
                      <p className="font-medium text-white">{type.name}</p>
                      <p className="text-sm text-slate-400 mt-1">{type.description}</p>
                    </div>
                    <ChevronRight className="w-5 h-5 text-slate-400 ml-auto mt-2" />
                  </button>
                ))}
              </div>
            </div>

            {/* Report Details */}
            <div className="bg-slate-800 rounded-xl border border-slate-700 p-6">
              <h2 className="text-lg font-semibold text-white mb-4">Report Details</h2>
              <div className="grid grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Report Title *
                  </label>
                  <input
                    type="text"
                    value={formData.title}
                    onChange={(e) => setFormData((prev) => ({ ...prev, title: e.target.value }))}
                    placeholder="Q4 2025 Security Assessment"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Subtitle (Optional)
                  </label>
                  <input
                    type="text"
                    value={formData.subtitle || ""}
                    onChange={(e) => setFormData((prev) => ({ ...prev, subtitle: e.target.value }))}
                    placeholder="Executive Summary and Findings"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Client Name
                  </label>
                  <input
                    type="text"
                    value={formData.clientName}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, clientName: e.target.value }))
                    }
                    placeholder="Acme Corporation"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">Author</label>
                  <input
                    type="text"
                    value={formData.author}
                    onChange={(e) => setFormData((prev) => ({ ...prev, author: e.target.value }))}
                    placeholder="Security Consultant"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Organization (Optional)
                  </label>
                  <input
                    type="text"
                    value={formData.organization || ""}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, organization: e.target.value }))
                    }
                    placeholder="Optio Security Services"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Export Format
                  </label>
                  <select
                    value={formData.format}
                    onChange={(e) => setFormData((prev) => ({ ...prev, format: e.target.value }))}
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    {exportFormats.map((fmt) => (
                      <option key={fmt.format} value={fmt.format.toLowerCase()}>
                        {fmt.name} ({fmt.extension})
                      </option>
                    ))}
                  </select>
                </div>
              </div>
            </div>

            {/* Report Options */}
            <div className="bg-slate-800 rounded-xl border border-slate-700 p-6">
              <h2 className="text-lg font-semibold text-white mb-4">Report Options</h2>
              <div className="grid grid-cols-2 gap-4">
                <label className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.includeToc}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, includeToc: e.target.checked }))
                    }
                    className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-blue-500 focus:ring-blue-500"
                  />
                  <span className="text-white">Include Table of Contents</span>
                </label>
                <label className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.includeExecutiveSummary}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, includeExecutiveSummary: e.target.checked }))
                    }
                    className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-blue-500 focus:ring-blue-500"
                  />
                  <span className="text-white">Include Executive Summary</span>
                </label>
                <label className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.includeCharts}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, includeCharts: e.target.checked }))
                    }
                    className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-blue-500 focus:ring-blue-500"
                  />
                  <span className="text-white">Include Charts & Visualizations</span>
                </label>
                <label className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.includeAppendices}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, includeAppendices: e.target.checked }))
                    }
                    className="w-4 h-4 rounded border-slate-600 bg-slate-700 text-blue-500 focus:ring-blue-500"
                  />
                  <span className="text-white">Include Appendices</span>
                </label>
              </div>

              {/* Classification */}
              <div className="mt-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Classification (Optional)
                </label>
                <select
                  value={formData.classification || ""}
                  onChange={(e) =>
                    setFormData((prev) => ({
                      ...prev,
                      classification: e.target.value || undefined,
                    }))
                  }
                  className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">No Classification</option>
                  <option value="CONFIDENTIAL">Confidential</option>
                  <option value="INTERNAL">Internal Use Only</option>
                  <option value="RESTRICTED">Restricted</option>
                  <option value="PUBLIC">Public</option>
                </select>
              </div>

              {/* Notes */}
              <div className="mt-6">
                <label className="block text-sm font-medium text-slate-300 mb-2">
                  Additional Notes (Optional)
                </label>
                <textarea
                  value={formData.notes || ""}
                  onChange={(e) =>
                    setFormData((prev) => ({ ...prev, notes: e.target.value || undefined }))
                  }
                  rows={3}
                  placeholder="Any additional notes or context for this report..."
                  className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none"
                />
              </div>
            </div>

            {/* Generate Button */}
            <div className="flex justify-end gap-4">
              <button
                onClick={() => setActiveTab("reports")}
                className="px-6 py-3 bg-slate-700 hover:bg-slate-600 text-white rounded-lg font-medium transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleGenerateReport}
                disabled={isGenerating || !formData.title}
                className="px-6 py-3 bg-blue-600 hover:bg-blue-500 disabled:bg-slate-600 disabled:cursor-not-allowed text-white rounded-lg font-medium flex items-center gap-2 transition-colors"
              >
                {isGenerating ? (
                  <>
                    <Loader2 className="w-5 h-5 animate-spin" />
                    Generating...
                  </>
                ) : (
                  <>
                    <FileText className="w-5 h-5" />
                    Generate Report
                  </>
                )}
              </button>
            </div>
          </div>
        ) : activeTab === "pdf" ? (
          /* PDF Generation Tab */
          <div className="max-w-4xl mx-auto space-y-6">
            {/* PDF Generation Header */}
            <div className="bg-gradient-to-r from-red-900/30 to-orange-900/30 rounded-xl border border-red-800/50 p-6">
              <div className="flex items-start gap-4">
                <div className="p-3 bg-red-500/20 rounded-xl">
                  <FileOutput className="w-8 h-8 text-red-400" />
                </div>
                <div>
                  <h2 className="text-xl font-bold text-white">Executive PDF Report Generator</h2>
                  <p className="text-slate-300 mt-1">
                    Generate professional PDF reports with integrated GRC compliance status,
                    network health scores, and executive summaries.
                  </p>
                </div>
              </div>
            </div>

            {/* PDF Configuration */}
            <div className="bg-slate-800 rounded-xl border border-slate-700 p-6">
              <h3 className="text-lg font-semibold text-white mb-4">Report Configuration</h3>
              <div className="grid grid-cols-2 gap-6">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Client Name
                  </label>
                  <input
                    type="text"
                    value={formData.clientName}
                    onChange={(e) =>
                      setFormData((prev) => ({ ...prev, clientName: e.target.value }))
                    }
                    placeholder="Acme Corporation"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-red-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Report Title
                  </label>
                  <input
                    type="text"
                    value={formData.title}
                    onChange={(e) => setFormData((prev) => ({ ...prev, title: e.target.value }))}
                    placeholder="Executive Security Assessment"
                    className="w-full px-4 py-2.5 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-red-500"
                  />
                </div>
              </div>

              {/* Report Contents */}
              <div className="mt-6">
                <h4 className="text-sm font-medium text-slate-300 mb-3">Report Contents</h4>
                <div className="grid grid-cols-2 gap-3">
                  <div className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg">
                    <Shield className="w-5 h-5 text-purple-400" />
                    <span className="text-white">NIST CSF Compliance Status</span>
                    <CheckCircle className="w-4 h-4 text-emerald-400 ml-auto" />
                  </div>
                  <div className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg">
                    <Network className="w-5 h-5 text-blue-400" />
                    <span className="text-white">Network Health Score</span>
                    <CheckCircle className="w-4 h-4 text-emerald-400 ml-auto" />
                  </div>
                  <div className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg">
                    <BarChart3 className="w-5 h-5 text-amber-400" />
                    <span className="text-white">Asset Inventory Summary</span>
                    <CheckCircle className="w-4 h-4 text-emerald-400 ml-auto" />
                  </div>
                  <div className="flex items-center gap-3 p-3 bg-slate-700/50 rounded-lg">
                    <AlertTriangle className="w-5 h-5 text-red-400" />
                    <span className="text-white">Risk Summary & Findings</span>
                    <CheckCircle className="w-4 h-4 text-emerald-400 ml-auto" />
                  </div>
                </div>
              </div>
            </div>

            {/* Generate Buttons */}
            <div className="bg-slate-800 rounded-xl border border-slate-700 p-6">
              <div className="flex items-center justify-between gap-4">
                <div>
                  <h3 className="text-lg font-semibold text-white">Generate PDF</h3>
                  <p className="text-sm text-slate-400 mt-1">
                    Creates a 5-page executive summary with all data integrated
                  </p>
                </div>
                <div className="flex gap-3">
                  <button
                    onClick={handleGenerateDemoPdf}
                    disabled={isGeneratingPdf}
                    className="px-5 py-2.5 bg-slate-700 hover:bg-slate-600 disabled:bg-slate-600 text-white rounded-lg font-medium flex items-center gap-2 transition-colors"
                  >
                    {isGeneratingPdf ? (
                      <Loader2 className="w-5 h-5 animate-spin" />
                    ) : (
                      <FileText className="w-5 h-5" />
                    )}
                    Demo Report
                  </button>
                  <button
                    onClick={handleGenerateExecutivePdf}
                    disabled={isGeneratingPdf}
                    className="px-5 py-2.5 bg-red-600 hover:bg-red-500 disabled:bg-slate-600 text-white rounded-lg font-medium flex items-center gap-2 transition-colors"
                  >
                    {isGeneratingPdf ? (
                      <>
                        <Loader2 className="w-5 h-5 animate-spin" />
                        Generating...
                      </>
                    ) : (
                      <>
                        <FileOutput className="w-5 h-5" />
                        Generate Executive PDF
                      </>
                    )}
                  </button>
                </div>
              </div>
            </div>

            {/* PDF Result */}
            {pdfResult && (
              <div className="bg-emerald-900/30 rounded-xl border border-emerald-700/50 p-6">
                <div className="flex items-start gap-4">
                  <div className="p-3 bg-emerald-500/20 rounded-xl">
                    <CheckCircle className="w-8 h-8 text-emerald-400" />
                  </div>
                  <div className="flex-1">
                    <h3 className="text-lg font-semibold text-white">PDF Generated Successfully!</h3>
                    <p className="text-slate-300 mt-1">{pdfResult.message}</p>
                    <div className="mt-4 grid grid-cols-3 gap-4">
                      <div className="bg-slate-800/50 rounded-lg p-3">
                        <p className="text-xs text-slate-400">File Size</p>
                        <p className="text-lg font-semibold text-white">
                          {formatFileSize(pdfResult.fileSize)}
                        </p>
                      </div>
                      <div className="bg-slate-800/50 rounded-lg p-3">
                        <p className="text-xs text-slate-400">Pages</p>
                        <p className="text-lg font-semibold text-white">{pdfResult.pageCount}</p>
                      </div>
                      <div className="bg-slate-800/50 rounded-lg p-3">
                        <p className="text-xs text-slate-400">Format</p>
                        <p className="text-lg font-semibold text-white">PDF</p>
                      </div>
                    </div>
                    <div className="mt-4 p-3 bg-slate-800/50 rounded-lg">
                      <p className="text-xs text-slate-400 mb-1">File Location</p>
                      <p className="text-sm text-slate-300 font-mono break-all">
                        {pdfResult.filePath}
                      </p>
                    </div>
                    <div className="mt-4">
                      <button
                        onClick={handleOpenPdfLocation}
                        className="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg font-medium flex items-center gap-2 transition-colors"
                      >
                        <FolderOpen className="w-5 h-5" />
                        Open File Location
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        ) : null}
      </div>

      {/* Export Preview Modal */}
      {exportContent && selectedReport && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-8">
          <div className="bg-slate-800 rounded-xl w-full max-w-4xl max-h-full flex flex-col border border-slate-700">
            <div className="p-4 border-b border-slate-700 flex items-center justify-between">
              <h3 className="text-lg font-semibold text-white">{selectedReport.title}</h3>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => {
                    const blob = new Blob([exportContent], { type: "text/plain" });
                    const url = URL.createObjectURL(blob);
                    const a = document.createElement("a");
                    a.href = url;
                    a.download = `${selectedReport.title.replace(/\s+/g, "_")}.txt`;
                    a.click();
                    URL.revokeObjectURL(url);
                  }}
                  className="px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg flex items-center gap-2 text-sm transition-colors"
                >
                  <Download className="w-4 h-4" />
                  Download
                </button>
                <button
                  onClick={() => {
                    setExportContent(null);
                    setSelectedReport(null);
                  }}
                  className="px-3 py-1.5 bg-slate-700 hover:bg-slate-600 text-white rounded-lg text-sm transition-colors"
                >
                  Close
                </button>
              </div>
            </div>
            <div className="flex-1 overflow-auto p-4">
              <pre className="text-sm text-slate-300 whitespace-pre-wrap font-mono">
                {exportContent}
              </pre>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
