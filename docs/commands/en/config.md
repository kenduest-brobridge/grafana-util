# config

## Root

Purpose: explain what the `grafana-util config` namespace is for, when to stay at the root, and when to move into `config profile`.

When to use:
- when you need the setup model before picking a profile subcommand
- when you are deciding between direct flags and repo-local `--profile` usage
- when you need to understand where URL, auth defaults, and secret handling should live

Key flags:
- the root page is a namespace overview; operational flags live under `config profile`

Examples:

```bash
# inspect the config namespace before choosing a profile subcommand.
grafana-util config --help
```

```bash
# initialize a starter config file in the current checkout.
grafana-util config profile init --overwrite
```

## Decide what you need first

| What you are trying to do | Open this page first | What it answers |
| --- | --- | --- |
| Save repo-local connection defaults so daily commands stay short | [config profile](./profile.md) | How to create, validate, and reuse profiles |
| Inspect the profile shape in the current checkout | [config profile](./profile.md#show) | See the resolved settings before you guess |
| Confirm which profile will actually be selected | [config profile](./profile.md#current) | Check current/default selection behavior |
| Validate the profile shape and live connectivity | [config profile](./profile.md#validate) | Verify auth shape and `/api/health` reachability |
| Initialize a new `grafana-util.yaml` | [config profile](./profile.md#init) | Start from a minimal repo-local config file |

## What this root is for

`grafana-util config` is the repo-local configuration namespace. The public surface is small today, but the role is still important: this is the entrypoint for connection defaults, secret sources, and repeatable profile-backed Grafana access.

Open this page first when the question is:

- how this checkout should store Grafana URL and auth defaults
- when to keep using direct flags vs when to switch to `--profile`
- whether secrets should live in env vars, an OS store, or `encrypted-file`

Then move into `config profile` once you know which profile workflow you actually need.

## Choose the right setup path

- **First connection proof**: run `status live` with direct flags first
- **Daily repo-local workflow**: use `config profile add`
- **CI / automation**: prefer `token_env`, `password_env`, or a secret store
- **Bootstrap / break-glass**: use direct Basic auth first, then decide whether it should become a profile

## How `config` and `config profile` fit together

- `config`: namespace / entry root that answers how this repo should store and reuse connection settings
- `config profile`: the actual command group that runs `list`, `show`, `current`, `validate`, `add`, `example`, and `init`

So:

- open `config` when you need the overall setup model
- open [config profile](./profile.md) when you already know you need a specific profile subcommand

## Before / After

- **Before**: live commands repeat `--url`, Basic auth, or token flags across shell history, scripts, and onboarding notes
- **After**: one repo-local profile carries the shared connection defaults so daily commands stay shorter and CI is easier to audit

## What success looks like

- one checkout has the connection defaults it actually needs
- repeated live commands can use `--profile` instead of restating credentials
- the chosen secret storage mode fits the machine and operating model
- the team can separate the `config` root from the `config profile` operational surface

## Failure checks

- if a saved profile does not behave like a direct command line, inspect it with [config profile show](./profile.md#show)
- if validation fails, confirm the selected secret mode is supported on the current machine
- if a repo still needs repeated auth flags, check whether the intended profile was created and selected
- if you are still mixing up `config` and `config profile`, stop here before going deeper into leaf pages

## Workflow entrypoints

| Workflow | Entry page | Common next pages |
| --- | --- | --- |
| Inventory and inspection | [config profile](./profile.md#list) | [show](./profile.md#show), [current](./profile.md#current) |
| Validation | [config profile](./profile.md#validate) | [show](./profile.md#show), [current](./profile.md#current) |
| Creation and initialization | [config profile](./profile.md#add) | [example](./profile.md#example), [init](./profile.md#init) |

## Examples

```bash
# inspect the config namespace before choosing a profile subcommand.
grafana-util config --help
```

```bash
# initialize a starter config file in the current checkout.
grafana-util config profile init --overwrite
```

```bash
# add a reusable production profile backed by prompt-based secrets.
grafana-util config profile add prod --url https://grafana.example.com --basic-user admin --prompt-password --store-secret encrypted-file
```

## Related commands

- [config profile](./profile.md)
- [status](./status.md)
- [workspace](./workspace.md)
- [version](./version.md)
