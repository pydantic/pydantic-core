import pytest

from pydantic_core import SchemaError, SchemaValidator


def test_on_error_default():
    s = SchemaValidator({'type': 'default', 'schema': 'int', 'default': 2, 'on_error': 'default'})
    assert s.validate_python(42) == 42
    assert s.validate_python('42') == 42
    assert s.validate_python('wrong') == 2


def test_on_error_default_not_int():
    s = SchemaValidator({'type': 'default', 'schema': 'int', 'default': [1, 2, 3], 'on_error': 'default'})
    assert s.validate_python(42) == 42
    assert s.validate_python('42') == 42
    a = s.validate_python('wrong')
    assert a == [1, 2, 3]
    # default is not copied, so mutating it mutates the default
    a.append(4)
    assert s.validate_python('wrong') == [1, 2, 3, 4]


def test_on_error_default_factory():
    s = SchemaValidator({'type': 'default', 'schema': 'int', 'default_factory': lambda: 17, 'on_error': 'default'})
    assert s.validate_python(42) == 42
    assert s.validate_python('42') == 42
    assert s.validate_python('wrong') == 17


def test_on_error_omit():
    s = SchemaValidator({'type': 'default', 'schema': 'int', 'on_error': 'omit'})
    assert s.validate_python(42) == 42
    with pytest.raises(ValueError, match='Uncaught Omit error, please check your usage of "default" validators.'):
        s.validate_python('wrong')


def test_on_error_wrong():
    with pytest.raises(SchemaError, match="'on_error = default' requires a `default` or `default_factory`"):
        SchemaValidator({'type': 'default', 'schema': 'int', 'on_error': 'default'})


def test_build_default_and_default_factory():
    with pytest.raises(SchemaError, match="'default' and 'default_factory' cannot be used together"):
        SchemaValidator({'type': 'default', 'schema': 'int', 'default_factory': lambda: 1, 'default': 2})
