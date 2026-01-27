import { useState, useEffect } from "react";
import { Sidebar } from "@/components/Sidebar";
import { Dashboard } from "@/components/Dashboard";
import { ClientOnboarding } from "@/components/ClientOnboarding";
import { GRCCommandCenter } from "@/components/GRCCommandCenter";
import { InfrastructureMigration } from "@/components/InfrastructureMigration";
import { NetworkIntelligence } from "@/components/NetworkIntelligence";
import { ReportingCenter } from "@/components/ReportingCenter";
import { ClientManagement } from "@/components/ClientManagement";
import { Settings } from "@/components/Settings";
import { Header } from "@/components/Header";
import { getSystemInfo } from "@/lib/commands";
import type { ViewMode, SystemInfo } from "@/types";

function App() {
  const [currentView, setCurrentView] = useState<ViewMode>("factory");
  const [systemInfo, setSystemInfo] = useState<SystemInfo | null>(null);

  useEffect(() => {
    // Load system info on startup
    getSystemInfo()
      .then(setSystemInfo)
      .catch(console.error);
  }, []);

  const renderContent = () => {
    switch (currentView) {
      case "dashboard":
        return <Dashboard />;
      case "factory":
        return <ClientOnboarding />;
      case "clients":
        return <ClientManagement />;
      case "grc":
        return <GRCCommandCenter />;
      case "infrastructure":
        return <InfrastructureMigration />;
      case "network":
        return <NetworkIntelligence />;
      case "reporting":
        return <ReportingCenter />;
      case "settings":
        return <Settings />;
      default:
        return <ClientOnboarding />;
    }
  };

  return (
    <div className="flex h-screen bg-slate-900">
      <Sidebar currentView={currentView} onViewChange={setCurrentView} />
      <div className="flex-1 flex flex-col overflow-hidden">
        <Header systemInfo={systemInfo} />
        <main className="flex-1 overflow-y-auto bg-slate-900">
          {renderContent()}
        </main>
      </div>
    </div>
  );
}

export default App;
