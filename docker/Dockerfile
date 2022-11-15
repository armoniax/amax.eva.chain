## create amax_eva buildtime env
FROM paritytech/ci-linux:974ba3ac-20201006
ARG BUILD_BRANCH=main
ARG PROFILE=release

RUN cd / && git clone -b $BUILD_BRANCH --recursive https://github.com/armoniax/amax.eva.chain.git
WORKDIR /amax.eva.chain
RUN bash -c "cargo build --${PROFILE}"

## 2nd stage
FROM phusion/baseimage:0.11
LABEL maintainer "Ray de Dragon Master"
ARG PROFILE=release
ARG NET=devnet
RUN mkdir -p /opt/$NET/amax.eva.chain
COPY --from=0 ./target/${PROFILE}/amax-eva /opt/$NET/amax.eva.chain/amax-eva