from decimal import Decimal

from pydantic_core import SchemaValidator


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


def test_chain2():
    validator = SchemaValidator(
        {
            'type': 'chain2',
            'schema1': {'type': 'str'},
            'schema2': {'type': 'function', 'mode': 'plain', 'function': lambda v, **kwargs: Decimal(v)},
        }
    )

    assert validator.validate_python('1.44') == Decimal('1.44')
    assert validator.validate_python(b'1.44') == Decimal('1.44')
