import { render, screen } from "@testing-library/react";
import { Dashboard } from "./Dashboard";
import type { Client, SystemInfo } from "@/types";

const mockClients: Client[] = [
  {
    id: "client-1",
    name: "Acme Corp",
    targetSubnet: "10.0.0.0/24",
    contactEmail: null,
    notes: null,
    createdAt: "2024-01-01T00:00:00Z",
    updatedAt: "2024-01-01T00:00:00Z",
  },
  {
    id: "client-2",
    name: "Beta LLC",
    targetSubnet: null,
    contactEmail: null,
    notes: null,
    createdAt: "2024-01-02T00:00:00Z",
    updatedAt: "2024-01-02T00:00:00Z",
  },
];

const mockSystemInfo: SystemInfo = {
  osName: "Linux",
  osVersion: "6.1",
  hostname: "optio-host",
  username: "alex",
  appVersion: "0.1.0",
  localIp: "10.0.0.5",
};

vi.mock("@/lib/commands", () => ({
  listClients: vi.fn(async () => mockClients),
  getSystemInfo: vi.fn(async () => mockSystemInfo),
}));

describe("Dashboard", () => {
  it("renders metrics once data loads", async () => {
    render(<Dashboard />);

    expect(await screen.findByText("Dashboard")).toBeInTheDocument();
    const activeClientsCard = await screen.findByText("Active Clients");
    expect(activeClientsCard.closest("div")).toHaveTextContent("2");
    expect(screen.getByText("System Status")).toBeInTheDocument();
    expect(screen.getByText("No clients yet. Create your first client to get started.")).not.toBeInTheDocument();
    expect(screen.getByText("Acme Corp")).toBeInTheDocument();
  });
});
