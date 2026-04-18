"""Repo-local Grafana connection profile management commands."""

import argparse
import sys
from typing import Any, List, Optional, Dict
from .cli_shared import dump_document, OUTPUT_FORMAT_CHOICES
from . import profile_config as pc


def build_parser(prog: Optional[str] = None) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog=prog or "grafana-util config",
        description="Repo-local Grafana connection profile management.",
    )
    subparsers = parser.add_subparsers(dest="command")
    subparsers.required = True

    # profile
    profile_parser = subparsers.add_parser("profile", help="Manage Grafana connection profiles.")
    profile_sub = profile_parser.add_subparsers(dest="subcommand")
    profile_sub.required = True

    # list
    profile_sub.add_parser("list", help="List profile names.")

    # show
    show_parser = profile_sub.add_parser("show", help="Show the selected profile.")
    show_parser.add_argument("name", nargs="?", help="Profile name.")
    show_parser.add_argument("--show-secrets", action="store_true", help="Reveal secrets.")
    show_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="text")

    # current
    current_parser = profile_sub.add_parser("current", help="Show the currently selected profile.")
    current_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="text")

    # example
    example_parser = profile_sub.add_parser("example", help="Render a profile config example.")
    example_parser.add_argument("--mode", choices=["basic", "full"], default="full")
    example_parser.add_argument("--output-format", choices=OUTPUT_FORMAT_CHOICES, default="yaml")

    # init
    init_parser = profile_sub.add_parser("init", help="Initialize grafana-util.yaml.")
    init_parser.add_argument("--overwrite", action="store_true", help="Allow overwrite.")

    return parser


def list_command(args: argparse.Namespace) -> int:
    doc = pc.load_profile_document()
    names = pc.list_profile_names(doc)
    for name in names:
        print(name)
    return 0


def show_command(args: argparse.Namespace) -> int:
    doc = pc.load_profile_document()
    try:
        name, profile = pc.select_profile(doc, args.name)
        summary = pc.build_profile_summary(name, profile, show_secrets=args.show_secrets)
        if args.output_format == "text":
            print("\n".join(pc.render_profile_summary_text(summary)))
        else:
            dump_document(summary, args.output_format)
    except ValueError as e:
        print(str(e), file=sys.stderr)
        return 1
    return 0


def current_command(args: argparse.Namespace) -> int:
    doc = pc.load_profile_document()
    path = pc.resolve_config_path()
    try:
        name, _ = pc.select_profile(doc)
        res = {"profile": name, "path": str(path.absolute())}
        if args.output_format == "text":
            print(f"Current profile: {name}")
            print(f"Config path: {path.absolute()}")
        else:
            dump_document(res, args.output_format)
    except ValueError as e:
        print(str(e), file=sys.stderr)
        return 1
    return 0


def example_command(args: argparse.Namespace) -> int:
    doc = pc.build_profile_example_document(mode=args.mode)
    dump_document(doc, args.output_format)
    return 0


def init_command(args: argparse.Namespace) -> int:
    path = pc.resolve_config_path()
    if path.exists() and not args.overwrite:
        print(f"Error: {path} already exists. Use --overwrite to replace it.", file=sys.stderr)
        return 1
    doc = pc.build_profile_example_document(mode="full")
    pc.save_profile_document(doc)
    print(f"Initialized {path}")
    return 0


def main(argv: Optional[list[str]] = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.subcommand == "list":
            return list_command(args)
        if args.subcommand == "show":
            return show_command(args)
        if args.subcommand == "current":
            return current_command(args)
        if args.subcommand == "example":
            return example_command(args)
        if args.subcommand == "init":
            return init_command(args)
    except Exception as e:
        print(str(e), file=sys.stderr)
        return 1
    return 0
