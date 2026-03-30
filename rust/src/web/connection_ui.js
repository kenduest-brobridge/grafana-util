import {
  actionDisabledReason,
  buildVerifiedConnectionMessage,
  hasConnectionCredentials,
  isActionDisabled,
  isConnectionPending,
  markConnectionDisconnected,
  markConnectionError,
  markConnectionPending,
  markConnectionStale,
  markConnectionVerified,
  numberOrUndefined,
  patchConnection,
  setConnectionDrawerOpen,
  textOrEmpty,
  trimToUndefined,
  toggleConnectionDrawer,
} from "./shell_state.js";

const DEFAULT_SELECTORS = {
  drawer: ".connection-drawer",
  fields: "#connection-fields",
  url: "#conn-url",
  authMode: "#conn-auth-mode",
  apiToken: "#conn-api-token",
  username: "#conn-username",
  password: "#conn-password",
  timeout: "#conn-timeout",
  verifySsl: "#conn-verify-ssl",
  status: "#connection-status",
  pill: "#connection-pill",
  testButton: "#test-connection-button",
  runButton: "#run-button",
};

function query(root, selector) {
  return selector ? root.querySelector(selector) : null;
}

function authSummary(connection) {
  return connection.authMode === "basic" ? "basic auth" : "API token";
}

function urlSummary(connection) {
  return connection.url ? connection.url : "no target";
}

export function readConnectionInputs(root = document, selectors = DEFAULT_SELECTORS) {
  const authModeNode = query(root, selectors.authMode);
  const authMode = authModeNode && authModeNode.value === "basic" ? "basic" : "token";

  return {
    url: trimToUndefined(query(root, selectors.url)?.value),
    authMode,
    apiToken: trimToUndefined(query(root, selectors.apiToken)?.value),
    username: trimToUndefined(query(root, selectors.username)?.value),
    password: trimToUndefined(query(root, selectors.password)?.value),
    timeout: numberOrUndefined(query(root, selectors.timeout)?.value),
    verifySsl: Boolean(query(root, selectors.verifySsl)?.checked),
  };
}

export function applyConnectionInputs(connection, root = document, selectors = DEFAULT_SELECTORS) {
  const fields = query(root, selectors.fields);
  const url = query(root, selectors.url);
  const authMode = query(root, selectors.authMode);
  const apiToken = query(root, selectors.apiToken);
  const username = query(root, selectors.username);
  const password = query(root, selectors.password);
  const timeout = query(root, selectors.timeout);
  const verifySsl = query(root, selectors.verifySsl);

  if (url) {
    url.value = connection.url || "";
  }
  if (authMode) {
    authMode.value = connection.authMode || "token";
  }
  if (apiToken) {
    apiToken.value = connection.apiToken || "";
  }
  if (username) {
    username.value = connection.username || "";
  }
  if (password) {
    password.value = connection.password || "";
  }
  if (timeout) {
    timeout.value = connection.timeout === undefined ? "" : connection.timeout;
  }
  if (verifySsl) {
    verifySsl.checked = Boolean(connection.verifySsl);
  }
  if (fields) {
    fields.dataset.authMode = connection.authMode || "token";
  }
}

export function connectionPayloadFromState(connection) {
  const payload = {
    url: trimToUndefined(connection.url),
    timeout: connection.timeout === "" ? undefined : connection.timeout,
    verifySsl: Boolean(connection.verifySsl),
  };

  if (connection.authMode === "basic") {
    payload.username = trimToUndefined(connection.username);
    payload.password = trimToUndefined(connection.password);
  } else {
    payload.apiToken = trimToUndefined(connection.apiToken);
  }

  return Object.fromEntries(
    Object.entries(payload).filter(([, value]) => value !== undefined)
  );
}

export function applyConnectionPresentation(state, options = {}) {
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const drawer = query(root, selectors.drawer);
  const fields = query(root, selectors.fields);
  const statusNode = query(root, selectors.status);
  const pill = query(root, selectors.pill);
  const testButton = query(root, selectors.testButton);
  const runButton = query(root, selectors.runButton);
  const currentAction = typeof options.getCurrentAction === "function"
    ? options.getCurrentAction()
    : null;

  applyConnectionInputs(state.connection, root, selectors);

  if (drawer) {
    drawer.dataset.open = state.ui.connectionDrawerOpen ? "true" : "false";
    drawer.dataset.connectionState = state.connectionStatus.state;
    drawer.hidden = !state.ui.connectionDrawerOpen;
  }
  if (fields) {
    fields.dataset.authMode = state.connection.authMode;
    fields.dataset.connectionState = state.connectionStatus.state;
  }
  if (statusNode) {
    statusNode.textContent = state.connectionStatus.message;
    statusNode.dataset.tone = state.connectionStatus.state;
    statusNode.dataset.connectionState = state.connectionStatus.state;
  }
  if (pill) {
    pill.dataset.connectionState = state.connectionStatus.state;
    pill.dataset.drawerOpen = state.ui.connectionDrawerOpen ? "true" : "false";
    pill.textContent = connectionPillLabel(state);
    pill.title = connectionPillTitle(state);
    pill.setAttribute("role", "button");
    pill.setAttribute("tabindex", "0");
    pill.setAttribute("aria-expanded", state.ui.connectionDrawerOpen ? "true" : "false");
  }
  if (testButton) {
    testButton.disabled = isConnectionPending(state);
    testButton.dataset.connectionState = state.connectionStatus.state;
  }
  if (runButton) {
    const disabled = isActionDisabled(state, currentAction);
    runButton.disabled = disabled;
    runButton.dataset.connectionState = state.connectionStatus.state;
    runButton.title = disabled ? actionDisabledReason(state, currentAction) : "";
  }
}

