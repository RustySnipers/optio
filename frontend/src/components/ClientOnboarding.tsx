import { useState, useEffect } from "react";
import {
  generateClientScript,
  listTemplates,
  getScriptPreview,
  validateConfig,
  getConsultantIp,
  generateAgentScript,
} from "@/lib/commands";
import type {
  TemplateInfo,
  ScriptConfigOptions,
  GenerateScriptResponse,
  ValidationResult,
  LogEntry,
  AgentScriptResponse,
} from "@/types";
import { cn } from "@/lib/utils";
import {
  Factory,
  Play,
  FileCode,
  Check,
  AlertTriangle,
  X,
  Copy,
  Download,
  RefreshCw,
  Server,
  Shield,
  Wifi,
  Terminal,
  Radio,
  Key,
} from "lucide-react";

export function ClientOnboarding() {
  // Form state
  const [clientName, setClientName] = useState("");
  const [targetSubnet, setTargetSubnet] = useState("");
  const [selectedTemplate, setSelectedTemplate] = useState("smart_prep");

  // Config options
  const [config, setConfig] = useState<ScriptConfigOptions>({
    enableWinrm: true,
    configureDns: false,
    dnsServers: [],
    installAgent: false,
    agentInstaller: "",
    enableFirewallLogging: true,
    customCommands: [],
  });

  // UI state
  const [templates, setTemplates] = useState<TemplateInfo[]>([]);
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [scriptPreview, setScriptPreview] = useState<string>("");
  const [generatedScript, setGeneratedScript] = useState<GenerateScriptResponse | null>(null);
  const [consultantIp, setConsultantIp] = useState<string>("");
  const [isGenerating, setIsGenerating] = useState(false);
  const [isValidating, setIsValidating] = useState(false);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [activeTab, setActiveTab] = useState<"config" | "preview" | "output" | "agent">("config");
  const [dnsInput, setDnsInput] = useState("");

  // Agent script state (Task A)
  const [agentClientIp, setAgentClientIp] = useState<string>("");
  const [agentAuthToken, setAgentAuthToken] = useState<string>("");
  const [agentCallbackPort, setAgentCallbackPort] = useState<number>(443);
  const [agentUseTls, setAgentUseTls] = useState<boolean>(true);
  const [agentHeartbeatInterval, setAgentHeartbeatInterval] = useState<number>(30);
  const [generatedAgentScript, setGeneratedAgentScript] = useState<AgentScriptResponse | null>(null);
  const [isGeneratingAgent, setIsGeneratingAgent] = useState(false);

  // Load initial data
  useEffect(() => {
    async function loadData() {
      try {
        const [templateData, ip] = await Promise.all([
          listTemplates(),
          getConsultantIp(),
        ]);
        setTemplates(templateData);
        setConsultantIp(ip);
        setAgentClientIp(ip); // Set default agent callback IP
        addLog("info", "Factory module initialized");
        addLog("info", `Consultant IP detected: ${ip}`);
      } catch (error) {
        addLog("error", `Failed to initialize: ${error}`);
      }
    }
    loadData();
  }, []);

  // Add log entry
  const addLog = (level: LogEntry["level"], message: string) => {
    setLogs((prev) => [
      { timestamp: new Date(), level, message },
      ...prev.slice(0, 99), // Keep last 100 entries
    ]);
  };

  // Handle validation
  const handleValidate = async () => {
    setIsValidating(true);
    try {
      const result = await validateConfig({
        clientName,
        targetSubnet,
        config,
      });
      setValidation(result);

      if (result.valid) {
        addLog("success", "Configuration validated successfully");
      } else {
        result.errors.forEach((err) => addLog("error", err));
      }
      result.warnings.forEach((warn) => addLog("warn", warn));
    } catch (error) {
      addLog("error", `Validation failed: ${error}`);
    } finally {
      setIsValidating(false);
    }
  };

  // Handle preview
  const handlePreview = async () => {
    if (!clientName || !targetSubnet) {
      addLog("warn", "Client name and target subnet are required for preview");
      return;
    }

    try {
      const preview = await getScriptPreview({
        templateName: selectedTemplate,
        config,
        clientName,
        targetSubnet,
      });
      setScriptPreview(preview);
      setActiveTab("preview");
      addLog("info", "Script preview generated");
    } catch (error) {
      addLog("error", `Preview failed: ${error}`);
    }
  };

  // Handle generation
  const handleGenerate = async () => {
    if (!validation?.valid) {
      addLog("error", "Please validate configuration before generating");
      return;
    }

    setIsGenerating(true);
    addLog("info", "Starting script generation...");

    try {
      const result = await generateClientScript({
        clientId: crypto.randomUUID(),
        clientName,
        targetSubnet,
        templateName: selectedTemplate,
        config,
      });

      setGeneratedScript(result);
      setActiveTab("output");
      addLog("success", `Script generated: ${result.scriptId}`);
      addLog("info", `Output path: ${result.outputPath}`);
      result.warnings.forEach((warn) => addLog("warn", warn));
    } catch (error) {
      addLog("error", `Generation failed: ${error}`);
    } finally {
      setIsGenerating(false);
    }
  };

  // Handle DNS server input
  const handleAddDns = () => {
    if (dnsInput && !config.dnsServers?.includes(dnsInput)) {
      setConfig((prev) => ({
        ...prev,
        dnsServers: [...(prev.dnsServers || []), dnsInput],
      }));
      setDnsInput("");
    }
  };

  const handleRemoveDns = (dns: string) => {
    setConfig((prev) => ({
      ...prev,
      dnsServers: prev.dnsServers?.filter((d) => d !== dns) || [],
    }));
  };

  // Copy to clipboard
  const handleCopy = async () => {
    const content = generatedScript?.scriptContent || scriptPreview;
    if (content) {
      await navigator.clipboard.writeText(content);
      addLog("info", "Script copied to clipboard");
    }
  };

  // Generate agent script (Task A)
  const handleGenerateAgentScript = async () => {
    if (!agentClientIp) {
      addLog("error", "Client IP is required for agent script generation");
      return;
    }

    // Auto-generate auth token if empty
    const authToken = agentAuthToken || crypto.randomUUID();
    if (!agentAuthToken) {
      setAgentAuthToken(authToken);
      addLog("info", "Generated new authentication token");
    }

    setIsGeneratingAgent(true);
    addLog("info", "Generating agent callback script...");

    try {
      const result = await generateAgentScript({
        clientIp: agentClientIp,
        authToken,
        callbackPort: agentCallbackPort,
        useTls: agentUseTls,
        heartbeatInterval: agentHeartbeatInterval,
      });

      setGeneratedAgentScript(result);
      addLog("success", `Agent script generated: ${result.scriptId}`);
      result.warnings.forEach((warn) => addLog("warn", warn));
    } catch (error) {
      addLog("error", `Agent script generation failed: ${error}`);
    } finally {
      setIsGeneratingAgent(false);
    }
  };

  // Copy agent script to clipboard
  const handleCopyAgentScript = async () => {
    if (generatedAgentScript?.scriptContent) {
      await navigator.clipboard.writeText(generatedAgentScript.scriptContent);
      addLog("info", "Agent script copied to clipboard");
    }
  };

  return (
    <div className="h-full flex">
      {/* Main Content */}
      <div className="flex-1 p-6 overflow-y-auto">
        {/* Header */}
        <div className="mb-6">
          <div className="flex items-center gap-3 mb-2">
            <div className="p-2 bg-optio-600/20 rounded-lg">
              <Factory className="w-6 h-6 text-optio-400" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-white">The Factory</h1>
              <p className="text-slate-400">Dynamic Client Provisioning Engine</p>
            </div>
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-2 mb-6 border-b border-slate-700 pb-2">
          {[
            { id: "config", label: "Configuration", icon: Server },
            { id: "preview", label: "Preview", icon: FileCode },
            { id: "output", label: "Output", icon: Terminal },
            { id: "agent", label: "Agent Script", icon: Radio },
          ].map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id as typeof activeTab)}
              className={cn(
                "flex items-center gap-2 px-4 py-2 rounded-t-lg transition-colors",
                activeTab === tab.id
                  ? "bg-slate-800 text-optio-400 border-b-2 border-optio-500"
                  : "text-slate-400 hover:text-white hover:bg-slate-800/50"
              )}
            >
              <tab.icon className="w-4 h-4" />
              {tab.label}
            </button>
          ))}
        </div>

        {/* Configuration Tab */}
        {activeTab === "config" && (
          <div className="space-y-6">
            {/* Client Info */}
            <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Server className="w-5 h-5 text-optio-400" />
                Client Information
              </h2>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Client Name *
                  </label>
                  <input
                    type="text"
                    value={clientName}
                    onChange={(e) => setClientName(e.target.value)}
                    placeholder="Acme Corporation"
                    className="w-full px-4 py-2.5 bg-slate-900 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-optio-500 focus:border-transparent"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Target Subnet *
                  </label>
                  <input
                    type="text"
                    value={targetSubnet}
                    onChange={(e) => setTargetSubnet(e.target.value)}
                    placeholder="192.168.1.0/24"
                    className="w-full px-4 py-2.5 bg-slate-900 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-optio-500 focus:border-transparent"
                  />
                </div>
              </div>

              {/* Consultant IP Display */}
              <div className="mt-4 flex items-center gap-2 text-sm">
                <Wifi className="w-4 h-4 text-secure" />
                <span className="text-slate-400">Consultant IP:</span>
                <code className="text-optio-400 bg-slate-900 px-2 py-0.5 rounded">
                  {consultantIp || "Detecting..."}
                </code>
              </div>
            </div>

            {/* Template Selection */}
            <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <FileCode className="w-5 h-5 text-optio-400" />
                Script Template
              </h2>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                {templates.map((template) => (
                  <button
                    key={template.name}
                    onClick={() => setSelectedTemplate(template.name)}
                    className={cn(
                      "p-4 rounded-lg border text-left transition-all",
                      selectedTemplate === template.name
                        ? "bg-optio-600/20 border-optio-500"
                        : "bg-slate-900/50 border-slate-700 hover:border-slate-600"
                    )}
                  >
                    <div className="flex items-start justify-between">
                      <div>
                        <p className="font-medium text-white">{template.name}</p>
                        <p className="text-sm text-slate-400 mt-1">
                          {template.description}
                        </p>
                        <span className="inline-block mt-2 text-xs bg-slate-700 text-slate-300 px-2 py-0.5 rounded">
                          {template.category}
                        </span>
                      </div>
                      {selectedTemplate === template.name && (
                        <Check className="w-5 h-5 text-optio-400" />
                      )}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {/* Configuration Options */}
            <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Shield className="w-5 h-5 text-optio-400" />
                Target State Configuration
              </h2>
              <div className="space-y-4">
                {/* Toggle Options */}
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <ToggleOption
                    label="Enable WinRM"
                    description="Configure Windows Remote Management"
                    checked={config.enableWinrm}
                    onChange={(checked) =>
                      setConfig((prev) => ({ ...prev, enableWinrm: checked }))
                    }
                  />
                  <ToggleOption
                    label="Firewall Logging"
                    description="Enable Windows Firewall logging"
                    checked={config.enableFirewallLogging}
                    onChange={(checked) =>
                      setConfig((prev) => ({ ...prev, enableFirewallLogging: checked }))
                    }
                  />
                  <ToggleOption
                    label="Configure DNS"
                    description="Set custom DNS servers"
                    checked={config.configureDns}
                    onChange={(checked) =>
                      setConfig((prev) => ({ ...prev, configureDns: checked }))
                    }
                  />
                  <ToggleOption
                    label="Install Agent"
                    description="Deploy security monitoring agent"
                    checked={config.installAgent}
                    onChange={(checked) =>
                      setConfig((prev) => ({ ...prev, installAgent: checked }))
                    }
                  />
                </div>

                {/* DNS Servers */}
                {config.configureDns && (
                  <div className="mt-4 p-4 bg-slate-900/50 rounded-lg">
                    <label className="block text-sm font-medium text-slate-300 mb-2">
                      DNS Servers
                    </label>
                    <div className="flex gap-2 mb-2">
                      <input
                        type="text"
                        value={dnsInput}
                        onChange={(e) => setDnsInput(e.target.value)}
                        onKeyDown={(e) => e.key === "Enter" && handleAddDns()}
                        placeholder="8.8.8.8"
                        className="flex-1 px-3 py-2 bg-slate-800 border border-slate-700 rounded text-white text-sm"
                      />
                      <button
                        onClick={handleAddDns}
                        className="px-4 py-2 bg-optio-600 text-white rounded hover:bg-optio-700 transition-colors"
                      >
                        Add
                      </button>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {config.dnsServers?.map((dns) => (
                        <span
                          key={dns}
                          className="inline-flex items-center gap-1 px-2 py-1 bg-slate-700 rounded text-sm text-white"
                        >
                          {dns}
                          <button
                            onClick={() => handleRemoveDns(dns)}
                            className="hover:text-critical"
                          >
                            <X className="w-3 h-3" />
                          </button>
                        </span>
                      ))}
                    </div>
                  </div>
                )}

                {/* Agent Installer */}
                {config.installAgent && (
                  <div className="mt-4 p-4 bg-slate-900/50 rounded-lg">
                    <label className="block text-sm font-medium text-slate-300 mb-2">
                      Agent Installer Path/URL
                    </label>
                    <input
                      type="text"
                      value={config.agentInstaller || ""}
                      onChange={(e) =>
                        setConfig((prev) => ({ ...prev, agentInstaller: e.target.value }))
                      }
                      placeholder="https://download.example.com/agent.exe"
                      className="w-full px-3 py-2 bg-slate-800 border border-slate-700 rounded text-white text-sm"
                    />
                  </div>
                )}
              </div>
            </div>

            {/* Validation Status */}
            {validation && (
              <div
                className={cn(
                  "p-4 rounded-lg border",
                  validation.valid
                    ? "bg-secure/10 border-secure/30"
                    : "bg-critical/10 border-critical/30"
                )}
              >
                <div className="flex items-center gap-2 mb-2">
                  {validation.valid ? (
                    <Check className="w-5 h-5 text-secure" />
                  ) : (
                    <AlertTriangle className="w-5 h-5 text-critical" />
                  )}
                  <span
                    className={cn(
                      "font-medium",
                      validation.valid ? "text-secure" : "text-critical"
                    )}
                  >
                    {validation.valid ? "Configuration Valid" : "Validation Failed"}
                  </span>
                </div>
                {validation.errors.length > 0 && (
                  <ul className="text-sm text-critical space-y-1 ml-7">
                    {validation.errors.map((err, i) => (
                      <li key={i}>{err}</li>
                    ))}
                  </ul>
                )}
                {validation.warnings.length > 0 && (
                  <ul className="text-sm text-warning space-y-1 ml-7 mt-2">
                    {validation.warnings.map((warn, i) => (
                      <li key={i}>{warn}</li>
                    ))}
                  </ul>
                )}
              </div>
            )}

            {/* Action Buttons */}
            <div className="flex gap-3">
              <button
                onClick={handleValidate}
                disabled={isValidating}
                className="flex items-center gap-2 px-6 py-2.5 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors disabled:opacity-50"
              >
                {isValidating ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Check className="w-4 h-4" />
                )}
                Validate
              </button>
              <button
                onClick={handlePreview}
                disabled={!clientName || !targetSubnet}
                className="flex items-center gap-2 px-6 py-2.5 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors disabled:opacity-50"
              >
                <FileCode className="w-4 h-4" />
                Preview
              </button>
              <button
                onClick={handleGenerate}
                disabled={isGenerating || !validation?.valid}
                className="flex items-center gap-2 px-6 py-2.5 bg-optio-600 text-white rounded-lg hover:bg-optio-700 transition-colors disabled:opacity-50"
              >
                {isGenerating ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Play className="w-4 h-4" />
                )}
                Generate Script
              </button>
            </div>
          </div>
        )}

        {/* Preview Tab */}
        {activeTab === "preview" && (
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-white">Script Preview</h2>
              <button
                onClick={handleCopy}
                className="flex items-center gap-2 px-4 py-2 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors"
              >
                <Copy className="w-4 h-4" />
                Copy
              </button>
            </div>
            <div className="bg-slate-950 border border-slate-800 rounded-lg p-4 overflow-auto max-h-[600px]">
              <pre className="code-preview text-slate-300 whitespace-pre-wrap">
                {scriptPreview || "Click 'Preview' to generate a script preview."}
              </pre>
            </div>
          </div>
        )}

        {/* Output Tab */}
        {activeTab === "output" && (
          <div className="space-y-4">
            {generatedScript ? (
              <>
                <div className="bg-secure/10 border border-secure/30 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Check className="w-5 h-5 text-secure" />
                    <span className="font-medium text-secure">Script Generated Successfully</span>
                  </div>
                  <div className="text-sm text-slate-300 space-y-1">
                    <p>
                      <strong>Script ID:</strong>{" "}
                      <code className="bg-slate-800 px-2 py-0.5 rounded">{generatedScript.scriptId}</code>
                    </p>
                    <p>
                      <strong>Output Path:</strong>{" "}
                      <code className="bg-slate-800 px-2 py-0.5 rounded">{generatedScript.outputPath}</code>
                    </p>
                    <p>
                      <strong>Generated:</strong> {new Date(generatedScript.generatedAt).toLocaleString()}
                    </p>
                  </div>
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={handleCopy}
                    className="flex items-center gap-2 px-4 py-2 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors"
                  >
                    <Copy className="w-4 h-4" />
                    Copy Script
                  </button>
                  <button className="flex items-center gap-2 px-4 py-2 bg-optio-600 text-white rounded-lg hover:bg-optio-700 transition-colors">
                    <Download className="w-4 h-4" />
                    Open in Explorer
                  </button>
                </div>

                <div className="bg-slate-950 border border-slate-800 rounded-lg p-4 overflow-auto max-h-[500px]">
                  <pre className="code-preview text-slate-300 whitespace-pre-wrap">
                    {generatedScript.scriptContent}
                  </pre>
                </div>
              </>
            ) : (
              <div className="text-center py-12 text-slate-400">
                <FileCode className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>No script generated yet.</p>
                <p className="text-sm">Configure and generate a script to see output here.</p>
              </div>
            )}
          </div>
        )}

        {/* Agent Script Tab (Task A) */}
        {activeTab === "agent" && (
          <div className="space-y-6">
            {/* Agent Configuration */}
            <div className="bg-slate-800/50 border border-slate-700/50 rounded-xl p-6">
              <h2 className="text-lg font-semibold text-white mb-4 flex items-center gap-2">
                <Radio className="w-5 h-5 text-optio-400" />
                Agent Callback Configuration
              </h2>
              <p className="text-sm text-slate-400 mb-6">
                Generate a PowerShell agent script that establishes a reverse connection to the Optio server.
                The script includes hardcoded connection parameters for secure callback communication.
              </p>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Callback IP Address *
                  </label>
                  <input
                    type="text"
                    value={agentClientIp}
                    onChange={(e) => setAgentClientIp(e.target.value)}
                    placeholder="192.168.1.100"
                    className="w-full px-4 py-2.5 bg-slate-900 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-optio-500 focus:border-transparent"
                  />
                  <p className="text-xs text-slate-500 mt-1">
                    The IP address of this Optio server
                  </p>
                </div>
                <div>
                  <label className="block text-sm font-medium text-slate-300 mb-2">
                    Callback Port
                  </label>
                  <input
                    type="number"
                    value={agentCallbackPort}
                    onChange={(e) => setAgentCallbackPort(parseInt(e.target.value) || 443)}
                    placeholder="443"
                    min={1}
                    max={65535}
                    className="w-full px-4 py-2.5 bg-slate-900 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-optio-500 focus:border-transparent"
                  />
                  <p className="text-xs text-slate-500 mt-1">
                    Port for callback (443 recommended)
                  </p>
                </div>
                <div className="md:col-span-2">
                  <label className="block text-sm font-medium text-slate-300 mb-2 flex items-center gap-2">
                    <Key className="w-4 h-4" />
                    Authentication Token
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={agentAuthToken}
                      onChange={(e) => setAgentAuthToken(e.target.value)}
                      placeholder="Leave empty to auto-generate"
                      className="flex-1 px-4 py-2.5 bg-slate-900 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-optio-500 focus:border-transparent font-mono text-sm"
                    />
                    <button
                      onClick={() => setAgentAuthToken(crypto.randomUUID())}
                      className="px-4 py-2.5 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors"
                    >
                      Generate
                    </button>
                  </div>
                  <p className="text-xs text-slate-500 mt-1">
                    Unique token for authenticating the agent connection
                  </p>
                </div>
              </div>

              {/* Options */}
              <div className="mt-6 grid grid-cols-1 md:grid-cols-2 gap-4">
                <ToggleOption
                  label="Enable TLS"
                  description="Encrypt callback communication (recommended)"
                  checked={agentUseTls}
                  onChange={setAgentUseTls}
                />
                <div className="p-3 bg-slate-900/50 rounded-lg">
                  <label className="block text-sm font-medium text-white mb-2">
                    Heartbeat Interval
                  </label>
                  <div className="flex items-center gap-2">
                    <input
                      type="number"
                      value={agentHeartbeatInterval}
                      onChange={(e) => setAgentHeartbeatInterval(parseInt(e.target.value) || 30)}
                      min={5}
                      max={3600}
                      className="w-24 px-3 py-2 bg-slate-800 border border-slate-700 rounded text-white text-sm"
                    />
                    <span className="text-sm text-slate-400">seconds</span>
                  </div>
                </div>
              </div>
            </div>

            {/* Generate Button */}
            <div className="flex gap-3">
              <button
                onClick={handleGenerateAgentScript}
                disabled={isGeneratingAgent || !agentClientIp}
                className="flex items-center gap-2 px-6 py-2.5 bg-optio-600 text-white rounded-lg hover:bg-optio-700 transition-colors disabled:opacity-50"
              >
                {isGeneratingAgent ? (
                  <RefreshCw className="w-4 h-4 animate-spin" />
                ) : (
                  <Play className="w-4 h-4" />
                )}
                Generate Agent Script
              </button>
            </div>

            {/* Generated Agent Script Output */}
            {generatedAgentScript && (
              <div className="space-y-4">
                <div className="bg-secure/10 border border-secure/30 rounded-lg p-4">
                  <div className="flex items-center gap-2 mb-2">
                    <Check className="w-5 h-5 text-secure" />
                    <span className="font-medium text-secure">Agent Script Generated Successfully</span>
                  </div>
                  <div className="text-sm text-slate-300 space-y-1">
                    <p>
                      <strong>Script ID:</strong>{" "}
                      <code className="bg-slate-800 px-2 py-0.5 rounded">{generatedAgentScript.scriptId}</code>
                    </p>
                    <p>
                      <strong>Generated:</strong> {new Date(generatedAgentScript.generatedAt).toLocaleString()}
                    </p>
                    <p>
                      <strong>Callback:</strong>{" "}
                      <code className="bg-slate-800 px-2 py-0.5 rounded">{agentClientIp}:{agentCallbackPort}</code>
                    </p>
                  </div>
                  {generatedAgentScript.warnings.length > 0 && (
                    <div className="mt-3 space-y-1">
                      {generatedAgentScript.warnings.map((warn, i) => (
                        <div key={i} className="flex items-center gap-2 text-warning text-sm">
                          <AlertTriangle className="w-4 h-4" />
                          {warn}
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                <div className="flex gap-3">
                  <button
                    onClick={handleCopyAgentScript}
                    className="flex items-center gap-2 px-4 py-2 bg-slate-700 text-white rounded-lg hover:bg-slate-600 transition-colors"
                  >
                    <Copy className="w-4 h-4" />
                    Copy Script
                  </button>
                </div>

                <div className="bg-slate-950 border border-slate-800 rounded-lg p-4 overflow-auto max-h-[500px]">
                  <pre className="code-preview text-slate-300 whitespace-pre-wrap text-xs">
                    {generatedAgentScript.scriptContent}
                  </pre>
                </div>
              </div>
            )}

            {!generatedAgentScript && (
              <div className="text-center py-12 text-slate-400 bg-slate-800/50 border border-slate-700/50 rounded-xl">
                <Radio className="w-12 h-12 mx-auto mb-4 opacity-50" />
                <p>Configure the callback parameters above and click "Generate Agent Script"</p>
                <p className="text-sm mt-2">
                  The generated script will establish a reverse connection to this Optio server.
                </p>
              </div>
            )}
          </div>
        )}
      </div>

      {/* Status Log Sidebar */}
      <div className="w-80 bg-slate-950 border-l border-slate-800 flex flex-col">
        <div className="p-4 border-b border-slate-800">
          <h3 className="text-sm font-semibold text-slate-300">Status Log</h3>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-2">
          {logs.map((log, i) => (
            <div
              key={i}
              className={cn(
                "text-xs p-2 rounded",
                log.level === "error" && "bg-critical/10 text-critical",
                log.level === "warn" && "bg-warning/10 text-warning",
                log.level === "success" && "bg-secure/10 text-secure",
                log.level === "info" && "bg-slate-800/50 text-slate-400"
              )}
            >
              <span className="text-slate-500">
                {log.timestamp.toLocaleTimeString()}
              </span>
              <span className="ml-2">{log.message}</span>
            </div>
          ))}
          {logs.length === 0 && (
            <p className="text-sm text-slate-500 text-center py-4">
              No log entries yet.
            </p>
          )}
        </div>
      </div>
    </div>
  );
}

// Toggle Option Component
interface ToggleOptionProps {
  label: string;
  description: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
}

function ToggleOption({ label, description, checked, onChange }: ToggleOptionProps) {
  return (
    <label className="flex items-start gap-3 p-3 bg-slate-900/50 rounded-lg cursor-pointer hover:bg-slate-900 transition-colors">
      <div className="relative mt-0.5">
        <input
          type="checkbox"
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          className="sr-only"
        />
        <div
          className={cn(
            "w-10 h-6 rounded-full transition-colors",
            checked ? "bg-optio-600" : "bg-slate-700"
          )}
        />
        <div
          className={cn(
            "absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform",
            checked && "translate-x-4"
          )}
        />
      </div>
      <div>
        <p className="text-sm font-medium text-white">{label}</p>
        <p className="text-xs text-slate-500">{description}</p>
      </div>
    </label>
  );
}
