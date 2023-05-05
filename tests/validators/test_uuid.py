from pydantic_core import Uuid, core_schema

from ..conftest import PyAndJson


def test_uuid_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.uuid_schema())
    uuid = v.validate_test('12345678-1234-5678-1234-567812345678')

    assert isinstance(uuid, Uuid)
    assert str(uuid) == '12345678-1234-5678-1234-567812345678'
    assert repr(uuid) == "Uuid('12345678-1234-5678-1234-567812345678')"
    assert uuid.version == 4
    assert uuid.urn == 'foo'
    assert uuid.variant == 'bar'


def test_uuid_from_constructor_ok():
    uuid = Uuid('12345678-1234-5678-1234-567812345678')

    assert isinstance(uuid, Uuid)
    assert str(uuid) == '12345678-1234-5678-1234-567812345678'
    assert repr(uuid) == "Uuid('12345678-1234-5678-1234-567812345678')"
    assert uuid.version == 4
    assert uuid.urn == 'foo'
    assert uuid.variant == 'bar'
