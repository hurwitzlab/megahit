FROM rust:1.39.0-stretch as rust
RUN git clone https://github.com/hurwitzlab/megahit.git
RUN cd megahit/run_megahit && cargo build --release && cp target/release/run_megahit /usr/local/bin

FROM python:3.7.4-buster
RUN apt-get -y update
RUN apt-get install -y parallel wget
RUN wget -q https://github.com/voutcn/megahit/releases/download/v1.2.9/MEGAHIT-1.2.9-Linux-x86_64-static.tar.gz && tar zvxf MEGAHIT-1.2.9-Linux-x86_64-static.tar.gz && cp MEGAHIT-1.2.9-Linux-x86_64-static/bin/* /usr/local/bin
COPY --from=rust /usr/local/bin /usr/local/bin
COPY --from=rust /usr/local/lib /usr/local/lib
ENV LD_LIBRARY_PATH /usr/local/lib

# Run when the container launches
CMD ["run_megahit"]
