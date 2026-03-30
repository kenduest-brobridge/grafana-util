import { renderVisualResult, responseDocument, responseTextLines } from "./renderers.js";

export const DEFAULT_RESULT_STAGE_SELECTORS = {
  stage: ".result-stage",
  tabs: "#result-tabs",
  tabButtons: ".result-tab[data-result-tab]",
  fullscreenToggle: "#result-fullscreen-toggle",
  visualHost: "#result-visual",
  sourceGrid: ".result-source-grid",
  logPanel: "#response-text",
  sourcePanel: "#response-json",
};

function resolveElement(target) {
  if (!target) {
    return null;
  }
  if (typeof target === "string") {
    return document.querySelector(target);
  }
  return target;
}

function visibleTextLines(response) {
  const lines = responseTextLines(response);
  return lines.length ? lines.join("\n") : "";
}

function visibleSourceJson(response) {
  if (!response) {
    return "";
  }
  const value = responseDocument(response) !== null ? responseDocument(response) : response;
  return JSON.stringify(value, null, 2);
}

function setButtonPressed(button, pressed) {
  button.classList.toggle("active", pressed);
  button.setAttribute("aria-pressed", pressed ? "true" : "false");
}

export function setupResultStage(options = {}) {
  const selectors = { ...DEFAULT_RESULT_STAGE_SELECTORS, ...(options.selectors || {}) };
  const stage = resolveElement(options.stage || selectors.stage);
  const tabs = resolveElement(options.tabs || selectors.tabs);
  const fullscreenToggle = resolveElement(options.fullscreenToggle || selectors.fullscreenToggle);
  const visualHost = resolveElement(options.visualHost || selectors.visualHost);
  const sourceGrid = resolveElement(options.sourceGrid || selectors.sourceGrid);
  const logPre = resolveElement(options.logPre || selectors.logPanel);
  const sourcePre = resolveElement(options.sourcePre || selectors.sourcePanel);

  if (!stage || !tabs || !visualHost || !logPre || !sourcePre) {
    throw new Error("Result stage setup requires stage, tabs, visual host, and source panes.");
  }

  const logPanel = logPre.closest(".result-source-panel") || logPre;
  const sourcePanel = sourcePre.closest(".result-source-panel") || sourcePre;

  const state = {
    activeTab: options.defaultTab || "visual",
    fullscreen: false,
    maximized: false,
    lastAction: null,
    lastResponse: null,
    rendererKey: "empty",
  };

  const controller = {
    stage,
    tabs,
    visualHost,
    logPre,
    sourcePre,
    logPanel,
    sourcePanel,
    sourceGrid,
    state,
    render(action, response, renderOptions = {}) {
      state.lastAction = action || null;
      state.lastResponse = response || null;
      const renderMeta = renderVisualResult(visualHost, action || null, response || null, renderOptions);
      state.rendererKey = renderMeta.rendererKey;
      logPre.textContent = visibleTextLines(response);
      sourcePre.textContent = visibleSourceJson(response);
      applyPaneVisibility();
      return renderMeta;
    },
    activateTab(tabName) {
      if (!["visual", "log", "source"].includes(tabName)) {
        return state.activeTab;
      }
      state.activeTab = tabName;
      applyPaneVisibility();
      return state.activeTab;
    },
    toggleFullscreen() {
      controller.setFullscreen(!state.fullscreen);
      return state.fullscreen;
    },
    setFullscreen(enabled) {
      state.fullscreen = Boolean(enabled);
      if (state.fullscreen) {
        state.maximized = true;
      }
      applyStageState();
      return state.fullscreen;
    },
    toggleMaximized() {
      controller.setMaximized(!state.maximized);
      return state.maximized;
    },
    setMaximized(enabled) {
      state.maximized = Boolean(enabled);
      if (!state.maximized) {
        state.fullscreen = false;
      }
      applyStageState();
      return state.maximized;
    },
    reset() {
      state.lastAction = null;
      state.lastResponse = null;
      state.rendererKey = "empty";
      controller.render(null, null);
    },
    getState() {
      return {
        activeTab: state.activeTab,
        fullscreen: state.fullscreen,
        maximized: state.maximized,
        rendererKey: state.rendererKey,
      };
    },
  };

  function tabButtons() {
    return Array.from(tabs.querySelectorAll(selectors.tabButtons));
  }

  function applyStageState() {
    stage.dataset.resultTab = state.activeTab;
    stage.dataset.resultFullscreen = state.fullscreen ? "true" : "false";
    stage.dataset.resultMaximized = state.maximized ? "true" : "false";
    stage.classList.toggle("is-result-fullscreen", state.fullscreen);
    stage.classList.toggle("is-result-maximized", state.maximized);
    if (fullscreenToggle) {
      fullscreenToggle.textContent = state.fullscreen ? "Exit Fullscreen" : "Fullscreen";
      fullscreenToggle.setAttribute("aria-pressed", state.fullscreen ? "true" : "false");
    }
  }

  function applyPaneVisibility() {
    tabButtons().forEach((button) => {
      setButtonPressed(button, button.dataset.resultTab === state.activeTab);
    });

    const visualActive = state.activeTab === "visual";
    const logActive = state.activeTab === "log";
    const sourceActive = state.activeTab === "source";

    visualHost.hidden = !visualActive;
    if (sourceGrid) {
      sourceGrid.hidden = visualActive;
    }
    logPanel.hidden = !logActive;
    sourcePanel.hidden = !sourceActive;
    applyStageState();
  }

  tabs.addEventListener("click", (event) => {
    const button = event.target.closest(selectors.tabButtons);
    if (!button) {
      return;
    }
    controller.activateTab(button.dataset.resultTab || "visual");
  });

  if (fullscreenToggle) {
    fullscreenToggle.addEventListener("click", () => {
      controller.toggleFullscreen();
    });
  }

  applyPaneVisibility();

  return controller;
}

export function renderResultStage(controller, action, response, options = {}) {
  return controller.render(action, response, options);
}
