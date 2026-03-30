export const API_PREFIX = "/api/1.0";

export const STORAGE_KEYS = {
  connection: "grafana-utils.web.connection.v2",
  selection: "grafana-utils.web.selection.v2",
  drafts: "grafana-utils.web.drafts.v2",
  shellUi: "grafana-utils.web.shell-ui.v1",
};

const AUTH_MODES = new Set(["token", "basic"]);
const CONNECTION_STATES = new Set(["disconnected", "stale", "pending", "verified", "error"]);

function storageOrNull(storage) {
  if (storage) {
    return storage;
  }

  try {
    return window.sessionStorage;
  } catch {
    return null;
  }
}

export function safeStorageGet(key, fallback, storage) {
  try {
    const target = storageOrNull(storage);
    if (!target) {
      return fallback;
    }
    const raw = target.getItem(key);
    return raw ? JSON.parse(raw) : fallback;
  } catch {
    return fallback;
  }
}

export function safeStorageSet(key, value, storage) {
  try {
    const target = storageOrNull(storage);
    if (target) {
      target.setItem(key, JSON.stringify(value));
    }
  } catch {
    // Storage can fail in private or restricted browser contexts.
  }
}

export function textOrEmpty(value) {
  return value == null ? "" : String(value);
}

export function trimToUndefined(value) {
  const trimmed = textOrEmpty(value).trim();
  return trimmed ? trimmed : undefined;
}

export function numberOrUndefined(value) {
  const trimmed = textOrEmpty(value).trim();
  if (!trimmed) {
    return undefined;
  }
  const parsed = Number(trimmed);
  return Number.isFinite(parsed) ? parsed : NaN;
}

export function cloneValue(value) {
  return value == null ? value : JSON.parse(JSON.stringify(value));
}

export function loadConnectionState(storage) {
  const stored = safeStorageGet(STORAGE_KEYS.connection, null, storage) || {};
  const authMode = AUTH_MODES.has(stored.authMode)
    ? stored.authMode
    : stored.username || stored.password
      ? "basic"
      : "token";

  return {
    url: stored.url || "",
    authMode,
    apiToken: stored.apiToken || "",
    username: stored.username || "",
    password: stored.password || "",
    timeout: stored.timeout === undefined ? "" : stored.timeout,
    verifySsl: Boolean(stored.verifySsl),
  };
}

export function loadSelectionState(storage) {
  const stored = safeStorageGet(STORAGE_KEYS.selection, null, storage) || {};
  const actionByWorkspace = stored.actionByWorkspace && typeof stored.actionByWorkspace === "object"
    ? stored.actionByWorkspace
    : {};

  return {
    workspaceId: stored.workspaceId || null,
    actionByWorkspace,
  };
}

export function loadDraftsState(storage) {
  const stored = safeStorageGet(STORAGE_KEYS.drafts, null, storage);
  return stored && typeof stored === "object" ? stored : {};
}

export function loadShellUiState(storage) {
  const stored = safeStorageGet(STORAGE_KEYS.shellUi, null, storage) || {};
  return {
    connectionDrawerOpen: stored.connectionDrawerOpen === true,
    paletteQuery: typeof stored.paletteQuery === "string" ? stored.paletteQuery : "",
  };
}

export function persistConnectionState(state, storage = state.storage) {
  safeStorageSet(STORAGE_KEYS.connection, state.connection, storage);
}

export function persistSelectionState(state, storage = state.storage) {
  safeStorageSet(STORAGE_KEYS.selection, state.selection, storage);
}

export function persistDraftsState(state, storage = state.storage) {
  safeStorageSet(STORAGE_KEYS.drafts, state.drafts, storage);
}

export function persistShellUiState(state, storage = state.storage) {
  safeStorageSet(STORAGE_KEYS.shellUi, state.ui, storage);
}

export function normalizeConnection(connection = {}) {
  return {
    url: connection.url || "",
    authMode: AUTH_MODES.has(connection.authMode) ? connection.authMode : "token",
    apiToken: connection.apiToken || "",
    username: connection.username || "",
    password: connection.password || "",
    timeout: connection.timeout === undefined ? "" : connection.timeout,
    verifySsl: Boolean(connection.verifySsl),
  };
}

export function connectionFingerprint(connection) {
  const normalized = normalizeConnection(connection);
  return JSON.stringify({
    url: normalized.url,
    authMode: normalized.authMode,
    apiToken: normalized.apiToken,
    username: normalized.username,
    password: normalized.password,
    timeout: normalized.timeout,
    verifySsl: normalized.verifySsl,
  });
}

