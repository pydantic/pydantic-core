import pytest

from pydantic_core import ValidationError

from ..conftest import PyOrJson


def test_none(py_or_json: PyOrJson):
    v = py_or_json('none')
    assert v.validate_test(None) is None
    with pytest.raises(ValidationError) as exc_info:
        v.validate_test(1)
    assert exc_info.value.errors() == [
        {'kind': 'none_required', 'loc': [], 'message': 'Value must be None/null', 'input_value': 1}
    ]
