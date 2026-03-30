function isPlainObject(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

function createElement(tagName, className, textContent) {
  const node = document.createElement(tagName);
  if (className) {
    node.className = className;
  }
  if (textContent !== undefined) {
    node.textContent = textContent;
  }
  return node;
}

function clearNode(node) {
  node.replaceChildren();
}

function appendIf(parent, child) {
  if (child) {
    parent.appendChild(child);
  }
}

function valueAsText(value) {
  if (value === null || value === undefined) {
    return "";
  }
  if (Array.isArray(value)) {
    return value.map((item) => valueAsText(item)).filter(Boolean).join(", ");
  }
  if (typeof value === "object") {
    try {
      return JSON.stringify(value);
    } catch (_error) {
      return String(value);
    }
  }
  return String(value);
}

function valueSortKey(value) {
  if (value === null || value === undefined) {
    return "";
  }
  if (typeof value === "number") {
    return value;
  }
  if (typeof value === "boolean") {
    return value ? 1 : 0;
  }
  return valueAsText(value).toLowerCase();
}

function titleCaseFromKey(key) {
  return String(key || "")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/^\w/, (match) => match.toUpperCase());
}

function createSectionTitle(title, subtitle) {
  const header = createElement("div", "section-title");
  header.appendChild(createElement("strong", "", title));
  if (subtitle) {
    header.appendChild(createElement("span", "", subtitle));
  }
  return header;
}

function createEmptyState(message, compact = false) {
  const className = compact ? "empty-state compact" : "empty-state";
  return createElement("div", className, message);
}

function createCodeBlock(value) {
  const pre = createElement("pre", "result-code");
  pre.textContent = typeof value === "string" ? value : JSON.stringify(value, null, 2);
  return pre;
}

function createChipRow(summary) {
  const entries = Object.entries(summary || {}).filter(([_key, value]) => {
    return value !== null && value !== undefined && value !== "";
  });
  if (!entries.length) {
    return null;
  }
  const row = createElement("div", "chip-row");
  entries.forEach(([key, value]) => {
    row.appendChild(createElement("span", "chip", `${titleCaseFromKey(key)}: ${valueAsText(value)}`));
  });
  return row;
}

function collectColumns(rows) {
  const columns = [];
  rows.forEach((row) => {
    Object.keys(row).forEach((key) => {
      if (!columns.includes(key)) {
        columns.push(key);
      }
    });
  });
  return columns;
}

function rowArrayDocument(documentValue) {
  if (Array.isArray(documentValue)) {
    return documentValue.every((row) => isPlainObject(row)) ? documentValue : null;
  }
  if (!isPlainObject(documentValue)) {
    return null;
  }
  if (Array.isArray(documentValue.rows) && documentValue.rows.every((row) => isPlainObject(row))) {
    return documentValue.rows;
  }
  return null;
}

function objectEntriesExcluding(value, excludedKeys) {
  return Object.entries(value || {}).filter(([key, entryValue]) => {
    return !excludedKeys.includes(key) && entryValue !== undefined;
  });
}

function createDefinitionList(data, options = {}) {
  const entries = Object.entries(data || {}).filter(([_key, value]) => value !== undefined);
  if (!entries.length) {
    return null;
  }
  const list = createElement("dl", options.className || "detail-list");
  entries.forEach(([key, value]) => {
    list.appendChild(createElement("dt", "", titleCaseFromKey(key)));
    const dd = createElement("dd");
    if (isPlainObject(value) || Array.isArray(value)) {
      dd.textContent = valueAsText(value) || "-";
    } else {
      dd.textContent = valueAsText(value) || "-";
    }
    list.appendChild(dd);
  });
  return list;
}

function createObjectSection(title, value, subtitle) {
  const section = createElement("section", "document-subsection");
  section.appendChild(createSectionTitle(title, subtitle));
  const list = createDefinitionList(value);
  appendIf(section, list);
  if (!list) {
    section.appendChild(createEmptyState("No fields available.", true));
  }
  return section;
}

function createArraySection(title, rows, options = {}) {
  const section = createElement("section", "document-subsection");
  const subtitle = options.subtitle || `${rows.length} item${rows.length === 1 ? "" : "s"}`;
  section.appendChild(createSectionTitle(title, subtitle));
  section.appendChild(renderTableResult(rows, {
    title: options.tableTitle || title,
    compact: true,
    emptyMessage: options.emptyMessage,
    maxRows: options.maxRows,
    searchable: options.searchable !== false,
    sortable: options.sortable !== false,
    showHeader: false,
  }));
  return section;
}

function arrayOfObjects(value) {
  return Array.isArray(value) && value.every((item) => isPlainObject(item));
}