export function hasConnectionCredentials(connection) {
  const normalized = normalizeConnection(connection);
  if (!normalized.url.trim()) {
    return false;
  }
  if (normalized.authMode === "basic") {
    return Boolean(normalized.username.trim() && normalized.password.trim());
  }
  return Boolean(normalized.apiToken.trim());
}

export function defaultConnectionStatus() {
  return {
    state: "disconnected",
    message: "Disconnected. Enter a Grafana URL and credentials to start a session.",
    verifiedAt: null,
    changedAt: null,
    details: null,
    fingerprint: null,
  };
}

export function createShellState(options = {}) {
  const storage = storageOrNull(options.storage);
  const connection = normalizeConnection(options.connection || loadConnectionState(storage));
  return {
    storage,
    workspaces: Array.isArray(options.workspaces) ? options.workspaces.slice() : [],
    activeWorkspaceId: options.activeWorkspaceId || null,
    activeActionId: options.activeActionId || null,
    selection: loadSelectionState(storage),
    drafts: loadDraftsState(storage),
    connection,
    connectionStatus: defaultConnectionStatus(),
    ui: loadShellUiState(storage),
    statusText: options.statusText || "",
  };
}

export function workspaceById(state, workspaceId) {
  return state.workspaces.find((item) => item.id === workspaceId) || null;
}

export function actionById(workspace, actionId) {
  if (!workspace || !Array.isArray(workspace.actions)) {
    return null;
  }
  return workspace.actions.find((item) => item.id === actionId) || null;
}

export function currentWorkspace(state) {
  return workspaceById(state, state.activeWorkspaceId);
}

export function currentAction(state) {
  return actionById(currentWorkspace(state), state.activeActionId);
}

export function initialWorkspaceSelection(state) {
  if (state.selection.workspaceId && workspaceById(state, state.selection.workspaceId)) {
    return state.selection.workspaceId;
  }
  return state.workspaces[0] ? state.workspaces[0].id : null;
}

export function initialActionSelection(state, workspace) {
  if (!workspace) {
    return null;
  }
  const stored = state.selection.actionByWorkspace[workspace.id];
  if (stored && actionById(workspace, stored)) {
    return stored;
  }
  return workspace.actions[0] ? workspace.actions[0].id : null;
}

export function setWorkspaces(state, workspaces) {
  state.workspaces = Array.isArray(workspaces) ? workspaces.slice() : [];

  const workspaceId = initialWorkspaceSelection(state);
  if (!workspaceId) {
    state.activeWorkspaceId = null;
    state.activeActionId = null;
    return { workspace: null, action: null };
  }

  return selectWorkspace(state, workspaceId);
}

export function selectWorkspace(state, workspaceId, preferredActionId) {
  const workspace = workspaceById(state, workspaceId);
  if (!workspace) {
    return { workspace: null, action: null };
  }

  state.activeWorkspaceId = workspace.id;
  state.selection.workspaceId = workspace.id;
  persistSelectionState(state);

  const actionId = preferredActionId || initialActionSelection(state, workspace);
  if (!actionId) {
    state.activeActionId = null;
    return { workspace, action: null };
  }

  const action = actionById(workspace, actionId);
  if (!action) {
    state.activeActionId = null;
    return { workspace, action: null };
  }

  state.activeActionId = action.id;
  state.selection.actionByWorkspace[workspace.id] = action.id;
  persistSelectionState(state);
  return { workspace, action };
}

export function selectAction(state, actionId) {
  const workspace = currentWorkspace(state);
  const action = actionById(workspace, actionId);
  if (!workspace || !action) {
    return { workspace, action: null };
  }

  state.activeActionId = action.id;
  state.selection.actionByWorkspace[workspace.id] = action.id;
  persistSelectionState(state);
  return { workspace, action };
}

export function getActionDraft(state, actionId) {
  return cloneValue(state.drafts[actionId]) || {};
}

export function setActionDraft(state, actionId, draft) {
  state.drafts[actionId] = cloneValue(draft) || {};
  persistDraftsState(state);
}

export function clearActionDraft(state, actionId) {
  delete state.drafts[actionId];
  persistDraftsState(state);
}

