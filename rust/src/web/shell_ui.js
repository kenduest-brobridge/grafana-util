import {
  actionDisabledReason,
  clearActionDraft,
  currentAction,
  currentWorkspace,
  getActionDraft,
  isActionDisabled,
  selectAction,
  selectWorkspace,
  setActionDraft,
  textOrEmpty,
} from "./shell_state.js";

const DEFAULT_SELECTORS = {
  workspaceNav: "#workspace-nav",
  workspaceTitle: "#workspace-title",
  workspaceDescription: "#workspace-description",
  actionTabs: "#action-tabs",
  actionTitle: "#action-title",
  actionDescription: "#action-description",
  statusLine: "#status-line",
  actionForm: "#action-form",
  runButton: "#run-button",
  resetButton: "#reset-button",
  searchInput: "#global-search",
};

const navExpansionState = new WeakMap();

function query(root, selector) {
  return selector ? root.querySelector(selector) : null;
}

function replaceText(node, text) {
  if (node) {
    node.textContent = text;
  }
}

function getExpandedWorkspaceIds(nav, state) {
  let expanded = navExpansionState.get(nav);
  if (!expanded) {
    expanded = new Set();
    navExpansionState.set(nav, expanded);
  }

  if (state.activeWorkspaceId) {
    expanded.add(state.activeWorkspaceId);
  }
  return expanded;
}

function workspaceActionCount(workspace) {
  return Array.isArray(workspace?.actions) ? workspace.actions.length : 0;
}

function fieldDefaultValue(field) {
  if (field.defaultValue === null || field.defaultValue === undefined) {
    return field.kind === "checkbox" ? false : "";
  }
  if (field.kind === "checkbox") {
    return Boolean(field.defaultValue);
  }
  return field.kind === "number"
    ? String(field.defaultValue)
    : textOrEmpty(field.defaultValue);
}

function collectFieldValue(field, control) {
  if (field.kind === "checkbox") {
    return Boolean(control.checked);
  }

  const raw = textOrEmpty(control.value).trim();
  if (!raw) {
    return field.required ? "" : undefined;
  }
  if (field.kind === "number") {
    const parsed = Number(raw);
    return Number.isFinite(parsed) ? parsed : NaN;
  }
  return raw;
}

function renderField(field, action, draftValues) {
  const wrapper = document.createElement("div");
  wrapper.className = `field field-${field.kind}`;

  const controlId = `field-${action.id}-${field.id}`;
  const initialValue = Object.prototype.hasOwnProperty.call(draftValues, field.id)
    ? draftValues[field.id]
    : fieldDefaultValue(field);

  if (field.kind === "checkbox") {
    const label = document.createElement("label");
    label.className = "checkbox-control";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.id = controlId;
    input.name = field.id;
    input.dataset.fieldId = field.id;
    input.dataset.fieldKind = field.kind;
    input.checked = Boolean(initialValue);
    const span = document.createElement("span");
    span.textContent = field.label;
    label.append(input, span);
    wrapper.appendChild(label);
  } else {
    const label = document.createElement("label");
    label.className = "field-label";
    label.setAttribute("for", controlId);

    const labelText = document.createElement("span");
    labelText.className = "field-label-text";
    labelText.textContent = field.label;

    const labelMeta = document.createElement("span");
    labelMeta.className = "field-label-meta";
    labelMeta.textContent = field.kind;

    label.append(labelText, labelMeta);
    wrapper.appendChild(label);

    const input = field.kind === "select"
      ? document.createElement("select")
      : document.createElement("input");

    input.id = controlId;
    input.name = field.id;
    input.dataset.fieldId = field.id;
    input.dataset.fieldKind = field.kind;

    if (field.kind === "select") {
      (field.options || []).forEach((option) => {
        const node = document.createElement("option");
        node.value = option.value;
        node.textContent = option.label;
        input.appendChild(node);
      });
      input.value = textOrEmpty(initialValue);
    } else {
      input.type = field.kind === "number" ? "number" : "text";
      input.value = textOrEmpty(initialValue);
      if (field.placeholder) {
        input.placeholder = field.placeholder;
      }
      if (field.kind === "number") {
        input.inputMode = "numeric";
        input.step = "any";
      }
    }

    if (field.required) {
      input.required = true;
    }
    wrapper.appendChild(input);
  }

  if (field.help) {
    const help = document.createElement("p");
    help.className = "field-help";
    help.textContent = field.help;
    wrapper.appendChild(help);
  }

  return wrapper;
}

