from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, Dict, List, Any, Tuple


@dataclass
class ChangeStagedInputs:
    workspace: Path = field(default_factory=Path)
    desired_file: Optional[Path] = None
    source_bundle: Optional[Path] = None
    dashboard_export_dir: Optional[Path] = None
    dashboard_provisioning_dir: Optional[Path] = None
    alert_export_dir: Optional[Path] = None
    datasource_export_file: Optional[Path] = None
    datasource_provisioning_file: Optional[Path] = None
    access_user_export_dir: Optional[Path] = None
    access_team_export_dir: Optional[Path] = None
    access_org_export_dir: Optional[Path] = None
    access_service_account_export_dir: Optional[Path] = None


@dataclass
class NormalizedChangeDashboardInputs:
    dashboard_export_dir: Optional[Path] = None
    dashboard_provisioning_dir: Optional[Path] = None


def build_overview_args(
    inputs: ChangeStagedInputs,
    output_format: str,
    discovered_dashboard_export_dir: Optional[Path] = None,
    discovered_dashboard_provisioning_dir: Optional[Path] = None,
    discovered_datasource_provisioning_file: Optional[Path] = None,
    discovered_access_user_export_dir: Optional[Path] = None,
    discovered_access_team_export_dir: Optional[Path] = None,
    discovered_access_org_export_dir: Optional[Path] = None,
    discovered_access_service_account_export_dir: Optional[Path] = None,
    discovered_desired_file: Optional[Path] = None,
    discovered_source_bundle: Optional[Path] = None,
    discovered_target_inventory: Optional[Path] = None,
    discovered_alert_export_dir: Optional[Path] = None,
    discovered_availability_file: Optional[Path] = None,
    discovered_mapping_file: Optional[Path] = None,
) -> Dict[str, Any]:
    return {
        "dashboard_export_dir": inputs.dashboard_export_dir or discovered_dashboard_export_dir,
        "dashboard_provisioning_dir": inputs.dashboard_provisioning_dir or discovered_dashboard_provisioning_dir,
        "datasource_export_dir": None,
        "datasource_provisioning_file": inputs.datasource_provisioning_file or discovered_datasource_provisioning_file,
        "access_user_export_dir": inputs.access_user_export_dir or discovered_access_user_export_dir,
        "access_team_export_dir": inputs.access_team_export_dir or discovered_access_team_export_dir,
        "access_org_export_dir": inputs.access_org_export_dir or discovered_access_org_export_dir,
        "access_service_account_export_dir": inputs.access_service_account_export_dir or discovered_access_service_account_export_dir,
        "desired_file": inputs.desired_file or discovered_desired_file,
        "source_bundle": inputs.source_bundle or discovered_source_bundle,
        "target_inventory": discovered_target_inventory,
        "alert_export_dir": inputs.alert_export_dir or discovered_alert_export_dir,
        "availability_file": discovered_availability_file,
        "mapping_file": discovered_mapping_file,
        "output_format": output_format,
    }


def build_status_args(
    inputs: ChangeStagedInputs,
    output_format: str,
    target_inventory: Optional[Path] = None,
    availability_file: Optional[Path] = None,
    mapping_file: Optional[Path] = None,
    discovered_dashboard_export_dir: Optional[Path] = None,
    discovered_dashboard_provisioning_dir: Optional[Path] = None,
    discovered_datasource_provisioning_file: Optional[Path] = None,
    discovered_access_user_export_dir: Optional[Path] = None,
    discovered_access_team_export_dir: Optional[Path] = None,
    discovered_access_org_export_dir: Optional[Path] = None,
    discovered_access_service_account_export_dir: Optional[Path] = None,
    discovered_desired_file: Optional[Path] = None,
    discovered_source_bundle: Optional[Path] = None,
    discovered_target_inventory: Optional[Path] = None,
    discovered_alert_export_dir: Optional[Path] = None,
    discovered_availability_file: Optional[Path] = None,
    discovered_mapping_file: Optional[Path] = None,
) -> Dict[str, Any]:
    return {
        "dashboard_export_dir": inputs.dashboard_export_dir or discovered_dashboard_export_dir,
        "dashboard_provisioning_dir": inputs.dashboard_provisioning_dir or discovered_dashboard_provisioning_dir,
        "datasource_export_dir": None,
        "datasource_provisioning_file": inputs.datasource_provisioning_file or discovered_datasource_provisioning_file,
        "access_user_export_dir": inputs.access_user_export_dir or discovered_access_user_export_dir,
        "access_team_export_dir": inputs.access_team_export_dir or discovered_access_team_export_dir,
        "access_org_export_dir": inputs.access_org_export_dir or discovered_access_org_export_dir,
        "access_service_account_export_dir": inputs.access_service_account_export_dir or discovered_access_service_account_export_dir,
        "desired_file": inputs.desired_file or discovered_desired_file,
        "source_bundle": inputs.source_bundle or discovered_source_bundle,
        "target_inventory": target_inventory or discovered_target_inventory,
        "alert_export_dir": inputs.alert_export_dir or discovered_alert_export_dir,
        "availability_file": availability_file or discovered_availability_file,
        "mapping_file": mapping_file or discovered_mapping_file,
        "output_format": output_format,
    }


