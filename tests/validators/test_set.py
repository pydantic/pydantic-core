import re

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [([], set()), ([1, 2, 3], {1, 2, 3}), ([1, 2, '3'], {1, 2, 3}), ([1, 2, 3, 2, 3], {1, 2, 3})],
)
def test_set_ints_both(py_or_json, input_value, expected):
    v = py_or_json({'type': 'set', 'items': {'type': 'int'}})
    assert v.validate_test(input_value) == expected


@pytest.mark.parametrize('input_value,expected', [([1, 2.5, '3'], {1, 2.5, '3'})])
def test_set_no_validators_both(py_or_json, input_value, expected):
    v = py_or_json({'type': 'set'})
    assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({1, 2, 3}, {1, 2, 3}),
        (set(), set()),
        ([1, 2, 3, 2, 3], {1, 2, 3}),
        ([], set()),
        ((1, 2, 3, 2, 3), {1, 2, 3}),
        ((), set()),
        ({'abc'}, Err('0\n  Value must be a valid integer')),
        ({1: 2}, Err('1 validation error for set-int\n  Value must be a valid list/array')),
        ('abc', Err('Value must be a valid list/array')),
        # Technically correct, but does anyone actually need this? I think needs a new type in pyo3
        pytest.param({1: 10, 2: 20, 3: 30}.keys(), {1, 2, 3}, marks=pytest.mark.xfail(raises=ValidationError)),
    ],
)
def test_set_ints_python(input_value, expected):
    v = SchemaValidator({'type': 'set', 'items': {'type': 'int'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected


@pytest.mark.parametrize('input_value,expected', [([1, 2.5, '3'], {1, 2.5, '3'}), ([(1, 2), (3, 4)], {(1, 2), (3, 4)})])
def test_set_no_validators_python(input_value, expected):
    v = SchemaValidator({'type': 'set'})
    assert v.validate_python(input_value) == expected


def test_set_multiple_errors():
    v = SchemaValidator({'type': 'set', 'items': {'type': 'int'}})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(['a', (1, 2), []])
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': [0],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'a',
        },
        {'kind': 'int_type', 'loc': [1], 'message': 'Value must be a valid integer', 'input_value': (1, 2)},
        {'kind': 'int_type', 'loc': [2], 'message': 'Value must be a valid integer', 'input_value': []},
    ]
