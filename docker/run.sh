source .env
CONF_DIR=~/.amax_eva_$NET

if ! -d $CONF_DIR; then mkdir -p $CONF_DIR && cp .env $CONF_DIR/eva.env; fi

mkdir -p $DATA_SHARE

docker-compose up -d