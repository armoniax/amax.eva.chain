## create amax_eva buildtime env
FROM paritytech/ci-linux:production AS builder
ARG BUILD_BRANCH
ARG PROFILE
ARG FEATURES

RUN cd / && git clone -b $BUILD_BRANCH --recursive https://github.com/armoniax/amax.eva.chain.git
RUN cd /amax.eva.chain && git pull && cargo build --$PROFILE --features="${FEATURES}"

## 2nd stage
FROM parity/substrate:latest
LABEL maintainer "Ray de Dragon Master"
ARG PROFILE
COPY --from=builder --chmod=755 "/amax.eva.chain/target/${PROFILE}/amax-eva" "/opt/amax.eva.chain/bin/"
WORKDIR /opt/amax.eva.chain
ENTRYPOINT ["./bin/amax-eva"]

EXPOSE 1992
EXPOSE 1993
EXPOSE 1994