export function collectActionDraft(root, action, selectors = DEFAULT_SELECTORS) {
  const form = query(root, selectors.actionForm);
  const draft = {};
  if (!form || !action) {
    return draft;
  }

  (action.fields || []).forEach((field) => {
    const control = form.querySelector(`#field-${action.id}-${field.id}`);
    if (!control) {
      return;
    }
    const value = collectFieldValue(field, control);
    if (value !== undefined) {
      draft[field.id] = value;
    }
  });
  return draft;
}

export function validateActionDraft(action, draft) {
  for (const field of action?.fields || []) {
    const value = draft[field.id];
    if (field.required && (value === undefined || value === "" || Number.isNaN(value))) {
      return `${field.label} is required.`;
    }
    if (field.kind === "number" && value !== undefined && Number.isNaN(value)) {
      return `${field.label} must be a number.`;
    }
  }
  return null;
}

export function renderWorkspaceNav(state, options = {}) {
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const nav = query(root, selectors.workspaceNav);
  if (!nav) {
    return;
  }

  nav.replaceChildren();
  nav.setAttribute("role", "tree");

  if (state.workspaces.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state compact";
    empty.textContent = "No workspaces were returned by the registry.";
    nav.appendChild(empty);
    return;
  }

  const expandedWorkspaceIds = getExpandedWorkspaceIds(nav, state);

  state.workspaces.forEach((workspace) => {
    const actionCount = workspaceActionCount(workspace);
    const activeWorkspace = workspace.id === state.activeWorkspaceId;
    const expanded = activeWorkspace || expandedWorkspaceIds.has(workspace.id);
    const subnavId = `workspace-subnav-${workspace.id}`;

    const group = document.createElement("section");
    group.className = "workspace-group";
    group.dataset.workspaceId = workspace.id;
    group.dataset.expanded = expanded ? "true" : "false";
    group.setAttribute("role", "treeitem");
    group.setAttribute("aria-expanded", expanded ? "true" : "false");

    const row = document.createElement("div");
    row.className = "workspace-tree-row";

    const toggle = document.createElement("button");
    toggle.type = "button";
    toggle.className = "workspace-toggle";
    toggle.dataset.workspaceId = workspace.id;
    toggle.setAttribute("aria-label", `${expanded ? "Collapse" : "Expand"} ${workspace.title}`);
    toggle.setAttribute("aria-controls", subnavId);
    toggle.setAttribute("aria-expanded", expanded ? "true" : "false");
    toggle.textContent = expanded ? "−" : "+";
    toggle.addEventListener("click", (event) => {
      event.stopPropagation();
      if (expandedWorkspaceIds.has(workspace.id)) {
        expandedWorkspaceIds.delete(workspace.id);
      } else {
        expandedWorkspaceIds.add(workspace.id);
      }
      renderWorkspaceNav(state, options);
      renderShellSelection(state, { root, selectors });
    });

    const button = document.createElement("button");
    button.type = "button";
    button.className = "workspace-button";
    button.dataset.workspaceId = workspace.id;
    button.dataset.paletteLabel = workspace.title;
    button.setAttribute("aria-pressed", activeWorkspace ? "true" : "false");

    const text = document.createElement("span");
    text.className = "workspace-button-text";

    const title = document.createElement("strong");
    title.textContent = workspace.title;

    const description = document.createElement("span");
    description.className = "workspace-button-description";
    description.textContent = workspace.description || "";

    text.append(title, description);

    const meta = document.createElement("span");
    meta.className = "workspace-button-meta";
    meta.textContent = `${actionCount} action${actionCount === 1 ? "" : "s"}`;

    button.append(text, meta);
    button.addEventListener("click", () => {
      if (typeof options.onSelectWorkspace === "function") {
        options.onSelectWorkspace(workspace.id);
      }
    });

    row.append(toggle, button);
    group.appendChild(row);

    const subnav = document.createElement("div");
    subnav.className = "workspace-subnav";
    subnav.id = subnavId;
    subnav.setAttribute("role", "group");
    subnav.hidden = !expanded;
    (workspace.actions || []).forEach((action) => {
      const actionButton = document.createElement("button");
      actionButton.type = "button";
      actionButton.className = "workspace-action-link";
      actionButton.dataset.workspaceId = workspace.id;
      actionButton.dataset.actionId = action.id;
      actionButton.setAttribute("aria-selected", action.id === state.activeActionId ? "true" : "false");
      const marker = document.createElement("span");
      marker.className = "workspace-action-marker";
      marker.textContent = action.uiMode === "browse" ? "◉" : "·";
      const label = document.createElement("span");
      label.className = "workspace-action-label";
      label.textContent = action.title;
      const meta = document.createElement("span");
      meta.className = "workspace-action-meta";
      meta.textContent = [action.uiMode, action.readOnly ? "read-only" : "", action.requiresConnection ? "needs connection" : ""]
        .filter(Boolean)
        .join(" · ");
      const copy = document.createElement("span");
      copy.className = "workspace-action-copy";
      copy.append(label, meta);
      actionButton.append(marker, copy);
      actionButton.addEventListener("click", () => {
        if (typeof options.onSelectWorkspace === "function") {
          options.onSelectWorkspace(workspace.id);
        }
        if (typeof options.onSelectAction === "function") {
          options.onSelectAction(action.id);
        }
      });
      subnav.appendChild(actionButton);
    });
    group.appendChild(subnav);
    nav.appendChild(group);
  });
}

