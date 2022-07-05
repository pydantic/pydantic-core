"""
Logic required for running tests with webassembly/emscripten
"""
import json
import os
from pathlib import Path

import nox

PYODIDE_VERSION = '0.21.0-alpha.2'


def append_to_github_env(name: str, value: str):
    if github_env := os.getenv('GITHUB_ENV'):
        with open(github_env, 'w+') as f:
            f.write(f'{name}={value}\n')


@nox.session(name='setup-pyodide')
def setup_pyodide(session: nox.Session):
    root_dir = Path(__file__).resolve().parent.parent
    with session.chdir(root_dir / 'node_modules' / 'pyodide'):
        session.run('node', '../prettier/bin-prettier.js', '-w', 'pyodide.asm.js', external=True)
        with open('repodata.json') as f:
            emscripten_version = json.load(f)['info']['platform'].split('_', 1)[1].replace('_', '.')
            append_to_github_env('EMSCRIPTEN_VERSION', emscripten_version)


@nox.session(name='test-emscripten')
def test_emscripten(session: nox.Session):
    root_dir = Path(__file__).resolve().parent.parent
    session.run('node', 'emscripten_runner.js', str(root_dir), external=True)
