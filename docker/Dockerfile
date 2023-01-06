FROM debian:buster
RUN apt update -y && apt install -y grub nasm make curl binutils build-essential xorriso grub-pc-bin
RUN curl --proto '=https' --tlsv1.3 https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly -c rust-src rust-std rustc cargo
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo search --limit 0
WORKDIR /tmp/builder