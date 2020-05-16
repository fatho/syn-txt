#!/usr/bin/env python3

"""
Fetch the latest commit info from upstream nixpkgs/mozilla-nixpkgs.
"""

import os
import subprocess
import sys
import urllib.request
import json
from typing import Dict
from pathlib import Path

SCRIPT_DIR = Path(os.path.dirname(os.path.realpath(__file__)))

def get_github_commit_info(user: str, repo: str, branch: str) -> Dict[str, str]:
    parts = subprocess.check_output(['git', 'ls-remote', f"git@github.com:{user}/{repo}", branch]).split()

    commit = parts[0].decode()
    archive_url = f"https://github.com/{user}/{repo}/archive/{commit}.tar.gz"

    nix_hash = subprocess.check_output(['nix-prefetch-url', '--unpack', archive_url]).strip().decode()

    return {
        'url': archive_url,
        'sha256': nix_hash
    }


def update_upstream(user: str, repo: str, branch: str, output_name: str) -> None:
    print(f'Updating {user}/{repo}...')
    info = get_github_commit_info(user, repo, branch)
    with open(SCRIPT_DIR / f'{output_name}.json', mode='w', encoding='utf-8') as f:
        json.dump(info, f, indent=2)


def update_rustanalyzer():
    with urllib.request.urlopen('https://api.github.com/repos/rust-analyzer/rust-analyzer/releases') as r:
        if str(r.getcode()) != "200":
            raise Exception(f"Non 200 status: {r.getcode()}")
        data = json.load(r)
        releases = filter(lambda x: not x['prerelease'] and not x['draft'], data)
        latest = sorted(releases, key=lambda x: x['created_at'], reverse=True)[0]
        version = latest['tag_name']
        download_url = f"https://github.com/rust-analyzer/rust-analyzer/releases/download/{version}/rust-analyzer-linux"
        nix_hash = subprocess.check_output(['nix-prefetch-url', download_url]).strip().decode()
        info = {
            'version': version,
            'url': download_url,
            'sha256': nix_hash
        }
        with open(SCRIPT_DIR / f'rust-analyzer.json', mode='w', encoding='utf-8') as f:
            json.dump(info, f, indent=2)


if __name__ == '__main__':
    update_rustanalyzer()
    update_upstream('NixOS', 'nixpkgs', 'nixpkgs-unstable', 'nixpkgs')