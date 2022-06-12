import re

import pytest
from dirty_equals import IsNonNegative, IsTuple

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import Err


@pytest.mark.parametrize(
    'tuple_variant,items,input_value,expected',
    [
        ('tuple-var-len', {'type': 'int'}, [1, 2, 3], (1, 2, 3)),
        (
            'tuple-var-len',
            {'type': 'int'},
            1,
            Err('Value must be a valid tuple [kind=tuple_type, input_value=1, input_type=int]'),
        ),
        ('tuple-fix-len', [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}], [1, 2, '3'], (1, 2, 3)),
        (
            'tuple-fix-len',
            [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}],
            5,
            Err('Value must be a valid tuple [kind=tuple_type, input_value=5, input_type=int]'),
        ),
    ],
)
def test_tuple_json(py_or_json, tuple_variant, items, input_value, expected):
    v = py_or_json({'type': tuple_variant, 'items': items})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'tuple_variant,items,input,expected',
    [
        ('tuple-var-len', {'type': 'int'}, (1, 2, '33'), (1, 2, 33)),
        ('tuple-var-len', {'type': 'str'}, (1, 2, '33'), ('1', '2', '33')),
        ('tuple-fix-len', [{'type': 'int'}, {'type': 'str'}, {'type': 'float'}], (1, 2, 33), (1, '2', 33.0)),
    ],
)
def test_tuple_strict_passes_with_tuple(tuple_variant, items, input, expected):
    v = SchemaValidator({'type': tuple_variant, 'items': items, 'strict': True})
    assert v.validate_python(input) == expected


@pytest.mark.parametrize(
    'tuple_variant,items',
    [('tuple-var-len', {'type': 'int'}), ('tuple-fix-len', [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}])],
)
@pytest.mark.parametrize('wrong_coll_type', [list, set, frozenset])
def test_tuple_strict_fails_without_tuple(wrong_coll_type, tuple_variant, items):
    v = SchemaValidator({'type': tuple_variant, 'items': items, 'strict': True})
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
    'kwargs,input_value,expected',
    [
        ({'min_items': 3}, (1, 2), Err('Tuple must have at least 3 items [kind=tuple_too_short,')),
        ({'max_items': 3}, (1, 2, 3, 4), Err('Tuple must have at most 3 items [kind=tuple_too_long,')),
    ],
)
def test_tuple_var_len_kwargs(kwargs, input_value, expected):
    v = SchemaValidator({'type': 'tuple-var-len', **kwargs})
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_python(input_value)
    else:
        assert v.validate_python(input_value) == expected


@pytest.mark.parametrize(
    'tuple_variant,items',
    [('tuple-var-len', {'type': 'int'}), ('tuple-fix-len', [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}])],
)
@pytest.mark.parametrize(
    'input_value,expected',
    [
        ((1, 2, '3'), (1, 2, 3)),
        ({1, 2, '3'}, IsTuple(1, 2, 3, check_order=False)),
        (frozenset([1, 2, '3']), IsTuple(1, 2, 3, check_order=False)),
    ],
)
def test_tuple_validate(input_value, expected, tuple_variant, items):
    v = SchemaValidator({'type': tuple_variant, 'items': items})
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
def test_tuple_var_len_errors(input_value, index):
    v = SchemaValidator({'type': 'tuple-var-len', 'items': {'type': 'int'}})
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


@pytest.mark.parametrize(
    'input_value,items,index',
    [
        (['wrong'], [{'type': 'int'}], 0),
        (('wrong',), [{'type': 'int'}], 0),
        ({'wrong'}, [{'type': 'int'}], 0),
        ((1, 2, 3, 'wrong'), [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}, {'type': 'int'}], 3),
        (
            (1, 2, 3, 'wrong', 4),
            [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}, {'type': 'int'}, {'type': 'int'}],
            3,
        ),
        ((1, 2, 'wrong'), [{'type': 'int'}, {'type': 'int'}, {'type': 'int'}], IsNonNegative()),
    ],
)
def test_tuple_fix_len_errors(input_value, items, index):
    v = SchemaValidator({'type': 'tuple-fix-len', 'items': items})
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
