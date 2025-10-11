"""Test script to demonstrate Fraction and Decimal serialization in Pydantic."""

from decimal import Decimal
from fractions import Fraction
from pydantic import BaseModel


class MyModel(BaseModel):
    fraction_field: Fraction
    decimal_field: Decimal


# Create instance
model = MyModel(
    fraction_field=Fraction(3, 4),
    decimal_field=Decimal("3.14159")
)

print("=" * 60)
print("Python mode serialization (mode='python'):")
python_serialized = model.model_dump(mode='python')
print(f"  Result: {python_serialized}")
print()

print("=" * 60)
print("JSON mode serialization (mode='json'):")
json_serialized = model.model_dump(mode='json')
print(f"  Result: {json_serialized}")
print(f"  fraction_field: {json_serialized['fraction_field']} (type: {type(json_serialized['fraction_field']).__name__})")
print(f"  decimal_field: {json_serialized['decimal_field']} (type: {type(json_serialized['decimal_field']).__name__})")
print()

print("=" * 60)
print("JSON string serialization (model_dump_json()):")
json_string = model.model_dump_json()
print(f"  Result: {json_string}")
print()

print("=" * 60)
print("Expected behavior:")
print("  Python mode:")
print("    - Fraction should be serialized as Fraction object (like Decimal)")
print("    - Decimal should be serialized as Decimal object")
print("  JSON mode:")
print("    - Both should be serialized as strings (or numbers)")
print()
