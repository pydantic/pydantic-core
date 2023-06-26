"""
General benchmarks that attempt to cover all field types, through by no means all uses of all field types.
"""
import json
import sys
from datetime import date, datetime, time
from pathlib import Path
from typing import Annotated, List, Literal, Union

import pytest
from pydantic.networks import EmailStr

from pydantic_core import SchemaValidator, ValidationError

from .complete_schema import input_data_lax, input_data_strict, input_data_wrong, pydantic_model, schema
from .test_micro_benchmarks import skip_pydantic

pytestmark = pytest.mark.skipif(sys.version_info < (3, 10), reason='requires python3.10 or higher')


def test_complete_valid():
    lax_schema = schema()
    cls = lax_schema['cls']
    lax_validator = SchemaValidator(lax_schema)
    output = lax_validator.validate_python(input_data_lax())
    assert isinstance(output, cls)
    assert len(output.__pydantic_fields_set__) == 39
    output_dict = output.__dict__
    assert output_dict == {
        'field_str': 'fo',
        'field_str_con': 'fooba',
        'field_int': 1,
        'field_int_con': 8,
        'field_float': 1.0,
        'field_float_con': 10.0,
        'field_bool': True,
        'field_bytes': b'foobar',
        'field_bytes_con': b'foobar',
        'field_date': date(2010, 2, 3),
        'field_date_con': date(2020, 1, 1),
        'field_time': time(12, 0),
        'field_time_con': time(12, 0),
        'field_datetime': datetime(2020, 1, 1, 12, 13, 14),
        'field_datetime_con': datetime(2020, 1, 1),
        'field_list_any': ['a', b'b', True, 1.0, None] * 10,
        'field_list_str': ['a', 'b', 'c'] * 10,
        'field_list_str_con': ['a', 'b', 'c'] * 10,
        'field_set_any': {'a', b'b', True, 1.0, None},
        'field_set_int': set(range(100)),
        'field_set_int_con': set(range(42)),
        'field_frozenset_any': frozenset({'a', b'b', True, 1.0, None}),
        'field_frozenset_bytes': frozenset([f'{i}'.encode() for i in range(100)]),
        'field_frozenset_bytes_con': frozenset([f'{i}'.encode() for i in range(42)]),
        'field_tuple_var_len_any': ('a', b'b', True, 1.0, None),
        'field_tuple_var_len_float': tuple((i + 0.5 for i in range(100))),
        'field_tuple_var_len_float_con': tuple((i + 0.5 for i in range(42))),
        'field_tuple_fix_len': ('a', 1, 1.0, True),
        'field_dict_any': {'a': 'b', 1: True, 1.0: 1.0},
        'field_dict_str_float': {f'{i}': i + 0.5 for i in range(100)},
        'field_literal_1_int': 1,
        'field_literal_1_str': 'foobar',
        'field_literal_mult_int': 3,
        'field_literal_mult_str': 'foo',
        'field_literal_assorted': 'foo',
        'field_list_nullable_int': [1, None, 2, None, 3, None, 4, None],
        'field_union': {'field_str': 'foo', 'field_int': 1, 'field_float': 1.0},
        'field_functions_model': {
            'field_before': 'foo Changed',
            'field_after': 'foo Changed',
            'field_wrap': 'Input foo Changed',
            'field_plain': 'foo Changed',
        },
        'field_recursive': {
            'name': 'foo',
            'sub_branch': {'name': 'bar', 'sub_branch': {'name': 'baz', 'sub_branch': None}},
        },
    }

    strict_validator = SchemaValidator(schema(strict=True))
    output2 = strict_validator.validate_python(input_data_strict())
    assert output_dict == output2.__dict__

    model = pydantic_model()
    if model is None:
        print('pydantic is not installed, skipping pydantic tests')
        return

    output_pydantic = model.parse_obj(input_data_lax())
    assert output_pydantic.dict() == output_dict


def test_complete_invalid():
    lax_schema = schema()
    lax_validator = SchemaValidator(lax_schema)
    with pytest.raises(ValidationError) as exc_info:
        lax_validator.validate_python(input_data_wrong())
    assert len(exc_info.value.errors(include_url=False)) == 738

    model = pydantic_model()
    if model is None:
        print('pydantic is not installed, skipping pydantic tests')
        return

    from pydantic import ValidationError as PydanticValidationError

    with pytest.raises(PydanticValidationError) as exc_info:
        model.parse_obj(input_data_wrong())
    assert len(exc_info.value.errors()) == 530


@pytest.mark.benchmark(group='complete')
def test_complete_core_lax(benchmark):
    v = SchemaValidator(schema())
    benchmark(v.validate_python, input_data_lax())


@pytest.mark.benchmark(group='complete')
def test_complete_core_strict(benchmark):
    v = SchemaValidator(schema(strict=True))
    benchmark(v.validate_python, input_data_strict())


@skip_pydantic
@pytest.mark.benchmark(group='complete')
def test_complete_pyd(benchmark):
    model = pydantic_model()
    assert model is not None
    benchmark(model.parse_obj, input_data_lax())