function createScalarSummarySection(documentValue, excludedKeys) {
  if (!isPlainObject(documentValue)) {
    return null;
  }
  const summary = {};
  objectEntriesExcluding(documentValue, excludedKeys).forEach(([key, value]) => {
    if (!isPlainObject(value) && !Array.isArray(value)) {
      summary[key] = value;
    }
  });
  const entries = Object.keys(summary);
  if (!entries.length) {
    return null;
  }
  return createObjectSection("Overview", summary, `${entries.length} field${entries.length === 1 ? "" : "s"}`);
}

export function responseTextLines(response) {
  if (Array.isArray(response?.textLines)) {
    return response.textLines;
  }
  if (Array.isArray(response?.text_lines)) {
    return response.text_lines;
  }
  return [];
}

export function responseDocument(response) {
  if (!response || !Object.prototype.hasOwnProperty.call(response, "document")) {
    return null;
  }
  return response.document;
}

export function resolveRendererKey(action, response) {
  const documentValue = responseDocument(response);
  const actionOverride = action?.resultRenderer ||
    action?.result_renderer ||
    action?.renderer ||
    action?.rendererKey ||
    action?.visualRenderer;
  if (typeof actionOverride === "string" && actionOverride) {
    return actionOverride;
  }

  const kind = typeof documentValue?.kind === "string" ? documentValue.kind : "";
  if (kind === "grafana-utils-dashboard-browser") {
    return "browse";
  }
  if (kind.startsWith("grafana-utils-sync-")) {
    return "sync-diff";
  }
  if (
    kind === "grafana-utils-dashboard-list" ||
    kind === "grafana-utils-dashboard-dependency-contract" ||
    kind.endsWith("-export-index")
  ) {
    return "table";
  }
  if (kind) {
    return "document";
  }

  const uiMode = action?.uiMode || action?.ui_mode || response?.uiMode || response?.ui_mode;
  if (uiMode === "browse") {
    return "browse";
  }
  if (uiMode === "table") {
    return "table";
  }
  if (uiMode === "analysis" || uiMode === "document") {
    return "document";
  }

  return "generic";
}

function renderKeyValueGrid(title, value) {
  if (!isPlainObject(value)) {
    return null;
  }
  return createObjectSection(title, value, `${Object.keys(value).length} field${Object.keys(value).length === 1 ? "" : "s"}`);
}

function renderListOfStrings(title, values) {
  if (!Array.isArray(values) || !values.length) {
    return null;
  }
  const section = createElement("section", "document-subsection");
  section.appendChild(createSectionTitle(title, `${values.length} item${values.length === 1 ? "" : "s"}`));
  const list = createElement("ul", "result-bullet-list");
  values.forEach((value) => {
    list.appendChild(createElement("li", "", valueAsText(value)));
  });
  section.appendChild(list);
  return section;
}

function countLabel(count, singular, plural) {
  const suffix = count === 1 ? singular : (plural || `${singular}s`);
  return `${count} ${suffix}`;
}

function uniqueTextValues(values) {
  return Array.from(new Set((Array.isArray(values) ? values : [])
    .map((value) => valueAsText(value).trim())
    .filter(Boolean)));
}

function browseDashboardRows(documentValue) {
  const rows = rowArrayDocument(documentValue);
  if (!rows) {
    return [];
  }
  return rows.map((row, index) => {
    const uid = valueAsText(row?.uid || `dashboard-${index + 1}`) || `dashboard-${index + 1}`;
    const title = valueAsText(row?.title || row?.uid || `Dashboard ${index + 1}`) || `Dashboard ${index + 1}`;
    const orgName = valueAsText(row?.orgName || "Current Org") || "Current Org";
    const folderPath = valueAsText(row?.folderPath || row?.folderTitle || "General") || "General";
    const folderSegments = folderPath.split("/").map((part) => part.trim()).filter(Boolean);
    const folderTitle = folderSegments[folderSegments.length - 1] || valueAsText(row?.folderTitle || folderPath) || "General";
    const sources = uniqueTextValues([
      ...uniqueTextValues(row?.sources),
      ...uniqueTextValues(row?.sourceUids),
    ]);
    return {
      uid,
      title,
      orgName,
      orgId: row?.orgId,
      folderPath,
      folderTitle,
      folderUid: valueAsText(row?.folderUid),
      sources,
      sourceUids: uniqueTextValues(row?.sourceUids),
      raw: row,
    };
  });
}

