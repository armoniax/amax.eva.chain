NET=$1

CONF_DIR=~/.amax_eva_$NET

[ ! -f "$CONF_DIR/eva.env" ] && mkdir -p $CONF_DIR && cp .env $CONF_DIR/eva.env
[ -f "$CONF_DIR/eva.env" ] && source $CONF_DIR/eva.env

mkdir -p $DATA_SHARE

docker-compose up --build -d