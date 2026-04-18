from dataclasses import dataclass, field
from enum import Enum, auto
from pathlib import Path
from typing import Dict, List, Optional, Any, Tuple, Union


class DiscoveryInputKind(Enum):
    DashboardExportDir = "dashboardExportDir"
    DashboardProvisioningDir = "dashboardProvisioningDir"
    DatasourceProvisioningFile = "datasourceProvisioningFile"
    DatasourceExportFile = "datasourceExportFile"
    AccessUserExportDir = "accessUserExportDir"
    AccessTeamExportDir = "accessTeamExportDir"
    AccessOrgExportDir = "accessOrgExportDir"
    AccessServiceAccountExportDir = "accessServiceAccountExportDir"
    AlertExportDir = "alertExportDir"
    DesiredFile = "desiredFile"
    SourceBundle = "sourceBundle"
    TargetInventory = "targetInventory"
    AvailabilityFile = "availabilityFile"
    MappingFile = "mappingFile"
    ReviewedPlanFile = "reviewedPlanFile"
    MetadataFile = "metadataFile"

    @property
    def json_key(self) -> str:
        return self.value

    @property
    def summary_label(self) -> str:
        labels = {
            DiscoveryInputKind.DashboardExportDir: "dashboard-export",
            DiscoveryInputKind.DashboardProvisioningDir: "dashboard-provisioning",
            DiscoveryInputKind.DatasourceProvisioningFile: "datasource-provisioning",
            DiscoveryInputKind.DatasourceExportFile: "datasource-export",
            DiscoveryInputKind.AccessUserExportDir: "access-users",
            DiscoveryInputKind.AccessTeamExportDir: "access-teams",
            DiscoveryInputKind.AccessOrgExportDir: "access-orgs",
            DiscoveryInputKind.AccessServiceAccountExportDir: "access-service-accounts",
            DiscoveryInputKind.AlertExportDir: "alert-export",
            DiscoveryInputKind.DesiredFile: "desired-file",
            DiscoveryInputKind.SourceBundle: "source-bundle",
            DiscoveryInputKind.TargetInventory: "target-inventory",
            DiscoveryInputKind.AvailabilityFile: "availability-file",
            DiscoveryInputKind.MappingFile: "mapping-file",
            DiscoveryInputKind.ReviewedPlanFile: "reviewed-plan-file",
            DiscoveryInputKind.MetadataFile: "metadata-file",
        }
        return labels[self]

    @classmethod
    def from_json_key(cls, key: str) -> Optional["DiscoveryInputKind"]:
        for kind in cls:
            if kind.value == key:
                return kind
        return None


@dataclass
class DiscoveryInput:
    kind: DiscoveryInputKind
    path: Path


@dataclass
class ChangeDiscoveryDocument:
    workspace_root: Optional[Path] = None
    inputs: Dict[DiscoveryInputKind, Path] = field(default_factory=dict)

    def insert(self, kind: DiscoveryInputKind, path: Union[Path, str]) -> None:
        self.inputs[kind] = Path(path)

    def extend(self, inputs: List[DiscoveryInput]) -> None:
        for input_item in inputs:
            self.insert(input_item.kind, input_item.path)

    def is_empty(self) -> bool:
        return not self.inputs

    def summary_line(self) -> Optional[str]:
        if self.is_empty() or not self.workspace_root:
            return None
        sorted_kinds = sorted(self.inputs.keys(), key=lambda k: k.summary_label)
        sources = ", ".join(kind.summary_label for kind in sorted_kinds)
        return f"Discovery: workspace-root={self.workspace_root} sources={sources}"

    def provenance_line(self) -> Optional[str]:
        if self.is_empty() or not self.workspace_root:
            return None
        sorted_items = sorted(self.inputs.items(), key=lambda x: x[0].summary_label)
        sources = ", ".join(f"{kind.summary_label}={path}" for kind, path in sorted_items)
        return f"Discovered change workspace root {self.workspace_root} from {sources}."

    def to_dict(self) -> Dict[str, Any]:
        result = {}
        if self.workspace_root:
            result["workspaceRoot"] = str(self.workspace_root)
        result["inputCount"] = len(self.inputs)
        result["inputs"] = {
            kind.json_key: str(path) for kind, path in self.inputs.items()
        }
        return result