function buildBrowseWorkspace(documentValue) {
  const rows = browseDashboardRows(documentValue);
  const orgMap = new Map();
  const datasourceMap = new Map();

  rows.forEach((row) => {
    if (!orgMap.has(row.orgName)) {
      orgMap.set(row.orgName, {
        orgName: row.orgName,
        orgId: row.orgId,
        folders: new Map(),
        dashboards: [],
      });
    }
    const orgEntry = orgMap.get(row.orgName);
    if (!orgEntry.folders.has(row.folderPath)) {
      orgEntry.folders.set(row.folderPath, {
        folderPath: row.folderPath,
        folderTitle: row.folderTitle,
        folderUid: row.folderUid,
        dashboards: [],
        sourceSet: new Set(),
      });
    }
    const folderEntry = orgEntry.folders.get(row.folderPath);
    folderEntry.dashboards.push(row);
    orgEntry.dashboards.push(row);
    row.sources.forEach((source) => {
      folderEntry.sourceSet.add(source);
      if (!datasourceMap.has(source)) {
        datasourceMap.set(source, {
          name: source,
          dashboards: new Set(),
          orgs: new Set(),
          folders: new Set(),
        });
      }
      const datasourceEntry = datasourceMap.get(source);
      datasourceEntry.dashboards.add(row.uid);
      datasourceEntry.orgs.add(row.orgName);
      datasourceEntry.folders.add(row.folderPath);
    });
  });

  const orgs = Array.from(orgMap.values())
    .map((entry) => {
      const folders = Array.from(entry.folders.values())
        .map((folder) => ({
          folderPath: folder.folderPath,
          folderTitle: folder.folderTitle,
          folderUid: folder.folderUid,
          dashboards: folder.dashboards.sort((left, right) => left.title.localeCompare(right.title)),
          sourceCount: folder.sourceSet.size,
          sources: Array.from(folder.sourceSet).sort((left, right) => left.localeCompare(right)),
        }))
        .sort((left, right) => {
          return right.dashboards.length - left.dashboards.length || left.folderPath.localeCompare(right.folderPath);
        });
      return {
        orgName: entry.orgName,
        orgId: entry.orgId,
        dashboardCount: entry.dashboards.length,
        folderCount: folders.length,
        sourceCount: new Set(entry.dashboards.flatMap((dashboard) => dashboard.sources)).size,
        folders,
      };
    })
    .sort((left, right) => {
      return right.dashboardCount - left.dashboardCount || left.orgName.localeCompare(right.orgName);
    });

  const datasources = Array.from(datasourceMap.values())
    .map((entry) => ({
      name: entry.name,
      dashboardCount: entry.dashboards.size,
      orgCount: entry.orgs.size,
      folderCount: entry.folders.size,
      dashboardUids: Array.from(entry.dashboards),
      orgs: Array.from(entry.orgs).sort((left, right) => left.localeCompare(right)),
      folders: Array.from(entry.folders).sort((left, right) => left.localeCompare(right)),
    }))
    .sort((left, right) => {
      return right.dashboardCount - left.dashboardCount || left.name.localeCompare(right.name);
    });

  if (!orgs.length && Array.isArray(documentValue?.tree)) {
    documentValue.tree.forEach((orgNode, orgIndex) => {
      const folderNodes = Array.isArray(orgNode?.children) ? orgNode.children : [];
      orgs.push({
        orgName: valueAsText(orgNode?.label || orgNode?.title || orgNode?.id || `Org ${orgIndex + 1}`) || `Org ${orgIndex + 1}`,
        orgId: valueAsText(orgNode?.id),
        dashboardCount: Number(orgNode?.dashboardCount || 0),
        folderCount: folderNodes.length,
        sourceCount: 0,
        folders: folderNodes.map((folderNode, folderIndex) => ({
          folderPath: valueAsText(folderNode?.folderPath || folderNode?.label || folderNode?.title || `Folder ${folderIndex + 1}`) || `Folder ${folderIndex + 1}`,
          folderTitle: valueAsText(folderNode?.label || folderNode?.title || folderNode?.folderPath || `Folder ${folderIndex + 1}`) || `Folder ${folderIndex + 1}`,
          folderUid: valueAsText(folderNode?.id),
          dashboards: [],
          sourceCount: 0,
          sources: [],
          dashboardCount: Number(folderNode?.dashboardCount || folderNode?.count || 0),
        })),
      });
    });
  }

  return { orgs, datasources, rows };
}

function browseSelectedDetail(documentValue, workspace, selectedUid) {
  const detail = isPlainObject(documentValue?.detail) ? documentValue.detail : null;
  if (selectedUid) {
    const selectedRow = workspace.rows.find((row) => row.uid === selectedUid);
    if (selectedRow) {
      return selectedRow.raw;
    }
  }
  return detail;
}

function createBrowseMetricCard(label, value, tone) {
  const card = createElement("div", tone ? `browse-metric-card ${tone}` : "browse-metric-card");
  card.appendChild(createElement("span", "browse-metric-value", value));
  card.appendChild(createElement("span", "browse-metric-label", label));
  return card;
}

