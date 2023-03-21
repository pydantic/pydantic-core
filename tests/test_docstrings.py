import sys

import pytest
from pytest_examples import CodeExample, ExampleRunner, find_examples


@pytest.mark.skipif(sys.platform != 'linux', reason='README.md is not mounted in wasm file system')
@pytest.mark.parametrize('example', find_examples('pydantic_core/core_schema.py', 'README.md'))
def test_docstrings(example: CodeExample, run_example: ExampleRunner):
    run_example.run(example)
