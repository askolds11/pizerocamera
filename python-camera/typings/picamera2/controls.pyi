"""
This type stub file was generated by pyright.
"""
from typing import Any, Self


class Controls:
    _VIRTUAL_FIELDS_MAP_ = ...
    def __init__(self, picam2, controls=...) -> None:
        ...
    
    def __setattr__(self, name, value) -> None:
        ...
    
    def __getattribute__(self, name) -> tuple[Any, Any] | Any:
        ...
    
    def __repr__(self) -> str:
        ...
    
    def __enter__(self) -> Self:
        ...
    
    def __exit__(self, exc_type, exc_value, tb) -> None:
        ...
    
    def set_controls(self, controls) -> None:
        ...
    
    def get_libcamera_controls(self) -> dict[Any, Any]:
        ...
    
    def make_dict(self) -> dict[Any, Any]:
        ...
    


