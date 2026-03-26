#!/usr/bin/env bash
set -euo pipefail

GRAFANA_URL="${GRAFANA_URL:-http://localhost:3000}"
GRAFANA_USER="${GRAFANA_USER:-admin}"
GRAFANA_PASSWORD="${GRAFANA_PASSWORD:-admin}"
DESTROY_MODE=false
RESET_ALL_DATA_MODE=false
CONFIRMED_RESET=false

fail() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

usage() {
  cat <<'EOF'
Usage: seed-grafana-sample-data.sh [OPTIONS]

Seed or destroy reusable developer sample data in a Grafana instance.

Options:
  --url URL           Grafana base URL (default: http://localhost:3000)
  --basic-user USER   Grafana admin username (default: admin)
  --basic-password PW Grafana admin password (default: admin)
  --destroy           Delete the sample data created by this script
  --reset-all-data    Delete all repo-relevant developer test data from Grafana
  --yes               Required with --reset-all-data
  -h, --help          Show this help text

Environment overrides:
  GRAFANA_URL
  GRAFANA_USER
  GRAFANA_PASSWORD

The script is idempotent:
- reuses existing orgs, folders, and datasources by fixed uid or name
- upserts dashboards with overwrite=true
- `--destroy` removes only the known sample resources and extra sample orgs
- `--reset-all-data --yes` is a destructive developer reset for disposable Grafana instances

Seeded sample layout:
- Org 1 Main Org.
  - Datasources: Smoke Prometheus, Smoke Prometheus 2, Smoke Loki
  - Users: browse-admin (Admin, grafanaAdmin), browse-editor (Editor), browse-viewer (Viewer), browse-auditor (Viewer)
  - Teams: platform-ops, qa-observers, api-editors
  - Folders: Platform, Platform / Infra, Platform / Team / Apps / Prod, Platform / Team / Apps / API
  - Dashboards: smoke-main, smoke-prom-only, query-smoke, mixed-query-smoke, two-prom-query-smoke, subfolder-main, subfolder-chain-smoke
- Org 2 Org Two
  - Dashboard: org-two-main
- Org 3 QA Org
  - Dashboard: qa-overview
- Org 4 Audit Org
  - Dashboard: audit-home

Reset-all-data scope:
- deletes all non-default orgs
- clears dashboards, folders, datasources, teams, service accounts, and alert rules in org 1
- deletes non-admin global users except the current login user
EOF
}

require_tool() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required tool: $1"
}

urlencode() {
  jq -rn --arg value "$1" '$value|@uri'
}

request_raw() {
  local method="$1"
  local path="$2"
  local payload="${3:-}"
  local org_id="${4:-}"
  local response
  local headers=(-u "${GRAFANA_USER}:${GRAFANA_PASSWORD}" -X "${method}")

  if [[ -n "${org_id}" ]]; then
    headers+=(-H "X-Grafana-Org-Id: ${org_id}")
  fi
  if [[ -n "${payload}" ]]; then
    headers+=(-H 'Content-Type: application/json' --data-binary "${payload}")
  fi

  response="$(
    curl --silent --show-error \
      "${headers[@]}" \
      "${GRAFANA_URL}${path}" \
      -w $'\n%{http_code}'
  )"
  HTTP_STATUS="${response##*$'\n'}"
  HTTP_BODY="${response%$'\n'*}"
}

request_json() {
  request_raw "$@"
  if [[ "${HTTP_STATUS}" != 2* ]]; then
    fail "request failed: $1 $2 -> HTTP ${HTTP_STATUS} ${HTTP_BODY}"
  fi
}

request_optional() {
  request_raw "$@"
  if [[ "${HTTP_STATUS}" == "404" ]]; then
    return 1
  fi
  if [[ "${HTTP_STATUS}" != 2* ]]; then
    fail "request failed: $1 $2 -> HTTP ${HTTP_STATUS} ${HTTP_BODY}"
  fi
  return 0
}

current_admin_login() {
  request_json GET "/api/user"
  printf '%s' "${HTTP_BODY}" | jq -r '.login // empty'
}

