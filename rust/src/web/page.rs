pub(crate) fn html_page() -> &'static str {
    "<!doctype html>\
<html lang=\"en\">\
<head>\
  <meta charset=\"utf-8\">\
  <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
  <title>Grafana Utils Web Workbench</title>\
  <link rel=\"stylesheet\" href=\"/assets/index.css\">\
</head>\
<body>\
  <div id=\"app\" class=\"workbench-shell\">\
    <header class=\"top-bar panel\">\
      <div class=\"brand-block\">\
        <p class=\"eyebrow\">Rust Local Workbench</p>\
        <h1>Grafana Utils Web</h1>\
        <p class=\"lede\">A dense, local-first workspace for Grafana operators. Credentials stay in the browser session and are sent per request only.</p>\
      </div>\
      <div class=\"top-bar-controls\">\
        <label class=\"search-field\" for=\"global-search\">\
          <span>Search</span>\
          <input id=\"global-search\" type=\"search\" placeholder=\"Jump to a workspace or action\">\
        </label>\
        <div class=\"top-bar-rail\">\
          <span id=\"connection-pill\" class=\"connection-pill\">Session credentials only</span>\
          <a id=\"native-link\" class=\"native-link\" href=\"#\" rel=\"noreferrer\">Open Grafana</a>\
        </div>\
      </div>\
    </header>\
    <div class=\"shell-grid\">\
      <aside class=\"global-sidebar panel\">\
        <div class=\"panel-heading\">Global Sidebar</div>\
        <nav id=\"workspace-nav\" class=\"sidebar-nav\"></nav>\
      </aside>\
      <main class=\"workspace-stage\">\
        <section class=\"connection-drawer panel\" aria-label=\"Connection drawer\" data-open=\"false\" hidden>\
          <div class=\"drawer-header\">\
            <div>\
              <div class=\"panel-heading\">Connection Drawer</div>\
              <h2 class=\"section-headline\">Connect To Grafana</h2>\
            </div>\
            <button id=\"test-connection-button\" type=\"button\" class=\"secondary\">Test Connection</button>\
          </div>\
          <p class=\"section-intro\">Enter the Grafana URL and one authentication method here. Test the connection before running any workspace action.</p>\
          <div id=\"connection-fields\" class=\"connection-grid\">\
            <label class=\"field\"><span>Grafana URL</span><input id=\"conn-url\" type=\"text\" placeholder=\"http://localhost:3000\"></label>\
            <label class=\"field\"><span>Auth Mode</span><select id=\"conn-auth-mode\"><option value=\"token\">Token</option><option value=\"basic\">Username / Password</option></select></label>\
            <label class=\"field auth auth-token\"><span>API Token</span><input id=\"conn-api-token\" type=\"password\" placeholder=\"glsa_...\"></label>\
            <label class=\"field auth auth-basic\"><span>Username</span><input id=\"conn-username\" type=\"text\" placeholder=\"admin\"></label>\
            <label class=\"field auth auth-basic\"><span>Password</span><input id=\"conn-password\" type=\"password\" placeholder=\"admin\"></label>\
            <label class=\"field\"><span>Timeout Seconds</span><input id=\"conn-timeout\" type=\"number\" min=\"1\" placeholder=\"30\"></label>\
            <label class=\"checkbox-control checkbox-row\"><input id=\"conn-verify-ssl\" type=\"checkbox\"><span>Verify SSL</span></label>\
          </div>\
          <div id=\"connection-status\" class=\"connection-status\">Connection not tested yet. Credentials are kept in browser session storage only.</div>\
          <p class=\"connection-note\">The workbench sends credentials with each request. Nothing is persisted server-side, and the browser session is the only local storage.</p>\
        </section>\
        <section class=\"workspace-grid\">\
          <section class=\"parameter-pane panel\">\
            <div class=\"parameter-pane-header\">\
              <div>\
                <div class=\"panel-heading\">Parameter Pane</div>\
                <div id=\"workspace-title\" class=\"workspace-title\"></div>\
                <div id=\"workspace-description\" class=\"workspace-description\"></div>\
              </div>\
              <div id=\"status-line\" class=\"status-line\">Loading workspace registry...</div>\
            </div>\
            <div id=\"action-tabs\" class=\"action-tabs\" aria-label=\"Workspace action tabs\"></div>\
            <div class=\"action-meta\">\
              <div class=\"panel-heading\">Active Action</div>\
              <h2 id=\"action-title\"></h2>\
              <p id=\"action-description\"></p>\
            </div>\
            <form id=\"action-form\" class=\"field-grid\"></form>\
            <div class=\"sticky-action-bar\">\
              <button id=\"run-button\" type=\"button\">Run Action</button>\
              <button id=\"reset-button\" type=\"button\" class=\"secondary\">Reset Inputs</button>\
            </div>\
          </section>\
          <section class=\"result-stage panel\">\
            <div class=\"result-stage-header\">\
              <div>\
                <div class=\"panel-heading\">Result Stage</div>\
                <h2>Visual, Log, and Source</h2>\
              </div>\
              <button id=\"result-fullscreen-toggle\" type=\"button\" class=\"secondary\">Fullscreen</button>\
            </div>\
            <div id=\"result-tabs\" class=\"result-tabs\" aria-label=\"Result tabs\">\
              <button type=\"button\" class=\"result-tab active\" data-result-tab=\"visual\">Visual</button>\
              <button type=\"button\" class=\"result-tab\" data-result-tab=\"log\">Log</button>\
              <button type=\"button\" class=\"result-tab\" data-result-tab=\"source\">Source</button>\
            </div>\
            <div class=\"result-stage-body\">\
              <div id=\"result-visual\" class=\"result-visual\"></div>\
              <div class=\"result-source-grid\">\
                <section class=\"result-source-panel\">\
                  <h3>Text Preview</h3>\
                  <pre id=\"response-text\"></pre>\
                </section>\
                <section class=\"result-source-panel\">\
                  <h3>JSON Document</h3>\
                  <pre id=\"response-json\"></pre>\
                </section>\
              </div>\
            </div>\
          </section>\
        </section>\
      </main>\
    </div>\
  </div>\
  <script type=\"module\" src=\"/assets/index.js\"></script>\
</body>\
</html>"
}
