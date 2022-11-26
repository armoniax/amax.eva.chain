NET=$1
CONF_DIR=~/.amax_eva/$NET
[ ! -f "$ENV_FILE" ] && mkdir -p $CONF_DIR && cp .env $CONF_DIR/

source $CONF_DIR/.env
docker build -t $IMGTAG \
  --build-arg BUILD_BRANCH=$BUILD_BRANCH \
  --build-arg PROFILE=$PROFILE \
  --build-arg IMGTAG=$IMGTAG .