iter_global_users() {
  local page=1
  local batch_len
  while true; do
    request_json GET "/api/users?perpage=100&page=${page}"
    batch_len="$(printf '%s' "${HTTP_BODY}" | jq 'length')"
    printf '%s\n' "${HTTP_BODY}"
    [[ "${batch_len}" -lt 100 ]] && break
    page=$((page + 1))
  done
}

lookup_user_id_by_login() {
  local login="$1"
  iter_global_users | jq -sr --arg login "${login}" 'add | map(select(.login == $login)) | .[0].id // empty'
}

resolve_user_ids_csv() {
  local identities_csv="$1"
  local resolved=()
  local identity
  local user_id

  IFS=',' read -r -a raw_identities <<< "${identities_csv}"
  for identity in "${raw_identities[@]}"; do
    identity="$(printf '%s' "${identity}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [[ -n "${identity}" ]] || continue
    user_id="$(lookup_user_id_by_login "${identity}")"
    [[ -n "${user_id}" ]] || fail "failed to resolve user id for login ${identity}"
    resolved+=("${user_id}")
  done

  printf '%s\n' "${resolved[@]}"
}

resolve_user_identities_json() {
  local identities_csv="$1"
  local identity
  local resolved=()

  IFS=',' read -r -a raw_identities <<< "${identities_csv}"
  for identity in "${raw_identities[@]}"; do
    identity="$(printf '%s' "${identity}" | sed 's/^[[:space:]]*//; s/[[:space:]]*$//')"
    [[ -n "${identity}" ]] || continue
    resolved+=("$(
      iter_global_users | jq -sr --arg login "${identity}" 'add | map(select(.login == $login)) | .[0].email // .[0].login // empty'
    )")
  done

  printf '%s\n' "${resolved[@]}" | jq -R 'select(length > 0)' | jq -s .
}

ensure_user() {
  local login="$1"
  local email="$2"
  local name="$3"
  local org_role="$4"
  local grafana_admin="$5"
  local user_id

  user_id="$(lookup_user_id_by_login "${login}")"
  if [[ -z "${user_id}" ]]; then
    request_json POST "/api/admin/users" "$(
      jq -cn \
        --arg login "${login}" \
        --arg email "${email}" \
        --arg name "${name}" \
        '{login: $login, email: $email, name: $name, password: "secret123!"}'
    )"
    user_id="$(printf '%s' "${HTTP_BODY}" | jq -r '.id // empty')"
    [[ -n "${user_id}" ]] || fail "failed to create user ${login}"
    printf 'Created user %s (id=%s)\n' "${login}" "${user_id}"
  else
    printf 'Reused user %s (id=%s)\n' "${login}" "${user_id}"
  fi

  request_json PATCH "/api/org/users/${user_id}" "$(
    jq -cn --arg role "${org_role}" '{role: $role}'
  )"
  request_json PUT "/api/admin/users/${user_id}/permissions" "$(
    jq -cn --argjson isGrafanaAdmin "${grafana_admin}" '{isGrafanaAdmin: $isGrafanaAdmin}'
  )"
  printf 'Configured user %s role=%s grafanaAdmin=%s\n' "${login}" "${org_role}" "${grafana_admin}"
}

delete_user_if_present() {
  local login="$1"
  local user_id

  user_id="$(lookup_user_id_by_login "${login}")"
  if [[ -z "${user_id}" ]]; then
    printf 'Skipped user %s: not found\n' "${login}"
    return
  fi

  request_json DELETE "/api/admin/users/${user_id}"
  printf 'Deleted user %s (id=%s)\n' "${login}" "${user_id}"
}

lookup_team_id_by_name() {
  local name="$1"
  request_json GET "/api/teams/search?query=${name}&perpage=100&page=1"
  printf '%s' "${HTTP_BODY}" | jq -r --arg name "${name}" '.teams[]? | select(.name == $name) | .id' | head -n 1
}

list_team_members() {
  local team_id="$1"
  request_json GET "/api/teams/${team_id}/members"
  printf '%s' "${HTTP_BODY}"
}

