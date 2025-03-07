import re

import pytest

from pydantic_core import ArgsKwargs, ValidationError
from pydantic_core import core_schema as cs

from ...conftest import Err, PyAndJson


@pytest.mark.parametrize(
    ['input_value', 'expected'],
    (
        [ArgsKwargs((1,)), ((1,), {})],
        [ArgsKwargs((), {'Foo': 1}), ((), {'a': 1})],
        [ArgsKwargs((), {'a': 1}), Err('Foo\n  Missing required argument [type=missing_argument,')],
        [{'Foo': 1}, ((1,), {})],
        [{'a': 1}, Err('Foo\n  Missing required argument [type=missing_argument,')],
    ),
    ids=repr,
)
def test_alias(py_and_json: PyAndJson, input_value, expected) -> None:
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(name='a', schema=cs.int_schema(), alias='Foo', mode='positional_or_keyword'),
            ]
        )
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        assert v.validate_test(input_value) == expected


@pytest.mark.parametrize(
    ['input_value', 'expected'],
    (
        [ArgsKwargs((1,)), ((1,), {})],
        [ArgsKwargs((), {'Foo': 1}), ((), {'a': 1})],
        [ArgsKwargs((), {'a': 1}), ((), {'a': 1})],
        [ArgsKwargs((), {'a': 1, 'b': 2}), Err('b\n  Unexpected keyword argument [type=unexpected_keyword_argument,')],
        [
            ArgsKwargs((), {'a': 1, 'Foo': 2}),
            Err('a\n  Unexpected keyword argument [type=unexpected_keyword_argument,'),
        ],
        [{'Foo': 1}, ((1,), {})],
        [{'a': 1}, ((1,), {})],
    ),
    ids=repr,
)
def test_alias_validate_by_name(py_and_json: PyAndJson, input_value, expected):
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(name='a', schema=cs.int_schema(), alias='Foo', mode='positional_or_keyword'),
            ],
            validate_by_name=True,
        )
    )
    if isinstance(expected, Err):
        with pytest.raises(ValidationError, match=re.escape(expected.message)):
            v.validate_test(input_value)
    else:
        assert v.validate_test(input_value) == expected


def test_only_validate_by_name(py_and_json) -> None:
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(
                    name='a', schema=cs.str_schema(), alias='FieldA', mode='positional_or_keyword'
                ),
            ],
            validate_by_name=True,
            validate_by_alias=False,
        )
    )

    assert v.validate_test(ArgsKwargs((), {'a': 'hello'})) == ((), {'a': 'hello'})
    assert v.validate_test({'a': 'hello'}) == (('hello',), {})

    with pytest.raises(ValidationError, match=r'a\n +Missing required argument \[type=missing_argument,'):
        assert v.validate_test(ArgsKwargs((), {'FieldA': 'hello'}))
    with pytest.raises(ValidationError, match=r'a\n +Missing required argument \[type=missing_argument,'):
        assert v.validate_test({'FieldA': 'hello'})


def test_only_allow_alias(py_and_json) -> None:
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(
                    name='a', schema=cs.str_schema(), alias='FieldA', mode='positional_or_keyword'
                ),
            ],
            validate_by_name=False,
            validate_by_alias=True,
        )
    )
    assert v.validate_test(ArgsKwargs((), {'FieldA': 'hello'})) == ((), {'a': 'hello'})
    assert v.validate_test({'FieldA': 'hello'}) == (('hello',), {})

    with pytest.raises(ValidationError, match=r'FieldA\n +Missing required argument \[type=missing_argument,'):
        assert v.validate_test(ArgsKwargs((), {'a': 'hello'}))
    with pytest.raises(ValidationError, match=r'FieldA\n +Missing required argument \[type=missing_argument,'):
        assert v.validate_test({'a': 'hello'})
