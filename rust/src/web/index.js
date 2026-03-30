import {
  API_PREFIX,
  createShellState,
  currentAction,
  currentWorkspace,
  setWorkspaces,
} from "./shell_state.js";
import { createShellUi, validateActionDraft } from "./shell_ui.js";
import {
  connectionPayloadFromState,
  createConnectionController,
} from "./connection_ui.js";
import { createPaletteController } from "./palette.js";
import { setupResultStage } from "./result_stage.js";

const state = createShellState();

let shellUi;
let connectionUi;
let paletteUi;
let resultStage;

function byId(id) {
  return document.getElementById(id);
}

async function parseJsonResponse(response) {
  const raw = await response.text();
  if (!raw.trim()) {
    return {};
  }
  try {
    return JSON.parse(raw);
  } catch (_error) {
    return { raw };
  }
}

async function fetchJson(path, options = {}) {
  const response = await fetch(path, options);
  const parsed = await parseJsonResponse(response);
  if (!response.ok) {
    const message = parsed.error || parsed.message || `HTTP ${response.status}`;
    const error = new Error(message);
    error.status = response.status;
    error.payload = parsed;
    throw error;
  }
  return parsed;
}

function setStatus(text) {
  state.statusText = text;
  if (shellUi) {
    shellUi.setStatus(text);
  } else {
    const statusLine = byId("status-line");
    if (statusLine) {
      statusLine.textContent = text;
    }
  }
}

function updateNativeLink() {
  const link = byId("native-link");
  if (!link) {
    return;
  }
  const url = (state.connection.url || "").trim();
  if (url) {
    link.href = url;
    link.classList.remove("is-disabled");
    link.textContent = "Open Grafana";
  } else {
    link.href = "#";
    link.classList.add("is-disabled");
    link.textContent = "Open Grafana";
  }
}

function buildActionPayload(action, draft) {
  const payload = { ...(draft || {}) };
  if (action?.requiresConnection) {
    Object.assign(payload, connectionPayloadFromState(state.connection));
  }
  return Object.fromEntries(
    Object.entries(payload).filter(([, value]) => value !== undefined)
  );
}

function buildLoadingResponse(action) {
  return {
    action: action?.id || "loading",
    title: action?.title || "Loading",
    uiMode: action?.uiMode || action?.ui_mode || "document",
    readOnly: true,
    textLines: [`Running ${action?.title || "request"}...`],
    document: {
      kind: "grafana-utils-web-pending",
      summary: {
        state: "loading",
      },
      action: action?.id || "",
      title: action?.title || "",
    },
  };
}

function buildErrorResponse(action, error) {
  return {
    action: action?.id || "error",
    title: action?.title || "Error",
    uiMode: action?.uiMode || action?.ui_mode || "document",
    readOnly: true,
    textLines: [error.message],
    document: {
      kind: "grafana-utils-web-error",
      summary: {
        state: "error",
        status: error.status || "request-failed",
      },
      message: error.message,
      payload: error.payload || null,
    },
  };
}

function refreshChrome() {
  if (shellUi) {
    shellUi.rerender();
  }
  if (connectionUi) {
    connectionUi.apply();
  }
  updateNativeLink();
}

function resetResultStage(message) {
  if (resultStage) {
    resultStage.activateTab("visual");
    resultStage.reset();
  }
  if (message) {
    setStatus(message);
  }
}

function handleSelectionChange() {
  const workspace = currentWorkspace(state);
  const action = currentAction(state);
  refreshChrome();
  resetResultStage(
    workspace && action
      ? `Ready: ${workspace.title} / ${action.title}`
      : "Select a workspace action."
  );
}

async function handleRun({ action, draft }) {
  if (!action) {
    setStatus("Select an action first.");
    return;
  }

  const validationError = validateActionDraft(action, draft);
  if (validationError) {
    setStatus(validationError);
    return;
  }

  const payload = buildActionPayload(action, draft);
  setStatus(`Submitting ${action.method} ${action.path}...`);
  resultStage.activateTab("visual");
  resultStage.render(action, buildLoadingResponse(action));

  try {
    const response = await fetchJson(action.path, {
      method: action.method,
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify(payload),
    });
    resultStage.render(action, response);
    setStatus(`Completed ${action.title}`);
  } catch (error) {
    resultStage.activateTab("source");
    resultStage.render(action, buildErrorResponse(action, error));
    setStatus(`Request failed: ${error.message}`);
  }
}

function bindGlobalShellEvents() {
  document.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
      event.preventDefault();
      byId("global-search")?.focus();
    }
    if (event.key === "Escape" && resultStage?.getState().fullscreen) {
      resultStage.setFullscreen(false);
    }
  });
}

async function bootstrap() {
  resultStage = setupResultStage();
  shellUi = createShellUi({
    state,
    onRun: handleRun,
    onReset: ({ action }) => {
      resetResultStage(action ? `Reset ${action.title}` : "Inputs reset.");
    },
    onSelectionChange: handleSelectionChange,
  });
  connectionUi = createConnectionController({
    state,
    getCurrentAction: () => currentAction(state),
    onChange: ({ reason }) => {
      refreshChrome();
      if (reason === "connection-test-success") {
        connectionUi.setDrawerOpen(false);
      }
      setStatus(state.connectionStatus.message);
    },
    onTest: async ({ payload }) => {
      return fetchJson(`${API_PREFIX}/connection/test`, {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify(payload),
      });
    },
  });

  shellUi.bind();
  connectionUi.bind();
  bindGlobalShellEvents();
  updateNativeLink();
  resultStage.reset();

  setStatus("Loading workspace registry...");
  const workspaces = await fetchJson(`${API_PREFIX}/workspaces`);
  setWorkspaces(state, workspaces);

  shellUi.rerender();
  connectionUi.apply();

  paletteUi = createPaletteController({
    state,
    onSelect: () => {
      handleSelectionChange();
    },
  });
  paletteUi.bind();

  const workspace = currentWorkspace(state);
  const action = currentAction(state);
  setStatus(
    workspace && action
      ? `Loaded ${state.workspaces.length} workspaces. Ready: ${workspace.title} / ${action.title}`
      : `Loaded ${state.workspaces.length} workspaces.`
  );
}

bootstrap().catch((error) => {
  setStatus(`Failed to load workspace registry: ${error.message}`);
  resultStage?.render(
    null,
    buildErrorResponse(null, {
      message: error.message,
      payload: error.payload || null,
      status: error.status || "bootstrap-failed",
    })
  );
});