ensure_team() {
  local name="$1"
  local email="$2"
  local members_csv="$3"
  local admins_csv="$4"
  local team_id
  local current_members_json
  local target_member
  local target_user_id
  local target_member_identities
  local target_admin_identities

  team_id="$(lookup_team_id_by_name "${name}")"
  if [[ -z "${team_id}" ]]; then
    request_json POST "/api/teams" "$(
      jq -cn --arg name "${name}" --arg email "${email}" '{name: $name, email: $email}'
    )"
    team_id="$(printf '%s' "${HTTP_BODY}" | jq -r '.teamId // .id // empty')"
    [[ -n "${team_id}" ]] || fail "failed to create team ${name}"
    printf 'Created team %s (id=%s)\n' "${name}" "${team_id}"
  else
    printf 'Reused team %s (id=%s)\n' "${name}" "${team_id}"
  fi

  current_members_json="$(list_team_members "${team_id}")"

  while IFS= read -r target_member; do
    [[ -n "${target_member}" ]] || continue
    target_user_id="$(lookup_user_id_by_login "${target_member}")"
    [[ -n "${target_user_id}" ]] || fail "failed to resolve user id for login ${target_member}"
    if ! printf '%s' "${current_members_json}" | jq -e --arg login "${target_member}" '
      .[]? | select((.login // "") == $login or (.email // "") == $login)
    ' >/dev/null; then
      request_json POST "/api/teams/${team_id}/members" "$(
        jq -cn --argjson userId "${target_user_id}" '{userId: $userId}'
      )"
      printf 'Added team member %s -> %s\n' "${name}" "${target_member}"
    fi
  done < <(printf '%s' "${members_csv}" | tr ',' '\n' | sed 's/^[[:space:]]*//; s/[[:space:]]*$//' )

  target_member_identities="$(resolve_user_identities_json "${members_csv}")"
  target_admin_identities="$(resolve_user_identities_json "${admins_csv}")"
  request_json PUT "/api/teams/${team_id}/members" "$(
    jq -cn \
      --argjson members "${target_member_identities:-[]}" \
      --argjson admins "${target_admin_identities:-[]}" '
      {
        members: $members,
        admins: $admins
      }'
  )"
  printf 'Configured team %s members=[%s] admins=[%s]\n' "${name}" "${members_csv}" "${admins_csv}"
}

delete_team_if_present() {
  local name="$1"
  local team_id

  team_id="$(lookup_team_id_by_name "${name}")"
  if [[ -z "${team_id}" ]]; then
    printf 'Skipped team %s: not found\n' "${name}"
    return
  fi

  request_json DELETE "/api/teams/${team_id}"
  printf 'Deleted team %s (id=%s)\n' "${name}" "${team_id}"
}

ensure_health() {
  request_json GET "/api/health"
}

lookup_org_id_by_name() {
  request_json GET "/api/orgs"
  printf '%s' "${HTTP_BODY}" | jq -r --arg name "$1" '.[] | select(.name == $name) | .id' | head -n 1
}

list_org_ids() {
  request_json GET "/api/orgs"
  printf '%s' "${HTTP_BODY}" | jq -r '.[].id'
}

ensure_org() {
  local name="$1"
  local org_id

  org_id="$(lookup_org_id_by_name "${name}")"
  if [[ -n "${org_id}" ]]; then
    printf '%s\n' "${org_id}"
    return
  fi

  request_json POST "/api/orgs" "$(jq -cn --arg name "${name}" '{name: $name}')"
  org_id="$(printf '%s' "${HTTP_BODY}" | jq -r '.orgId // .id // empty')"
  [[ -n "${org_id}" ]] || fail "failed to create org ${name}"
  printf '%s\n' "${org_id}"
}

lookup_datasource_uid() {
  local org_id="$1"
  local uid="$2"
  local name="$3"
  request_json GET "/api/datasources" "" "${org_id}"
  printf '%s' "${HTTP_BODY}" |
    jq -r --arg uid "${uid}" --arg name "${name}" \
      '.[] | select(.uid == $uid or .name == $name) | .uid' | head -n 1
}

