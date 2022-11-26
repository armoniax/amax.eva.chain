source ./.env
CONF_DIR=~/.amax_eva_$NET

[ ! -d "$CONF_DIR/eva.env" ] && mkdir -p $CONF_DIR && cp .env $CONF_DIR/eva.env

mkdir -p $DATA_SHARE

docker-compose up -d