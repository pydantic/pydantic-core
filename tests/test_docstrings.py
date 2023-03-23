import sys

import pytest

try:
    from pytest_examples import CodeExample, EvalExample, find_examples
except ImportError:
    # pytest_examples is not installed on emscripten
    CodeExample = EvalExample = None

    def find_examples(*_directories):
        return []


@pytest.mark.skipif(sys.platform not in {'linux', 'darwin'}, reason='Only both on linux and macos')
@pytest.mark.parametrize('example', find_examples('pydantic_core/core_schema.py'))
def test_docstrings(example: CodeExample, eval_example: EvalExample):
    if eval_example.update_examples:
        eval_example.format(example)
        eval_example.run_print_check(example, rewrite_assertions=True)
    else:
        eval_example.lint(example)
        eval_example.run(example, rewrite_assertions=True)


@pytest.mark.skipif(sys.platform not in {'linux', 'darwin'}, reason='Only both on linux and macos')
@pytest.mark.parametrize('example', find_examples('README.md'))
def test_readme(example: CodeExample, eval_example: EvalExample):
    eval_example.set_config(line_length=100, quotes='single')
    eval_example.lint(example)
    eval_example.run(example, rewrite_assertions=True)
