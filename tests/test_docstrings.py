import sys

import pytest

try:
    from pytest_examples import CodeExample, ExampleRunner, find_examples
except ImportError:
    # pytest_examples is not installed on emscripten
    def find_examples(*_directories):
        return []


@pytest.mark.skipif(sys.platform != 'linux', reason='README.md is not mounted in wasm file system')
@pytest.mark.parametrize('example', find_examples('pydantic_core/core_schema.py', 'README.md'))
def test_docstrings(example: CodeExample, run_example: ExampleRunner):
    run_example.run(example)
    # run_example.ruff(example)
    # run_example.black(example, line_length=100)
