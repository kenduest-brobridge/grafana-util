import { currentWorkspace, selectAction, selectWorkspace, setPaletteQuery } from "./shell_state.js";

const DEFAULT_SELECTORS = {
  input: "#global-search",
};

function query(root, selector) {
  return selector ? root.querySelector(selector) : null;
}

function ensureResultsHost(root, input) {
  const existing = root.querySelector("[data-shell-palette-results]");
  if (existing) {
    return existing;
  }

  const host = document.createElement("div");
  host.dataset.shellPaletteResults = "true";
  host.className = "shell-palette-results";
  host.hidden = true;

  const topBarControls = input?.closest(".top-bar-controls");
  if (topBarControls) {
    topBarControls.appendChild(host);
  } else if (input?.parentElement) {
    input.parentElement.appendChild(host);
  } else {
    root.body ? root.body.appendChild(host) : root.appendChild(host);
  }

  return host;
}

export function buildPaletteIndex(workspaces) {
  const entries = [];

  (workspaces || []).forEach((workspace) => {
    entries.push({
      kind: "workspace",
      workspaceId: workspace.id,
      actionId: null,
      title: workspace.title,
      subtitle: workspace.description || "",
      searchText: `${workspace.title} ${workspace.description || ""}`.toLowerCase(),
    });

    (workspace.actions || []).forEach((action) => {
      entries.push({
        kind: "action",
        workspaceId: workspace.id,
        actionId: action.id,
        title: action.title,
        subtitle: workspace.title,
        searchText: `${workspace.title} ${workspace.description || ""} ${action.title} ${action.description || ""}`.toLowerCase(),
      });
    });
  });

  return entries;
}

export function filterPaletteEntries(entries, queryText) {
  const query = String(queryText || "").trim().toLowerCase();
  if (!query) {
    return entries.slice(0, 24);
  }

  const tokens = query.split(/\s+/).filter(Boolean);
  return entries
    .filter((entry) => tokens.every((token) => entry.searchText.includes(token)))
    .slice(0, 24);
}

export function renderPaletteResults(resultsHost, results, activeIndex = 0) {
  resultsHost.replaceChildren();

  if (results.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state compact";
    empty.textContent = "No matching workspaces or actions.";
    resultsHost.appendChild(empty);
    return;
  }

  results.forEach((result, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "shell-palette-option";
    button.dataset.paletteIndex = String(index);
    button.dataset.paletteKind = result.kind;
    button.dataset.workspaceId = result.workspaceId;
    button.dataset.actionId = result.actionId || "";
    button.setAttribute("aria-selected", index === activeIndex ? "true" : "false");

    const title = document.createElement("strong");
    title.textContent = result.title;
    const meta = document.createElement("span");
    meta.textContent = result.subtitle;

    button.append(title, meta);
    resultsHost.appendChild(button);
  });
}

export function createPaletteController(options) {
  const state = options.state;
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const input = query(root, selectors.input);
  const resultsHost = options.resultsHost || ensureResultsHost(root, input);
  const onChange = typeof options.onChange === "function" ? options.onChange : null;
  const onSelect = typeof options.onSelect === "function" ? options.onSelect : null;

  let activeIndex = 0;
  let lastResults = [];

  function index() {
    return buildPaletteIndex(state.workspaces);
  }

  function notify(reason, result) {
    if (onChange) {
      onChange({
        reason,
        query: state.ui.paletteQuery,
        results: lastResults,
        result: result || null,
      });
    }
  }

  function close() {
    resultsHost.hidden = true;
    activeIndex = 0;
    notify("close");
  }

  function open() {
    resultsHost.hidden = false;
    notify("open");
  }

  function applyResult(result) {
    if (!result) {
      return;
    }

    if (result.kind === "action") {
      selectWorkspace(state, result.workspaceId, result.actionId);
      selectAction(state, result.actionId);
    } else {
      selectWorkspace(state, result.workspaceId);
    }

    if (onSelect) {
      onSelect({
        result,
        workspace: currentWorkspace(state),
      });
    }
  }

  function refresh(queryText = state.ui.paletteQuery) {
    setPaletteQuery(state, queryText);
    lastResults = filterPaletteEntries(index(), queryText);
    if (activeIndex >= lastResults.length) {
      activeIndex = 0;
    }

    renderPaletteResults(resultsHost, lastResults, activeIndex);
    resultsHost.hidden = lastResults.length === 0 || !state.ui.paletteQuery.trim();
    notify("refresh");
    return lastResults;
  }

  function chooseActiveResult() {
    const result = lastResults[activeIndex];
    if (!result) {
      return null;
    }

    applyResult(result);
    if (input) {
      input.value = "";
    }
    setPaletteQuery(state, "");
    close();
    return result;
  }

  function moveActive(delta) {
    if (lastResults.length === 0) {
      return;
    }
    activeIndex = (activeIndex + delta + lastResults.length) % lastResults.length;
    renderPaletteResults(resultsHost, lastResults, activeIndex);
  }

  function bind() {
    if (!input) {
      return;
    }

    input.value = state.ui.paletteQuery || "";
    refresh(input.value);
    close();

    input.addEventListener("focus", () => {
      if (state.ui.paletteQuery.trim()) {
        open();
      }
    });

    input.addEventListener("input", () => {
      const results = refresh(input.value);
      if (results.length > 0 && input.value.trim()) {
        open();
      } else {
        close();
      }
    });

    input.addEventListener("keydown", (event) => {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        moveActive(1);
        open();
      } else if (event.key === "ArrowUp") {
        event.preventDefault();
        moveActive(-1);
        open();
      } else if (event.key === "Enter") {
        if (!resultsHost.hidden) {
          event.preventDefault();
          const result = chooseActiveResult();
          notify("choose", result);
        }
      } else if (event.key === "Escape") {
        close();
      }
    });

    resultsHost.addEventListener("mousedown", (event) => {
      const button = event.target.closest(".shell-palette-option");
      if (!button) {
        return;
      }
      event.preventDefault();
      activeIndex = Number(button.dataset.paletteIndex || 0);
      const result = chooseActiveResult();
      notify("choose", result);
    });

    document.addEventListener("click", (event) => {
      if (event.target === input || resultsHost.contains(event.target)) {
        return;
      }
      close();
    });
  }

  return {
    bind,
    refresh,
    open,
    close,
    chooseActiveResult,
    getResults: () => lastResults.slice(),
  };
}
