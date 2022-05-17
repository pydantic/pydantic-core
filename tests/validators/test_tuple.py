import pytest
from dirty_equals import IsNonNegative, IsTuple

from pydantic_core import SchemaValidator, ValidationError


@pytest.mark.parametrize('input_value,expected', [([1, 2, 3], (1, 2, 3)), ([1, 2, '3'], (1, 2, 3))])
def test_tuple_json(py_or_json, input_value, expected):
    v = py_or_json({'type': 'tuple', 'items': {'type': 'int'}})
    assert v.validate_test(input_value) == expected


def test_tuple_strict_passes_with_tuple():
    v = SchemaValidator({'type': 'tuple', 'items': {'type': 'int'}, 'strict': True})
    assert v.validate_python((1, 2, '33')) == (1, 2, 33)


@pytest.mark.parametrize('wrong_coll_type', [list, set, frozenset])
def test_tuple_strict_fails_without_tuple(wrong_coll_type):
    v = SchemaValidator({'type': 'tuple', 'items': {'type': 'int'}, 'strict': True})
    with pytest.raises(ValidationError) as exc_info:
        v.validate_python(wrong_coll_type([1, 2, '33']))
    assert exc_info.value.errors() == [
        {
            'kind': 'tuple_type',
            'loc': [],
            'message': 'Value must be a valid tuple',
            'input_value': wrong_coll_type([1, 2, '33']),
        }
    ]


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ((1, 2, '3'), (1, 2, 3)),
        ({1, 2, '3'}, IsTuple(1, 2, 3, check_order=False)),
        (frozenset([1, 2, '3']), IsTuple(1, 2, 3, check_order=False)),
    ],
)
def test_tuple(input_value, expected):
    v = SchemaValidator({'type': 'tuple', 'items': {'type': 'int'}})
    assert v.validate_python(input_value) == expected


@pytest.mark.parametrize(
    'input_value,index',
    [
        (['wrong'], 0),
        (('wrong',), 0),
        ({'wrong'}, 0),
        ((1, 2, 3, 'wrong'), 3),
        ((1, 2, 3, 'wrong', 4), 3),
        ((1, 2, 'wrong'), IsNonNegative()),
    ],
)
def test_tuple_error(input_value, index):
    v = SchemaValidator({'type': 'tuple', 'items': {'type': 'int'}})
    with pytest.raises(ValidationError) as exc_info:
        assert v.validate_python(input_value)
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': [index],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'wrong',
        }
    ]
