NET=$1
CONF_DIR=~/.amax_eva/$NET
[ ! -f "$ENV_FILE" ] && mkdir -p $CONF_DIR && cp .env $CONF_DIR/ && cp docker-compose.yml $CONF_DIR/

source $CONF_DIR/.env
DOCKER_BUILDKIT=1 docker build -t $IMGTAG \
  --build-arg BUILD_BRANCH=$BUILD_BRANCH \
  --build-arg PROFILE=$PROFILE \
  --build-arg FEATURES=$FEATURES \
  --build-arg IMGTAG=$IMGTAG .
