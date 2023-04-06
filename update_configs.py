"""
Update configurations for VSCode and other editors.
VSCode at least does not support composing configurations (https://github.com/microsoft/vscode/issues/15909)
and we don't want to clobber any existing configurations.
"""
import json
from pathlib import Path
from typing import Any, Dict

VSCODE_CONFIG_DIR = Path('.vscode')
VSCODE_CONFIG_DIR.mkdir(exist_ok=True)

settings: Dict[str, Any]
with (VSCODE_CONFIG_DIR / 'settings.json').open(mode='r') as f:
    settings = json.load(f)

settings.update(
    {
        'rust-analyzer.cargo.features': ['auto-initialize'],
        'rust-analyzer.cargo.noDefaultFeatures': True,
    }
)

with (VSCODE_CONFIG_DIR / 'settings.json').open(mode='w') as f:
    json.dump(settings, f, indent=4)
