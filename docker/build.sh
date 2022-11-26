NET=$1
CONF_DIR=~/.amax_eva/$NET

[ ! -f "$ENV_FILE" ] && mkdir -p $CONF_DIR && cp .env $CONF_DIR/
source $CONF_DIR/.env
docker build -t $EVA_IMG_TAG .