export function setConnection(state, nextConnection, options = {}) {
  const previousFingerprint = connectionFingerprint(state.connection);
  state.connection = normalizeConnection(nextConnection);
  persistConnectionState(state);

  const nextFingerprint = connectionFingerprint(state.connection);
  if (options.keepStatus || previousFingerprint === nextFingerprint) {
    return state.connection;
  }

  if (!hasConnectionCredentials(state.connection)) {
    markConnectionDisconnected(
      state,
      "Disconnected. Enter a Grafana URL and credentials to start a session.",
    );
    return state.connection;
  }

  markConnectionStale(
    state,
    "Connection changed. Re-test this session before running connection-backed actions.",
  );
  return state.connection;
}

export function patchConnection(state, patch, options = {}) {
  return setConnection(state, { ...state.connection, ...patch }, options);
}

export function setConnectionStatus(state, nextStatus) {
  const nextState = CONNECTION_STATES.has(nextStatus?.state)
    ? nextStatus.state
    : "disconnected";

  state.connectionStatus = {
    ...defaultConnectionStatus(),
    ...nextStatus,
    state: nextState,
  };
  return state.connectionStatus;
}

export function markConnectionPending(state, message = "Testing connection...") {
  return setConnectionStatus(state, {
    state: "pending",
    message,
    changedAt: Date.now(),
    fingerprint: connectionFingerprint(state.connection),
  });
}

export function markConnectionVerified(state, details = {}, message) {
  const now = Date.now();
  return setConnectionStatus(state, {
    state: "verified",
    message: message || buildVerifiedConnectionMessage(details),
    verifiedAt: now,
    changedAt: now,
    details: cloneValue(details),
    fingerprint: connectionFingerprint(state.connection),
  });
}

export function markConnectionStale(state, message = "Connection changed. Re-test this session.") {
  return setConnectionStatus(state, {
    state: "stale",
    message,
    verifiedAt: state.connectionStatus.verifiedAt || null,
    changedAt: Date.now(),
    details: state.connectionStatus.details,
    fingerprint: connectionFingerprint(state.connection),
  });
}

export function markConnectionDisconnected(state, message = "Disconnected.") {
  return setConnectionStatus(state, {
    state: "disconnected",
    message,
    verifiedAt: null,
    changedAt: Date.now(),
    details: null,
    fingerprint: connectionFingerprint(state.connection),
  });
}

export function markConnectionError(state, message = "Connection test failed.", details = null) {
  return setConnectionStatus(state, {
    state: "error",
    message,
    verifiedAt: state.connectionStatus.verifiedAt || null,
    changedAt: Date.now(),
    details: details == null ? null : cloneValue(details),
    fingerprint: connectionFingerprint(state.connection),
  });
}

export function buildVerifiedConnectionMessage(details = {}) {
  const authMode = details.authMode ? `auth=${details.authMode}` : "verified";
  const orgName = details.orgName || "";
  const orgId = details.orgId == null ? "" : ` (org ${details.orgId})`;
  const target = orgName ? ` target=${orgName}${orgId}` : "";
  return `Connected. ${authMode}${target}`.trim();
}

export function isConnectionVerified(state) {
  return state.connectionStatus.state === "verified";
}

export function isConnectionPending(state) {
  return state.connectionStatus.state === "pending";
}

export function actionRequiresConnection(action) {
  return Boolean(action && action.requiresConnection);
}

export function isActionDisabled(state, action = currentAction(state)) {
  if (!action) {
    return true;
  }
  if (!actionRequiresConnection(action)) {
    return false;
  }
  return !isConnectionVerified(state);
}

export function actionDisabledReason(state, action = currentAction(state)) {
  if (!action) {
    return "Select an action first.";
  }
  if (!actionRequiresConnection(action)) {
    return "";
  }

  switch (state.connectionStatus.state) {
    case "verified":
      return "";
    case "pending":
      return "Connection test still running.";
    case "stale":
      return "Connection changed. Test the current session before running this action.";
    case "error":
      return "Connection test failed. Fix the session and try again.";
    default:
      return "This action requires a verified Grafana connection.";
  }
}

export function setConnectionDrawerOpen(state, open) {
  state.ui.connectionDrawerOpen = Boolean(open);
  persistShellUiState(state);
  return state.ui.connectionDrawerOpen;
}

export function toggleConnectionDrawer(state) {
  return setConnectionDrawerOpen(state, !state.ui.connectionDrawerOpen);
}

export function setPaletteQuery(state, query) {
  state.ui.paletteQuery = textOrEmpty(query);
  persistShellUiState(state);
  return state.ui.paletteQuery;
}
