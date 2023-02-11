import pytest

from pydantic_core import SchemaValidator


@pytest.mark.parametrize(
    'input_value,expected',
    [
        # TODO: fix tests
        pytest.param('abc def <abcdef@gmail.com>', 'abcdef@gmail.com', id='test_1')
    ],
)
def test_date(input_value, expected):
    v = SchemaValidator({'type': 'name-email'})
    output = v.validate_python(input_value)
    assert output == expected
    assert v.isinstance_python(input_value) is True