@pytest.mark.benchmark(group='complete-wrong')
def test_complete_core_error(benchmark):
    v = SchemaValidator(schema())
    data = input_data_wrong()

    @benchmark
    def f():
        try:
            v.validate_python(data)
        except ValueError:
            pass
        else:
            raise RuntimeError('expected ValueError')


@pytest.mark.benchmark(group='complete-wrong')
def test_complete_core_isinstance(benchmark):
    v = SchemaValidator(schema())
    data = input_data_wrong()
    assert v.isinstance_python(data) is False

    @benchmark
    def f():
        v.isinstance_python(data)


@skip_pydantic
@pytest.mark.benchmark(group='complete-wrong')
def test_complete_pyd_error(benchmark):
    model = pydantic_model()
    assert model is not None
    data = input_data_wrong()

    @benchmark
    def f():
        try:
            model.parse_obj(data)
        except ValueError:
            pass
        else:
            raise RuntimeError('expected ValueError')


def default_json_encoder(obj):
    if isinstance(obj, bytes):
        return obj.decode('utf-8')
    if isinstance(obj, (set, frozenset)):
        return list(obj)
    else:
        raise TypeError(f'Object of type {type(obj)} is not JSON serializable')


@pytest.mark.benchmark(group='complete-json')
def test_complete_core_json(benchmark):
    v = SchemaValidator(schema())
    json_data = json.dumps(input_data_lax(), default=default_json_encoder)
    benchmark(v.validate_json, json_data)


@skip_pydantic
@pytest.mark.benchmark(group='complete-json')
def test_complete_pyd_json(benchmark):
    model = pydantic_model()
    assert model is not None
    json_data = json.dumps(input_data_lax(), default=default_json_encoder)

    @benchmark
    def t():
        model.parse_raw(json_data, content_type='application/json')


@pytest.mark.benchmark(group='build')
def test_build_schema(benchmark):
    lax_schema = schema()
    benchmark(SchemaValidator, lax_schema)


@pytest.fixture(scope='module')
def pydantic_v2_type_adapter():
    from pydantic import BaseModel, Field, TypeAdapter
    from pydantic.networks import AnyHttpUrl

    class Blog(BaseModel):
        type: Literal['blog']
        title: str
        post_count: int
        readers: int
        avg_post_rating: float
        url: AnyHttpUrl

    class SocialProfileBase(BaseModel):
        type: Literal['profile']
        network: Literal['facebook', 'twitter', 'linkedin']
        username: str
        join_date: date

    class FacebookProfile(SocialProfileBase):
        network: Literal['facebook']
        friends: int

    class TwitterProfile(SocialProfileBase):
        network: Literal['twitter']
        followers: int

    class LinkedinProfile(SocialProfileBase):
        network: Literal['linkedin']
        connections: Annotated[int, Field(le=500)]

    SocialProfile = Annotated[Union[FacebookProfile, TwitterProfile, LinkedinProfile], Field(discriminator='network')]

    Website = Annotated[Union[Blog, SocialProfile], Field(discriminator='type')]

    class Person(BaseModel):
        name: str
        email: EmailStr
        entry_created_date: date
        entry_created_time: time
        entry_updated_at: datetime
        websites: List[Website] = Field(default_factory=list)

    return TypeAdapter(List[Person])


@pytest.fixture(scope='module')
def sample_data_bytes():
    sample_data_path = Path(__file__).parent / 'sample_data.json'
    return sample_data_path.read_bytes()


def test_north_star_validate_json(pydantic_v2_type_adapter, sample_data_bytes, benchmark):
    benchmark(pydantic_v2_type_adapter.validate_json, sample_data_bytes)


def test_north_star_validate_json_strict(pydantic_v2_type_adapter, sample_data_bytes, benchmark):
    coerced_sample_data = pydantic_v2_type_adapter.dump_json(pydantic_v2_type_adapter.validate_json(sample_data_bytes))
    benchmark(pydantic_v2_type_adapter.validate_json, coerced_sample_data, strict=True)


def test_north_star_dump_json(pydantic_v2_type_adapter, sample_data_bytes, benchmark):
    parsed = pydantic_v2_type_adapter.validate_json(sample_data_bytes)
    benchmark(pydantic_v2_type_adapter.dump_json, parsed)


def test_north_star_validate_python(pydantic_v2_type_adapter, sample_data_bytes, benchmark):
    benchmark(pydantic_v2_type_adapter.validate_python, json.loads(sample_data_bytes))


def test_north_star_validate_python_strict(pydantic_v2_type_adapter, sample_data_bytes, benchmark):
    coerced_sample_data = pydantic_v2_type_adapter.dump_python(
        pydantic_v2_type_adapter.validate_json(sample_data_bytes)
    )
    benchmark(pydantic_v2_type_adapter.validate_python, coerced_sample_data, strict=True)


def test_north_star_dump_python(pydantic_v2_type_adapter, sample_data_bytes, benchmark):
    parsed = pydantic_v2_type_adapter.validate_python(json.loads(sample_data_bytes))
    benchmark(pydantic_v2_type_adapter.dump_python, parsed)


def test_north_star_json_loads(sample_data_bytes, benchmark):
    benchmark(json.loads, sample_data_bytes)


def test_north_star_json_dumps(sample_data_bytes, benchmark):
    parsed = json.loads(sample_data_bytes)
    benchmark(json.dumps, parsed)