ensure_datasource() {
  local org_id="$1"
  local uid="$2"
  local name="$3"
  local ds_type="$4"
  local url="$5"
  local is_default="$6"
  local existing_uid
  local recreated=false

  request_json GET "/api/datasources" "" "${org_id}"
  existing_uid="$(
    printf '%s' "${HTTP_BODY}" |
      jq -r --arg uid "${uid}" --arg name "${name}" \
        '.[] | select(.uid == $uid or .name == $name) | .uid' | head -n 1
  )"
  if [[ -n "${existing_uid}" ]]; then
    if [[ "${existing_uid}" != "${uid}" ]]; then
      request_json DELETE "/api/datasources/uid/${existing_uid}" "" "${org_id}"
      recreated=true
    else
      printf 'Reused datasource %s (org %s)\n' "${name}" "${org_id}"
      return
    fi
  fi
  request_json POST "/api/datasources" "$(
    jq -cn \
      --arg uid "${uid}" \
      --arg name "${name}" \
      --arg type "${ds_type}" \
      --arg url "${url}" \
      --argjson isDefault "${is_default}" \
      '{
        uid: $uid,
        name: $name,
        type: $type,
        access: "proxy",
        url: $url,
        isDefault: $isDefault
      }'
  )" "${org_id}"
  if [[ "${recreated}" == true ]]; then
    printf 'Recreated datasource %s (org %s): replaced uid %s with %s\n' "${name}" "${org_id}" "${existing_uid}" "${uid}"
  else
    printf 'Created datasource %s (org %s)\n' "${name}" "${org_id}"
  fi
}

delete_datasource() {
  local org_id="$1"
  local uid="$2"
  local name="$3"
  local existing_uid

  existing_uid="$(lookup_datasource_uid "${org_id}" "${uid}" "${name}")"
  if [[ -z "${existing_uid}" ]]; then
    printf 'Skipped datasource %s (org %s): not found\n' "${name}" "${org_id}"
    return
  fi

  request_json DELETE "/api/datasources/uid/${existing_uid}" "" "${org_id}"
  printf 'Deleted datasource %s (org %s)\n' "${name}" "${org_id}"
}

lookup_folder_uid() {
  local org_id="$1"
  local uid="$2"
  request_raw GET "/api/folders/${uid}" "" "${org_id}"
  if [[ "${HTTP_STATUS}" == "200" ]]; then
    printf '%s\n' "${uid}"
  fi
}

ensure_folder() {
  local org_id="$1"
  local uid="$2"
  local title="$3"
  local parent_uid="${4:-}"
  local existing_uid
  local payload

  existing_uid="$(lookup_folder_uid "${org_id}" "${uid}")"
  if [[ -n "${existing_uid}" ]]; then
    printf 'Reused folder %s (org %s)\n' "${title}" "${org_id}"
    return
  fi

  if [[ -n "${parent_uid}" ]]; then
    payload="$(jq -cn --arg uid "${uid}" --arg title "${title}" --arg parentUid "${parent_uid}" \
      '{uid: $uid, title: $title, parentUid: $parentUid}')"
  else
    payload="$(jq -cn --arg uid "${uid}" --arg title "${title}" '{uid: $uid, title: $title}')"
  fi
  request_json POST "/api/folders" "${payload}" "${org_id}"
  printf 'Created folder %s (org %s)\n' "${title}" "${org_id}"
}

delete_folder() {
  local org_id="$1"
  local uid="$2"
  local title="$3"

  if ! request_optional DELETE "/api/folders/${uid}" "" "${org_id}"; then
    printf 'Skipped folder %s (org %s): not found\n' "${title}" "${org_id}"
    return
  fi
  printf 'Deleted folder %s (org %s)\n' "${title}" "${org_id}"
}

upsert_dashboard() {
  local org_id="$1"
  local folder_uid="$2"
  local dashboard_json="$3"
  local uid
  local title
  local payload

  uid="$(printf '%s' "${dashboard_json}" | jq -r '.uid')"
  title="$(printf '%s' "${dashboard_json}" | jq -r '.title')"
  payload="$(jq -cn \
    --arg folderUid "${folder_uid}" \
    --argjson dashboard "${dashboard_json}" \
    '{dashboard: $dashboard, folderUid: $folderUid, overwrite: true, message: "developer sample seed"}'
  )"
  request_json POST "/api/dashboards/db" "${payload}" "${org_id}"
  printf 'Upserted dashboard %s (%s) in org %s\n' "${title}" "${uid}" "${org_id}"
}

delete_dashboard() {
  local org_id="$1"
  local uid="$2"

  if ! request_optional DELETE "/api/dashboards/uid/${uid}" "" "${org_id}"; then
    printf 'Skipped dashboard %s (org %s): not found\n' "${uid}" "${org_id}"
    return
  fi
  printf 'Deleted dashboard %s (org %s)\n' "${uid}" "${org_id}"
}

