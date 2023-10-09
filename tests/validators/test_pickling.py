import dataclasses
import pickle
import re
from datetime import datetime, timedelta, timezone

import pytest

from pydantic_core import core_schema
from pydantic_core._pydantic_core import SchemaValidator, ValidationError

from ..conftest import PyAndJson


def test_basic_schema_validator(py_and_json: PyAndJson):
    v = py_and_json({'type': 'dict', 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    v = pickle.loads(pickle.dumps(v))
    assert v.validate_test({'1': 2, '3': 4}) == {1: 2, 3: 4}

    v = py_and_json({'type': 'dict', 'strict': True, 'keys_schema': {'type': 'int'}, 'values_schema': {'type': 'int'}})
    v = pickle.loads(pickle.dumps(v))
    assert v.validate_test({'1': 2, '3': 4}) == {1: 2, 3: 4}
    assert v.validate_test({}) == {}
    with pytest.raises(ValidationError, match=re.escape('[type=dict_type, input_value=[], input_type=list]')):
        v.validate_test([])


def test_schema_validator_containing_config():
    """
    Verify that the config object is not lost during (de)serialization.
    """

    @dataclasses.dataclass
    class MyModel:
        f: str

    v = SchemaValidator(
        core_schema.dataclass_schema(
            MyModel,
            core_schema.dataclass_args_schema('MyModel', [core_schema.dataclass_field('f', core_schema.str_schema())]),
            ['f'],
            config=core_schema.CoreConfig(extra_fields_behavior='allow'),
        )
    )

    m: MyModel = v.validate_python({'f': 'x', 'extra_field': '123'})
    assert m.f == 'x'
    assert getattr(m, 'extra_field') == '123'

    # If the config was lost during (de)serialization, the validation call below would
    # fail due to the `extra_field`.
    v = pickle.loads(pickle.dumps(v))
    m: MyModel = v.validate_python({'f': 'x', 'extra_field': '123'})
    assert m.f == 'x'
    assert getattr(m, 'extra_field') == '123'


def test_schema_validator_tz_pickle() -> None:
    """
    https://github.com/pydantic/pydantic-core/issues/589
    """
    v = SchemaValidator(core_schema.datetime_schema())
    original = datetime(2022, 6, 8, 12, 13, 14, tzinfo=timezone(timedelta(hours=-12, minutes=-15)))
    validated = v.validate_python('2022-06-08T12:13:14-12:15')
    assert validated == original
    assert pickle.loads(pickle.dumps(validated)) == validated == original
