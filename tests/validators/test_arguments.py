import re

import pytest
from dirty_equals import IsListOrTuple

from pydantic_core import SchemaError, SchemaValidator, ValidationError

from ..conftest import Err, PyAndJson, plain_repr


@pytest.mark.parametrize(
    'input_value,expected',
    [
        [((1, 'a', True), None), ((1, 'a', True), {})],
        [((1, 'a', True), {}), ((1, 'a', True), {})],
        [([1, 'a', True], None), ((1, 'a', True), {})],
        [((1, 'a', 'true'), None), ((1, 'a', True), {})],
        ['x', Err('kind=arguments_type,')],
        [((1, 'a', True), ()), Err('kind=arguments_type,')],
        [(4, {}), Err('kind=arguments_type,')],
        [
            ([1, 'a', True], {'x': 1}),
            Err(
                '',
                [
                    {
                        'kind': 'unexpected_keyword_argument',
                        'loc': ['x'],
                        'message': 'Unexpected keyword argument',
                        'input_value': 1,
                    }
                ],
            ),
        ],
        [
            ([1], None),
            Err(
                '',
                [
                    {
                        'kind': 'missing_positional_argument',
                        'loc': [1],
                        'message': 'Missing positional argument',
                        'input_value': IsListOrTuple([1], None),
                    },
                    {
                        'kind': 'missing_positional_argument',
                        'loc': [2],
                        'message': 'Missing positional argument',
                        'input_value': IsListOrTuple([1], None),
                    },
                ],
            ),
        ],
        [
            ([1, 'a', True, 4], None),
            Err(
                '',
                [
                    {
                        'kind': 'unexpected_positional_arguments',
                        'loc': [],
                        'message': '1 unexpected positional argument',
                        'input_value': IsListOrTuple([1, 'a', True, 4], None),
                        'context': {'unexpected_count': 1},
                    }
                ],
            ),
        ],
        [
            ([1, 'a', True, 4, 5], None),
            Err(
                '',
                [
                    {
                        'kind': 'unexpected_positional_arguments',
                        'loc': [],
                        'message': '2 unexpected positional arguments',
                        'input_value': IsListOrTuple([1, 'a', True, 4, 5], None),
                        'context': {'unexpected_count': 2},
                    }
                ],
            ),
        ],
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
        [
            (None, None),
            Err(
                '3 validation errors for arguments',
                [
                    {
                        'kind': 'missing_positional_argument',
                        'loc': [0],
                        'message': 'Missing positional argument',
                        'input_value': IsListOrTuple(None, None),
                    },
                    {
                        'kind': 'missing_positional_argument',
                        'loc': [1],
                        'message': 'Missing positional argument',
                        'input_value': IsListOrTuple(None, None),
                    },
                    {
                        'kind': 'missing_positional_argument',
                        'loc': [2],
                        'message': 'Missing positional argument',
                        'input_value': IsListOrTuple(None, None),
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
        [
            ((), {'a': 1, 'b': 'a', 'c': True, 'd': 'wrong'}),
            Err(
                'kind=unexpected_keyword_argument,',
                [
                    {
                        'kind': 'unexpected_keyword_argument',
                        'loc': ['d'],
                        'message': 'Unexpected keyword argument',
                        'input_value': 'wrong',
                    }
                ],
            ),
        ],
        [
            ([], {'a': 1, 'b': 'a'}),
            Err(
                'kind=missing_keyword_argument,',
                [
                    {
                        'kind': 'missing_keyword_argument',
                        'loc': ['c'],
                        'message': 'Missing keyword argument',
                        'input_value': IsListOrTuple([], {'a': 1, 'b': 'a'}),
                    }
                ],
            ),
        ],
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
        [
            (None, None),
            Err(
                '',
                [
                    {
                        'kind': 'missing_keyword_argument',
                        'loc': ['a'],
                        'message': 'Missing keyword argument',
                        'input_value': IsListOrTuple(None, None),
                    },
                    {
                        'kind': 'missing_keyword_argument',
                        'loc': ['b'],
                        'message': 'Missing keyword argument',
                        'input_value': IsListOrTuple(None, None),
                    },
                    {
                        'kind': 'missing_keyword_argument',
                        'loc': ['c'],
                        'message': 'Missing keyword argument',
                        'input_value': IsListOrTuple(None, None),
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
        [
            ([1, 'bb', 'cc'], {'b': 'bb', 'c': True}),
            Err(
                'kind=unexpected_positional_arguments,',
                [
                    {
                        'kind': 'unexpected_positional_arguments',
                        'loc': [],
                        'message': '1 unexpected positional argument',
                        'input_value': IsListOrTuple([1, 'bb', 'cc'], {'b': 'bb', 'c': True}),
                        'context': {'unexpected_count': 1},
                    }
                ],
            ),
        ],
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
        # debug(exc_info.value.errors())
        if expected.errors:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


def test_arguments_mapping_build():
    v = SchemaValidator(
        {
            'type': 'arguments',
            'keyword_args_schema': {
                'type': 'typed-dict',
                'fields': {'a': {'schema': 'int'}, 'b': {'schema': 'str'}, 'c': {'schema': 'bool'}},
            },
        }
    )
    assert re.search(r'arguments_mapping: (\w+)', repr(v)).group(1) == 'None'
    v = SchemaValidator(
        {
            'type': 'arguments',
            'arguments_mapping': {1: 'b', 0: 'a'},
            'keyword_args_schema': {
                'type': 'typed-dict',
                'fields': {'a': {'schema': 'int'}, 'b': {'schema': 'str'}, 'c': {'schema': 'bool'}},
            },
        }
    )
    assert re.search(r'arguments_mapping: (\w+)', repr(v)).group(1) == 'Some'
    arguments_mapping = re.search('arguments_mapping:(.*?),pargs_validator', plain_repr(v)).group(1)
    # check that mapping has been sorted
    assert arguments_mapping == 'Some(ArgumentsMapping{slice_at:0,max_length:2,mapping:[(0,"a"),(1,"b"),]})'
    v = SchemaValidator(
        {
            'type': 'arguments',
            'arguments_mapping': {},
            'keyword_args_schema': {
                'type': 'typed-dict',
                'fields': {'a': {'schema': 'int'}, 'b': {'schema': 'str'}, 'c': {'schema': 'bool'}},
            },
        }
    )
    assert re.search(r'arguments_mapping: (\w+)', repr(v)).group(1) == 'None'


def test_build_no_args():
    m = "Arguments schema must have either 'positional_args_schema' or 'keyword_args_schema' defined"
    with pytest.raises(SchemaError, match=m):
        SchemaValidator({'type': 'arguments'})


@pytest.mark.parametrize(
    'input_value,expected',
    [
        [(None, {'a': 1}), ((), {'a': 1})],
        [(None, None), ((), {'a': 1})],
        [((), {'a': 1}), ((), {'a': 1})],
        [((), None), ((), {'a': 1})],
    ],
    ids=repr,
)
def test_all_optional(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'arguments',
            'arguments_mapping': {0: 'a'},
            'keyword_args_schema': {
                'type': 'typed-dict',
                'extra_behavior': 'forbid',
                'fields': {'a': {'schema': 'int', 'default': 1}},
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
        [([1, 2, 3], None), ((1, 2, 3), {})],
        [([1], None), ((1,), {})],
        [([], None), ((), {})],
        [([], {}), ((), {})],
        [([1, 2, 3], {'a': 1}), Err('a\n  Unexpected keyword argument [kind=unexpected_keyword_argument,')],
    ],
    ids=repr,
)
def test_var_args_only(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'arguments',
            'positional_args_schema': {
                'type': 'tuple',
                'mode': 'positional',
                'items_schema': [],
                'extra_schema': 'int',
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
        [([1, 'a', 'true'], {'b': 'bb', 'c': 3}), ((1, 'a'), {'a': True, 'b': 'bb', 'c': 3})],
        [([1, 'a'], {'a': 'true', 'b': 'bb', 'c': 3}), ((1, 'a'), {'a': True, 'b': 'bb', 'c': 3})],
        [
            ([1, 'a', 'true', 4, 5], {'b': 'bb', 'c': 3}),
            Err(
                'kind=unexpected_positional_arguments,',
                [
                    {
                        'kind': 'unexpected_positional_arguments',
                        'loc': [],
                        'message': '2 unexpected positional arguments',
                        'input_value': IsListOrTuple([1, 'a', 'true', 4, 5], {'b': 'bb', 'c': 3}),
                        'context': {'unexpected_count': 2},
                    }
                ],
            ),
        ],
    ],
    ids=repr,
)
def test_both(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {
            'type': 'arguments',
            'arguments_mapping': {2: 'a'},
            'positional_args_schema': {'type': 'tuple', 'mode': 'positional', 'items_schema': ['int', 'str']},
            'keyword_args_schema': {
                'type': 'typed-dict',
                'extra_behavior': 'forbid',
                'fields': {'a': {'schema': 'bool'}, 'b': {'schema': 'str'}, 'c': {'schema': 'int'}},
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


@pytest.mark.parametrize(
    'input_value,expected',
    [
        [([], {}), ((), {})],
        [(None, None), ((), {})],
        [([1], None), Err('1 unexpected positional argument [kind=unexpected_positional_arguments,')],
    ],
    ids=repr,
)
def test_no_args(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        {'type': 'arguments', 'positional_args_schema': {'type': 'tuple', 'mode': 'positional', 'items_schema': []}}
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)
        # debug(exc_info.value.errors())
        if expected.errors:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected
