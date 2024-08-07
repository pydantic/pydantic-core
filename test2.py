from pydantic_core import core_schema as cs, SchemaValidator


generic_schema = cs.definitions_schema(
    cs.definition_reference_schema('T'),
    [
        cs.int_schema(ref='T')
    ]
)

v = SchemaValidator(generic_schema)
assert v.validate_python('1') == 1

concrete_schema = cs.definitions_schema(
    generic_schema,
    [
        cs.str_schema(ref='T')
    ]
)

v = SchemaValidator(concrete_schema)

assert v.validate_python('1.2') == '1.2'
