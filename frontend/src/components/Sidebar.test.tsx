import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Sidebar } from "./Sidebar";
import type { ViewMode } from "@/types";

describe("Sidebar", () => {
  it("highlights the active view and fires view change", async () => {
    const user = userEvent.setup();
    const onViewChange = vi.fn<(view: ViewMode) => void>();

    render(<Sidebar currentView="dashboard" onViewChange={onViewChange} />);

    const grcButton = screen.getByRole("button", { name: /GRC Center/i });
    await user.click(grcButton);

    expect(onViewChange).toHaveBeenCalledWith("grc");
    expect(screen.getByRole("button", { name: /Dashboard/i })).toBeInTheDocument();
  });
});
