import { beforeEach, describe, expect, it, vi } from "vitest";
import { type InspectResult } from "./main";

const listenMock = vi.fn();

vi.mock("@tauri-apps/api/event", () => ({
  listen: listenMock,
}));

function setupDom() {
  document.body.innerHTML = `
    <div id="empty-state"></div>
    <div id="panel" class="hidden"></div>
    <div id="tabs"></div>
    <div id="viewport"></div>
    <div id="info"></div>
  `;
}

const sampleResult: InspectResult = {
  is_asset: true,
  source: "C:\\models",
  assets: [
    { path: "C:\\models\\a.glb", name: "a.glb", kind: "glb" },
    { path: "C:\\models\\b.vrm", name: "b.vrm", kind: "vrm" },
  ],
};

describe("preview panel", () => {
  beforeEach(() => {
    setupDom();
    listenMock.mockClear();
    vi.resetModules();
  });

  it("shows the panel and renders one tab per asset", async () => {
    const { showPreview } = await import("./main");
    showPreview(sampleResult);

    expect(document.getElementById("panel")!.classList.contains("hidden")).toBe(
      false,
    );
    const tabs = document.getElementById("tabs")!;
    expect(tabs.children).toHaveLength(2);
    expect(tabs.children[0].textContent).toContain("a.glb");
    expect(tabs.children[1].textContent).toContain("b.vrm");
  });

  it("switches the active asset when a tab is clicked", async () => {
    const { showPreview } = await import("./main");
    showPreview(sampleResult);

    const tabs = document.querySelectorAll<HTMLButtonElement>("#tabs .tab");
    tabs[1].click();

    const info = document.getElementById("info")!;
    expect(info.textContent).toContain("b.vrm");
    expect(info.textContent).toContain("VRM");
  });

  it("registers a listener for the preview event on startup", async () => {
    await import("./main");
    expect(listenMock).toHaveBeenCalledWith("preview", expect.any(Function));
  });

  it("opens the panel when the backend emits a preview event", async () => {
    await import("./main");
    const handler = listenMock.mock.calls[0][1] as (event: {
      payload: InspectResult;
    }) => void;
    handler({ payload: sampleResult });

    expect(document.getElementById("panel")!.classList.contains("hidden")).toBe(
      false,
    );
    expect(document.getElementById("tabs")!.children).toHaveLength(2);
  });
});
