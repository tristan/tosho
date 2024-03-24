FROM rust:latest

WORKDIR /usr/src/tosho
COPY . .

RUN cargo install --path .

WORKDIR /
RUN rm -rf /usr/src/tosho
RUN mkdir -p /root/.config/tosho
RUN touch /root/.config/tosho.toml

CMD ["tosho"]