function createBrowseFactList(detail) {
  if (!isPlainObject(detail)) {
    return null;
  }
  const facts = {};
  [
    "uid",
    "orgName",
    "orgId",
    "folderTitle",
    "folderUid",
    "folderPath",
  ].forEach((key) => {
    if (detail[key] !== undefined && detail[key] !== null && detail[key] !== "") {
      facts[key] = detail[key];
    }
  });
  return createDefinitionList(facts, { className: "browse-detail-list" });
}

function renderBrowseSpotlight(documentValue, workspace, selectedUid) {
  const section = createElement("section", "browse-spotlight");
  const detail = browseSelectedDetail(documentValue, workspace, selectedUid);
  const title = detail?.title || detail?.uid || "No dashboard selected";

  section.appendChild(createSectionTitle("Dashboard spotlight", title));
  if (!detail) {
    section.appendChild(createEmptyState("Browse results did not include a dashboard detail record.", true));
    return section;
  }

  const hero = createElement("div", "browse-spotlight-hero");
  const header = createElement("div", "browse-spotlight-header");
  header.appendChild(createElement("div", "browse-kicker", detail?.orgName || "Current Org"));
  header.appendChild(createElement("h3", "browse-spotlight-title", detail?.title || detail?.uid || "Dashboard"));
  header.appendChild(createElement("p", "browse-spotlight-subtitle", detail?.folderPath || detail?.folderTitle || "Folder path unavailable"));
  hero.appendChild(header);

  const sourceList = uniqueTextValues([
    ...uniqueTextValues(detail?.sources),
    ...uniqueTextValues(detail?.sourceUids),
  ]);
  const metrics = createElement("div", "browse-metric-grid");
  metrics.appendChild(createBrowseMetricCard("Datasource links", String(sourceList.length), "accent"));
  metrics.appendChild(createBrowseMetricCard("Folder", detail?.folderTitle || "General"));
  metrics.appendChild(createBrowseMetricCard("UID", detail?.uid || "-"));
  hero.appendChild(metrics);
  section.appendChild(hero);

  if (sourceList.length) {
    const sourceSection = createElement("div", "browse-spotlight-sources");
    sourceSection.appendChild(createElement("div", "browse-mini-title", "Attached datasources"));
    const sourceRow = createElement("div", "browse-source-row");
    sourceList.forEach((source) => {
      sourceRow.appendChild(createElement("span", "browse-source-pill", source));
    });
    sourceSection.appendChild(sourceRow);
    section.appendChild(sourceSection);
  }

  appendIf(section, createChipRow(detail.summary));
  appendIf(section, createBrowseFactList(detail));
  return section;
}

function createBrowseDatasourceConstellation(workspace, selectedUid) {
  const section = createElement("section", "browse-datasource-constellation");
  section.appendChild(createSectionTitle(
    "Datasource constellation",
    workspace.datasources.length ? countLabel(workspace.datasources.length, "cluster") : "No datasource edges",
  ));

  if (!workspace.datasources.length) {
    section.appendChild(createEmptyState("Datasource relationships are only available when browse rows include source metadata.", true));
    return section;
  }

  const cluster = createElement("div", "browse-datasource-cluster");
  workspace.datasources.forEach((datasource) => {
    const card = document.createElement("details");
    card.className = "browse-datasource-card";
    if (selectedUid && datasource.dashboardUids.includes(selectedUid)) {
      card.open = true;
    }
    const summary = createElement("summary", "browse-datasource-summary");
    const heading = createElement("div", "browse-datasource-heading");
    heading.appendChild(createElement("strong", "", datasource.name));
    heading.appendChild(createElement(
      "span",
      "",
      `${countLabel(datasource.dashboardCount, "dashboard")} | ${countLabel(datasource.folderCount, "folder")} | ${countLabel(datasource.orgCount, "org")}`,
    ));
    summary.appendChild(heading);
    card.appendChild(summary);

    const body = createElement("div", "browse-datasource-body");
    if (datasource.folders.length) {
      body.appendChild(createElement("div", "browse-mini-title", "Folders"));
      const folderList = createElement("div", "browse-inline-list");
      datasource.folders.slice(0, 6).forEach((folder) => {
        folderList.appendChild(createElement("span", "browse-inline-pill", folder));
      });
      body.appendChild(folderList);
    }
    if (datasource.orgs.length) {
      body.appendChild(createElement("div", "browse-mini-title", "Organizations"));
      const orgList = createElement("div", "browse-inline-list");
      datasource.orgs.forEach((orgName) => {
        orgList.appendChild(createElement("span", "browse-inline-pill muted", orgName));
      });
      body.appendChild(orgList);
    }
    card.appendChild(body);
    cluster.appendChild(card);
  });
  section.appendChild(cluster);
  return section;
}

