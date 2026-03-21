# Future Feature Proposals & Enhancements

Reviewed: 2026-03-21

This file should only keep ideas that are still future-facing. Several earlier
items are now partially covered by existing inspection, governance, sync, or
workbench paths, so each section below calls out current status before listing
the remaining gap.

## 1. Resource Housekeeping

Current status:

- Partially covered.
- Dashboard inspection already reports datasource usage, datasource inventory,
  orphaned datasources, mixed-datasource dashboards, and governance risks.
- What is still missing is cleanup automation and richer stale-resource
  lifecycle handling.

Remaining proposals:

- **Zombie Dashboard Detection**
  - Use Grafana API access timestamps or equivalent signals to identify
    dashboards not accessed for a configurable period.
- **Orphaned Resource Checker**
  - Extend beyond datasource orphan detection to cover alerts, library panels,
    and other resources with missing upstream references.
- **Bulk Deletion/Archiving**
  - Safe mechanisms to remove or move identified stale resources after review.

## 2. Performance & Quality Audit

Current status:

- Partially covered.
- Query-family analyzers and dashboard inspection contracts exist, but they do
  not yet provide stronger operator-facing quality scoring.

Remaining proposals:

- **Query Efficiency Analysis**
  - Flag Prometheus queries missing label filters or using high-cardinality
    wildcard patterns.
- **Panel Load Auditing**
  - Warn if a single dashboard exceeds a threshold of panels that degrades UI
    performance.
- **Variable Dependency Mapping**
  - Detect circular dependencies or broken filter chains in template variables.

## 3. Migration & Provisioning Support

Current status:

- Largely not started.
- Export/import/diff contracts are much stronger now, but they are still
  Grafana-native workflows, not provisioning or IaC emitters.

Remaining proposals:

- **Provisioning YAML Generator**
  - Convert existing live dashboards or datasources into Grafana
    `provisioning/*.yaml` layouts for GitOps migration.
- **UID Conflict Resolver**
  - Automate re-mapping of UIDs and datasource IDs when migrating resources
    between different Grafana instances.

## 4. Advanced Security & Secret Management

Current status:

- Partially staged only.
- Datasource secret-provider workbench and bundle-preflight coverage exist, but
  provider IO is still intentionally unwired.

Remaining proposals:

- **External Secret Provider Integration**
  - Formalize support for Vault, AWS Secrets Manager, Azure Key Vault, or
    equivalent backends on top of the current staged contract.
- **Service Account Audit**
  - Generate reports on service-account or API-key expiration, usage, and
    rotation posture for compliance.

## 5. Dependency Mapping & Impact Analysis

Current status:

- Partially covered.
- Dashboard inspection and governance already expose datasource usage,
  orphaned datasource signals, mixed dashboards, and some blast-radius style
  inventory views.
- What is still missing is a broader cross-resource, operator-facing impact
  analysis surface.

Remaining proposals:

- **Blast Radius Analysis**
  - When a datasource is modified or removed, generate a report showing exactly
    which dashboards and alerts will be affected.
- **Visual Resource Hierarchy**
  - Generate a Markdown table or DOT graph showing the relationship between
    folders, dashboards, datasources, and alerts.

## 6. Infrastructure as Code Bridging

Current status:

- Mostly not started.
- Today the project supports export/import/diff and stronger replay contracts,
  but not direct Terraform/Pulumi generation or two-live-instance structural
  diffing as a first-class workflow.

Remaining proposals:

- **Terraform/Pulumi Exporter**
  - Export existing manual Grafana configurations into HCL or Pulumi code for
    formal IaC management.
- **Cross-Environment Diffing**
  - Deep diff entire folder structures between two Grafana instances to
    identify drift without relying on manual export staging.
