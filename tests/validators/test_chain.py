from decimal import Decimal

import pytest

from pydantic_core import SchemaValidator, ValidationError

from ..conftest import PyAndJson


def test_chain():
    validator = SchemaValidator(
        {
            'type': 'chain',
            'steps': [
                {'type': 'str'},
                {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: Decimal(v)},
            ],
        }
    )

    assert validator.validate_python('1.44') == Decimal('1.44')
    assert validator.validate_python(b'1.44') == Decimal('1.44')


def test_chain_many():
    validator = SchemaValidator(
        {
            'type': 'chain',
            'steps': [
                {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: f'{v}-1'},
                {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: f'{v}-2'},
                {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: f'{v}-3'},
                {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: f'{v}-4'},
            ],
        }
    )

    assert validator.validate_python('input') == 'input-1-2-3-4'


def test_chain_error():
    validator = SchemaValidator({'type': 'chain', 'steps': [{'type': 'str'}, {'type': 'int'}]})

    assert validator.validate_python('123') == 123
    assert validator.validate_python(b'123') == 123

    with pytest.raises(ValidationError) as exc_info:
        validator.validate_python('abc')
    # insert_assert(exc_info.value.errors())
    assert exc_info.value.errors() == [
        {
            'kind': 'int_parsing',
            'loc': [],
            'message': 'Input should be a valid integer, unable to parse string as an integer',
            'input_value': 'abc',
        }
    ]


@pytest.mark.parametrize(
    'input_value,expected', [('1.44', Decimal('1.44')), (1, Decimal(1)), (1.44, pytest.approx(1.44))]
)
def test_json(py_and_json: PyAndJson, input_value, expected):
    validator = py_and_json(
        {
            'type': 'chain',
            'steps': [
                {'type': 'union', 'choices': [{'type': 'str'}, {'type': 'float'}]},
                {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: Decimal(v)},
            ],
        }
    )
    output = validator.validate_test(input_value)
    assert output == expected
    assert isinstance(output, Decimal)