@dataclass
class DiscoveredChangeInputs:
    workspace_root: Optional[Path] = None
    dashboard_export_dir: Optional[Path] = None
    dashboard_provisioning_dir: Optional[Path] = None
    datasource_provisioning_file: Optional[Path] = None
    access_user_export_dir: Optional[Path] = None
    access_team_export_dir: Optional[Path] = None
    access_org_export_dir: Optional[Path] = None
    access_service_account_export_dir: Optional[Path] = None
    alert_export_dir: Optional[Path] = None
    desired_file: Optional[Path] = None
    source_bundle: Optional[Path] = None
    target_inventory: Optional[Path] = None
    availability_file: Optional[Path] = None
    mapping_file: Optional[Path] = None
    reviewed_plan_file: Optional[Path] = None


ACCESS_USER_EXPORT_DIR_NAME = "access-users"
ACCESS_TEAM_EXPORT_DIR_NAME = "access-teams"
ACCESS_ORG_EXPORT_DIR_NAME = "access-orgs"
ACCESS_SERVICE_ACCOUNT_EXPORT_DIR_NAME = "access-service-accounts"


def infer_workspace_root(base_dir: Path) -> Path:
    for ancestor in [base_dir] + list(base_dir.parents):
        name = ancestor.name
        if name in [
            ACCESS_USER_EXPORT_DIR_NAME,
            ACCESS_TEAM_EXPORT_DIR_NAME,
            ACCESS_ORG_EXPORT_DIR_NAME,
            ACCESS_SERVICE_ACCOUNT_EXPORT_DIR_NAME,
        ]:
            return ancestor.parent if ancestor.parent != ancestor else ancestor

    return _infer_dashboard_workspace_root(base_dir)


def _infer_dashboard_workspace_root(input_dir: Path) -> Path:
    if root := _infer_workspace_root_from_layout_ancestors(input_dir):
        return root
    return _infer_dashboard_workspace_root_fallback(input_dir)


def _infer_workspace_root_from_layout_ancestors(input_dir: Path) -> Optional[Path]:
    if input_dir.is_file() and input_dir.name == "datasources.yaml":
        parent = input_dir.parent
        grandparent = parent.parent
        great_grandparent = grandparent.parent
        if parent.name == "provisioning" and grandparent.name == "datasources":
            return great_grandparent if great_grandparent != grandparent else input_dir

    for ancestor in [input_dir] + list(input_dir.parents):
        name = ancestor.name
        if name in ["dashboards", "alerts", "datasources"]:
            if ancestor.parent.name == "provisioning":
                continue
            return ancestor.parent if ancestor.parent != ancestor else ancestor
        if name == "git-sync":
            parent = ancestor.parent
            grandparent = parent.parent
            if parent.name == "dashboards":
                return grandparent if grandparent != parent else ancestor

    return None


def _infer_dashboard_workspace_root_fallback(input_dir: Path) -> Path:
    name = input_dir.name
    if not name:
        return input_dir

    if input_dir.is_file():
        if name == "datasources.yaml":
            parent = input_dir.parent
            grandparent = parent.parent
            great_grandparent = grandparent.parent
            if parent.name == "provisioning" and grandparent.name == "datasources":
                return great_grandparent if great_grandparent != grandparent else input_dir
        return input_dir.parent if input_dir.parent != input_dir else input_dir

    parent = input_dir.parent
    grandparent = parent.parent
    if name in ["dashboards", "alerts", "datasources"]:
        return parent if parent != input_dir else input_dir
    if name == "git-sync":
        if parent.name == "dashboards":
            return grandparent if grandparent != parent else input_dir
        else:
            return input_dir
    if name in ["raw", "provisioning"]:
        if parent.name == "git-sync":
            return grandparent.parent if grandparent.parent != grandparent else input_dir
        if parent.name in ["dashboards", "alerts", "datasources"]:
            return grandparent if grandparent != parent else input_dir

    return input_dir


