from typing import Any, Dict, Union, Optional
from typing_extensions import TypedDict
from dataclasses import dataclass
from pydantic_core import core_schema as cs


@dataclass
class ASTClass:
    name: str


class CoreConfig(TypedDict, total=False):
    """Schema configuration options matching pydantic_core's CoreConfig"""

    title: str
    strict: bool
    extra_fields_behavior: str  # 'allow', 'forbid', 'ignore'
    typed_dict_total: bool
    from_attributes: bool
    loc_by_alias: bool
    revalidate_instances: str  # 'always', 'never', 'subclass-instances'
    validate_default: bool
    populate_by_name: bool
    str_max_length: int
    str_min_length: int
    str_strip_whitespace: bool
    str_to_lower: bool
    str_to_upper: bool
    allow_inf_nan: bool
    ser_json_timedelta: str  # 'iso8601', 'float'
    ser_json_bytes: str  # 'utf8', 'base64', 'hex'
    ser_json_inf_nan: str  # 'null', 'constants', 'strings'
    val_json_bytes: str  # 'utf8', 'base64', 'hex'
    hide_input_in_errors: bool
    validation_error_cause: bool
    coerce_numbers_to_str: bool
    regex_engine: str  # 'rust-regex', 'python-re'
    cache_strings: Union[bool, str]  # bool or 'all', 'keys', 'none'


SCHEMA_TYPE_TO_FUNCTION = {
    'str': 'str_schema',
    'int': 'int_schema',
    'float': 'float_schema',
    'bool': 'bool_schema',
    'dict': 'dict_schema',
    'list': 'list_schema',
    'set': 'set_schema',
    'tuple': 'tuple_schema',
    'frozenset': 'frozenset_schema',
    'any': 'any_schema',
    'none': 'none_schema',
    'model': 'model_schema',
    'model-field': 'model_field',
    'model-fields': 'model_fields_schema',
    'typed-dict': 'typed_dict_schema',
    'typed-dict-field': 'typed_dict_field',
    'union': 'union_schema',
    'nullable': 'nullable_schema',
    'function-before': 'before_validator_function',
    'function-after': 'after_validator_function',
    'function-wrap': 'wrap_validator_function',
    'function-plain': 'plain_validator_function',
}

# Parameters that require special handling
SPECIAL_PARAMS = {'ref', 'metadata', 'serialization', 'type'}


def format_value(value: Any) -> str:
    """Format a value for inclusion in a function call."""
    if isinstance(value, str):
        return f"'{value}'"
    elif isinstance(value, ASTClass):
        return value.name
    elif isinstance(value, dict):
        return dict_to_schema_call(value)
    elif isinstance(value, list):
        items = [format_value(item) for item in value]
        return f'[{", ".join(items)}]'
    elif isinstance(value, bool):
        return str(value)
    elif value is None:
        return 'None'
    return str(value)


def format_kwarg(key: str, value: Any) -> str:
    """Format a keyword argument for a function call."""
    formatted_value = format_value(value)
    return f'{key}={formatted_value}'


def dict_to_schema_call(schema_dict: Dict[str, Any]) -> str:
    """Convert a schema dictionary to a CoreSchema function call."""
    if not isinstance(schema_dict, dict):
        return format_value(schema_dict)

    schema_type = schema_dict.get('type')
    if not schema_type:
        # Handle dict literals that aren't schema definitions
        items = [f"'{k}': {format_value(v)}" for k, v in schema_dict.items()]
        return f'{{{", ".join(items)}}}'

    function_name = SCHEMA_TYPE_TO_FUNCTION.get(schema_type)
    if not function_name:
        raise ValueError(f'Unknown schema type: {schema_type}')

    # Collect kwargs
    kwargs = []
    for key, value in schema_dict.items():
        if key in SPECIAL_PARAMS:
            continue

        # Handle special cases
        if key == 'config':
            config = schema_dict.get('config', {})
            if isinstance(config, ASTClass) and config.name == 'config':
                kwargs.append(f'config={config.name}')
                continue
            if not config or not isinstance(config, dict):
                continue
            kwargs.append(f'config={config_to_CoreConfig(config)}')
        elif key == 'schema' and isinstance(value, dict):
            kwargs.append(f'schema={dict_to_schema_call(value)}')
        elif key == 'fields' and isinstance(value, dict):
            # Handle fields mapping for typed-dict and model schemas
            field_items = []
            for field_name, field_schema in value.items():
                formatted_field = dict_to_schema_call(field_schema)
                field_items.append(f"'{field_name}': {formatted_field}")
            kwargs.append(f'fields={{{", ".join(field_items)}}}')
        else:
            kwargs.append(format_kwarg(key, value))

    # Add reference if present
    if 'ref' in schema_dict:
        kwargs.append(format_kwarg('ref', schema_dict['ref']))

    # Add metadata if present
    if 'metadata' in schema_dict:
        kwargs.append(format_kwarg('metadata', schema_dict['metadata']))

    # Add serialization if present
    if 'serialization' in schema_dict:
        kwargs.append(format_kwarg('serialization', schema_dict['serialization']))

    return f'cs.{function_name}({", ".join(kwargs)})'


def config_to_CoreConfig(config: Dict[str, Any] | ASTClass) -> str:
    """Convert a config dictionary to a CoreConfig TypedDict instantiation."""
    if isinstance(config, ASTClass):
        return config.name
    config_items = []
    for key, value in config.items():
        if isinstance(value, str):
            config_items.append(f"{key}='{value}'")
        else:
            config_items.append(f'{key}={value}')
    return f'CoreConfig({", ".join(config_items)})'


def convert_schema(schema: dict[str, Any]|ASTClass) -> str:
    """Convert a schema dictionary to a CoreSchema function call with proper formatting."""
    if isinstance(schema, ASTClass):
        return schema.name
    result = dict_to_schema_call(schema)

    return result
