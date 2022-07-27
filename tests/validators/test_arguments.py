import re

import pytest
from dirty_equals import IsListOrTuple

from pydantic_core import ValidationError

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize(
    'input_value,expected',
    [
        [((1, 'a', True), None), ((1, 'a', True), {})],
        [((1, 'a', True), {}), ((1, 'a', True), {})],
        [([1, 'a', True], None), ((1, 'a', True), {})],
        [((1, 'a', 'true'), None), ((1, 'a', True), {})],
        ['x', Err('kind=arguments_type,')],
        [((1, 'a', True), ()), Err('kind=arguments_type,')],
        [
            ([1, 'a', True], {'x': 1}),
            Err(
                'kind=unexpected_keyword_arguments,',
                [
                    {
                        'kind': 'unexpected_keyword_arguments',
                        'loc': [],
                        'message': 'Input included 1 unexpected key word argument',
                        'input_value': IsListOrTuple([1, 'a', True], {'x': 1}),
                        'context': {'count': 1},
                    }
                ],
            ),
        ],
        [((1, 'a', True, 4), None), Err('kind=too_long,')],
        [
            (('x', 'a', 'wrong'), None),
            Err(
                '',
                [
                    {
                        'kind': 'int_parsing',
                        'loc': [0],
                        'message': 'Value must be a valid integer, unable to parse string as an integer',
                        'input_value': 'x',
                    },
                    {
                        'kind': 'bool_parsing',
                        'loc': [2],
                        'message': 'Value must be a valid boolean, unable to interpret input',
                        'input_value': 'wrong',
                    },
                ],
            ),
        ],
    ],
    ids=repr,
)
def test_positional_args(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'arguments',
            'positional_args_schema': {'type': 'tuple', 'mode': 'positional', 'items_schema': ['int', 'str', 'bool']},
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)
        # debug(exc_info.value.errors())
        if expected.errors:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected

    with pytest.raises(ValidationError, match='kind=arguments_type,'):
        # lists are not allowed from python, but no equivalent restriction in JSON
        v.validate_python([(1, 'a', True), None])


@pytest.mark.parametrize(
    'input_value,expected',
    [
        [(None, {'a': 1, 'b': 'a', 'c': True}), ((), {'a': 1, 'b': 'a', 'c': True})],
        [{'a': 1, 'b': 'a', 'c': True}, ((), {'a': 1, 'b': 'a', 'c': True})],
        [(None, {'a': '1', 'b': 'a', 'c': 'True'}), ((), {'a': 1, 'b': 'a', 'c': True})],
        [((), {'a': 1, 'b': 'a', 'c': True}), ((), {'a': 1, 'b': 'a', 'c': True})],
        [((1,), {'a': 1, 'b': 'a', 'c': True}), Err('kind=unexpected_positional_arguments,')],
        [((), {'a': 1, 'b': 'a', 'c': True, 'd': 'wrong'}), Err('kind=extra_forbidden,')],
        [
            ((), {'a': 'x', 'b': 'a', 'c': 'wrong'}),
            Err(
                '',
                [
                    {
                        'kind': 'int_parsing',
                        'loc': ['a'],
                        'message': 'Value must be a valid integer, unable to parse string as an integer',
                        'input_value': 'x',
                    },
                    {
                        'kind': 'bool_parsing',
                        'loc': ['c'],
                        'message': 'Value must be a valid boolean, unable to interpret input',
                        'input_value': 'wrong',
                    },
                ],
            ),
        ],
    ],
    ids=repr,
)
def test_keyword_args(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'arguments',
            'keyword_args_schema': {
                'type': 'typed-dict',
                'extra_behavior': 'forbid',
                'fields': {'a': {'schema': 'int'}, 'b': {'schema': 'str'}, 'c': {'schema': 'bool'}},
            },
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)
        # debug(exc_info.value.errors())
        if expected.errors:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        [(None, {'a': 1, 'b': 'bb', 'c': True}), ((), {'a': 1, 'b': 'bb', 'c': True})],
        [((1, 'bb'), {'c': True}), ((), {'a': 1, 'b': 'bb', 'c': True})],
        [((1,), {'b': 'bb', 'c': True}), ((), {'a': 1, 'b': 'bb', 'c': True})],
    ],
    ids=repr,
)
def test_arguments_mapping(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'arguments',
            'arguments_mapping': {0: 'a', 1: 'b'},
            'keyword_args_schema': {
                'type': 'typed-dict',
                'extra_behavior': 'forbid',
                'fields': {'a': {'schema': 'int'}, 'b': {'schema': 'str'}, 'c': {'schema': 'bool'}},
            },
        }
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)
        if expected.errors:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected
