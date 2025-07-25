import io
import logging
import socketserver
import threading
from http import server
from http.server import BaseHTTPRequestHandler
from threading import Condition
from typing import Any

import numpy as np

import libcamera
from picamera2 import Picamera2
from picamera2.encoders import MJPEGEncoder
from picamera2.outputs import FileOutput


class StreamingOutput(io.BufferedIOBase):
    def __init__(self):
        self.frame = None
        self.condition = Condition()

    def write(self, buf):
        with self.condition:
            self.frame = buf
            self.condition.notify_all()
        # Convert buffer to bytes to get length
        return len(buf)


class StreamingHandler(BaseHTTPRequestHandler):
    def __init__(self, streaming_output: StreamingOutput, *args, **kwargs):
        self.streaming_output = streaming_output
        super().__init__(*args, **kwargs)

    def do_GET(self):
        if self.path == '/stream.mjpg':
            self.send_response(200)
            self.send_header('Age', '0')
            self.send_header('Cache-Control', 'no-cache, private')
            self.send_header('Pragma', 'no-cache')
            self.send_header('Content-Type', 'multipart/x-mixed-replace; boundary=FRAME')
            self.end_headers()

            try:
                while True:
                    with self.streaming_output.condition:
                        self.streaming_output.condition.wait()
                        frame = self.streaming_output.frame

                    if frame is None:
                        continue

                    self.wfile.write(b'--FRAME\r\n')
                    self.send_header('Content-Type', 'image/jpeg')
                    self.send_header('Content-Length', str(len(frame)))
                    self.end_headers()
                    self.wfile.write(frame)
                    self.wfile.write(b'\r\n')
            except Exception as e:
                logging.warning("Removed streaming client %s: %s", self.client_address, str(e))
        else:
            self.send_error(404)
            self.end_headers()

class StreamingServer(socketserver.ThreadingMixIn, server.HTTPServer):
    allow_reuse_address = True
    daemon_threads = True


class CameraService:
    cam: Picamera2
    config: dict[str, Any]
    # Streaming
    file_output: FileOutput | None
    streaming_output: StreamingOutput
    encoder: MJPEGEncoder | None
    http_server: StreamingServer | None
    server_thread: threading.Thread | None

    def __init__(self, still_controls: dict[str, Any] | None = None):
        """
        Initializes the camera service
        """
        print("Python - CameraService init")
        self.cam = Picamera2()
        self.file_output = None
        self.encoder = None
        self.http_server = None
        self.server_thread = None
        self.streaming_output = StreamingOutput()

        self.set_still_configuration(still_controls)
        print("Python - Starting camera")
        self.cam.start()
        print("Python - Camera started")

    def set_still_configuration(self, still_controls: dict[str, Any] | None = None):
        print("Python - Configuring camera")
        if still_controls is None:
            still_controls = {}
        still_config = self.cam.create_still_configuration(
            main={"size": (3280, 2464), "format": "BGR888"},
            lores=None,
            #raw
            transform=libcamera.Transform(),
            colour_space=libcamera.ColorSpace.Sycc(),
            buffer_count=1,
            controls=still_controls,
            display=None,
            encode=None,
            queue=True,
            sensor={"output_size": (3280, 2464), "bit_depth": 10},
            use_case="still"
        )
        self.cam.stop()
        self.cam.configure(still_config)

    def capture(self) -> tuple[np.ndarray, int, int, dict[str, Any]]:
        """
        :return: Jpeg bytes and metadata
        """
        # TODO: Maybe can use this instead of awaiting in rust?
        # request = picam2.capture_request(flush=time.monotonic_ns())
        request = self.cam.capture_request()
        # image = request.make_image("main")
        array = request.make_array("main")
        metadata = request.get_metadata()
        request.release()

        # jpeg_buffer = io.BytesIO()
        # image.save(jpeg_buffer, format='JPEG', quality=95)
        # jpeg_bytes = jpeg_buffer.getvalue()

        flattened_array = array.flatten()
        height, width, _ = array.shape

        # print(metadata)

        return flattened_array, width, height, metadata

    def stop(self):
        print("Stopping camera")
        self.cam.stop()

    def start_preview(self, video_controls: dict[str, Any] | None = None):
        self.cam.stop()
        if video_controls is None:
            video_controls = {}
        video_config = self.cam.create_video_configuration(
            main={"size": (1640, 1232), "format": "XBGR8888"},
            lores=None,
            # raw
            transform=libcamera.Transform(),
            colour_space=libcamera.ColorSpace.Rec709(),
            buffer_count=6,
            controls=video_controls,
            display=None,
            encode="main",
            queue=True,
            sensor={"output_size": (1640, 1232), "bit_depth": 10},
            use_case="still"
        )

        self.cam.configure(video_config)
        self.cam.start()

        # Create encoder and start streaming
        self.encoder = MJPEGEncoder()
        self.file_output = FileOutput(self.streaming_output)
        self.cam.start_encoder(self.encoder, self.file_output, name="main")

        if not self.http_server:
            # Start HTTP server with threading support
            def handler_factory(*args, **kwargs):
                return StreamingHandler(self.streaming_output, *args, **kwargs)

            self.http_server = StreamingServer(('', 8000), handler_factory)
            self.server_thread = threading.Thread(target=self.http_server.serve_forever, daemon=True)
            self.server_thread.start()


    def stop_preview(self, still_controls: dict[str, Any] | None = None):
        # Stop HTTP server
        if self.http_server:
            self.http_server.shutdown()
            self.http_server.server_close()
            self.http_server = None

        # Stop encoder
        if self.encoder:
            self.cam.stop_encoder()
            self.encoder = None

        if self.file_output:
            self.file_output.close()
            self.file_output = None

        # Switch back to still configuration
        self.cam.stop()
        if still_controls is None:
            still_controls = {}
        still_config = self.cam.create_still_configuration(still_controls)
        self.cam.configure(still_config)
        self.cam.start()

    def set_controls(self, controls: dict[str, Any]):
        print("Settings controls:\n", controls)
        self.cam.set_controls(controls)

    def get_controls(self,):
        print("Getting controls:\n", self.cam.camera_controls)
        return self.cam.camera_controls

# cameraService = CameraService()
# cameraService.preview_full()
#
# while True:
#     x = 1