def select_preview_dashboard_sources(
    inputs: ChangeStagedInputs,
    dashboard_export_dir: Optional[Path] = None,
    dashboard_provisioning_dir: Optional[Path] = None,
) -> Tuple[Optional[Path], Optional[Path]]:
    if inputs.dashboard_export_dir and inputs.dashboard_provisioning_dir:
        raise ValueError(
            "Workspace preview accepts only one dashboard source: --dashboard-export-dir or --dashboard-provisioning-dir."
        )

    if inputs.dashboard_export_dir:
        return inputs.dashboard_export_dir, None
    if inputs.dashboard_provisioning_dir:
        return None, inputs.dashboard_provisioning_dir
    if dashboard_export_dir:
        return dashboard_export_dir, None
    return None, dashboard_provisioning_dir


def build_change_bundle_specs(
    inputs: ChangeStagedInputs,
    dashboard_export_dir: Optional[Path] = None,
    dashboard_provisioning_dir: Optional[Path] = None,
    discovered_alert_export_dir: Optional[Path] = None,
    discovered_datasource_provisioning_file: Optional[Path] = None,
) -> Optional[List[Dict[str, Any]]]:
    # In Rust, this calls load_sync_bundle_input_artifacts which we don't have yet.
    # We will just return None or raise an error if needed, but for now we follow the porting logic.
    # Since I don't have the implementation of load_sync_bundle_input_artifacts in Python,
    # I'll keep the function signature and a placeholder or basic logic if possible.
    
    # For now, let's just return None to indicate we can't build it without the other loaders.
    return None


def load_preview_desired_specs(
    inputs: ChangeStagedInputs,
    discovered_desired_file: Optional[Path] = None,
    discovered_source_bundle: Optional[Path] = None,
    dashboard_export_dir: Optional[Path] = None,
    dashboard_provisioning_dir: Optional[Path] = None,
    discovered_alert_export_dir: Optional[Path] = None,
    discovered_datasource_provisioning_file: Optional[Path] = None,
) -> List[Dict[str, Any]]:
    import json
    
    path = inputs.desired_file or discovered_desired_file
    if path:
        with open(path, "r") as f:
            return json.load(f)
            
    path = inputs.source_bundle or discovered_source_bundle
    if path:
        with open(path, "r") as f:
            source_bundle = json.load(f)
            if not isinstance(source_bundle, dict):
                raise ValueError("Workspace package input must be a JSON object.")
            desired_specs = []
            for key in ["dashboards", "datasources", "folders", "alerts"]:
                if key in source_bundle and isinstance(source_bundle[key], list):
                    desired_specs.extend(source_bundle[key])
            return desired_specs
            
    specs = build_change_bundle_specs(
        inputs,
        dashboard_export_dir,
        dashboard_provisioning_dir,
        discovered_alert_export_dir,
        discovered_datasource_provisioning_file,
    )
    if specs is not None:
        return specs
        
    raise ValueError(
        "Workspace preview could not find a staged desired file, workspace package, or staged export/provisioning inputs."
    )