def discover_from_workspace_root(base_dir: Path) -> DiscoveredChangeInputs:
    datasources_dir = base_dir / "datasources"
    alerts_dir = base_dir / "alerts"

    dashboard_export_dir, dashboard_provisioning_dir = _dashboard_workspace_roots(base_dir)

    (
        access_user_export_dir,
        access_team_export_dir,
        access_org_export_dir,
        access_service_account_export_dir,
    ) = _access_workspace_dirs(base_dir)

    return DiscoveredChangeInputs(
        workspace_root=base_dir,
        dashboard_export_dir=dashboard_export_dir,
        dashboard_provisioning_dir=dashboard_provisioning_dir,
        datasource_provisioning_file=_first_existing(
            [datasources_dir / "provisioning" / "datasources.yaml"]
        ),
        access_user_export_dir=access_user_export_dir,
        access_team_export_dir=access_team_export_dir,
        access_org_export_dir=access_org_export_dir,
        access_service_account_export_dir=access_service_account_export_dir,
        alert_export_dir=_first_existing([alerts_dir / "raw", alerts_dir]),
        desired_file=_first_existing([base_dir / "desired.json"]),
        source_bundle=_first_existing(
            [base_dir / "sync-source-bundle.json", base_dir / "bundle.json"]
        ),
        target_inventory=_first_existing(
            [base_dir / "target-inventory.json", base_dir / "target.json"]
        ),
        availability_file=_first_existing([base_dir / "availability.json"]),
        mapping_file=_first_existing(
            [
                base_dir / "promotion-map.json",
                base_dir / "promotion-mapping.json",
                base_dir / "mapping.json",
            ]
        ),
        reviewed_plan_file=_first_existing(
            [
                base_dir / "sync-plan-reviewed.json",
                base_dir / "reviewed-plan.json",
                base_dir / "sync-plan.json",
            ]
        ),
    )


def _first_existing(paths: List[Path]) -> Optional[Path]:
    for path in paths:
        if path.exists():
            return path
    return None


def _dashboard_workspace_roots(base_dir: Path) -> Tuple[Optional[Path], Optional[Path]]:
    dashboards_dir = base_dir if base_dir.name == "dashboards" else base_dir / "dashboards"

    raw_dir = _resolve_dashboard_workspace_dir(
        dashboards_dir, "raw", "git-sync"
    ) or _resolve_dashboard_workspace_dir(dashboards_dir, "raw", None)

    provisioning_dir = _resolve_dashboard_workspace_dir(
        dashboards_dir, "provisioning", "git-sync"
    ) or _resolve_dashboard_workspace_dir(dashboards_dir, "provisioning", None)

    return raw_dir, provisioning_dir


def _resolve_dashboard_workspace_dir(
    dashboards_dir: Path, subdir: str, wrapper: Optional[str]
) -> Optional[Path]:
    if wrapper:
        candidate = dashboards_dir / wrapper / subdir
    else:
        candidate = dashboards_dir / subdir

    if candidate.is_dir():
        return candidate
    return None


def _access_workspace_dirs(
    base_dir: Path,
) -> Tuple[Optional[Path], Optional[Path], Optional[Path], Optional[Path]]:
    return (
        _first_existing([base_dir / ACCESS_USER_EXPORT_DIR_NAME]),
        _first_existing([base_dir / ACCESS_TEAM_EXPORT_DIR_NAME]),
        _first_existing([base_dir / ACCESS_ORG_EXPORT_DIR_NAME]),
        _first_existing([base_dir / ACCESS_SERVICE_ACCOUNT_EXPORT_DIR_NAME]),
    )


def discover_change_staged_inputs(base_dir: Optional[Path] = None) -> DiscoveredChangeInputs:
    if base_dir is None:
        base_dir = Path.cwd()

    workspace_root = infer_workspace_root(base_dir)
    discovered = discover_from_workspace_root(workspace_root)
    overlay_direct_workspace_input(discovered, base_dir)
    return discovered


def overlay_direct_workspace_input(discovered: DiscoveredChangeInputs, base_dir: Path) -> None:
    name = base_dir.name
    if not name:
        return

    if base_dir.is_file():
        _overlay_direct_workspace_file(discovered, base_dir, name)
    else:
        _overlay_direct_workspace_dir(discovered, base_dir, name)


def _overlay_direct_workspace_file(
    discovered: DiscoveredChangeInputs, base_dir: Path, name: str
) -> None:
    if name == "desired.json":
        discovered.desired_file = base_dir
    elif name in ["sync-source-bundle.json", "bundle.json"]:
        discovered.source_bundle = base_dir
    elif name in ["target-inventory.json", "target.json"]:
        discovered.target_inventory = base_dir
    elif name == "availability.json":
        discovered.availability_file = base_dir
    elif name in ["promotion-map.json", "promotion-mapping.json", "mapping.json"]:
        discovered.mapping_file = base_dir
    elif name in ["sync-plan-reviewed.json", "reviewed-plan.json", "sync-plan.json"]:
        discovered.reviewed_plan_file = base_dir
    elif name == "datasources.yaml":
        parent = base_dir.parent
        grandparent = parent.parent
        if parent.name == "provisioning" and grandparent.name == "datasources":
            discovered.datasource_provisioning_file = base_dir