list_dashboard_uids() {
  local org_id="$1"
  local page=1
  local page_data

  while true; do
    request_json GET "/api/search?type=dash-db&limit=500&page=${page}" "" "${org_id}"
    page_data="$(printf '%s' "${HTTP_BODY}" | jq -r '.[].uid')"
    if [[ -z "${page_data}" ]]; then
      break
    fi
    printf '%s\n' "${page_data}"
    if [[ "$(printf '%s' "${HTTP_BODY}" | jq 'length')" -lt 500 ]]; then
      break
    fi
    page=$((page + 1))
  done
}

list_folder_uids() {
  local org_id="$1"
  request_json GET "/api/folders" "" "${org_id}"
  printf '%s' "${HTTP_BODY}" | jq -r '.[].uid'
}

delete_all_dashboards_in_org() {
  local org_id="$1"
  local uid
  while IFS= read -r uid; do
    [[ -n "${uid}" ]] || continue
    delete_dashboard "${org_id}" "${uid}"
  done < <(list_dashboard_uids "${org_id}")
}

delete_all_folders_in_org() {
  local org_id="$1"
  local uid
  while IFS= read -r uid; do
    [[ -n "${uid}" ]] || continue
    delete_folder "${org_id}" "${uid}" "${uid}"
  done < <(list_folder_uids "${org_id}")
}

delete_all_datasources_in_org() {
  local org_id="$1"
  local uid name
  request_json GET "/api/datasources" "" "${org_id}"
  while IFS=$'\t' read -r uid name; do
    [[ -n "${uid}" ]] || continue
    delete_datasource "${org_id}" "${uid}" "${name}"
  done < <(printf '%s' "${HTTP_BODY}" | jq -r '.[] | [.uid, .name] | @tsv')
}

delete_all_alert_rules_in_org() {
  local org_id="$1"
  local uid
  request_json GET "/api/v1/provisioning/alert-rules" "" "${org_id}"
  while IFS= read -r uid; do
    [[ -n "${uid}" ]] || continue
    request_json DELETE "/api/v1/provisioning/alert-rules/${uid}" "" "${org_id}"
    printf 'Deleted alert rule %s (org %s)\n' "${uid}" "${org_id}"
  done < <(printf '%s' "${HTTP_BODY}" | jq -r '.[]?.uid // empty')
}

delete_all_teams_in_org() {
  local org_id="$1"
  local team_id name
  request_json GET "/api/teams/search?perpage=1000&page=1" "" "${org_id}"
  while IFS=$'\t' read -r team_id name; do
    [[ -n "${team_id}" ]] || continue
    request_json DELETE "/api/teams/${team_id}" "" "${org_id}"
    printf 'Deleted team %s (org %s)\n' "${name}" "${org_id}"
  done < <(printf '%s' "${HTTP_BODY}" | jq -r '.teams[]? | [.id, .name] | @tsv')
}

delete_all_service_accounts_in_org() {
  local org_id="$1"
  local sa_id name
  request_json GET "/api/serviceaccounts/search?perpage=1000&page=1" "" "${org_id}"
  while IFS=$'\t' read -r sa_id name; do
    [[ -n "${sa_id}" ]] || continue
    request_json DELETE "/api/serviceaccounts/${sa_id}" "" "${org_id}"
    printf 'Deleted service account %s (org %s)\n' "${name}" "${org_id}"
  done < <(printf '%s' "${HTTP_BODY}" | jq -r '.serviceAccounts[]? | [.id, .name] | @tsv')
}

delete_non_admin_users() {
  local keep_login="$1"
  local user_id login is_admin
  request_json GET "/api/users?perpage=1000&page=1"
  while IFS=$'\t' read -r user_id login is_admin; do
    [[ -n "${user_id}" ]] || continue
    if [[ "${login}" == "${keep_login}" ]]; then
      continue
    fi
    if [[ "${is_admin}" == "true" ]]; then
      continue
    fi
    request_json DELETE "/api/admin/users/${user_id}"
    printf 'Deleted user %s\n' "${login}"
  done < <(printf '%s' "${HTTP_BODY}" | jq -r '.[]? | [.id, .login, (.isGrafanaAdmin // false)] | @tsv')
}

