from uuid import UUID

from pydantic_core import core_schema

from ..conftest import PyAndJson


def test_uuid_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.uuid_schema())
    uuid = v.validate_test('8aff5d5f-e7e2-4d46-8557-d7eb125ea546')

    assert isinstance(uuid, UUID) is True
    assert uuid.urn == 'urn:uuid:8aff5d5f-e7e2-4d46-8557-d7eb125ea546'
    assert str(uuid) == '8aff5d5f-e7e2-4d46-8557-d7eb125ea546'
    assert 'RFC 4122' in uuid.variant
    assert uuid.version == 4


def test_uuid_from_constructor_ok(py_and_json: PyAndJson):
    original = UUID('12345678-1234-5678-1234-567812345678')
    v = py_and_json(core_schema.uuid_schema())
    uuid = v.validate_test(original)

    assert isinstance(uuid, UUID)
    assert str(uuid) == str(original)
    assert repr(uuid) == repr(original)
    assert uuid.urn == original.urn
    assert uuid.variant == original.variant
    assert uuid.version == original.version
