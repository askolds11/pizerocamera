FROM tttapa/rpi-cross-toolchain:armv6-rpi-linux-gnueabihf

ARG LIBCAMERA_VERSION=0.5.1
ARG SYSROOT=/home/develop/x-tools/armv6-rpi-linux-gnueabihf/armv6-rpi-linux-gnueabihf/sysroot
ARG LIBCAMERA_COMPILE_PATH=/home/develop/libcamera-compile

USER root

# Install apt packages
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        python3-pip cmake pkg-config curl \
        libyaml-dev python3-yaml python3-ply python3-jinja2 ninja-build clang \
        python3-dev python-dev \
        nano \
        software-properties-common && \
        add-apt-repository ppa:deadsnakes/ppa && \
        apt install python3.11 python3.11-dev python3.11-venv -y \
        && apt-get clean && rm -rf /var/lib/apt/lists/*

# Install pip packages
RUN pip install --no-cache-dir meson

# clone
RUN git clone https://git.libcamera.org/libcamera/libcamera.git $LIBCAMERA_COMPILE_PATH --branch v$LIBCAMERA_VERSION

WORKDIR $LIBCAMERA_COMPILE_PATH

# Copy cross file
COPY cross-file.txt .

# compile, install
RUN meson setup build --cross-file cross-file.txt --prefix=/usr && \
    DESTDIR=$SYSROOT ninja -C build install

# clean up files
RUN rm -r $LIBCAMERA_COMPILE_PATH

USER develop
WORKDIR /home/develop

# Install rust
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable
ENV PATH="/home/develop/.cargo/bin:${PATH}"
RUN /home/develop/.cargo/bin/rustup target add arm-unknown-linux-gnueabihf

# ./build-python-cross.sh 3.11.10 armv6-rpi-linux-gnueabihf
# Compile python
RUN git clone https://github.com/tttapa/python-dev.git python-dev --branch 0.0.7
COPY --chown=develop:develop pythoncrossbuild.sh python-dev
RUN cd python-dev && chmod +x pythoncrossbuild.sh && ./pythoncrossbuild.sh 3.11.10 armv6-rpi-linux-gnueabihf

USER root
RUN cp -r /home/develop/python-dev-3.11.10-armv6-rpi-linux-gnueabihf/usr/local $SYSROOT/usr
RUN mkdir /keepfiles && chown develop:develop /keepfiles
RUN cp -r /home/develop/.cargo /keepfiles
RUN cp -r /home/develop/python-dev /keepfiles
RUN cp -r /home/develop/python-dev-3.11.10-armv6-rpi-linux-gnueabihf /keepfiles
RUN cp -r /home/develop/zlib-1.3 /keepfiles
USER develop
# set env variables for compilation
ENV PKG_CONFIG_SYSROOT_DIR=$SYSROOT \
    PKG_CONFIG_PATH=$SYSROOT/usr/lib/pkgconfig \
    PKG_CONFIG_LIBDIR=$SYSROOT/usr/lib/pkgconfig:$SYSROOT/usr/share/pkgconfig \
    PKG_CONFIG_ALLOW_CROSS=1 \
    TARGET_CXX="armv6-rpi-linux-gnueabihf-g++" \
    PYO3_CROSS="1" \
    PYO3_PYTHON="python3.11" \
    PYO3_CROSS_LIB_DIR=/home/develop/python-dev-3.11.10-armv6-rpi-linux-gnueabihf/usr/local/lib