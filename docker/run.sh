NET=$1
TAG=$2

CONF=~/.amax_eva/$NET/$TAG.env
source $CONF && mkdir -p $EVA_SHARE

docker-compose up -f $CONF -d