export function renderActionTabs(state, options = {}) {
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const tabs = query(root, selectors.actionTabs);
  if (!tabs) {
    return;
  }

  tabs.replaceChildren();
  const workspace = currentWorkspace(state);
  if (!workspace) {
    return;
  }

  workspace.actions.forEach((action) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "action-tab";
    button.dataset.actionId = action.id;
    button.dataset.workspaceId = workspace.id;
    button.dataset.paletteLabel = action.title;
    button.setAttribute("aria-selected", action.id === state.activeActionId ? "true" : "false");

    const title = document.createElement("strong");
    title.textContent = action.title;

    const meta = document.createElement("span");
    meta.className = "action-tab-meta";
    const tags = [action.uiMode];
    if (action.readOnly) {
      tags.push("read-only");
    }
    if (action.requiresConnection) {
      tags.push("connection");
    }
    meta.textContent = tags.filter(Boolean).join(" · ");

    button.append(title, meta);
    button.addEventListener("click", () => {
      if (typeof options.onSelectAction === "function") {
        options.onSelectAction(action.id);
      }
    });
    tabs.appendChild(button);
  });
}

export function renderActionForm(state, options = {}) {
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const form = query(root, selectors.actionForm);
  const action = currentAction(state);

  if (!form) {
    return;
  }

  form.replaceChildren();
  if (!action) {
    return;
  }

  const fields = Array.isArray(action.fields) ? action.fields : [];
  if (fields.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state compact";
    empty.textContent = "This action does not expose any editable fields.";
    form.appendChild(empty);
    return;
  }

  const draftValues = getActionDraft(state, action.id);
  const grid = document.createElement("div");
  grid.className = "field-grid";
  fields.forEach((field) => {
    grid.appendChild(renderField(field, action, draftValues));
  });
  form.appendChild(grid);
}

export function renderShellSelection(state, options = {}) {
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const workspace = currentWorkspace(state);
  const action = currentAction(state);
  const statusLine = query(root, selectors.statusLine);

  replaceText(query(root, selectors.workspaceTitle), workspace ? workspace.title : "");
  replaceText(query(root, selectors.workspaceDescription), workspace ? workspace.description : "");
  replaceText(query(root, selectors.actionTitle), action ? action.title : "");
  replaceText(query(root, selectors.actionDescription), action ? action.description : "");

  root.querySelectorAll(".workspace-button").forEach((button) => {
    const active = button.dataset.workspaceId === state.activeWorkspaceId;
    button.classList.toggle("active", active);
    button.setAttribute("aria-pressed", active ? "true" : "false");
  });

  root.querySelectorAll(".workspace-group").forEach((group) => {
    const active = group.dataset.workspaceId === state.activeWorkspaceId;
    group.classList.toggle("active", active);
  });

  root.querySelectorAll(".workspace-action-link").forEach((button) => {
    const active = button.dataset.actionId === state.activeActionId
      && button.dataset.workspaceId === state.activeWorkspaceId;
    button.classList.toggle("active", active);
    button.setAttribute("aria-selected", active ? "true" : "false");
  });

  root.querySelectorAll(".action-tab").forEach((button) => {
    const active = button.dataset.actionId === state.activeActionId;
    button.classList.toggle("active", active);
    button.setAttribute("aria-selected", active ? "true" : "false");
  });

  if (statusLine && !state.statusText && workspace && action) {
    statusLine.textContent = `Ready: ${workspace.title} / ${action.title}`;
  } else if (statusLine && state.statusText) {
    statusLine.textContent = state.statusText;
  }
}

