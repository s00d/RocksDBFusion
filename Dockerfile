# Define versions used to select image versions
# (ARGs declared before FROM can't be used outside of FROMs)
FROM debian:bullseye-slim

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true

RUN apt-get update && apt install curl build-essential gcc libclang-dev make openssl libssl-dev git -y

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y

RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /code
ENTRYPOINT [ "" ]