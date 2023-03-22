import sys

import pytest

try:
    from pytest_examples import CodeExample, EvalExample, find_examples
except ImportError:
    # pytest_examples is not installed on emscripten
    def find_examples(*_directories):
        return []


@pytest.mark.skipif(sys.platform not in {'linux', 'darwin'}, reason='README.md is not mounted in wasm file system')
@pytest.mark.parametrize('example', find_examples('pydantic_core/core_schema.py', 'README.md'))
def test_docstrings(example: CodeExample, eval_example: EvalExample):
    if example.path.name != 'README.md':
        eval_example.lint(example, line_length=100)
    eval_example.run(example, rewrite_assertions=True)