function createBrowseDashboardCard(row, selectedUid, onSelect) {
  const button = createElement("button", "browse-dashboard-card");
  button.type = "button";
  button.dataset.uid = row.uid;
  button.dataset.selected = row.uid === selectedUid ? "true" : "false";

  const top = createElement("div", "browse-dashboard-topline");
  top.appendChild(createElement("strong", "", row.title));
  top.appendChild(createElement("span", "", row.uid));
  button.appendChild(top);

  const meta = createElement("div", "browse-dashboard-meta");
  meta.appendChild(createElement("span", "", row.folderTitle || row.folderPath));
  meta.appendChild(createElement("span", "", countLabel(row.sources.length, "source")));
  button.appendChild(meta);

  if (row.sources.length) {
    const sourceRow = createElement("div", "browse-dashboard-sources");
    row.sources.slice(0, 4).forEach((source) => {
      sourceRow.appendChild(createElement("span", "browse-inline-pill", source));
    });
    if (row.sources.length > 4) {
      sourceRow.appendChild(createElement("span", "browse-inline-pill muted", `+${row.sources.length - 4}`));
    }
    button.appendChild(sourceRow);
  }

  button.addEventListener("click", () => onSelect(row.uid));
  return button;
}

function createBrowseFolderCard(folder, selectedUid, onSelect) {
  const card = document.createElement("details");
  card.className = "browse-folder-card";
  const dashboardCount = folder.dashboards.length || Number(folder.dashboardCount || 0);
  if (!dashboardCount || folder.dashboards.some((row) => row.uid === selectedUid)) {
    card.open = true;
  }

  const summary = createElement("summary", "browse-folder-summary");
  const heading = createElement("div", "browse-folder-heading");
  heading.appendChild(createElement("strong", "", folder.folderTitle || folder.folderPath));
  heading.appendChild(createElement("span", "", folder.folderPath));
  summary.appendChild(heading);
  const stats = createElement("div", "browse-folder-stats");
  stats.appendChild(createElement("span", "browse-inline-pill", countLabel(dashboardCount, "dashboard")));
  if (folder.sourceCount) {
    stats.appendChild(createElement("span", "browse-inline-pill muted", countLabel(folder.sourceCount, "datasource")));
  }
  summary.appendChild(stats);
  card.appendChild(summary);

  const body = createElement("div", "browse-folder-body");
  if (folder.dashboards.length) {
    const list = createElement("div", "browse-dashboard-stack");
    folder.dashboards.forEach((row) => {
      list.appendChild(createBrowseDashboardCard(row, selectedUid, onSelect));
    });
    body.appendChild(list);
  } else {
    body.appendChild(createEmptyState("Browse tree returned folder counts but not dashboard rows for this path.", true));
  }
  card.appendChild(body);
  return card;
}

function renderBrowseLanes(workspace, selectedUid, onSelect) {
  const section = createElement("section", "browse-lanes");
  section.appendChild(createSectionTitle(
    "Relationship lanes",
    workspace.orgs.length ? countLabel(workspace.orgs.length, "org") : "No organization groups",
  ));

  if (!workspace.orgs.length) {
    section.appendChild(createEmptyState("No browse rows or tree groups were returned.", true));
    return section;
  }

  const laneList = createElement("div", "browse-lane-list");
  workspace.orgs.forEach((org, index) => {
    const lane = createElement("section", "browse-org-lane");
    lane.dataset.laneIndex = String(index);

    const header = createElement("div", "browse-org-header");
    const title = createElement("div", "browse-org-title");
    title.appendChild(createElement("div", "browse-kicker", `Org ${index + 1}`));
    title.appendChild(createElement("h3", "", org.orgName));
    title.appendChild(createElement(
      "p",
      "",
      `${countLabel(org.folderCount, "folder")} flowing into ${countLabel(org.dashboardCount, "dashboard")}`,
    ));
    header.appendChild(title);

    const stats = createElement("div", "browse-org-stats");
    stats.appendChild(createElement("span", "browse-inline-pill", countLabel(org.dashboardCount, "dashboard")));
    stats.appendChild(createElement("span", "browse-inline-pill muted", countLabel(org.folderCount, "folder")));
    if (org.sourceCount) {
      stats.appendChild(createElement("span", "browse-inline-pill muted", countLabel(org.sourceCount, "datasource")));
    }
    header.appendChild(stats);
    lane.appendChild(header);

    const folders = createElement("div", "browse-folder-grid");
    org.folders.forEach((folder) => {
      folders.appendChild(createBrowseFolderCard(folder, selectedUid, onSelect));
    });
    lane.appendChild(folders);
    laneList.appendChild(lane);
  });
  section.appendChild(laneList);
  return section;
}

