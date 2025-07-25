"""
This type stub file was generated by pyright.
"""

"""Null preview"""
_log = ...
class NullPreview:
    """Null Preview"""
    def thread_func(self, picam2) -> None:
        """Thread function

        :param picam2: picamera2 object
        :type picam2: Picamera2
        """
        ...
    
    def __init__(self, x=..., y=..., width=..., height=..., transform=...) -> None:
        """Initialise null preview

        :param x: X position, defaults to None
        :type x: int, optional
        :param y: Y position, defaults to None
        :type y: int, optional
        :param width: Width, defaults to None
        :type width: int, optional
        :param height: Height, defaults to None
        :type height: int, optional
        :param transform: Transform, defaults to None
        :type transform: libcamera.Transform, optional
        """
        ...
    
    def start(self, picam2) -> None:
        """Starts null preview

        :param picam2: Picamera2 object
        :type picam2: Picamera2
        """
        ...
    
    def set_overlay(self, overlay) -> None:
        """Sets overlay

        :param overlay: Overlay
        """
        ...
    
    def render_request(self, completed_request) -> None:
        """Draw the camera image. For the NullPreview, there is nothing to do."""
        ...
    
    def handle_request(self, picam2) -> None:
        """Handle requests

        :param picam2: picamera2 object
        :type picam2: Picamera2
        """
        ...
    
    def stop(self) -> None:
        """Stop preview"""
        ...
    
    def set_title_function(self, function) -> None:
        ...
    


