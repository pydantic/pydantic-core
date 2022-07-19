import re

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ([], []),
        ([1, 2, 3], [1, 2, 3]),
        ([1, 2, '3'], [1, 2, 3]),
        ((), ()),
        ((1, 2, '3'), (1, 2, 3)),
        (set(), set()),
        ({1, 2, 3}, {1, 2, 3}),
        ({1, 2, '3'}, {1, 2, 3}),
        (frozenset(), frozenset()),
        (frozenset([1, 2, '3']), frozenset([1, 2, 3])),
        ({}, Err('Value must be a valid list/array')),
        ('xxx', Err('Value must be a valid list/array')),
    ],
)
def test_sequence_ints_py(input_value, expected):
    v = SchemaValidator({'type': 'sequence', 'items_schema': {'type': 'int'}})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        output = v.validate_python(input_value)
        assert output == expected
        assert output.__class__ == input_value.__class__


def test_sequence_ints_json():
    v = SchemaValidator({'type': 'sequence', 'items_schema': {'type': 'int'}})
    assert v.validate_json('[1, 2, 3]') == [1, 2, 3]
    assert v.validate_json('[1, 2, "3"]') == [1, 2, 3]