export function renderBrowseResult(documentValue, options = {}) {
  const wrapper = createElement("section", "browse-panel");
  const dashboardCount = documentValue?.summary?.dashboardCount;
  wrapper.appendChild(createSectionTitle(
    options.title || "Browse",
    dashboardCount !== undefined ? `${dashboardCount} dashboards` : "Structured browse view",
  ));

  appendIf(wrapper, createChipRow(documentValue?.summary));
  appendIf(wrapper, createChipRow(documentValue?.filters));

  const workspace = buildBrowseWorkspace(documentValue);
  let selectedUid = valueAsText(documentValue?.summary?.selectedUid || documentValue?.detail?.uid || workspace.rows[0]?.uid);
  const layout = createElement("div", "browse-layout");
  const spotlightHost = createElement("div", "browse-spotlight-host");
  const mapHost = createElement("div", "browse-map");

  const renderSelection = () => {
    clearNode(spotlightHost);
    spotlightHost.appendChild(renderBrowseSpotlight(documentValue, workspace, selectedUid));
    mapHost.querySelectorAll(".browse-dashboard-card").forEach((button) => {
      button.dataset.selected = button.dataset.uid === selectedUid ? "true" : "false";
    });
    mapHost.querySelectorAll(".browse-datasource-card").forEach((card) => {
      const summary = card.querySelector(".browse-datasource-summary strong");
      const datasourceName = summary ? summary.textContent : "";
      const datasource = workspace.datasources.find((entry) => entry.name === datasourceName);
      if (datasource) {
        card.dataset.related = selectedUid && datasource.dashboardUids.includes(selectedUid) ? "true" : "false";
      }
    });
  };

  const handleSelect = (uid) => {
    selectedUid = uid;
    renderSelection();
  };

  mapHost.appendChild(createBrowseDatasourceConstellation(workspace, selectedUid));
  mapHost.appendChild(renderBrowseLanes(workspace, selectedUid, handleSelect));
  layout.appendChild(spotlightHost);
  layout.appendChild(mapHost);
  renderSelection();
  wrapper.appendChild(layout);

  const rows = rowArrayDocument(documentValue);
  if (rows) {
    wrapper.appendChild(renderTableResult(rows, {
      title: "Rows",
      subtitle: `${rows.length} row${rows.length === 1 ? "" : "s"}`,
      compact: true,
    }));
  }

  return wrapper;
}

export function renderTableResult(rowsOrDocument, options = {}) {
  const rows = Array.isArray(rowsOrDocument) ? rowsOrDocument : rowArrayDocument(rowsOrDocument) || [];
  const wrapper = createElement("section", options.compact ? "table-panel compact" : "table-panel");
  if (options.showHeader !== false) {
    const subtitle = options.subtitle || `${rows.length} row${rows.length === 1 ? "" : "s"}`;
    wrapper.appendChild(createSectionTitle(options.title || "Rows", subtitle));
  }

  if (!rows.length) {
    wrapper.appendChild(createEmptyState(options.emptyMessage || "No rows were returned.", true));
    return wrapper;
  }

  const columns = collectColumns(rows);
  const controls = createElement("div", "table-controls");
  const searchInput = createElement("input");
  searchInput.type = "search";
  searchInput.className = "table-filter-input";
  searchInput.placeholder = options.searchPlaceholder || "Filter rows";
  if (options.searchable === false) {
    searchInput.hidden = true;
  }
  controls.appendChild(searchInput);

  const summary = createElement("span", "table-controls-summary");
  controls.appendChild(summary);
  wrapper.appendChild(controls);

  const scroll = createElement("div", "table-scroll");
  const table = createElement("table", "data-table");
  const thead = createElement("thead");
  const headRow = createElement("tr");
  const tbody = createElement("tbody");

  const state = {
    query: "",
    sortKey: "",
    sortDirection: "asc",
  };

  function filteredRows() {
    const query = state.query.trim().toLowerCase();
    let visible = rows;
    if (query) {
      visible = visible.filter((row) => {
        return columns.some((column) => valueAsText(row[column]).toLowerCase().includes(query));
      });
    }
    if (state.sortKey) {
      visible = [...visible].sort((left, right) => {
        const leftValue = valueSortKey(left[state.sortKey]);
        const rightValue = valueSortKey(right[state.sortKey]);
        if (leftValue < rightValue) {
          return state.sortDirection === "asc" ? -1 : 1;
        }
        if (leftValue > rightValue) {
          return state.sortDirection === "asc" ? 1 : -1;
        }
        return 0;
      });
    }
    if (typeof options.maxRows === "number") {
      visible = visible.slice(0, options.maxRows);
    }
    return visible;
  }

  function renderHeader() {
    clearNode(headRow);
    columns.forEach((column) => {
      const th = createElement("th");
      const button = createElement("button", "table-sort-button", titleCaseFromKey(column));
      button.type = "button";
      button.dataset.column = column;
      if (state.sortKey === column) {
        button.dataset.sortDirection = state.sortDirection;
        button.setAttribute("aria-pressed", "true");
      } else {
        button.setAttribute("aria-pressed", "false");
      }
      if (options.sortable !== false) {
        button.addEventListener("click", () => {
          if (state.sortKey === column) {
            state.sortDirection = state.sortDirection === "asc" ? "desc" : "asc";
          } else {
            state.sortKey = column;
            state.sortDirection = "asc";
          }
          renderBody();
          renderHeader();
        });
      } else {
        button.disabled = true;
      }
      th.appendChild(button);
      headRow.appendChild(th);
    });
  }

  function renderBody() {
    const visibleRows = filteredRows();
    clearNode(tbody);
    if (!visibleRows.length) {
      const tr = createElement("tr", "table-empty-row");
      const td = createElement("td", "table-empty-cell", "No rows match the current filter.");
      td.colSpan = columns.length;
      tr.appendChild(td);
      tbody.appendChild(tr);
      summary.textContent = "0 visible";
      return;
    }
    visibleRows.forEach((row) => {
      const tr = createElement("tr");
      columns.forEach((column) => {
        tr.appendChild(createElement("td", "", valueAsText(row[column]) || "-"));
      });
      tbody.appendChild(tr);
    });
    summary.textContent = `${visibleRows.length} visible of ${rows.length}`;
  }

  searchInput.addEventListener("input", () => {
    state.query = searchInput.value;
    renderBody();
  });

  thead.appendChild(headRow);
  table.append(thead, tbody);
  scroll.appendChild(table);
  wrapper.appendChild(scroll);

  renderHeader();
  renderBody();

  return wrapper;
}