export function applyShellDisabledState(state, options = {}) {
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const runButton = query(root, selectors.runButton);
  const resetButton = query(root, selectors.resetButton);
  const searchInput = query(root, selectors.searchInput);
  const action = currentAction(state);
  const disabled = isActionDisabled(state, action);
  const reason = actionDisabledReason(state, action);

  if (runButton) {
    runButton.disabled = disabled;
    runButton.title = disabled ? reason : "";
    runButton.dataset.actionId = action ? action.id : "";
  }
  if (resetButton) {
    resetButton.disabled = !action;
  }
  if (searchInput) {
    searchInput.disabled = state.workspaces.length === 0;
  }
}

export function createShellUi(options) {
  const state = options.state;
  const root = options.root || document;
  const selectors = { ...DEFAULT_SELECTORS, ...(options.selectors || {}) };
  const onRun = typeof options.onRun === "function" ? options.onRun : null;
  const onReset = typeof options.onReset === "function" ? options.onReset : null;
  const onSelectionChange = typeof options.onSelectionChange === "function"
    ? options.onSelectionChange
    : null;

  function setStatus(text) {
    state.statusText = text;
    replaceText(query(root, selectors.statusLine), text);
  }

  function rerender() {
    renderWorkspaceNav(state, {
      root,
      selectors,
      onSelectWorkspace: handleWorkspaceSelect,
    });
    renderActionTabs(state, {
      root,
      selectors,
      onSelectAction: handleActionSelect,
    });
    renderActionForm(state, { root, selectors });
    renderShellSelection(state, { root, selectors });
    applyShellDisabledState(state, { root, selectors });
    bindActionForm();
  }

  function emitSelection(reason) {
    if (onSelectionChange) {
      onSelectionChange({
        reason,
        workspace: currentWorkspace(state),
        action: currentAction(state),
        state,
      });
    }
  }

  function handleWorkspaceSelect(workspaceId) {
    selectWorkspace(state, workspaceId);
    rerender();
    emitSelection("workspace");
  }

  function handleActionSelect(actionId) {
    selectAction(state, actionId);
    rerender();
    emitSelection("action");
  }

  function bindActionForm() {
    const form = query(root, selectors.actionForm);
    const action = currentAction(state);
    if (!form) {
      return;
    }

    form.onsubmit = (event) => {
      event.preventDefault();
      if (onRun) {
        onRun({ action, draft: collectActionDraft(root, action, selectors), state });
      }
    };

    form.oninput = () => {
      if (!action) {
        return;
      }
      setActionDraft(state, action.id, collectActionDraft(root, action, selectors));
    };

    form.onchange = form.oninput;
  }

  function bindButtons() {
    const runButton = query(root, selectors.runButton);
    const resetButton = query(root, selectors.resetButton);

    if (runButton) {
      runButton.addEventListener("click", () => {
        const action = currentAction(state);
        if (onRun) {
          onRun({ action, draft: collectActionDraft(root, action, selectors), state });
        }
      });
    }

    if (resetButton) {
      resetButton.addEventListener("click", () => {
        const action = currentAction(state);
        if (!action) {
          return;
        }
        clearActionDraft(state, action.id);
        renderActionForm(state, { root, selectors });
        applyShellDisabledState(state, { root, selectors });
        bindActionForm();
        if (onReset) {
          onReset({ action, state });
        }
      });
    }
  }

  return {
    bind() {
      bindButtons();
      rerender();
    },
    rerender,
    setStatus,
    getCurrentWorkspace: () => currentWorkspace(state),
    getCurrentAction: () => currentAction(state),
    selectWorkspace(workspaceId, preferredActionId) {
      selectWorkspace(state, workspaceId, preferredActionId);
      rerender();
      emitSelection("workspace");
    },
    selectAction(actionId) {
      selectAction(state, actionId);
      rerender();
      emitSelection("action");
    },
    collectDraft() {
      return collectActionDraft(root, currentAction(state), selectors);
    },
  };
}
