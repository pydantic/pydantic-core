import re
from functools import wraps
from inspect import Parameter, signature
from typing import Any, get_type_hints

import pytest
from dirty_equals import IsListOrTuple, IsStr

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
        [(1, 2, 3), Err('kind=arguments_type,')],
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
        [
            ((1,), {'a': 11, 'b': 'bb', 'c': True}),
            Err(
                'kind=multiple_argument_values,',
                [
                    {
                        'kind': 'multiple_argument_values',
                        'loc': [0],
                        'message': "Got multiple values for argument 'a'",
                        'input_value': 1,
                        'context': {'arg': 'a'},
                    }
                ],
            ),
        ],
        [
            ((1, 'bb'), {'a': 11, 'b': 'bb', 'c': True}),
            Err(
                'kind=multiple_argument_values,',
                [
                    {
                        'kind': 'multiple_argument_values',
                        'loc': [0],
                        'message': "Got multiple values for argument 'a'",
                        'input_value': 1,
                        'context': {'arg': 'a'},
                    },
                    {
                        'kind': 'multiple_argument_values',
                        'loc': [1],
                        'message': "Got multiple values for argument 'b'",
                        'input_value': 'bb',
                        'context': {'arg': 'b'},
                    },
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
    assert arguments_mapping == IsStr(
        regex=(
            r'Some\('
            r'ArgumentsMapping{'
            r'slice_at:0,'
            r'max_length:2,'
            r'mapping:\[\(0,Py\(0x[0-9a-f]+\)\),\(1,Py\(0x[0-9a-f]+\)\),\]}'
            r'\)'
        )
    )
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


def double_or_bust(input_value, **kwargs):
    if input_value == 1:
        raise RuntimeError('bust')
    return input_value * 2


def test_positional_internal_error(py_and_json: PyAndJson):

    v = py_and_json(
        {
            'type': 'arguments',
            'positional_args_schema': {
                'type': 'tuple',
                'mode': 'positional',
                'items_schema': ['int', {'type': 'function', 'mode': 'plain', 'function': double_or_bust}],
            },
        }
    )
    assert v.validate_test(((1, 2), None)) == ((1, 4), {})
    with pytest.raises(RuntimeError, match='bust'):
        v.validate_test(((1, 1), None))


def test_kwarg_internal_error(py_and_json: PyAndJson):
    v = py_and_json(
        {
            'type': 'arguments',
            'keyword_args_schema': {
                'type': 'typed-dict',
                'extra_behavior': 'forbid',
                'fields': {
                    'a': {'schema': 'int'},
                    'b': {'schema': {'type': 'function', 'mode': 'plain', 'function': double_or_bust}},
                },
            },
        }
    )
    assert v.validate_test((None, {'a': 1, 'b': 2})) == ((), {'a': 1, 'b': 4})
    with pytest.raises(RuntimeError, match='bust'):
        v.validate_test((None, {'a': 1, 'b': 1}))


def validate(function):
    """
    a demo validation decorator to test arguments
    """
    parameters = signature(function).parameters

    type_hints = get_type_hints(function)

    arguments_mapping = {}
    positional_args = []
    extra_args_schema = None
    keyword_args = {}
    extra_kwargs_schema = None
    for i, (name, p) in enumerate(parameters.items()):
        if p.annotation is p.empty:
            annotation = Any
        else:
            annotation = type_hints[name]

        assert annotation in (bool, int, float, str, Any), f'schema for {annotation} not implemented'
        schema = annotation.__name__.lower()

        if p.kind == Parameter.POSITIONAL_ONLY:
            positional_args.append(schema)
            if p.default is not p.empty:
                raise NotImplementedError('default values for positional only arguments are not supported')
        elif p.kind == Parameter.POSITIONAL_OR_KEYWORD:
            keyword_args[name] = field = {'schema': schema}
            if p.default is not p.empty:
                field['default'] = p.default
            arguments_mapping[i] = name
        elif p.kind == Parameter.KEYWORD_ONLY:
            keyword_args[name] = field = {'schema': schema}
            if p.default is not p.empty:
                field['default'] = p.default
        elif p.kind == Parameter.VAR_POSITIONAL:
            extra_args_schema = schema
        else:
            assert p.kind == Parameter.VAR_KEYWORD, p.kind
            extra_kwargs_schema = schema

    schema = {
        'type': 'arguments',
        'arguments_mapping': arguments_mapping,
        'keyword_args_schema': {
            'type': 'typed-dict',
            'extra_behavior': 'forbid',
            'fields': {'a': {'schema': 'bool'}, 'b': {'schema': 'str'}, 'c': {'schema': 'int'}},
        },
    }
    if positional_args or extra_args_schema:
        schema['positional_args_schema'] = {'type': 'tuple', 'mode': 'positional', 'items_schema': positional_args}
        if extra_args_schema:
            schema['positional_args_schema']['extra_schema'] = extra_args_schema
    if keyword_args or extra_kwargs_schema:
        schema['keyword_args_schema'] = {'type': 'typed-dict', 'extra_behavior': 'forbid', 'fields': keyword_args}
        if extra_kwargs_schema:
            schema['keyword_args_schema']['extra_behavior'] = 'allow'
            schema['keyword_args_schema']['extra_validator'] = extra_kwargs_schema

    validator = SchemaValidator(schema)

    @wraps(function)
    def wrapper(*args, **kwargs):
        validated_args, validated_kwargs = validator.validate_python((args, kwargs))
        return function(*validated_args, **validated_kwargs)

    return wrapper


def test_function_any():
    @validate
    def foobar(a, b, c):
        return a, b, c

    assert foobar(1, 2, 3) == (1, 2, 3)
    assert foobar(a=1, b=2, c=3) == (1, 2, 3)
    assert foobar(1, b=2, c=3) == (1, 2, 3)

    with pytest.raises(ValidationError, match='1 unexpected positional argument'):
        foobar(1, 2, 3, 4)

    with pytest.raises(ValidationError, match='d\n  Unexpected keyword argument'):
        foobar(1, 2, 3, d=4)


def test_function_types():
    @validate
    def foobar(a: int, b: int, *, c: int):
        return a, b, c

    assert foobar(1, 2, c='3') == (1, 2, 3)
    assert foobar(a=1, b='2', c=3) == (1, 2, 3)

    with pytest.raises(ValidationError, match='1 unexpected positional argument'):
        foobar(1, 2, 3)

    with pytest.raises(ValidationError) as exc_info:
        foobar(1, 'b')

    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': ['b'],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'b',
        },
        {
            'kind': 'missing_keyword_argument',
            'loc': ['c'],
            'message': 'Missing keyword argument',
            'input_value': ((1, 'b'), {}),
        },
    ]

    with pytest.raises(ValidationError) as exc_info:
        foobar(1, 'b', c='c')
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': ['b'],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'b',
        },
        {
            'kind': 'int_parsing',
            'loc': ['c'],
            'message': 'Value must be a valid integer, unable to parse string as an integer',
            'input_value': 'c',
        },
    ]


def test_function_positional_only():
    @validate
    def foobar(a: int, b: int, /, c: int):
        return a, b, c

    assert foobar('1', 2, 3) == (1, 2, 3)
    assert foobar('1', 2, c=3) == (1, 2, 3)
    with pytest.raises(ValidationError) as exc_info:
        foobar('1', b=2, c=3)
    assert exc_info.value.errors() == [
        {
            'kind': 'missing_positional_argument',
            'loc': [1],
            'message': 'Missing positional argument',
            'input_value': (('1',), {'b': 2, 'c': 3}),
        },
        {
            'kind': 'unexpected_keyword_argument',
            'loc': ['b'],
            'message': 'Unexpected keyword argument',
            'input_value': 2,
        },
    ]


def test_function_args_kwargs():
    @validate
    def foobar(*args, **kwargs):
        return args, kwargs

    assert foobar(1, 2, 3, a=4, b=5) == ((1, 2, 3), {'a': 4, 'b': 5})
    assert foobar(1, 2, 3) == ((1, 2, 3), {})
    assert foobar(a=1, b=2, c=3) == ((), {'a': 1, 'b': 2, 'c': 3})
    assert foobar() == ((), {})