export function connectionPillLabel(state) {
  switch (state.connectionStatus.state) {
    case "verified":
      return `Connected: ${urlSummary(state.connection)}`;
    case "pending":
      return "Testing connection";
    case "stale":
      return "Connection changed";
    case "error":
      return "Connection failed";
    default:
      return hasConnectionCredentials(state.connection)
        ? `Session draft: ${authSummary(state.connection)}`
        : "Session credentials only";
  }
}

export function connectionPillTitle(state) {
  const prefix = `${connectionPillLabel(state)}.`;
  const suffix = state.ui.connectionDrawerOpen ? " Connection drawer open." : " Connection drawer closed.";
  return `${prefix}${suffix}`;
}

export function createConnectionController(options) {
  const state = options.state;
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const onChange = typeof options.onChange === "function" ? options.onChange : () => {};
  const onTest = typeof options.onTest === "function" ? options.onTest : null;
  const onToggleDrawer = typeof options.onToggleDrawer === "function" ? options.onToggleDrawer : null;

  function notify(reason) {
    applyConnectionPresentation(state, {
      root,
      selectors,
      getCurrentAction: options.getCurrentAction,
    });
    onChange({ reason, state });
  }

  function handleInputChange() {
    const nextConnection = readConnectionInputs(root, selectors);
    patchConnection(state, nextConnection);
    if (!hasConnectionCredentials(state.connection)) {
      markConnectionDisconnected(
        state,
        "Disconnected. Enter a Grafana URL and credentials to start a session.",
      );
    } else if (state.connectionStatus.state === "verified") {
      markConnectionStale(
        state,
        "Connection changed. Re-test this session before running connection-backed actions.",
      );
    }
    notify("connection-input");
  }

  async function runConnectionTest() {
    if (!hasConnectionCredentials(state.connection)) {
      markConnectionDisconnected(
        state,
        "Set a Grafana URL and one authentication method before testing the session.",
      );
      notify("connection-test-invalid");
      return null;
    }

    markConnectionPending(state);
    notify("connection-test-start");

    if (!onTest) {
      return state.connection;
    }

    try {
      const result = await onTest({
        connection: state.connection,
        payload: connectionPayloadFromState(state.connection),
        state,
      });
      markConnectionVerified(
        state,
        result || {},
        buildVerifiedConnectionMessage(result || {}),
      );
      notify("connection-test-success");
      return result;
    } catch (error) {
      markConnectionError(state, error?.message || "Connection test failed.");
      notify("connection-test-error");
      throw error;
    }
  }

  function setDrawerOpen(open) {
    setConnectionDrawerOpen(state, open);
    applyConnectionPresentation(state, {
      root,
      selectors,
      getCurrentAction: options.getCurrentAction,
    });
    if (onToggleDrawer) {
      onToggleDrawer({ open: state.ui.connectionDrawerOpen, state });
    }
    return state.ui.connectionDrawerOpen;
  }

  function toggleDrawer() {
    toggleConnectionDrawer(state);
    applyConnectionPresentation(state, {
      root,
      selectors,
      getCurrentAction: options.getCurrentAction,
    });
    if (onToggleDrawer) {
      onToggleDrawer({ open: state.ui.connectionDrawerOpen, state });
    }
    return state.ui.connectionDrawerOpen;
  }

  function bind() {
    [
      selectors.url,
      selectors.authMode,
      selectors.apiToken,
      selectors.username,
      selectors.password,
      selectors.timeout,
      selectors.verifySsl,
    ].forEach((selector) => {
      const node = query(root, selector);
      if (node) {
        node.addEventListener("input", handleInputChange);
        node.addEventListener("change", handleInputChange);
      }
    });

    const pill = query(root, selectors.pill);
    if (pill) {
      pill.addEventListener("click", toggleDrawer);
      pill.addEventListener("keydown", (event) => {
        if (event.key === "Enter" || event.key === " ") {
          event.preventDefault();
          toggleDrawer();
        }
      });
    }

    const testButton = query(root, selectors.testButton);
    if (testButton) {
      testButton.addEventListener("click", () => {
        void runConnectionTest();
      });
    }

    applyConnectionPresentation(state, {
      root,
      selectors,
      getCurrentAction: options.getCurrentAction,
    });
  }

  return {
    bind,
    readInputs: () => readConnectionInputs(root, selectors),
    apply: () => applyConnectionPresentation(state, {
      root,
      selectors,
      getCurrentAction: options.getCurrentAction,
    }),
    runConnectionTest,
    setDrawerOpen,
    toggleDrawer,
    markDisconnected(message) {
      markConnectionDisconnected(state, message);
      notify("connection-disconnected");
    },
    markStale(message) {
      markConnectionStale(state, message);
      notify("connection-stale");
    },
    markVerified(details, message) {
      markConnectionVerified(state, details, message);
      notify("connection-verified");
    },
    markError(message, details) {
      markConnectionError(state, message, details);
      notify("connection-error");
    },
  };
}
