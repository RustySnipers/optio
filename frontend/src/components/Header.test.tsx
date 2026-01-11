import { render, screen } from "@testing-library/react";
import { Header } from "./Header";
import type { SystemInfo } from "@/types";

const baseSystemInfo: SystemInfo = {
  osName: "Linux",
  osVersion: "6.1",
  hostname: "optio-host",
  username: "alex",
  appVersion: "0.1.0",
  localIp: "10.0.0.5",
};

describe("Header", () => {
  it("shows offline status when system info is unavailable", () => {
    render(<Header systemInfo={null} />);

    expect(screen.getByText("Offline")).toBeInTheDocument();
    expect(screen.queryByText("optio-host")).not.toBeInTheDocument();
  });

  it("renders system info when available", () => {
    render(<Header systemInfo={baseSystemInfo} />);

    expect(screen.getByText("10.0.0.5")).toBeInTheDocument();
    expect(screen.getByText("optio-host")).toBeInTheDocument();
    expect(screen.getByText("v0.1.0")).toBeInTheDocument();
  });
});
