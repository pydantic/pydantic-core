import re

import pytest
from dirty_equals import IsListOrTuple

from pydantic_core import ValidationError, core_schema

from ..conftest import Err, PyAndJson


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (('hello', True), ({'a': 'hello', 'b': True}, None)),
        (['hello', True], ({'a': 'hello', 'b': True}, None)),
        ({'a': 'hello', 'b': True}, ({'a': 'hello', 'b': True}, None)),
        ({'a': 'hello', 'b': 'true'}, ({'a': 'hello', 'b': True}, None)),
        ({'__args__': ('hello', True), '__kwargs__': {}}, ({'a': 'hello', 'b': True}, None)),
        ({'__args__': (), '__kwargs__': {'a': 'hello', 'b': True}}, ({'a': 'hello', 'b': True}, None)),
        (
            {'__args__': ('hello',), '__kwargs__': {'a': 'hello', 'b': True}},
            Err(
                'Got multiple values for argument',
                errors=[
                    {
                        'type': 'multiple_argument_values',
                        'loc': ('a',),
                        'msg': 'Got multiple values for argument',
                        'input': 'hello',
                    }
                ],
            ),
        ),
        (
            {'a': 'hello'},
            Err(
                'Missing required keyword argument',
                errors=[
                    {
                        'type': 'missing_keyword_argument',
                        'loc': ('b',),
                        'msg': 'Missing required keyword argument',
                        'input': {'a': 'hello'},
                    }
                ],
            ),
        ),
    ],
)
def test_dataclass(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), positional=True),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), positional=True),
    )
    v = py_and_json(schema)
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)

        # debug(exc_info.value.errors())
        if expected.errors is not None:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        (('hello', True), ({'a': 'hello'}, {'b': True})),
        (['hello', True], ({'a': 'hello'}, {'b': True})),
        (('hello', 'true'), ({'a': 'hello'}, {'b': True})),
        ({'__args__': ('hello', True), '__kwargs__': {}}, ({'a': 'hello'}, {'b': True})),
        ({'__args__': (), '__kwargs__': {'a': 'hello', 'b': True}}, ({'a': 'hello'}, {'b': True})),
        (
            {'__args__': ('hello',), '__kwargs__': {'a': 'hello', 'b': True}},
            Err(
                'Got multiple values for argument',
                errors=[
                    {
                        'type': 'multiple_argument_values',
                        'loc': ('a',),
                        'msg': 'Got multiple values for argument',
                        'input': 'hello',
                    }
                ],
            ),
        ),
        (
            {'a': 'hello'},
            Err(
                'Missing required keyword argument',
                errors=[
                    {
                        'type': 'missing_keyword_argument',
                        'loc': ('b',),
                        'msg': 'Missing required keyword argument',
                        'input': {'a': 'hello'},
                    }
                ],
            ),
        ),
        (
            {'a': 'hello', 'b': 'wrong'},
            Err(
                'Input should be a valid boolean, unable to interpret input',
                errors=[
                    {
                        'type': 'bool_parsing',
                        'loc': ('b',),
                        'msg': 'Input should be a valid boolean, unable to interpret input',
                        'input': 'wrong',
                    }
                ],
            ),
        ),
    ],
)
def test_dataclass_init_only(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema(), positional=True),
        core_schema.dataclass_field(name='b', schema=core_schema.bool_schema(), positional=True, init_only=True),
        collect_init_only=True,
    )
    v = py_and_json(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)

        # debug(exc_info.value.errors())
        if expected.errors is not None:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    'input_value,expected',
    [
        ({'a': 'hello'}, ({'a': 'hello'}, {})),
        ({'__args__': (), '__kwargs__': {'a': 'hello'}}, ({'a': 'hello'}, {})),
        (
            ('hello',),
            Err(
                '2 validation errors for dataclass-args',
                errors=[
                    {
                        'type': 'missing_keyword_argument',
                        'loc': ('a',),
                        'msg': 'Missing required keyword argument',
                        'input': IsListOrTuple('hello'),
                    },
                    {
                        'type': 'unexpected_positional_argument',
                        'loc': (0,),
                        'msg': 'Unexpected positional argument',
                        'input': 'hello',
                    },
                ],
            ),
        ),
    ],
)
def test_dataclass_init_only_no_fields(py_and_json: PyAndJson, input_value, expected):
    schema = core_schema.dataclass_args_schema(
        core_schema.dataclass_field(name='a', schema=core_schema.str_schema()), collect_init_only=True
    )
    v = py_and_json(schema)

    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)) as exc_info:
            v.validate_test(input_value)

        # debug(exc_info.value.errors())
        if expected.errors is not None:
            assert exc_info.value.errors() == expected.errors
    else:
        assert v.validate_test(input_value) == expected
