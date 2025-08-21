"""
Tests for pattern properties functionality in TypedDict validators.

Pattern properties allow dynamic field validation based on regex patterns or functions,
similar to JSON Schema's patternProperties feature.
"""

import pytest

from pydantic_core import SchemaValidator, ValidationError, core_schema


def test_basic_pattern_properties():
    """Test basic pattern property functionality with regex patterns."""
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            fields={
                'name': core_schema.typed_dict_field(schema=core_schema.str_schema()),
                'age': core_schema.typed_dict_field(schema=core_schema.int_schema()),
            },
            pattern_properties={
                r'^user_\d+$': core_schema.typed_dict_field(schema=core_schema.str_schema()),
                r'^item_\d+$': core_schema.typed_dict_field(schema=core_schema.int_schema()),
            },
            extra_behavior='allow',
        )
    )

    assert v.validate_python(
        {
            'name': 'John',
            'age': 30,
            'user_123': 'alice',
            'user_456': 'bob',
            'item_789': '100',  # should be converted to int
            'extra_field': 'allowed',  # should be valid because extra_behavior='forbid'
        }
    ) == {
        'name': 'John',
        'age': 30,
        'user_123': 'alice',
        'user_456': 'bob',
        'item_789': 100,
        'extra_field': 'allowed',
    }


def test_pattern_validation_errors():
    """Test validation errors for pattern-matched fields."""
    v = SchemaValidator(
        core_schema.typed_dict_schema(
            fields={
                'name': core_schema.typed_dict_field(schema=core_schema.str_schema()),
            },
            pattern_properties={
                r'^item_\d+$': core_schema.typed_dict_field(schema=core_schema.int_schema()),
            },
        )
    )

    with pytest.raises(
        ValidationError, match='1 validation error for typed-dict\nitem_789\n  Input should be a valid integer'
    ):
        v.validate_python(
            {
                'name': 'John',
                'item_789': 'not_a_number',  # should fail int validation
            }
        )


def test_pattern_properties_with_forbid_extra_behavior():
    """Test pattern properties with different extra behaviors."""
    v_forbid = SchemaValidator(
        core_schema.typed_dict_schema(
            fields={
                'name': core_schema.typed_dict_field(schema=core_schema.str_schema()),
            },
            pattern_properties={
                r'^user_\d+$': core_schema.typed_dict_field(schema=core_schema.str_schema()),
            },
            extra_behavior='forbid',
        )
    )

    assert v_forbid.validate_python(
        {
            'name': 'John',
            'user_123': 'alice',
        }
    ) == {'name': 'John', 'user_123': 'alice'}

    with pytest.raises(
        ValidationError, match='1 validation error for typed-dict\ninvalid_key\n  Extra inputs are not permitted'
    ):
        v_forbid.validate_python(
            {
                'name': 'John',
                'user_123': 'alice',
                'invalid_key': 'reject',  # should fail due to extra_behavior='forbid'
            }
        )


def test_function_pattern_basic():
    """Test basic function pattern functionality."""

    def starts_with_api(key: str) -> bool:
        return key.startswith('api_')

    def starts_with_config(key: str) -> bool:
        return key.startswith('config_')

    v = SchemaValidator(
        core_schema.typed_dict_schema(
            fields={
                'name': core_schema.typed_dict_field(schema=core_schema.str_schema()),
            },
            pattern_properties={
                starts_with_api: core_schema.typed_dict_field(schema=core_schema.str_schema()),
                starts_with_config: core_schema.typed_dict_field(schema=core_schema.int_schema()),
            },
        )
    )

    assert v.validate_python(
        {
            'name': 'John',
            'api_token': 'secret123',
            'api_version': 'v2',
            'config_timeout': '30',  # Should be converted to int
            'other_field': 'ignored',  # Should be ignored
        }
    ) == {
        'name': 'John',
        'api_token': 'secret123',
        'api_version': 'v2',
        'config_timeout': 30,
    }


def test_function_pattern_with_errors():
    """Test function pattern error handling."""

    def starts_with_num(key: str) -> bool:
        return key.startswith('num_')

    v = SchemaValidator(
        core_schema.typed_dict_schema(
            fields={},
            pattern_properties={
                starts_with_num: core_schema.typed_dict_field(schema=core_schema.int_schema()),
            },
        )
    )

    assert v.validate_python({'num_items': '42'}) == {'num_items': 42}

    with pytest.raises(
        ValidationError, match='1 validation error for typed-dict\nnum_items\n  Input should be a valid integer'
    ):
        v.validate_python({'num_items': 'not_a_number'})
