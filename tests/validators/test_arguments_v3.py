import pytest

from pydantic_core import ArgsKwargs, ValidationError
from pydantic_core import core_schema as cs

from ..conftest import PyAndJson


@pytest.mark.parametrize(
    ['input_value', 'expected'],
    (
        [ArgsKwargs((1, True), {}), ((1, True), {})],
        [ArgsKwargs((1,), {}), ((1,), {})],
        [{'a': 1, 'b': True}, ((1, True), {})],
        [{'a': 1}, ((1,), {})],
    ),
)
def test_positional_only(py_and_json: PyAndJson, input_value, expected) -> None:
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(name='a', schema=cs.int_schema(), mode='positional_only'),
                cs.arguments_v3_parameter(
                    name='b', schema=cs.with_default_schema(cs.bool_schema()), mode='positional_only'
                ),
            ]
        )
    )

    assert v.validate_test(input_value) == expected


def test_positional_only_validation_error(py_and_json: PyAndJson) -> None:
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(name='a', schema=cs.int_schema(), mode='positional_only'),
            ]
        )
    )

    with pytest.raises(ValidationError) as exc_info:
        v.validate_test(ArgsKwargs(('not_an_int',), {}))

    error = exc_info.value.errors()[0]

    assert error['type'] == 'int_parsing'
    assert error['loc'] == (0,)

    with pytest.raises(ValidationError) as exc_info:
        v.validate_test({'a': 'not_an_int'})

    error = exc_info.value.errors()[0]

    assert error['type'] == 'int_parsing'
    assert error['loc'] == ('a',)


def test_positional_only_error_required(py_and_json: PyAndJson) -> None:
    v = py_and_json(
        cs.arguments_v3_schema(
            [
                cs.arguments_v3_parameter(name='a', schema=cs.int_schema(), mode='positional_only'),
            ]
        )
    )

    with pytest.raises(ValidationError) as exc_info:
        v.validate_test(ArgsKwargs(tuple(), {}))

    error = exc_info.value.errors()[0]

    assert error['type'] == 'missing_positional_only_argument'
    assert error['loc'] == (0,)

    with pytest.raises(ValidationError) as exc_info:
        v.validate_test({})

    error = exc_info.value.errors()[0]

    assert error['type'] == 'missing_positional_only_argument'
    assert error['loc'] == ('a',)
