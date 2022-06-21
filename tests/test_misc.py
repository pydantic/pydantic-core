import pytest

from pydantic_core._pydantic_core import SchemaError, SchemaValidator, ValidationError, __version__


@pytest.mark.parametrize('obj', [ValidationError, SchemaValidator, SchemaError])
def test_module(obj):
    assert obj.__module__ == 'pydantic_core._pydantic_core'


def test_version():
    assert isinstance(__version__, str)
    assert '.' in __version__


def test_schema_error():
    err = SchemaError('test')
    assert isinstance(err, Exception)
    assert str(err) == 'test'
    assert repr(err) == 'SchemaError("test")'