delete_non_default_orgs() {
  local org_id
  while IFS= read -r org_id; do
    [[ -n "${org_id}" ]] || continue
    if [[ "${org_id}" == "1" ]]; then
      continue
    fi
    request_json DELETE "/api/orgs/${org_id}"
    printf 'Deleted org %s\n' "${org_id}"
  done < <(list_org_ids)
}

reset_all_data() {
  local keep_login
  keep_login="$(current_admin_login)"
  [[ -n "${keep_login}" ]] || fail "failed to detect current Grafana login"

  delete_non_default_orgs
  delete_all_alert_rules_in_org "1"
  delete_all_dashboards_in_org "1"
  delete_all_folders_in_org "1"
  delete_all_datasources_in_org "1"
  delete_all_teams_in_org "1"
  delete_all_service_accounts_in_org "1"
  delete_non_admin_users "${keep_login}"
}

dashboard_smoke_main() {
  cat <<'EOF'
{
  "id": null,
  "uid": "smoke-main",
  "title": "Smoke Dashboard",
  "tags": ["sample", "smoke"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Up Query",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "smoke-prom"},
      "targets": [
        {"refId": "A", "expr": "up"}
      ],
      "gridPos": {"h": 8, "w": 12, "x": 0, "y": 0}
    },
    {
      "id": 2,
      "title": "Recent Logs",
      "type": "logs",
      "datasource": {"type": "loki", "uid": "smoke-loki"},
      "targets": [
        {"refId": "A", "expr": "{job=\"smoke\"}"}
      ],
      "gridPos": {"h": 8, "w": 12, "x": 12, "y": 0}
    }
  ]
}
EOF
}