def _overlay_direct_workspace_dir(
    discovered: DiscoveredChangeInputs, base_dir: Path, name: str
) -> None:
    # Dashboard layout discovery
    parent = base_dir.parent
    grandparent = parent.parent

    # Match (grandparent, parent, name)
    layout_match = None
    if grandparent.name == "dashboards" and parent.name == "git-sync":
        if name == "raw":
            layout_match = "raw"
        elif name == "provisioning":
            layout_match = "provisioning"
    elif parent.name == "dashboards":
        if name == "raw":
            layout_match = "raw"
        elif name == "provisioning":
            layout_match = "provisioning"

    if layout_match == "raw":
        discovered.dashboard_export_dir = base_dir
        return
    elif layout_match == "provisioning":
        discovered.dashboard_provisioning_dir = base_dir
        return

    if name == ACCESS_USER_EXPORT_DIR_NAME:
        discovered.access_user_export_dir = base_dir
    elif name == ACCESS_TEAM_EXPORT_DIR_NAME:
        discovered.access_team_export_dir = base_dir
    elif name == ACCESS_ORG_EXPORT_DIR_NAME:
        discovered.access_org_export_dir = base_dir
    elif name == ACCESS_SERVICE_ACCOUNT_EXPORT_DIR_NAME:
        discovered.access_service_account_export_dir = base_dir
    elif parent.name == "alerts" and name == "raw":
        discovered.alert_export_dir = base_dir
    elif parent.name == "datasources" and name == "provisioning":
        discovered.datasource_provisioning_file = _first_existing([base_dir / "datasources.yaml"])
    elif name == "alerts":
        discovered.alert_export_dir = _first_existing([base_dir / "raw", base_dir])
    elif name == "datasources":
        discovered.datasource_provisioning_file = _first_existing(
            [base_dir / "provisioning" / "datasources.yaml"]
        )


def build_change_discovery(discovered: DiscoveredChangeInputs) -> Optional[ChangeDiscoveryDocument]:
    if not discovered.workspace_root:
        return None

    doc = ChangeDiscoveryDocument(workspace_root=discovered.workspace_root)

    mapping = {
        DiscoveryInputKind.DashboardExportDir: discovered.dashboard_export_dir,
        DiscoveryInputKind.DashboardProvisioningDir: discovered.dashboard_provisioning_dir,
        DiscoveryInputKind.DatasourceProvisioningFile: discovered.datasource_provisioning_file,
        DiscoveryInputKind.AccessUserExportDir: discovered.access_user_export_dir,
        DiscoveryInputKind.AccessTeamExportDir: discovered.access_team_export_dir,
        DiscoveryInputKind.AccessOrgExportDir: discovered.access_org_export_dir,
        DiscoveryInputKind.AccessServiceAccountExportDir: discovered.access_service_account_export_dir,
        DiscoveryInputKind.AlertExportDir: discovered.alert_export_dir,
        DiscoveryInputKind.DesiredFile: discovered.desired_file,
        DiscoveryInputKind.SourceBundle: discovered.source_bundle,
        DiscoveryInputKind.TargetInventory: discovered.target_inventory,
        DiscoveryInputKind.AvailabilityFile: discovered.availability_file,
        DiscoveryInputKind.MappingFile: discovered.mapping_file,
        DiscoveryInputKind.ReviewedPlanFile: discovered.reviewed_plan_file,
    }

    for kind, path in mapping.items():
        if path:
            doc.insert(kind, path)

    return doc if not doc.is_empty() else None


def render_discovery_provenance(discovered: DiscoveredChangeInputs) -> Optional[str]:
    doc = build_change_discovery(discovered)
    return doc.provenance_line() if doc else None


def render_discovery_summary(discovered: DiscoveredChangeInputs) -> Optional[str]:
    doc = build_change_discovery(discovered)
    return doc.summary_line() if doc else None


def build_discovery_document(discovered: DiscoveredChangeInputs) -> Optional[Dict[str, Any]]:
    doc = build_change_discovery(discovered)
    return doc.to_dict() if doc else None
