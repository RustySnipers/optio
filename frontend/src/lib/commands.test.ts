import {
  createClient,
  getClient,
  getSystemInfo,
  listTemplates,
  updateClient,
} from "./commands";

const invokeMock = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

describe("commands", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("calls invoke for listTemplates", async () => {
    invokeMock.mockResolvedValue([]);

    await listTemplates();

    expect(invokeMock).toHaveBeenCalledWith("list_templates");
  });

  it("calls invoke with request payloads", async () => {
    invokeMock.mockResolvedValue({});

    await createClient({ name: "Acme" });
    await getClient("client-1");
    await updateClient({ id: "client-1", name: "Acme Updated" });
    await getSystemInfo();

    expect(invokeMock).toHaveBeenCalledWith("create_client", {
      request: { name: "Acme" },
    });
    expect(invokeMock).toHaveBeenCalledWith("get_client", { id: "client-1" });
    expect(invokeMock).toHaveBeenCalledWith("update_client", {
      request: { id: "client-1", name: "Acme Updated" },
    });
    expect(invokeMock).toHaveBeenCalledWith("get_system_info");
  });
});
