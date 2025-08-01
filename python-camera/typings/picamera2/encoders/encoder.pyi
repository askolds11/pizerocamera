"""
This type stub file was generated by pyright.
"""

from enum import Enum
from typing import Any

from picamera2.outputs import Output

"""Encoder functionality"""
class Quality(Enum):
    """Enum type to describe the quality wanted from an encoder.

    This may be passed if a specific value (such as bitrate) has not been set.
    """
    VERY_LOW = ...
    LOW = ...
    MEDIUM = ...
    HIGH = ...
    VERY_HIGH = ...


class Encoder:
    """
    Base class for encoders.

    Mostly this defines the API for derived encoder classes, but it also handles optional audio encoding.
    For audio, a separate thread is started, which encodes audio packets and forwards them to the
    encoder's output object(s). This only work when the output object understands the audio stream,
    meaning that (at the time of writing) this must be a PyavOutput (though you could send output there
    via a CircularOutput2).

    Additional audio parameters:
    audio - set to True to enable audio encoding and output.
    audio_input - list of parameters that is passed to PyAv.open to create the audio input.
    audio_output - list of parameters passed to PyAv add_stream to define the audio codec and output stream.
    audio_sync - value (in us) by which to advance the audio stream to better sync with the video.

    Reasonable defaults are supplied so that applications can often just set the audio property to True.
    The audio_input and audio_output parameters are passed directly to PyAV, so will accept whatever PyAV
    understands.
    """
    def __init__(self) -> None:
        """Initialises encoder"""
        ...
    
    @property
    def running(self) -> bool:
        ...
    
    @property
    def width(self) -> int:
        """Gets width

        :return: Width of frames
        :rtype: int
        """
        ...
    
    @width.setter
    def width(self, value) -> None:
        """Sets width

        :param value: Width
        :type value: int
        :raises RuntimeError: Failed to set width
        """
        ...
    
    @property
    def height(self) -> int:
        """Gets height

        :return: Height of frames
        :rtype: int
        """
        ...
    
    @height.setter
    def height(self, value) -> None:
        """Sets height

        :param value: Height
        :type value: int
        :raises RuntimeError: Failed to set height
        """
        ...
    
    @property
    def size(self) -> tuple[int, int]:
        """Gets size

        :return: Size of frames as (width, height)
        :rtype: tuple
        """
        ...
    
    @size.setter
    def size(self, value) -> None:
        """Sets size

        :param value: Size
        :type value: tuple
        :raises RuntimeError: Failed to set size
        """
        ...
    
    @property
    def stride(self) -> int:
        """Gets stride

        :return: Stride
        :rtype: int
        """
        ...
    
    @stride.setter
    def stride(self, value) -> None:
        """Sets stride

        :param value: Stride
        :type value: int
        :raises RuntimeError: Failed to set stride
        """
        ...
    
    @property
    def format(self) -> None:
        """Get current format

        :return: Current format
        :rtype: int
        """
        ...
    
    @format.setter
    def format(self, value) -> None:
        """Sets input format to encoder

        :param value: Format
        :type value: str
        :raises RuntimeError: Invalid format
        """
        ...
    
    @property
    def output(self) -> Output | list[Any] | list[Output]:
        """Gets output objects

        :return: Output object list or single Output object
        :rtype: List[Output]
        """
        ...
    
    @output.setter
    def output(self, value) -> None:
        """Sets output object, to write frames to

        :param value: Output object
        :type value: Output
        :raises RuntimeError: Invalid output passed
        """
        ...
    
    @property
    def name(self) -> str | None:
        """Gets stream name

        :return: Name
        :rtype: str
        """
        ...
    
    @name.setter
    def name(self, value) -> None:
        """Sets stream name

        :param value: Name
        :type value: str
        :raises RuntimeError: Failed to set name
        """
        ...
    
    def encode(self, stream, request) -> None:
        """Encode a frame

        :param stream: Stream
        :type stream: stream
        :param request: Request
        :type request: request
        """
        ...
    
    def start(self, quality=...) -> None:
        ...
    
    def stop(self) -> None:
        ...
    
    def outputframe(self, frame, keyframe=..., timestamp=..., packet=..., audio=...) -> None:
        """Writes a frame

        :param frame: Frame
        :type frame: bytes
        :param keyframe: Whether frame is a keyframe or not, defaults to True
        :type keyframe: bool, optional
        """
        ...
    