function syncSummarySubtitle(documentValue) {
  const kind = documentValue?.kind || "sync document";
  const resourceCount = documentValue?.summary?.resourceCount;
  if (resourceCount !== undefined) {
    return `${kind} | ${resourceCount} resources`;
  }
  return kind;
}

function renderSyncStatusSections(documentValue, wrapper) {
  appendIf(wrapper, createChipRow(documentValue?.summary));
  appendIf(wrapper, renderKeyValueGrid("Lineage", {
    stage: documentValue?.stage,
    stepIndex: documentValue?.stepIndex,
    traceId: documentValue?.traceId,
    parentTraceId: documentValue?.parentTraceId,
    reviewRequired: documentValue?.reviewRequired,
    reviewed: documentValue?.reviewed,
    approved: documentValue?.approved,
    executeLive: documentValue?.executeLive,
  }));

  [
    ["Preflight summary", documentValue?.preflightSummary],
    ["Bundle preflight summary", documentValue?.bundlePreflightSummary],
    ["Handoff summary", documentValue?.handoffSummary],
    ["Continuation summary", documentValue?.continuationSummary],
    ["Availability", documentValue?.availability],
  ].forEach(([title, value]) => {
    appendIf(wrapper, renderKeyValueGrid(title, value));
  });

  [
    ["Next actions", documentValue?.nextActions],
    ["Signal keys", documentValue?.signalKeys],
    ["Plugin ids", documentValue?.pluginIds],
    ["Datasource uids", documentValue?.datasourceUids],
    ["Contact points", documentValue?.contactPoints],
  ].forEach(([title, value]) => {
    appendIf(wrapper, renderListOfStrings(title, value));
  });

  [
    ["Blockers", documentValue?.blockers],
    ["Warnings", documentValue?.warnings],
    ["Operations", documentValue?.operations],
    ["Alerts", documentValue?.alerts],
    ["Drifts", documentValue?.drifts],
    ["Resources", documentValue?.resources],
  ].forEach(([title, value]) => {
    if (arrayOfObjects(value)) {
      wrapper.appendChild(createArraySection(title, value, { compact: true, maxRows: 200 }));
    }
  });

  if (isPlainObject(documentValue?.folders) || isPlainObject(documentValue?.datasources)) {
    const mapping = createElement("section", "document-subsection");
    mapping.appendChild(createSectionTitle("Mappings", "Promotion mapping surfaces"));
    appendIf(mapping, renderKeyValueGrid("Folders", documentValue.folders));
    appendIf(mapping, renderKeyValueGrid("Datasources", documentValue.datasources));
    wrapper.appendChild(mapping);
  }
}