dashboard_prom_only() {
  cat <<'EOF'
{
  "id": null,
  "uid": "smoke-prom-only",
  "title": "Prometheus Only",
  "tags": ["sample", "prometheus"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Only Prometheus",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "smoke-prom"},
      "targets": [
        {"refId": "A", "expr": "sum(up)"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_query_smoke() {
  cat <<'EOF'
{
  "id": null,
  "uid": "query-smoke",
  "title": "Query Smoke Dashboard",
  "tags": ["sample", "query"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Up Query",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "smoke-prom"},
      "targets": [
        {"refId": "A", "expr": "up{a=\"100\"}"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_mixed_query_smoke() {
  cat <<'EOF'
{
  "id": null,
  "uid": "mixed-query-smoke",
  "title": "Mixed Query Dashboard",
  "tags": ["sample", "mixed-datasource"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "refresh": "30s",
  "panels": [
    {
      "id": 1,
      "title": "Mixed Panel",
      "type": "timeseries",
      "datasource": {"type": "datasource", "uid": "-- Mixed --"},
      "targets": [
        {"refId": "A", "datasource": {"type": "prometheus", "uid": "smoke-prom"}, "expr": "up", "legendFormat": "prom"},
        {"refId": "B", "datasource": {"type": "loki", "uid": "smoke-loki"}, "expr": "{job=\"grafana\"}", "queryType": "range", "legendFormat": "loki"}
      ],
      "gridPos": {"h": 9, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_two_prom_query_smoke() {
  cat <<'EOF'
{
  "id": null,
  "uid": "two-prom-query-smoke",
  "title": "Two Prometheus Query Dashboard",
  "tags": ["sample", "two-prometheus"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "refresh": "30s",
  "panels": [
    {
      "id": 1,
      "title": "Two Prometheus Panel",
      "type": "timeseries",
      "datasource": {"type": "datasource", "uid": "-- Mixed --"},
      "targets": [
        {"refId": "A", "datasource": {"type": "prometheus", "uid": "smoke-prom"}, "expr": "up", "legendFormat": "prom-1"},
        {"refId": "B", "datasource": {"type": "prometheus", "uid": "smoke-prom-2"}, "expr": "up", "legendFormat": "prom-2"}
      ],
      "gridPos": {"h": 9, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_subfolder_main() {
  cat <<'EOF'
{
  "id": null,
  "uid": "subfolder-main",
  "title": "Subfolder Dashboard",
  "tags": ["sample", "folder"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Folder Query",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "smoke-prom"},
      "targets": [
        {"refId": "A", "expr": "rate(up[5m])"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_subfolder_chain_smoke() {
  cat <<'EOF'
{
  "id": null,
  "uid": "subfolder-chain-smoke",
  "title": "Subfolder Chain Dashboard",
  "tags": ["sample", "folder", "chain"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Prod Chain Query",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "smoke-prom"},
      "targets": [
        {"refId": "A", "expr": "sum(up)", "legendFormat": "prod"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_org_two() {
  cat <<'EOF'
{
  "id": null,
  "uid": "org-two-main",
  "title": "Org Two Dashboard",
  "tags": ["sample", "org-two"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Org Two Query",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "org-two-prom"},
      "targets": [
        {"refId": "A", "expr": "up"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_qa_overview() {
  cat <<'EOF'
{
  "id": null,
  "uid": "qa-overview",
  "title": "QA Overview",
  "tags": ["sample", "qa"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "QA Up",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "qa-prom"},
      "targets": [
        {"refId": "A", "expr": "up"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

dashboard_audit_home() {
  cat <<'EOF'
{
  "id": null,
  "uid": "audit-home",
  "title": "Audit Home",
  "tags": ["sample", "audit"],
  "timezone": "browser",
  "schemaVersion": 41,
  "version": 0,
  "panels": [
    {
      "id": 1,
      "title": "Audit Up",
      "type": "timeseries",
      "datasource": {"type": "prometheus", "uid": "audit-prom"},
      "targets": [
        {"refId": "A", "expr": "up"}
      ],
      "gridPos": {"h": 8, "w": 24, "x": 0, "y": 0}
    }
  ]
}
EOF
}

seed_main_org() {
  local org_id="$1"
  ensure_user "browse-admin" "browse-admin@example.com" "Browse Admin" "Admin" true
  ensure_user "browse-editor" "browse-editor@example.com" "Browse Editor" "Editor" false
  ensure_user "browse-viewer" "browse-viewer@example.com" "Browse Viewer" "Viewer" false
  ensure_user "browse-auditor" "browse-auditor@example.com" "Browse Auditor" "Viewer" false
  ensure_team "platform-ops" "platform-ops@example.com" "browse-admin,browse-editor" "browse-admin"
  ensure_team "qa-observers" "qa-observers@example.com" "browse-viewer,browse-auditor" "browse-auditor"
  ensure_team "api-editors" "api-editors@example.com" "browse-editor,browse-viewer" "browse-editor"
  ensure_datasource "${org_id}" "smoke-prom" "Smoke Prometheus" "prometheus" "http://prometheus:9090" true
  ensure_datasource "${org_id}" "smoke-prom-2" "Smoke Prometheus 2" "prometheus" "http://prometheus-two:9090" false
  ensure_datasource "${org_id}" "smoke-loki" "Smoke Loki" "loki" "http://loki:3100" false
  ensure_folder "${org_id}" "platform" "Platform"
  ensure_folder "${org_id}" "infra" "Infra" "platform"
  ensure_folder "${org_id}" "team" "Team" "platform"
  ensure_folder "${org_id}" "apps" "Apps" "team"
  ensure_folder "${org_id}" "prod" "Prod" "apps"
  ensure_folder "${org_id}" "api" "API" "apps"
  upsert_dashboard "${org_id}" "" "$(dashboard_smoke_main)"
  upsert_dashboard "${org_id}" "" "$(dashboard_prom_only)"
  upsert_dashboard "${org_id}" "" "$(dashboard_query_smoke)"
  upsert_dashboard "${org_id}" "" "$(dashboard_mixed_query_smoke)"
  upsert_dashboard "${org_id}" "" "$(dashboard_two_prom_query_smoke)"
  upsert_dashboard "${org_id}" "infra" "$(dashboard_subfolder_main)"
  upsert_dashboard "${org_id}" "prod" "$(dashboard_subfolder_chain_smoke)"
}

destroy_main_org() {
  local org_id="$1"
  delete_team_if_present "api-editors"
  delete_team_if_present "qa-observers"
  delete_team_if_present "platform-ops"
  delete_dashboard "${org_id}" "subfolder-chain-smoke"
  delete_dashboard "${org_id}" "subfolder-main"
  delete_dashboard "${org_id}" "two-prom-query-smoke"
  delete_dashboard "${org_id}" "mixed-query-smoke"
  delete_dashboard "${org_id}" "query-smoke"
  delete_dashboard "${org_id}" "smoke-prom-only"
  delete_dashboard "${org_id}" "smoke-main"
  delete_folder "${org_id}" "api" "API"
  delete_folder "${org_id}" "prod" "Prod"
  delete_folder "${org_id}" "apps" "Apps"
  delete_folder "${org_id}" "team" "Team"
  delete_folder "${org_id}" "infra" "Infra"
  delete_folder "${org_id}" "platform" "Platform"
  delete_datasource "${org_id}" "smoke-loki" "Smoke Loki"
  delete_datasource "${org_id}" "smoke-prom-2" "Smoke Prometheus 2"
  delete_datasource "${org_id}" "smoke-prom" "Smoke Prometheus"
  delete_user_if_present "browse-auditor"
  delete_user_if_present "browse-viewer"
  delete_user_if_present "browse-editor"
  delete_user_if_present "browse-admin"
}

seed_extra_org() {
  local org_name="$1"
  local datasource_uid="$2"
  local datasource_name="$3"
  local dashboard_json="$4"
  local org_id

  org_id="$(ensure_org "${org_name}")"
  ensure_datasource "${org_id}" "${datasource_uid}" "${datasource_name}" "prometheus" "http://prometheus:9090" true
  upsert_dashboard "${org_id}" "" "${dashboard_json}"
}

destroy_extra_org() {
  local org_name="$1"
  local datasource_uid="$2"
  local datasource_name="$3"
  local dashboard_uid="$4"
  local org_id

  org_id="$(lookup_org_id_by_name "${org_name}")"
  if [[ -z "${org_id}" ]]; then
    printf 'Skipped org %s: not found\n' "${org_name}"
    return
  fi

  delete_dashboard "${org_id}" "${dashboard_uid}"
  delete_datasource "${org_id}" "${datasource_uid}" "${datasource_name}"
  request_json DELETE "/api/orgs/${org_id}"
  printf 'Deleted org %s (%s)\n' "${org_name}" "${org_id}"
}

main() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --url)
        GRAFANA_URL="$2"
        shift 2
        ;;
      --basic-user)
        GRAFANA_USER="$2"
        shift 2
        ;;
      --basic-password)
        GRAFANA_PASSWORD="$2"
        shift 2
        ;;
      --destroy)
        DESTROY_MODE=true
        shift
        ;;
      --reset-all-data)
        RESET_ALL_DATA_MODE=true
        shift
        ;;
      --yes)
        CONFIRMED_RESET=true
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      *)
        fail "unknown argument: $1"
        ;;
    esac
  done

  require_tool curl
  require_tool jq
  ensure_health

  if [[ "${RESET_ALL_DATA_MODE}" == "true" ]]; then
    if [[ "${CONFIRMED_RESET}" != "true" ]]; then
      fail "--reset-all-data requires --yes"
    fi
    if [[ "${DESTROY_MODE}" == "true" ]]; then
      fail "choose either --destroy or --reset-all-data"
    fi
    reset_all_data
    printf 'Reset repo-relevant Grafana test data at %s\n' "${GRAFANA_URL}"
    return
  fi

  if [[ "${DESTROY_MODE}" == "true" ]]; then
    destroy_extra_org "Audit Org" "audit-prom" "Audit Prometheus" "audit-home"
    destroy_extra_org "QA Org" "qa-prom" "QA Prometheus" "qa-overview"
    destroy_extra_org "Org Two" "org-two-prom" "Org Two Prometheus" "org-two-main"
    destroy_main_org "1"
    printf 'Destroyed sample Grafana data at %s\n' "${GRAFANA_URL}"
    return
  fi

  seed_main_org "1"
  seed_extra_org "Org Two" "org-two-prom" "Org Two Prometheus" "$(dashboard_org_two)"
  seed_extra_org "QA Org" "qa-prom" "QA Prometheus" "$(dashboard_qa_overview)"
  seed_extra_org "Audit Org" "audit-prom" "Audit Prometheus" "$(dashboard_audit_home)"

  printf 'Seeded sample Grafana data at %s\n' "${GRAFANA_URL}"
}

main "$@"
