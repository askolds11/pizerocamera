"""
This type stub file was generated by pyright.
"""

from picamera2.encoders.v4l2_encoder import V4L2Encoder as V4L2Encoder

"""H264 encoder functionality"""
class H264Encoder(V4L2Encoder):
    """Uses functionality from V4L2Encoder"""
    def __init__(self, bitrate=..., repeat=..., iperiod=..., framerate=..., enable_sps_framerate=..., qp=..., profile=...) -> None:
        """H264 Encoder

        :param bitrate: Bitrate, default None
        :type bitrate: int
        :param repeat: Repeat seq header, defaults to True
        :type repeat: bool, optional
        :param iperiod: Iperiod, defaults to None
        :type iperiod: int, optional
        :param framerate: record a framerate in the stream (whether true or not)
        :type framerate: float, optional
        :param qp: Fixed quantiser from 1 to 51 (disables constant bitrate), default None
        :type qp: int
        """
        ...
    


