NET=$1

CONF_DIR=~/.amax_eva_$NET
ENV_FILE=$CONF_DIR/eva.env
[ ! -f "$ENV_FILE" ] && mkdir -p $CONF_DIR && cp .env $ENV_FILE
[ -f "$ENV_FILE" ] && source $ENV_FILE

mkdir -p $DATA_SHARE

docker-compose up --build -d