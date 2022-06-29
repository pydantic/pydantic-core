import os
from datetime import datetime, timezone
from typing import Optional, TypedDict

import pytest
from hypothesis import given, settings, strategies

from pydantic_core import SchemaValidator, ValidationError

settings_dict = {}
if hyp_max_examples := os.getenv('HYPOTHESIS_MAX_EXAMPLES'):
    settings_dict['max_examples'] = int(hyp_max_examples)
    print(f'hypothesis settings: {settings_dict}')


@pytest.fixture(scope='module')
def datetime_schema():
    return SchemaValidator({'type': 'datetime'})


@settings(**settings_dict)
@given(strategies.datetimes())
def test_datetime_datetime(datetime_schema, data):
    assert datetime_schema.validate_python(data) == data


@settings(**settings_dict)
@given(strategies.integers(min_value=-11_676_096_000, max_value=253_402_300_799_000))
def test_datetime_int(datetime_schema, data):
    if abs(data) > 20_000_000_000:
        microsecond = (data % 1000) * 1000
        expected = datetime.fromtimestamp(data // 1000, tz=timezone.utc).replace(tzinfo=None, microsecond=microsecond)
    else:
        expected = datetime.fromtimestamp(data, tz=timezone.utc).replace(tzinfo=None)

    assert datetime_schema.validate_python(data) == expected, data


@settings(**settings_dict)
@given(strategies.binary())
def test_datetime_binary(datetime_schema, data):
    try:
        datetime_schema.validate_python(data)
    except ValidationError:
        # that's fine
        pass


@pytest.fixture(scope='module')
def recursive_schema():
    return SchemaValidator(
        {
            'type': 'typed-dict',
            'ref': 'Branch',
            'fields': {
                'name': {'schema': {'type': 'str'}},
                'sub_branch': {
                    'schema': {'type': 'nullable', 'schema': {'type': 'recursive-ref', 'schema_ref': 'Branch'}},
                    'default': None,
                },
            },
        }
    )


def test_recursive_simple(recursive_schema):
    assert recursive_schema.validate_python({'name': 'root'}) == {'name': 'root', 'sub_branch': None}


class BranchModel(TypedDict):
    name: str
    sub_branch: Optional['BranchModel']


@settings(**settings_dict)
@given(strategies.from_type(BranchModel))
def test_recursive_hyp(recursive_schema, data):
    assert recursive_schema.validate_python(data) == data


@pytest.mark.skip(reason='recursion not yet detected, see #134, python/pytest currently crash with Segmentation fault')
def test_recursive_broken(recursive_schema):
    data = {'name': 'x'}
    data['sub_branch'] = data
    with pytest.raises(ValidationError):
        recursive_schema.validate_python(data)
