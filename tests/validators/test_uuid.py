from copy import deepcopy
from typing import Dict

import pytest

from pydantic_core import SchemaValidator, Uuid, ValidationError, core_schema

from ..conftest import Err, PyAndJson


def test_uuid_ok(py_and_json: PyAndJson):
    v = py_and_json(core_schema.uuid_schema())
    uuid = v.validate_test('12345678-1234-5678-1234-567812345678')

    assert isinstance(uuid, Uuid)
    assert str(uuid) == '12345678-1234-5678-1234-567812345678'
    assert repr(uuid) == "Uuid('12345678-1234-5678-1234-567812345678')"
    assert uuid.urn == 'urn:uuid:12345678-1234-5678-1234-567812345678'
    assert uuid.version == 5
    assert uuid.variant == 'NCS'


def test_uuid_from_constructor_ok():
    uuid = Uuid('12345678-1234-5678-1234-567812345678')

    assert isinstance(uuid, Uuid)
    assert str(uuid) == '12345678-1234-5678-1234-567812345678'
    assert repr(uuid) == "Uuid('12345678-1234-5678-1234-567812345678')"
    assert uuid.urn == 'urn:uuid:12345678-1234-5678-1234-567812345678'
    assert uuid.version == 5
    assert uuid.variant == 'NCS'


@pytest.fixture(scope='module', name='uuid_validator')
def uuid_validator_fixture():
    return SchemaValidator(core_schema.uuid_schema())


ORDERED_TEST_CASES = [
    ('00000000-0000-0000-0000-000000000000', 'urn:uuid:00000000-0000-0000-0000-000000000000', 'NCS', 0),
    ('00010203-0405-0607-0809-0a0b0c0d0e0f', 'urn:uuid:00010203-0405-0607-0809-0a0b0c0d0e0f', 'NCS', 0),
    ('02d9e6d5-9467-382e-8f9b-9300a64ac3cd', 'urn:uuid:02d9e6d5-9467-382e-8f9b-9300a64ac3cd', 'RFC4122', 3),
    ('6ba7b810-9dad-11d1-80b4-00c04fd430c8', 'urn:uuid:6ba7b810-9dad-11d1-80b4-00c04fd430c8', 'RFC4122', 1),
    ('6ba7b811-9dad-11d1-80b4-00c04fd430c8', 'urn:uuid:6ba7b811-9dad-11d1-80b4-00c04fd430c8', 'RFC4122', 1),
    ('6ba7b812-9dad-11d1-80b4-00c04fd430c8', 'urn:uuid:6ba7b812-9dad-11d1-80b4-00c04fd430c8', 'RFC4122', 1),
    ('6ba7b814-9dad-11d1-80b4-00c04fd430c8', 'urn:uuid:6ba7b814-9dad-11d1-80b4-00c04fd430c8', 'RFC4122', 1),
    ('7d444840-9dc0-11d1-b245-5ffdce74fad2', 'urn:uuid:7d444840-9dc0-11d1-b245-5ffdce74fad2', 'RFC4122', 1),
    ('e902893a-9d22-3c7e-a7b8-d6e313b71d9f', 'urn:uuid:e902893a-9d22-3c7e-a7b8-d6e313b71d9f', 'RFC4122', 3),
    ('eb424026-6f54-4ef8-a4d0-bb658a1fc6cf', 'urn:uuid:eb424026-6f54-4ef8-a4d0-bb658a1fc6cf', 'RFC4122', 4),
    ('f81d4fae-7dec-11d0-a765-00a0c91e6bf6', 'urn:uuid:f81d4fae-7dec-11d0-a765-00a0c91e6bf6', 'RFC4122', 1),
]


@pytest.mark.parametrize('uuid,urn,variant,version', ORDERED_TEST_CASES)
def test_uuid_cases_ok(uuid_validator, uuid, urn, variant, version):
    output_uuid = uuid_validator.validate_python(uuid)
    assert isinstance(output_uuid, Uuid)
    assert str(output_uuid) == uuid
    assert output_uuid.urn == urn
    assert output_uuid.version == version
    assert output_uuid.variant == variant


TEST_CASES_ERR = [
    ('', Err('invalid length: expected length 32 for simple format, found 0')),
    ('abc', Err('invalid length: expected length 32 for simple format, found 3')),
    ('1234567812345678123456781234567', Err('invalid length: expected length 32 for simple format, found 31')),
    ('123456781234567812345678123456789', Err('invalid length: expected length 32 for simple format, found 33')),
    (
        '123456781234567812345678z2345678',
        Err('invalid character: expected an optional prefix of `urn:uuid:` followed by [0-9a-fA-F-], found `z` at 25'),
    ),
]


@pytest.mark.parametrize('uuid,expected', TEST_CASES_ERR)
def test_uuid_error(uuid_validator, uuid, expected):
    with pytest.raises(ValidationError) as exc_info:
        uuid_validator.validate_python(uuid)
    assert exc_info.value.error_count() == 1
    error = exc_info.value.errors(include_url=False)[0]
    assert error['type'] == 'uuid_parsing'
    assert error['ctx']['error'] == expected.message


def test_uuid_comparison():
    ascending = []
    for uuid, _, _, _ in ORDERED_TEST_CASES:
        ascending.append(Uuid(uuid))

    for i in range(len(ascending)):
        for j in range(len(ascending)):
            assert (i < j) == (ascending[i] < ascending[j])
            assert (i <= j) == (ascending[i] <= ascending[j])
            assert (i == j) == (ascending[i] == ascending[j])
            assert (i > j) == (ascending[i] > ascending[j])
            assert (i >= j) == (ascending[i] >= ascending[j])
            assert (i != j) == (ascending[i] != ascending[j])


def test_uuid_hash():
    data: Dict[Uuid, int] = {}

    data[Uuid('12345678-1234-5678-1234-567812345678')] = 1
    assert data == {Uuid('12345678-1234-5678-1234-567812345678'): 1}

    data[Uuid('00010203-0405-0607-0809-0a0b0c0d0e0f')] = 2
    assert data == {Uuid('12345678-1234-5678-1234-567812345678'): 1, Uuid('00010203-0405-0607-0809-0a0b0c0d0e0f'): 2}

    data[Uuid('00010203-0405-0607-0809-0a0b0c0d0e0f')] = 3
    assert data == {Uuid('12345678-1234-5678-1234-567812345678'): 1, Uuid('00010203-0405-0607-0809-0a0b0c0d0e0f'): 3}


def test_uuid_bool():
    assert bool(Uuid('12345678-1234-5678-1234-567812345678')) is True


def test_uuid_deepcopy():
    assert deepcopy(Uuid('12345678-1234-5678-1234-567812345678')) == Uuid('12345678-1234-5678-1234-567812345678')