export function renderSyncDiffResult(documentValue, options = {}) {
  const wrapper = createElement("section", "sync-diff-panel");
  wrapper.appendChild(createSectionTitle(options.title || "Sync review", syncSummarySubtitle(documentValue)));
  renderSyncStatusSections(documentValue, wrapper);

  const remainingObjectEntries = objectEntriesExcluding(documentValue || {}, [
    "kind",
    "summary",
    "stage",
    "stepIndex",
    "traceId",
    "parentTraceId",
    "reviewRequired",
    "reviewed",
    "approved",
    "executeLive",
    "preflightSummary",
    "bundlePreflightSummary",
    "handoffSummary",
    "continuationSummary",
    "availability",
    "nextActions",
    "signalKeys",
    "pluginIds",
    "datasourceUids",
    "contactPoints",
    "blockers",
    "warnings",
    "operations",
    "alerts",
    "drifts",
    "resources",
    "folders",
    "datasources",
  ]);

  remainingObjectEntries.forEach(([key, value]) => {
    if (arrayOfObjects(value)) {
      wrapper.appendChild(createArraySection(titleCaseFromKey(key), value, { maxRows: 100 }));
      return;
    }
    if (isPlainObject(value)) {
      appendIf(wrapper, renderKeyValueGrid(titleCaseFromKey(key), value));
      return;
    }
    if (Array.isArray(value)) {
      appendIf(wrapper, renderListOfStrings(titleCaseFromKey(key), value));
      return;
    }
    if (value !== undefined) {
      wrapper.appendChild(createObjectSection(titleCaseFromKey(key), { value }));
    }
  });

  return wrapper;
}

export function renderDocumentResult(documentValue, options = {}) {
  const wrapper = createElement("section", "document-panel");
  const kind = typeof documentValue?.kind === "string" ? documentValue.kind : "document";
  wrapper.appendChild(createSectionTitle(options.title || "Document", kind));

  appendIf(wrapper, createChipRow(documentValue?.summary));
  appendIf(wrapper, createScalarSummarySection(documentValue, ["kind", "summary"]));

  objectEntriesExcluding(documentValue || {}, ["kind", "summary"]).forEach(([key, value]) => {
    if (arrayOfObjects(value)) {
      wrapper.appendChild(createArraySection(titleCaseFromKey(key), value, { maxRows: 100 }));
      return;
    }
    if (isPlainObject(value)) {
      appendIf(wrapper, renderKeyValueGrid(titleCaseFromKey(key), value));
      return;
    }
    if (Array.isArray(value)) {
      appendIf(wrapper, renderListOfStrings(titleCaseFromKey(key), value));
      return;
    }
  });

  if (wrapper.childNodes.length === 1) {
    wrapper.appendChild(createCodeBlock(documentValue));
  }

  return wrapper;
}

export function renderGenericResult(value, options = {}) {
  if (arrayOfObjects(value)) {
    return renderTableResult(value, { title: options.title || "Rows" });
  }
  if (isPlainObject(value)) {
    return renderDocumentResult(value, options);
  }
  if (Array.isArray(value)) {
    return renderListOfStrings(options.title || "Values", value) || createEmptyState("No values were returned.");
  }

  const wrapper = createElement("section", "document-panel");
  wrapper.appendChild(createSectionTitle(options.title || "Result", "Scalar response"));
  wrapper.appendChild(createCodeBlock(valueAsText(value)));
  return wrapper;
}

export function renderVisualResult(host, action, response, options = {}) {
  clearNode(host);

  if (!response) {
    host.appendChild(createEmptyState("Run an action to render a workspace view."));
    return { rendererKey: "empty", documentValue: null };
  }

  const documentValue = responseDocument(response);
  const rendererKey = resolveRendererKey(action, response);

  if (rendererKey === "browse" && isPlainObject(documentValue)) {
    host.appendChild(renderBrowseResult(documentValue, options));
    return { rendererKey, documentValue };
  }

  if (rendererKey === "sync-diff" && isPlainObject(documentValue)) {
    host.appendChild(renderSyncDiffResult(documentValue, options));
    return { rendererKey, documentValue };
  }

  if (rendererKey === "table") {
    const rows = rowArrayDocument(documentValue !== null ? documentValue : response);
    if (rows) {
      host.appendChild(renderTableResult(rows, options));
      return { rendererKey, documentValue };
    }
  }

  if (rendererKey === "document" && documentValue !== null) {
    host.appendChild(renderDocumentResult(documentValue, options));
    return { rendererKey, documentValue };
  }

  if (documentValue !== null) {
    const rows = rowArrayDocument(documentValue);
    if (rows) {
      host.appendChild(renderTableResult(rows, options));
      return { rendererKey: "table", documentValue };
    }
    host.appendChild(renderGenericResult(documentValue, options));
    return { rendererKey: "generic", documentValue };
  }

  host.appendChild(renderGenericResult(response, options));
  return { rendererKey: "generic", documentValue: null };
}
