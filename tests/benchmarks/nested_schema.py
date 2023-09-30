from typing import Any

from pydantic_core import core_schema as cs

N = 5  # arbitrary number that takes ~0.05s per run


def schema() -> cs.CoreSchema:
    class MyModel:
        # __slots__ is not required, but it avoids __pydantic_fields_set__ falling into __dict__
        __slots__ = '__dict__', '__pydantic_fields_set__', '__pydantic_extra__', '__pydantic_private__'

    definitions: list[cs.CoreSchema] = [
        cs.int_schema(ref='int'),
        cs.model_schema(
            MyModel,
            cs.model_fields_schema({str(c): cs.model_field(cs.definition_reference_schema('int')) for c in range(N)}),
            ref=f'model_{N}',
        ),
    ]
    level = N
    for level in reversed(range(N)):
        definitions.append(
            cs.model_schema(
                MyModel,
                cs.model_fields_schema(
                    {str(c): cs.model_field(cs.definition_reference_schema(f'model_{level+1}')) for c in range(N)}
                ),
                ref=f'model_{level}',
            )
        )
    return cs.definitions_schema(cs.definition_reference_schema('model_0'), definitions)


def input_data_valid(levels: int = N) -> Any:
    data = {str(c): 1 for c in range(N)}
    for _ in range(levels):
        data = {str(c): data for c in range(N)}
    return data


if __name__ == '__main__':
    from pydantic_core import SchemaValidator

    v = SchemaValidator(schema())
    v.validate_python(input_data_valid())
