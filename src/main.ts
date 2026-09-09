import { listen } from "@tauri-apps/api/event";

export interface AssetInfo {
  path: string;
  name: string;
  kind: string;
}

export interface InspectResult {
  is_asset: boolean;
  source: string;
  assets: AssetInfo[];
}

const emptyState = document.getElementById("empty-state")!;
const panel = document.getElementById("panel")!;
const tabsEl = document.getElementById("tabs")!;
const infoEl = document.getElementById("info")!;

let currentAssets: AssetInfo[] = [];
let activeIndex = 0;

export function renderTabs() {
  tabsEl.innerHTML = "";
  currentAssets.forEach((asset, i) => {
    const tab = document.createElement("button");
    tab.className = "tab" + (i === activeIndex ? " active" : "");
    tab.textContent = `${asset.name} (${asset.kind})`;
    tab.addEventListener("click", () => {
      activeIndex = i;
      renderTabs();
      renderInfo();
    });
    tabsEl.appendChild(tab);
  });
}

export function renderInfo() {
  const asset = currentAssets[activeIndex];
  if (!asset) return;
  infoEl.textContent = `${asset.name} — ${asset.kind.toUpperCase()} — ${asset.path}`;
}

export function showPreview(result: InspectResult) {
  currentAssets = result.assets;
  activeIndex = 0;
  emptyState.classList.add("hidden");
  panel.classList.remove("hidden");
  renderTabs();
  renderInfo();
}

async function main() {
  await listen<InspectResult>("preview", (event) => {
    showPreview(event.payload);
  });
}

main();
