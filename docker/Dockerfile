## create amax_eva buildtime env
FROM paritytech/ci-linux:production AS builder
ARG BUILD_BRANCH=main
ARG PROFILE=release

RUN cd / && git clone -b $BUILD_BRANCH --recursive https://github.com/armoniax/amax.eva.chain.git
RUN cd /amax.eva.chain && cargo build --${PROFILE}

## 2nd stage
FROM parity/substrate:latest
LABEL maintainer "Ray de Dragon Master"
ARG PROFILE=release
ARG NET=devnet
RUN mkdir -p /opt/$NET/
COPY --from=builder /amax.eva.chain/target/${PROFILE}/amax-eva /opt/$NET/amax-eva